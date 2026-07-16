use crate::animation;
use crate::assets;
use crate::bindings::ui;
use crate::context_menu_manager;
use crate::event;
use crate::frame_signal;
use crate::viewport;
use crate::Application;
use std::cell::{Cell, RefCell};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScrollEvent {
    pub handle: u64,
    pub offset_x: f32,
    pub offset_y: f32,
    pub content_width: f32,
    pub content_height: f32,
    pub viewport_width: f32,
    pub viewport_height: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ContextMenuRequest {
    pub handle: u64,
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AssetFailure {
    pub id: u32,
    pub error: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AssetReady {
    pub id: u32,
    pub width: f32,
    pub height: f32,
}

thread_local! {
    static CURRENT_ROUTE: RefCell<String> = const { RefCell::new(String::new()) };
    static LAST_SCROLL: RefCell<Option<ScrollEvent>> = const { RefCell::new(None) };
    static LAST_CONTEXT_MENU: RefCell<Option<ContextMenuRequest>> = const { RefCell::new(None) };
    static CONTEXT_MENU_VISIBLE: Cell<bool> = const { Cell::new(false) };
    static LAST_FONT_LOADED: Cell<Option<u32>> = const { Cell::new(None) };
    static LAST_SVG_LOADED: RefCell<Option<AssetReady>> = const { RefCell::new(None) };
    static LAST_SVG_FAILED: RefCell<Option<AssetFailure>> = const { RefCell::new(None) };
    static LAST_TEXTURE_LOADED: RefCell<Option<AssetReady>> = const { RefCell::new(None) };
    static LAST_TEXTURE_FAILED: RefCell<Option<AssetFailure>> = const { RefCell::new(None) };
    static PERSIST_CAPTURE_COUNT: Cell<u32> = const { Cell::new(0) };
    static PERSIST_RESTORE_COUNT: Cell<u32> = const { Cell::new(0) };
}

fn read_utf8(ptr: *const u8, len: u32) -> String {
    if ptr.is_null() || len == 0 {
        return String::new();
    }
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len as usize) };
    String::from_utf8_lossy(bytes).into_owned()
}

pub fn current_route() -> String {
    CURRENT_ROUTE.with(|route| route.borrow().clone())
}

pub fn last_scroll_event() -> Option<ScrollEvent> {
    LAST_SCROLL.with(|event| event.borrow().clone())
}

pub fn last_context_menu_request() -> Option<ContextMenuRequest> {
    LAST_CONTEXT_MENU.with(|event| event.borrow().clone())
}

pub fn is_context_menu_visible() -> bool {
    CONTEXT_MENU_VISIBLE.with(Cell::get)
}

pub fn last_font_loaded() -> Option<u32> {
    LAST_FONT_LOADED.with(Cell::get)
}

pub fn last_svg_loaded() -> Option<AssetReady> {
    LAST_SVG_LOADED.with(|event| event.borrow().clone())
}

pub fn last_svg_failed() -> Option<AssetFailure> {
    LAST_SVG_FAILED.with(|event| event.borrow().clone())
}

pub fn last_texture_loaded() -> Option<AssetReady> {
    LAST_TEXTURE_LOADED.with(|event| event.borrow().clone())
}

pub fn last_texture_failed() -> Option<AssetFailure> {
    LAST_TEXTURE_FAILED.with(|event| event.borrow().clone())
}

pub fn persisted_capture_count() -> u32 {
    PERSIST_CAPTURE_COUNT.with(Cell::get)
}

pub fn persisted_restore_count() -> u32 {
    PERSIST_RESTORE_COUNT.with(Cell::get)
}

#[no_mangle]
pub extern "C" fn __fui_on_viewport_changed(width: f32, height: f32) {
    ui::resize_window(width, height);
    viewport::set_viewport_size(width, height);
}

#[no_mangle]
pub extern "C" fn __fui_on_frame(timestamp_ms: f64) {
    frame_signal::set_frame_time(timestamp_ms);
    animation::tick_animations(timestamp_ms);
}

#[no_mangle]
pub extern "C" fn __fui_on_route_changed(route_ptr: *const u8, route_len: u32) {
    let route = read_utf8(route_ptr, route_len);
    CURRENT_ROUTE.with(|slot| slot.replace(route));
}

#[no_mangle]
pub extern "C" fn __fui_on_scroll(
    handle: u64,
    offset_x: f32,
    offset_y: f32,
    content_width: f32,
    content_height: f32,
    viewport_width: f32,
    viewport_height: f32,
) {
    LAST_SCROLL.with(|slot| {
        slot.replace(Some(ScrollEvent {
            handle,
            offset_x,
            offset_y,
            content_width,
            content_height,
            viewport_width,
            viewport_height,
        }));
    });
    event::dispatch_scroll(
        crate::node::NodeHandle::from_raw(handle),
        offset_x,
        offset_y,
        content_width,
        content_height,
        viewport_width,
        viewport_height,
    );
}

#[no_mangle]
pub extern "C" fn __fui_can_show_context_menu(handle: u64) -> bool {
    context_menu_manager::can_show_for_handle(handle)
}

#[no_mangle]
pub extern "C" fn __fui_on_context_menu(handle: u64, x: f32, y: f32) {
    LAST_CONTEXT_MENU.with(|slot| slot.replace(Some(ContextMenuRequest { handle, x, y })));
    let shown = context_menu_manager::show_for_current_selection(handle, x, y);
    CONTEXT_MENU_VISIBLE.with(|visible| visible.set(shown));
}

#[no_mangle]
pub extern "C" fn __fui_hide_active_context_menu() {
    context_menu_manager::hide_active_menu();
    CONTEXT_MENU_VISIBLE.with(|visible| visible.set(false));
}

#[no_mangle]
pub extern "C" fn __fui_on_font_loaded(font_id: u32) {
    LAST_FONT_LOADED.with(|slot| slot.set(Some(font_id)));
    assets::on_font_loaded(font_id);
    crate::typography::notify_font_loaded(font_id);
    crate::text::notify_font_loaded(font_id);
    crate::frame_scheduler::mark_needs_commit();
}

#[no_mangle]
pub extern "C" fn __fui_on_svg_loaded(svg_id: u32, width: f32, height: f32) {
    LAST_SVG_LOADED.with(|slot| {
        slot.replace(Some(AssetReady {
            id: svg_id,
            width,
            height,
        }))
    });
    assets::on_svg_loaded(svg_id, width, height);
}

#[no_mangle]
pub extern "C" fn __fui_on_svg_failed(svg_id: u32, error_ptr: *const u8, error_len: u32) {
    let error = read_utf8(error_ptr, error_len);
    LAST_SVG_FAILED.with(|slot| {
        slot.replace(Some(AssetFailure {
            id: svg_id,
            error: error.clone(),
        }))
    });
    assets::on_svg_failed(svg_id, error);
}

#[no_mangle]
pub extern "C" fn __fui_on_texture_loaded(texture_id: u32, width: f32, height: f32) {
    LAST_TEXTURE_LOADED.with(|slot| {
        slot.replace(Some(AssetReady {
            id: texture_id,
            width,
            height,
        }))
    });
    assets::on_texture_loaded(texture_id, width, height);
}

#[no_mangle]
pub extern "C" fn __fui_on_texture_failed(texture_id: u32, error_ptr: *const u8, error_len: u32) {
    let error = read_utf8(error_ptr, error_len);
    LAST_TEXTURE_FAILED.with(|slot| {
        slot.replace(Some(AssetFailure {
            id: texture_id,
            error: error.clone(),
        }))
    });
    assets::on_texture_failed(texture_id, error);
}

#[no_mangle]
pub extern "C" fn __fui_capture_persisted_ui_state() {
    PERSIST_CAPTURE_COUNT.with(|count| count.set(count.get() + 1));
    Application::capture_persisted_ui_state();
}

#[no_mangle]
pub extern "C" fn __fui_restore_persisted_ui_state() {
    PERSIST_RESTORE_COUNT.with(|count| count.set(count.get() + 1));
    Application::restore_persisted_ui_state();
}
