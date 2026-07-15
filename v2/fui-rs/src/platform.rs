use crate::ffi;
use crate::generated::framework_host_services;

#[cfg(feature = "native-runtime")]
use std::cell::RefCell;
#[cfg(feature = "native-runtime")]
use std::collections::HashMap;
#[cfg(feature = "native-runtime")]
use std::path::Path;
#[cfg(feature = "native-runtime")]
use std::path::PathBuf;
#[cfg(feature = "native-runtime")]
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(feature = "native-runtime")]
unsafe extern "C" {
    fn fui_dispatch_to_ui(callback_id: u64) -> bool;
    fn fui_cancel_ui_dispatch_async(callback_id: u64) -> bool;
    fn fui_native_clipboard_write(text: *const u8, length: u32) -> bool;
    fn fui_native_clipboard_text_length() -> u32;
    fn fui_native_clipboard_copy(destination: *mut u8, capacity: u32) -> u32;
    fn fui_native_open_external_url(value: *const u8, length: u32) -> bool;
    fn fui_native_open_file(value: *const u8, length: u32) -> bool;
    fn fui_native_reveal_file(value: *const u8, length: u32) -> bool;
    fn fui_native_show_file_dialog(
        kind: u32,
        request_id: u64,
        filters: *const u8,
        filters_length: u32,
        default_location: *const u8,
        default_location_length: u32,
        allow_multiple: bool,
    ) -> bool;
}

#[cfg(feature = "native-runtime")]
thread_local! {
    static UI_DISPATCH_CALLBACKS: RefCell<HashMap<u64, Box<dyn FnOnce()>>> = RefCell::new(HashMap::new());
}

#[cfg(feature = "native-runtime")]
static NEXT_UI_DISPATCH_ID: AtomicU64 = AtomicU64::new(1);

#[cfg(feature = "native-runtime")]
static NEXT_NATIVE_FILE_DIALOG_ID: AtomicU64 = AtomicU64::new(1);

#[cfg(feature = "native-runtime")]
type NativeFileDialogCallback = Box<dyn FnOnce(NativeFileDialogResult)>;

#[cfg(feature = "native-runtime")]
thread_local! {
    static NATIVE_FILE_DIALOG_CALLBACKS: RefCell<HashMap<u64, NativeFileDialogCallback>> = RefCell::new(HashMap::new());
}

#[cfg(feature = "native-runtime")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeFileFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

#[cfg(feature = "native-runtime")]
impl NativeFileFilter {
    pub fn new(
        name: impl Into<String>,
        extensions: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            name: name.into(),
            extensions: extensions.into_iter().map(Into::into).collect(),
        }
    }
}

#[cfg(feature = "native-runtime")]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NativeFileDialogOptions {
    pub filters: Vec<NativeFileFilter>,
    pub default_location: Option<PathBuf>,
    pub allow_multiple: bool,
}

#[cfg(feature = "native-runtime")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativeFileDialogResult {
    Selected {
        paths: Vec<PathBuf>,
        selected_filter: Option<usize>,
    },
    Cancelled,
    Error(String),
}

#[cfg(feature = "native-runtime")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeFileDialogRequest {
    request_id: u64,
}

#[cfg(feature = "native-runtime")]
impl NativeFileDialogRequest {
    pub fn id(self) -> u64 {
        self.request_id
    }
}

/// A one-shot, `Send` token for work whose closure remains owned by the UI thread.
#[cfg(feature = "native-runtime")]
pub struct UiDispatchHandle {
    callback_id: u64,
    pending: bool,
}

#[cfg(feature = "native-runtime")]
impl UiDispatchHandle {
    pub fn dispatch(mut self) -> bool {
        let dispatched = unsafe { fui_dispatch_to_ui(self.callback_id) };
        if dispatched {
            self.pending = false;
        }
        dispatched
    }
}

#[cfg(feature = "native-runtime")]
impl Drop for UiDispatchHandle {
    fn drop(&mut self) {
        if self.pending {
            unsafe {
                fui_cancel_ui_dispatch_async(self.callback_id);
            }
        }
    }
}

#[cfg(feature = "native-runtime")]
pub struct UiDispatcher;

#[cfg(feature = "native-runtime")]
impl UiDispatcher {
    /// Keeps retained work on the UI thread and returns a token that may be sent to a worker.
    pub fn prepare(callback: impl FnOnce() + 'static) -> UiDispatchHandle {
        let callback_id = NEXT_UI_DISPATCH_ID.fetch_add(1, Ordering::Relaxed);
        UI_DISPATCH_CALLBACKS.with(|callbacks| {
            callbacks
                .borrow_mut()
                .insert(callback_id, Box::new(callback));
        });
        UiDispatchHandle {
            callback_id,
            pending: true,
        }
    }
}

