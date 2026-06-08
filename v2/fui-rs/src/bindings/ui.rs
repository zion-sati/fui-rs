use crate::ffi;

pub fn reset() {
    unsafe { ffi::ui_reset() }
}

pub fn create_node(node_type: u32) -> u64 {
    unsafe { ffi::ui_create_node(node_type) }
}

pub fn delete_node(handle: u64) {
    unsafe { ffi::ui_delete_node(handle) }
}

pub fn add_child(parent: u64, child: u64) {
    unsafe { ffi::ui_node_add_child(parent, child) }
}

#[allow(dead_code)]
pub fn remove_child(parent: u64, child: u64) {
    unsafe { ffi::ui_node_remove_child(parent, child) }
}

pub fn set_root(handle: u64) {
    unsafe { ffi::ui_set_root(handle) }
}

pub fn set_width(handle: u64, value: f32, unit: u32) {
    unsafe { ffi::ui_set_width(handle, value, unit) }
}

pub fn set_height(handle: u64, value: f32, unit: u32) {
    unsafe { ffi::ui_set_height(handle, value, unit) }
}

pub fn set_bg_color(handle: u64, color: u32) {
    unsafe { ffi::ui_set_bg_color(handle, color) }
}

pub fn set_text(handle: u64, text: &str) {
    let bytes = text.as_bytes();
    unsafe {
        ffi::ui_set_text(
            handle,
            if bytes.is_empty() {
                std::ptr::null()
            } else {
                bytes.as_ptr()
            },
            bytes.len() as u32,
        )
    }
}

pub fn set_font(handle: u64, font_id: u32, size: f32) {
    unsafe { ffi::ui_set_font(handle, font_id, size) }
}

pub fn set_text_color(handle: u64, color: u32) {
    unsafe { ffi::ui_set_text_color(handle, color) }
}

pub fn set_padding(handle: u64, top: f32, right: f32, bottom: f32, left: f32) {
    unsafe { ffi::ui_set_padding(handle, top, right, bottom, left) }
}

pub fn set_flex_direction(handle: u64, direction: u32) {
    unsafe { ffi::ui_set_flex_direction(handle, direction) }
}

pub fn commit_frame() {
    unsafe { ffi::ui_commit_frame() }
}

pub fn resize_window(width: f32, height: f32) {
    unsafe { ffi::ui_resize_window(width, height) }
}

pub fn request_render() {
    unsafe { ffi::request_render() }
}

pub fn get_viewport_width() -> f32 {
    unsafe { ffi::get_viewport_width() }
}

pub fn get_viewport_height() -> f32 {
    unsafe { ffi::get_viewport_height() }
}
