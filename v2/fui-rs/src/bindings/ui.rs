use crate::ffi;
use crate::ffi::{GridUnit, ImageSamplingKind};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextRangeRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextSelectionEndpointRects {
    pub start: TextRangeRect,
    pub end: TextRangeRect,
}

fn with_utf8(text: &str, callback: impl FnOnce(*const u8, u32)) {
    let bytes = text.as_bytes();
    callback(
        if bytes.is_empty() {
            std::ptr::null()
        } else {
            bytes.as_ptr()
        },
        bytes.len() as u32,
    );
}

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

pub fn set_node_id(handle: u64, node_id: &str) {
    with_utf8(node_id, |ptr, len| unsafe {
        ffi::ui_set_node_id(handle, ptr, len)
    })
}

pub fn set_semantic_role(handle: u64, role: u32) {
    unsafe { ffi::ui_set_semantic_role(handle, role) }
}

pub fn set_semantic_label(handle: u64, label: &str) {
    with_utf8(label, |ptr, len| unsafe {
        ffi::ui_set_semantic_label(handle, ptr, len)
    })
}

pub fn set_semantic_checked(handle: u64, state: u32) {
    unsafe { ffi::ui_set_semantic_checked(handle, state) }
}

pub fn set_semantic_selected(handle: u64, has_selected: bool, selected: bool) {
    unsafe { ffi::ui_set_semantic_selected(handle, has_selected, selected) }
}

pub fn set_semantic_disabled(handle: u64, has_disabled: bool, disabled: bool) {
    unsafe { ffi::ui_set_semantic_disabled(handle, has_disabled, disabled) }
}

pub fn set_semantic_value_range(
    handle: u64,
    has_value_range: bool,
    value_now: f32,
    value_min: f32,
    value_max: f32,
) {
    unsafe {
        ffi::ui_set_semantic_value_range(handle, has_value_range, value_now, value_min, value_max)
    }
}

pub fn set_semantic_orientation(handle: u64, orientation: u32) {
    unsafe { ffi::ui_set_semantic_orientation(handle, orientation) }
}

pub fn set_semantic_expanded(handle: u64, has_expanded: bool, is_expanded: bool) {
    unsafe { ffi::ui_set_semantic_expanded(handle, has_expanded, is_expanded) }
}

pub fn request_semantic_announcement(handle: u64) {
    unsafe { ffi::ui_request_semantic_announcement(handle) }
}

pub fn request_focus(handle: u64) {
    unsafe { ffi::ui_request_focus(handle) }
}

pub fn push_semantic_scope(handle: u64) -> u32 {
    unsafe { ffi::ui_push_semantic_scope(handle) }
}

pub fn remove_semantic_scope(token: u32) {
    unsafe { ffi::ui_remove_semantic_scope(token) }
}

pub fn set_is_portal(handle: u64, is_portal: bool) {
    unsafe { ffi::ui_set_is_portal(handle, is_portal) }
}

pub fn set_visibility(handle: u64, visibility: u32) {
    unsafe { ffi::ui_set_visibility(handle, visibility) }
}

pub fn set_width(handle: u64, value: f32, unit: u32) {
    unsafe { ffi::ui_set_width(handle, value, unit) }
}

pub fn set_height(handle: u64, value: f32, unit: u32) {
    unsafe { ffi::ui_set_height(handle, value, unit) }
}

pub fn set_fill_width(handle: u64, fill: bool) {
    unsafe { ffi::ui_set_fill_width(handle, fill) }
}

pub fn set_fill_height(handle: u64, fill: bool) {
    unsafe { ffi::ui_set_fill_height(handle, fill) }
}

pub fn set_fill_width_percent(handle: u64, percent: f32) {
    unsafe { ffi::ui_set_fill_width_percent(handle, percent) }
}

pub fn set_fill_height_percent(handle: u64, percent: f32) {
    unsafe { ffi::ui_set_fill_height_percent(handle, percent) }
}

pub fn set_min_width(handle: u64, value: f32, unit: u32) {
    unsafe { ffi::ui_set_min_width(handle, value, unit) }
}

pub fn set_max_width(handle: u64, value: f32, unit: u32) {
    unsafe { ffi::ui_set_max_width(handle, value, unit) }
}