#[cfg(feature = "native-runtime")]
pub fn write_clipboard_text(text: &str) -> bool {
    unsafe { fui_native_clipboard_write(text.as_ptr(), text.len() as u32) }
}

#[cfg(feature = "native-runtime")]
pub fn read_clipboard_text() -> Option<String> {
    let length = unsafe { fui_native_clipboard_text_length() };
    let mut bytes = vec![0u8; length as usize];
    let copied = unsafe { fui_native_clipboard_copy(bytes.as_mut_ptr(), length) };
    bytes.truncate(copied as usize);
    String::from_utf8(bytes).ok()
}

#[cfg(feature = "native-runtime")]
pub fn open_external_url(url: &str) -> bool {
    unsafe { fui_native_open_external_url(url.as_ptr(), url.len() as u32) }
}

#[cfg(feature = "native-runtime")]
pub fn open_file(path: impl AsRef<Path>) -> bool {
    let Some(path) = path.as_ref().to_str() else {
        return false;
    };
    unsafe { fui_native_open_file(path.as_ptr(), path.len() as u32) }
}

#[cfg(feature = "native-runtime")]
pub fn reveal_file(path: impl AsRef<Path>) -> bool {
    let Some(path) = path.as_ref().to_str() else {
        return false;
    };
    unsafe { fui_native_reveal_file(path.as_ptr(), path.len() as u32) }
}

#[cfg(feature = "native-runtime")]
fn show_native_file_dialog(
    kind: u32,
    options: NativeFileDialogOptions,
    callback: impl FnOnce(NativeFileDialogResult) + 'static,
) -> Option<NativeFileDialogRequest> {
    let request_id = NEXT_NATIVE_FILE_DIALOG_ID.fetch_add(1, Ordering::Relaxed);
    let mut encoded_filters = Vec::new();
    for filter in &options.filters {
        if filter.name.is_empty() || filter.extensions.is_empty() {
            return None;
        }
        encoded_filters.extend_from_slice(filter.name.as_bytes());
        encoded_filters.push(0);
        encoded_filters.extend_from_slice(filter.extensions.join(";").as_bytes());
        encoded_filters.push(0);
    }
    let default_location = options
        .default_location
        .as_ref()
        .and_then(|path| path.to_str())
        .unwrap_or_default();
    NATIVE_FILE_DIALOG_CALLBACKS.with(|callbacks| {
        callbacks
            .borrow_mut()
            .insert(request_id, Box::new(callback));
    });
    let shown = unsafe {
        fui_native_show_file_dialog(
            kind,
            request_id,
            encoded_filters.as_ptr(),
            encoded_filters.len() as u32,
            default_location.as_ptr(),
            default_location.len() as u32,
            options.allow_multiple,
        )
    };
    if !shown {
        NATIVE_FILE_DIALOG_CALLBACKS.with(|callbacks| {
            callbacks.borrow_mut().remove(&request_id);
        });
        return None;
    }
    Some(NativeFileDialogRequest { request_id })
}

#[cfg(feature = "native-runtime")]
pub fn show_open_file_dialog(
    options: NativeFileDialogOptions,
    callback: impl FnOnce(NativeFileDialogResult) + 'static,
) -> Option<NativeFileDialogRequest> {
    show_native_file_dialog(0, options, callback)
}

#[cfg(feature = "native-runtime")]
pub fn show_save_file_dialog(
    options: NativeFileDialogOptions,
    callback: impl FnOnce(NativeFileDialogResult) + 'static,
) -> Option<NativeFileDialogRequest> {
    show_native_file_dialog(1, options, callback)
}

#[cfg(feature = "native-runtime")]
pub fn show_open_folder_dialog(
    options: NativeFileDialogOptions,
    callback: impl FnOnce(NativeFileDialogResult) + 'static,
) -> Option<NativeFileDialogRequest> {
    show_native_file_dialog(2, options, callback)
}

