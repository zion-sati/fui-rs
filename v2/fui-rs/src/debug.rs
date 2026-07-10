use crate::event::{self, PointerType};
use crate::ffi::{self, KeyEventType, PointerEventType};
use crate::node::NodeHandle;

pub fn pointer_event(
    event_type: PointerEventType,
    handle: NodeHandle,
    x: f32,
    y: f32,
    modifiers: u32,
    pointer_id: i32,
    pointer_type: PointerType,
    button: i32,
    buttons: u32,
    pressure: f32,
    width: f32,
    height: f32,
    click_count: i32,
) -> bool {
    event::dispatch_pointer_event(
        handle,
        event_type,
        x,
        y,
        modifiers,
        pointer_id,
        pointer_type,
        button,
        buttons,
        pressure,
        width,
        height,
        click_count,
    )
}

pub fn focus_changed(handle: NodeHandle, focused: bool) {
    event::dispatch_focus_changed(handle, focused);
}

pub fn key_event(event_type: KeyEventType, key: &str, modifiers: u32) -> bool {
    event::dispatch_key_event(event_type, key.to_string(), modifiers)
}

#[no_mangle]
pub extern "C" fn __fui_debug_pointer_event(
    event_type: u32,
    handle: u64,
    x: f32,
    y: f32,
    modifiers: u32,
) {
    event::dispatch_pointer_event(
        NodeHandle::from_raw(handle),
        match event_type {
            1 => PointerEventType::Down,
            2 => PointerEventType::Up,
            3 => PointerEventType::Move,
            4 => PointerEventType::Enter,
            5 => PointerEventType::Leave,
            _ => PointerEventType::Cancel,
        },
        x,
        y,
        modifiers,
        -1,
        PointerType::Mouse,
        0,
        0,
        0.0,
        0.0,
        0.0,
        0,
    );
}

#[no_mangle]
pub extern "C" fn __fui_debug_key_event(
    event_type: u32,
    key_ptr: *const u8,
    key_len: u32,
    modifiers: u32,
) {
    let key = if key_ptr.is_null() || key_len == 0 {
        String::new()
    } else {
        let bytes = unsafe { std::slice::from_raw_parts(key_ptr, key_len as usize) };
        String::from_utf8_lossy(bytes).into_owned()
    };
    event::dispatch_key_event(
        match event_type {
            2 => KeyEventType::Up,
            _ => KeyEventType::Down,
        },
        key,
        modifiers,
    );
}

#[no_mangle]
pub extern "C" fn __fui_debug_focus_changed(handle: u64, focused: bool) {
    event::dispatch_focus_changed(NodeHandle::from_raw(handle), focused);
}

#[no_mangle]
pub extern "C" fn __fui_debug_scroll(
    handle: u64,
    offset_x: f32,
    offset_y: f32,
    content_width: f32,
    content_height: f32,
    viewport_width: f32,
    viewport_height: f32,
) {
    event::dispatch_scroll(
        NodeHandle::from_raw(handle),
        offset_x,
        offset_y,
        content_width,
        content_height,
        viewport_width,
        viewport_height,
    );
}

pub fn debug_tree_words() -> Vec<u32> {
    let mut len = 0u32;
    let ptr = unsafe { ffi::ui_get_debug_tree_buffer(&mut len as *mut u32) };
    if ptr.is_null() || len == 0 {
        return Vec::new();
    }
    unsafe { std::slice::from_raw_parts(ptr, len as usize) }.to_vec()
}

#[cfg(test)]
mod tests {
    use super::debug_tree_words;
    use crate::ffi;

    #[test]
    fn reads_debug_tree_words_from_host() {
        ffi::test::reset();
        ffi::test::set_debug_tree_words(&[1, 2, 3, 4]);
        assert_eq!(debug_tree_words(), vec![1, 2, 3, 4]);
    }
}
