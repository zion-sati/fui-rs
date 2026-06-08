#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "effindom_v2_ui")]
unsafe extern "C" {
    pub fn ui_reset();
    pub fn ui_create_node(node_type: u32) -> u64;
    pub fn ui_delete_node(handle: u64);
    pub fn ui_node_add_child(parent: u64, child: u64);
    pub fn ui_node_remove_child(parent: u64, child: u64);
    pub fn ui_set_root(handle: u64);
    pub fn ui_set_width(handle: u64, value: f32, unit: u32);
    pub fn ui_set_height(handle: u64, value: f32, unit: u32);
    pub fn ui_set_bg_color(handle: u64, color: u32);
    pub fn ui_set_text(handle: u64, ptr: *const u8, len: u32);
    pub fn ui_set_font(handle: u64, font_id: u32, size: f32);
    pub fn ui_set_text_color(handle: u64, color: u32);
    pub fn ui_set_padding(handle: u64, top: f32, right: f32, bottom: f32, left: f32);
    pub fn ui_set_flex_direction(handle: u64, direction: u32);
    pub fn ui_commit_frame();
    pub fn ui_resize_window(width: f32, height: f32);
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "fui_host")]
unsafe extern "C" {
    pub fn request_render();
    pub fn get_viewport_width() -> f32;
    pub fn get_viewport_height() -> f32;
}

#[repr(u64)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandleValue {
    Invalid = 0,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    FlexBox = 0,
    Text = 1,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Pixel = 0,
    Auto = 1,
    Star = 2,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexDirection {
    Column = 0,
    Row = 1,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq)]
pub enum Call {
    Reset,
    CreateNode { node_type: u32, handle: u64 },
    DeleteNode { handle: u64 },
    AddChild { parent: u64, child: u64 },
    RemoveChild { parent: u64, child: u64 },
    SetRoot { handle: u64 },
    SetWidth { handle: u64, value: f32, unit: u32 },
    SetHeight { handle: u64, value: f32, unit: u32 },
    SetBgColor { handle: u64, color: u32 },
    SetText { handle: u64, text: String },
    SetFont { handle: u64, font_id: u32, size: f32 },
    SetTextColor { handle: u64, color: u32 },
    SetPadding { handle: u64, top: f32, right: f32, bottom: f32, left: f32 },
    SetFlexDirection { handle: u64, direction: u32 },
    CommitFrame,
    ResizeWindow { width: f32, height: f32 },
    RequestRender,
}

#[cfg(not(target_arch = "wasm32"))]
thread_local! {
    static CALLS: std::cell::RefCell<Vec<Call>> = std::cell::RefCell::new(Vec::new());
    static NEXT_HANDLE: std::cell::Cell<u64> = const { std::cell::Cell::new(1) };
    static VIEWPORT: std::cell::Cell<(f32, f32)> = const { std::cell::Cell::new((320.0, 220.0)) };
}

#[cfg(not(target_arch = "wasm32"))]
fn push_call(call: Call) {
    CALLS.with(|calls| calls.borrow_mut().push(call));
}

#[cfg(not(target_arch = "wasm32"))]
pub mod test {
    use super::{Call, CALLS, NEXT_HANDLE, VIEWPORT};

    pub fn reset() {
        CALLS.with(|calls| calls.borrow_mut().clear());
        NEXT_HANDLE.with(|next| next.set(1));
        VIEWPORT.with(|viewport| viewport.set((320.0, 220.0)));
    }

    pub fn take_calls() -> Vec<Call> {
        CALLS.with(|calls| std::mem::take(&mut *calls.borrow_mut()))
    }

    pub fn set_viewport(width: f32, height: f32) {
        VIEWPORT.with(|viewport| viewport.set((width, height)));
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn ui_reset() {
    push_call(Call::Reset);
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn ui_create_node(node_type: u32) -> u64 {
    let handle = NEXT_HANDLE.with(|next| {
        let handle = next.get();
        next.set(handle + 1);
        handle
    });
    push_call(Call::CreateNode { node_type, handle });
    handle
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn ui_delete_node(handle: u64) {
    push_call(Call::DeleteNode { handle });
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn ui_node_add_child(parent: u64, child: u64) {
    push_call(Call::AddChild { parent, child });
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn ui_node_remove_child(parent: u64, child: u64) {
    push_call(Call::RemoveChild { parent, child });
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn ui_set_root(handle: u64) {
    push_call(Call::SetRoot { handle });
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn ui_set_width(handle: u64, value: f32, unit: u32) {
    push_call(Call::SetWidth { handle, value, unit });
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn ui_set_height(handle: u64, value: f32, unit: u32) {
    push_call(Call::SetHeight { handle, value, unit });
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn ui_set_bg_color(handle: u64, color: u32) {
    push_call(Call::SetBgColor { handle, color });
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn ui_set_text(handle: u64, ptr: *const u8, len: u32) {
    let text = if ptr.is_null() || len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(ptr, len as usize)).into_owned()
    };
    push_call(Call::SetText { handle, text });
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn ui_set_font(handle: u64, font_id: u32, size: f32) {
    push_call(Call::SetFont { handle, font_id, size });
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn ui_set_text_color(handle: u64, color: u32) {
    push_call(Call::SetTextColor { handle, color });
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn ui_set_padding(handle: u64, top: f32, right: f32, bottom: f32, left: f32) {
    push_call(Call::SetPadding { handle, top, right, bottom, left });
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn ui_set_flex_direction(handle: u64, direction: u32) {
    push_call(Call::SetFlexDirection { handle, direction });
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn ui_commit_frame() {
    push_call(Call::CommitFrame);
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn ui_resize_window(width: f32, height: f32) {
    push_call(Call::ResizeWindow { width, height });
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn request_render() {
    push_call(Call::RequestRender);
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn get_viewport_width() -> f32 {
    VIEWPORT.with(|viewport| viewport.get().0)
}

#[cfg(not(target_arch = "wasm32"))]
pub unsafe fn get_viewport_height() -> f32 {
    VIEWPORT.with(|viewport| viewport.get().1)
}