pub fn set_min_height(handle: u64, value: f32, unit: u32) {
    unsafe { ffi::ui_set_min_height(handle, value, unit) }
}

pub fn set_max_height(handle: u64, value: f32, unit: u32) {
    unsafe { ffi::ui_set_max_height(handle, value, unit) }
}

pub fn set_bg_color(handle: u64, color: u32) {
    unsafe { ffi::ui_set_bg_color(handle, color) }
}

pub fn set_box_style(
    handle: u64,
    bg_color: u32,
    radius_tl: f32,
    radius_tr: f32,
    radius_br: f32,
    radius_bl: f32,
    border_width: f32,
    border_color: u32,
    border_style_enum: u32,
    border_dash_on: f32,
    border_dash_off: f32,
) {
    unsafe {
        ffi::ui_set_box_style(
            handle,
            bg_color,
            radius_tl,
            radius_tr,
            radius_br,
            radius_bl,
            border_width,
            border_color,
            border_style_enum,
            border_dash_on,
            border_dash_off,
        )
    }
}

pub fn set_linear_gradient(
    handle: u64,
    sx: f32,
    sy: f32,
    ex: f32,
    ey: f32,
    offsets: &[f32],
    colors: &[u32],
) {
    let count = offsets.len().min(colors.len());
    unsafe {
        ffi::ui_set_linear_gradient(
            handle,
            sx,
            sy,
            ex,
            ey,
            count as u32,
            if count == 0 {
                std::ptr::null()
            } else {
                offsets.as_ptr()
            },
            if count == 0 {
                std::ptr::null()
            } else {
                colors.as_ptr()
            },
        )
    }
}

pub fn set_drop_shadow(
    handle: u64,
    color: u32,
    offset_x: f32,
    offset_y: f32,
    blur_sigma: f32,
    spread: f32,
) {
    unsafe { ffi::ui_set_drop_shadow(handle, color, offset_x, offset_y, blur_sigma, spread) }
}

pub fn set_layer_effect(handle: u64, opacity: f32, blur_sigma: f32, blend_mode_enum: u32) {
    unsafe { ffi::ui_set_layer_effect(handle, opacity, blur_sigma, blend_mode_enum) }
}