#[cfg(feature = "native-runtime")]
#[no_mangle]
pub unsafe extern "C" fn __fui_complete_native_file_dialog(
    request_id: u64,
    status: u32,
    payload: *const u8,
    payload_length: u32,
    selected_filter: i32,
) -> bool {
    let callback =
        NATIVE_FILE_DIALOG_CALLBACKS.with(|callbacks| callbacks.borrow_mut().remove(&request_id));
    let Some(callback) = callback else {
        return false;
    };
    let bytes = if payload.is_null() || payload_length == 0 {
        &[][..]
    } else {
        unsafe { std::slice::from_raw_parts(payload, payload_length as usize) }
    };
    let result = match status {
        0 => NativeFileDialogResult::Selected {
            paths: bytes
                .split(|byte| *byte == 0)
                .filter(|path| !path.is_empty())
                .filter_map(|path| std::str::from_utf8(path).ok())
                .map(PathBuf::from)
                .collect(),
            selected_filter: usize::try_from(selected_filter).ok(),
        },
        1 => NativeFileDialogResult::Cancelled,
        _ => NativeFileDialogResult::Error(String::from_utf8_lossy(bytes).into_owned()),
    };
    callback(result);
    true
}

#[cfg(feature = "native-runtime")]
#[no_mangle]
pub extern "C" fn __fui_clear_native_file_dialog_callbacks() {
    NATIVE_FILE_DIALOG_CALLBACKS.with(|callbacks| callbacks.borrow_mut().clear());
}

#[cfg(feature = "native-runtime")]
#[no_mangle]
pub extern "C" fn __fui_run_ui_dispatch(callback_id: u64) -> bool {
    let callback =
        UI_DISPATCH_CALLBACKS.with(|callbacks| callbacks.borrow_mut().remove(&callback_id));
    let Some(callback) = callback else {
        return false;
    };
    callback();
    true
}

#[cfg(feature = "native-runtime")]
#[no_mangle]
pub extern "C" fn __fui_cancel_ui_dispatch(callback_id: u64) {
    UI_DISPATCH_CALLBACKS.with(|callbacks| {
        callbacks.borrow_mut().remove(&callback_id);
    });
}