pub fn set_background_blur(handle: u64, blur_sigma: f32) {
    unsafe { ffi::ui_set_background_blur(handle, blur_sigma) }
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

pub fn register_font_fallback(font_id: u32, fallback_font_id: u32) {
    unsafe { ffi::ui_register_font_fallback(font_id, fallback_font_id) }
}

pub fn set_line_height(handle: u64, line_height: f32) {
    unsafe { ffi::ui_set_line_height(handle, line_height) }
}

pub fn set_text_style_runs(handle: u64, words: &[u32]) {
    unsafe {
        ffi::ui_set_text_style_runs(
            handle,
            (words.len() / 7) as u32,
            if words.is_empty() {
                std::ptr::null()
            } else {
                words.as_ptr()
            },
        )
    }
}

pub fn prepare_node(handle: u64) -> u64 {
    unsafe { ffi::ui_prepare_node(handle).into() }
}

pub fn set_dynamic_text_charset(handle: u64, charset: &str) {
    with_utf8(charset, |ptr, len| unsafe {
        ffi::ui_set_dynamic_text_charset(handle, ptr, len)
    })
}

pub fn replace_text_range(handle: u64, start: u32, end: u32, text: &str, caret: u32) {
    with_utf8(text, |ptr, len| unsafe {
        ffi::ui_replace_text_range(handle, start, end, ptr, len, caret)
    })
}

pub fn get_text_metrics(handle: u64) -> Option<[f32; 5]> {
    let mut width = 0.0;
    let mut height = 0.0;
    let mut baseline = 0.0;
    let mut line_count = 0u32;
    let mut max_line_width = 0.0;
    let ok = unsafe {
        ffi::ui_get_text_metrics(
            handle,
            &mut width,
            &mut height,
            &mut baseline,
            &mut line_count,
            &mut max_line_width,
        )
    };
    if ok {
        Some([width, height, baseline, line_count as f32, max_line_width])
    } else {
        None
    }
}

pub fn set_text_color(handle: u64, color: u32) {
    unsafe { ffi::ui_set_text_color(handle, color) }
}

pub fn set_text_align(handle: u64, align_enum: u32) {
    unsafe { ffi::ui_set_text_align(handle, align_enum) }
}

pub fn set_text_vertical_align(handle: u64, align_enum: u32) {
    unsafe { ffi::ui_set_text_vertical_align(handle, align_enum) }
}

pub fn set_text_limits(handle: u64, max_chars: i32, max_lines: i32) {
    unsafe { ffi::ui_set_text_limits(handle, max_chars, max_lines) }
}

pub fn set_text_wrapping(handle: u64, wrap: bool) {
    unsafe { ffi::ui_set_text_wrapping(handle, wrap) }
}

pub fn set_text_obscured(handle: u64, obscured: bool) {
    unsafe { ffi::ui_set_text_obscured(handle, obscured) }
}

pub fn set_text_overflow(handle: u64, overflow_enum: u32) {
    unsafe { ffi::ui_set_text_overflow(handle, overflow_enum) }
}

pub fn set_text_overflow_fade(handle: u64, horizontal: bool, vertical: bool) {
    unsafe { ffi::ui_set_text_overflow_fade(handle, horizontal, vertical) }
}

pub fn set_selectable(handle: u64, selectable: bool, selection_color: u32) {
    unsafe { ffi::ui_set_selectable(handle, selectable, selection_color) }
}

pub fn set_editable(handle: u64, editable: bool) {
    unsafe { ffi::ui_set_editable(handle, editable) }
}

pub fn set_editor_command_keys(handle: u64, enabled: bool) {
    unsafe { ffi::ui_set_editor_command_keys(handle, enabled) }
}

pub fn set_editor_accepts_tab(handle: u64, enabled: bool) {
    unsafe { ffi::ui_set_editor_accepts_tab(handle, enabled) }
}

pub fn set_caret_color(handle: u64, color: u32) {
    unsafe { ffi::ui_set_caret_color(handle, color) }
}

pub fn set_text_selection_range(handle: u64, start: u32, end: u32) {
    unsafe { ffi::ui_set_text_selection_range(handle, start, end) }
}

pub fn register_text_input_metadata(handle: u64, is_password: bool, hint: Option<&str>) {
    with_utf8(hint.unwrap_or(""), |ptr, len| unsafe {
        ffi::fui_register_text_input_metadata(handle, is_password, ptr as usize, len)
    });
}

pub fn set_preserve_selection_on_pointer_down(handle: u64, preserve: bool) {
    unsafe { ffi::ui_set_preserve_selection_on_pointer_down(handle, preserve) }
}

pub fn select_word_at(handle: u64, logical_x: f32, logical_y: f32) -> bool {
    unsafe { ffi::ui_select_word_at(handle, logical_x, logical_y) }
}

pub fn begin_selection_endpoint_drag(handle: u64, endpoint: u32) -> bool {
    unsafe { ffi::ui_begin_selection_endpoint_drag(handle, endpoint) }
}

pub fn get_text_range_rects(handle: u64, start: u32, end: u32) -> Vec<TextRangeRect> {
    let rect_count = unsafe { ffi::ui_get_text_range_rect_count(handle, start, end) as usize };
    if rect_count == 0 {
        return Vec::new();
    }
    let mut rect_words = vec![0.0f32; rect_count * 4];
    let copied_count = unsafe {
        ffi::ui_copy_text_range_rects(
            handle,
            start,
            end,
            rect_words.as_mut_ptr(),
            rect_count as u32,
        ) as usize
    };
    if copied_count == 0 {
        return Vec::new();
    }
    rect_words
        .chunks_exact(4)
        .take(copied_count)
        .map(|words| TextRangeRect {
            x: words[0],
            y: words[1],
            width: words[2],
            height: words[3],
        })
        .collect()
}

pub fn get_cross_selection_endpoint_rects(area_handle: u64) -> Option<TextSelectionEndpointRects> {
    let mut rect_words = [0.0f32; 8];
    let ok = unsafe {
        ffi::ui_copy_cross_selection_endpoint_rects(area_handle, rect_words.as_mut_ptr())
    };
    if !ok {
        return None;
    }
    Some(TextSelectionEndpointRects {
        start: TextRangeRect {
            x: rect_words[0],
            y: rect_words[1],
            width: rect_words[2],
            height: rect_words[3],
        },
        end: TextRangeRect {
            x: rect_words[4],
            y: rect_words[5],
            width: rect_words[6],
            height: rect_words[7],
        },
    })
}

pub fn clear_current_selection() {
    unsafe { ffi::ui_clear_current_selection() }
}

pub fn is_point_in_selection(logical_x: f32, logical_y: f32) -> bool {
    unsafe { ffi::ui_is_point_in_selection(logical_x, logical_y) }
}

pub fn set_interactive(handle: u64, interactive: bool) {
    unsafe { ffi::ui_set_interactive(handle, interactive) }
}

pub fn set_scroll_proxy_target(handle: u64, scroll_handle: u64) {
    unsafe { ffi::ui_set_scroll_proxy_target(handle, scroll_handle) }
}

pub fn set_focusable(handle: u64, focusable: bool, tab_index: i32) {
    unsafe { ffi::ui_set_focusable(handle, focusable, tab_index) }
}

pub fn set_padding(handle: u64, left: f32, top: f32, right: f32, bottom: f32) {
    unsafe { ffi::ui_set_padding(handle, left, top, right, bottom) }
}

pub fn set_flex_direction(handle: u64, direction: u32) {
    unsafe { ffi::ui_set_flex_direction(handle, direction) }
}

pub fn set_flex_basis(handle: u64, basis: f32) {
    unsafe { ffi::ui_set_flex_basis(handle, basis) }
}

pub fn set_justify_content(handle: u64, justify: u32) {
    unsafe { ffi::ui_set_justify_content(handle, justify) }
}

pub fn set_align_items(handle: u64, align: u32) {
    unsafe { ffi::ui_set_align_items(handle, align) }
}

pub fn set_align_self(handle: u64, align: u32) {
    unsafe { ffi::ui_set_align_self(handle, align) }
}

pub fn set_margin(handle: u64, left: f32, top: f32, right: f32, bottom: f32) {
    unsafe { ffi::ui_set_margin(handle, left, top, right, bottom) }
}

pub fn set_position_type(handle: u64, position_type: u32) {
    unsafe { ffi::ui_set_position_type(handle, position_type) }
}

pub fn set_position(handle: u64, left: f32, top: f32, right: f32, bottom: f32) {
    unsafe { ffi::ui_set_position(handle, left, top, right, bottom) }
}

pub fn set_is_shared_size_scope(handle: u64, is_scope: bool) {
    unsafe { ffi::ui_set_is_shared_size_scope(handle, is_scope) }
}

pub fn set_custom_drawable(handle: u64, is_custom_drawable: bool) {
    unsafe { ffi::ui_set_custom_drawable(handle, is_custom_drawable) }
}

pub fn set_flex_wrap(handle: u64, wrap: u32) {
    unsafe { ffi::ui_set_flex_wrap(handle, wrap) }
}

pub fn set_clip_to_bounds(handle: u64, clip: bool) {
    unsafe { ffi::ui_set_clip_to_bounds(handle, clip) }
}

pub fn set_selection_area(handle: u64, is_area: bool) {
    unsafe { ffi::ui_set_selection_area(handle, is_area) }
}

pub fn set_selection_area_barrier(handle: u64, is_barrier: bool) {
    unsafe { ffi::ui_set_selection_area_barrier(handle, is_barrier) }
}

pub fn clear_selection(text_node_handle: u64) {
    unsafe { ffi::ui_clear_selection(text_node_handle) }
}

pub fn retarget_selection(from_text_node_handle: u64, to_text_node_handle: u64) {
    unsafe { ffi::ui_retarget_selection(from_text_node_handle, to_text_node_handle) }
}

pub fn grid_set_columns(handle: u64, values: &[f32], types: &[GridUnit]) {
    let count = values.len().min(types.len());
    let type_words: Vec<u8> = types[..count].iter().map(|value| *value as u8).collect();
    unsafe { ffi::ui_grid_set_columns(handle, count as u32, values.as_ptr(), type_words.as_ptr()) }
}

pub fn grid_set_rows(handle: u64, values: &[f32], types: &[GridUnit]) {
    let count = values.len().min(types.len());
    let type_words: Vec<u8> = types[..count].iter().map(|value| *value as u8).collect();
    unsafe { ffi::ui_grid_set_rows(handle, count as u32, values.as_ptr(), type_words.as_ptr()) }
}

pub fn grid_set_column_shared_size_group(handle: u64, index: u32, group: &str) {
    with_utf8(group, |ptr, len| unsafe {
        ffi::ui_grid_set_column_shared_size_group(handle, index, ptr, len)
    });
}

pub fn grid_set_row_shared_size_group(handle: u64, index: u32, group: &str) {
    with_utf8(group, |ptr, len| unsafe {
        ffi::ui_grid_set_row_shared_size_group(handle, index, ptr, len)
    });
}

pub fn set_grid_placement(handle: u64, row: u32, col: u32, row_span: u32, col_span: u32) {
    unsafe { ffi::ui_node_set_grid_placement(handle, row, col, row_span, col_span) }
}

pub fn set_image(
    handle: u64,
    texture_id: u32,
    object_fit: u32,
    sampling_kind: ImageSamplingKind,
    max_aniso: u32,
) {
    unsafe {
        ffi::ui_set_image(
            handle,
            texture_id,
            object_fit,
            sampling_kind as u32,
            max_aniso,
        )
    }
}

pub fn set_image_nine(
    handle: u64,
    texture_id: u32,
    inset_left: f32,
    inset_top: f32,
    inset_right: f32,
    inset_bottom: f32,
    sampling_kind: ImageSamplingKind,
    max_aniso: u32,
) {
    unsafe {
        ffi::ui_set_image_nine(
            handle,
            texture_id,
            inset_left,
            inset_top,
            inset_right,
            inset_bottom,
            sampling_kind as u32,
            max_aniso,
        )
    }
}

pub fn set_svg(
    handle: u64,
    svg_id: u32,
    tint_color: u32,
    sampling_kind: ImageSamplingKind,
    max_aniso: u32,
) {
    unsafe { ffi::ui_set_svg(handle, svg_id, tint_color, sampling_kind as u32, max_aniso) }
}

pub fn set_scroll_enabled(handle: u64, enabled_x: bool, enabled_y: bool) {
    unsafe { ffi::ui_set_scroll_enabled(handle, enabled_x, enabled_y) }
}

pub fn set_show_scrollbars(handle: u64, show_scrollbars: bool) {
    unsafe { ffi::ui_set_show_scrollbars(handle, show_scrollbars) }
}

pub fn set_scroll_friction(handle: u64, friction: f32) {
    unsafe { ffi::ui_set_scroll_friction(handle, friction) }
}

pub fn set_smooth_scrolling(handle: u64, smooth_scrolling: bool) {
    unsafe { ffi::ui_set_smooth_scrolling(handle, smooth_scrolling) }
}

pub fn set_scroll_offset(handle: u64, offset_x: f32, offset_y: f32) {
    unsafe { ffi::ui_set_scroll_offset(handle, offset_x, offset_y) }
}

pub fn set_scroll_content_size(handle: u64, content_width: f32, content_height: f32) {
    unsafe { ffi::ui_set_scroll_content_size(handle, content_width, content_height) }
}

pub fn clear_momentum_scroll() {
    unsafe { ffi::ui_clear_momentum_scroll() }
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

pub fn get_bounds(handle: u64) -> Option<[f32; 4]> {
    let mut x = 0.0;
    let mut y = 0.0;
    let mut width = 0.0;
    let mut height = 0.0;
    let ok = unsafe { ffi::ui_get_bounds(handle, &mut x, &mut y, &mut width, &mut height) };
    if ok {
        Some([x, y, width, height])
    } else {
        None
    }
}

pub fn get_visible_bounds(handle: u64) -> Option<[f32; 4]> {
    let mut x = 0.0;
    let mut y = 0.0;
    let mut width = 0.0;
    let mut height = 0.0;
    let ok = unsafe { ffi::ui_get_visible_bounds(handle, &mut x, &mut y, &mut width, &mut height) };
    if ok {
        Some([x, y, width, height])
    } else {
        None
    }
}