#[cfg(feature = "native-runtime")]
#[no_mangle]
pub extern "C" fn __fui_clear_ui_dispatches() {
    UI_DISPATCH_CALLBACKS.with(|callbacks| callbacks.borrow_mut().clear());
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlatformFamily {
    Unknown = 0,
    Apple = 1,
    Windows = 2,
    Linux = 3,
}

pub fn device_pixel_ratio() -> f32 {
    unsafe { ffi::get_device_pixel_ratio() }
}

pub fn platform_family() -> PlatformFamily {
    match framework_host_services::fui_get_platform_family() {
        1 => PlatformFamily::Apple,
        2 => PlatformFamily::Windows,
        3 => PlatformFamily::Linux,
        _ => PlatformFamily::Unknown,
    }
}

pub fn is_coarse_pointer() -> bool {
    framework_host_services::fui_is_coarse_pointer()
}

pub fn primary_shortcut_modifier() -> u32 {
    match platform_family() {
        PlatformFamily::Apple => ffi::KeyModifier::Meta as u32,
        _ => ffi::KeyModifier::Ctrl as u32,
    }
}

pub fn word_navigation_modifier() -> u32 {
    match platform_family() {
        PlatformFamily::Apple => ffi::KeyModifier::Alt as u32,
        _ => ffi::KeyModifier::Ctrl as u32,
    }
}

pub fn line_boundary_modifier() -> u32 {
    match platform_family() {
        PlatformFamily::Apple => ffi::KeyModifier::Meta as u32,
        _ => 0,
    }
}

pub fn document_boundary_modifier() -> u32 {
    match platform_family() {
        PlatformFamily::Apple => ffi::KeyModifier::Meta as u32,
        _ => ffi::KeyModifier::Ctrl as u32,
    }
}

fn has_modifier(modifiers: u32, expected: u32) -> bool {
    expected != 0 && (modifiers & expected) != 0
}

pub fn has_primary_shortcut_modifier(modifiers: u32) -> bool {
    has_modifier(modifiers, primary_shortcut_modifier())
}

pub fn has_word_navigation_modifier(modifiers: u32) -> bool {
    has_modifier(modifiers, word_navigation_modifier())
}

pub fn has_line_boundary_modifier(modifiers: u32) -> bool {
    has_modifier(modifiers, line_boundary_modifier())
}

pub fn has_document_boundary_modifier(modifiers: u32) -> bool {
    has_modifier(modifiers, document_boundary_modifier())
}

fn format_shortcut_key_token(key: &str, platform_family: PlatformFamily) -> String {
    match key {
        "ArrowLeft" => {
            if platform_family == PlatformFamily::Apple {
                "←".to_string()
            } else {
                "Left".to_string()
            }
        }
        "ArrowRight" => {
            if platform_family == PlatformFamily::Apple {
                "→".to_string()
            } else {
                "Right".to_string()
            }
        }
        "ArrowUp" => {
            if platform_family == PlatformFamily::Apple {
                "↑".to_string()
            } else {
                "Up".to_string()
            }
        }
        "ArrowDown" => {
            if platform_family == PlatformFamily::Apple {
                "↓".to_string()
            } else {
                "Down".to_string()
            }
        }
        "PageUp" => "PgUp".to_string(),
        "PageDown" => "PgDn".to_string(),
        _ if key.chars().count() == 1 => key.to_uppercase(),
        _ => key.to_string(),
    }
}

fn append_shortcut_modifier_tokens(
    tokens: &mut Vec<String>,
    modifiers: u32,
    platform: PlatformFamily,
) {
    if platform == PlatformFamily::Apple {
        if (modifiers & ffi::KeyModifier::Ctrl as u32) != 0 {
            tokens.push("⌃".to_string());
        }
        if (modifiers & ffi::KeyModifier::Alt as u32) != 0 {
            tokens.push("⌥".to_string());
        }
        if (modifiers & ffi::KeyModifier::Shift as u32) != 0 {
            tokens.push("⇧".to_string());
        }
        if (modifiers & ffi::KeyModifier::Meta as u32) != 0 {
            tokens.push("⌘".to_string());
        }
        return;
    }

    if (modifiers & ffi::KeyModifier::Ctrl as u32) != 0 {
        tokens.push("Ctrl".to_string());
    }
    if (modifiers & ffi::KeyModifier::Alt as u32) != 0 {
        tokens.push("Alt".to_string());
    }
    if (modifiers & ffi::KeyModifier::Shift as u32) != 0 {
        tokens.push("Shift".to_string());
    }
    if (modifiers & ffi::KeyModifier::Meta as u32) != 0 {
        tokens.push("Meta".to_string());
    }
}

pub fn format_shortcut_label(key: &str, modifiers: u32) -> String {
    let platform = platform_family();
    let mut tokens = Vec::new();
    append_shortcut_modifier_tokens(&mut tokens, modifiers, platform);
    tokens.push(format_shortcut_key_token(key, platform));
    if platform == PlatformFamily::Apple {
        tokens.join("")
    } else {
        tokens.join("+")
    }
}

pub fn format_primary_shortcut_label(key: &str) -> String {
    format_shortcut_label(key, primary_shortcut_modifier())
}

pub fn format_undo_shortcut_label() -> String {
    format_primary_shortcut_label("z")
}

pub fn format_redo_shortcut_label() -> String {
    match platform_family() {
        PlatformFamily::Apple => format_shortcut_label(
            "z",
            primary_shortcut_modifier() | ffi::KeyModifier::Shift as u32,
        ),
        _ => format_primary_shortcut_label("y"),
    }
}

fn matches_shortcut_key(key: &str, expected: &str) -> bool {
    key.eq_ignore_ascii_case(expected)
}

pub fn is_undo_shortcut(key: &str, modifiers: u32) -> bool {
    (modifiers & ffi::KeyModifier::Shift as u32) == 0
        && has_primary_shortcut_modifier(modifiers)
        && matches_shortcut_key(key, "z")
}

pub fn is_redo_shortcut(key: &str, modifiers: u32) -> bool {
    match platform_family() {
        PlatformFamily::Apple => {
            has_primary_shortcut_modifier(modifiers)
                && (modifiers & ffi::KeyModifier::Shift as u32) != 0
                && matches_shortcut_key(key, "z")
        }
        _ => has_primary_shortcut_modifier(modifiers) && matches_shortcut_key(key, "y"),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        document_boundary_modifier, is_coarse_pointer, is_redo_shortcut, is_undo_shortcut,
        line_boundary_modifier, platform_family, PlatformFamily,
    };
    use crate::ffi;

    #[test]
    fn returns_mock_device_pixel_ratio() {
        ffi::test::reset();
        ffi::test::set_device_pixel_ratio(2.5);
        assert_eq!(super::device_pixel_ratio(), 2.5);
    }

    #[test]
    fn reports_platform_family_and_pointer_mode() {
        ffi::test::reset();
        ffi::test::set_platform_family(1);
        ffi::test::set_coarse_pointer(true);
        assert_eq!(platform_family(), PlatformFamily::Apple);
        assert!(is_coarse_pointer());
        assert_eq!(line_boundary_modifier(), ffi::KeyModifier::Meta as u32);
        assert_eq!(document_boundary_modifier(), ffi::KeyModifier::Meta as u32);
        assert!(is_undo_shortcut("z", ffi::KeyModifier::Meta as u32));
        assert!(is_redo_shortcut(
            "z",
            ffi::KeyModifier::Meta as u32 | ffi::KeyModifier::Shift as u32
        ));
    }
}
