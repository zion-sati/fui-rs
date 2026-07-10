#pragma once

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#include "effindom.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef uint64_t ui_handle_t;
typedef uint32_t ui_color_t;

enum {
    UI_INVALID_HANDLE = 0,
    UI_ABI_VERSION = 1
};

typedef enum UiNodeType {
    UI_NODE_FLEX_BOX = 0,
    UI_NODE_TEXT = 1,
    UI_NODE_IMAGE = 2,
    UI_NODE_SVG = 3,
    UI_NODE_SCROLLVIEW = 4,
    UI_NODE_GRID = 5,
    UI_NODE_PATH = 6
} UiNodeType;

typedef enum UiEvent {
    UI_EVENT_POINTER_DOWN = 1,
    UI_EVENT_POINTER_UP = 2,
    UI_EVENT_POINTER_MOVE = 3,
    UI_EVENT_POINTER_ENTER = 4,
    UI_EVENT_POINTER_LEAVE = 5,
    UI_EVENT_POINTER_CANCEL = 6,
    UI_EVENT_CLICK = 7,
    UI_EVENT_RIGHT_CLICK = 8
} UiEvent;

typedef enum UiKeyEventType {
    UI_KEY_EVENT_DOWN = 1,
    UI_KEY_EVENT_UP = 2
} UiKeyEventType;

typedef enum UiKeyModifier {
    UI_KEY_MOD_SHIFT = 1 << 0,
    UI_KEY_MOD_CTRL = 1 << 1,
    UI_KEY_MOD_ALT = 1 << 2,
    UI_KEY_MOD_META = 1 << 3
} UiKeyModifier;

typedef enum UiPointerType {
    UI_POINTER_TYPE_UNKNOWN = 0,
    UI_POINTER_TYPE_MOUSE = 1,
    UI_POINTER_TYPE_TOUCH = 2,
    UI_POINTER_TYPE_PEN = 3
} UiPointerType;

typedef enum UiSizeUnit {
    UI_SIZE_UNIT_PIXEL = 0,
    UI_SIZE_UNIT_AUTO = 1,
    UI_SIZE_UNIT_PERCENT = 2
} UiSizeUnit;

typedef enum UiGridUnit {
    UI_GRID_UNIT_PIXEL = 0,
    UI_GRID_UNIT_AUTO = 1,
    UI_GRID_UNIT_STAR = 2
} UiGridUnit;

typedef enum UiPositionType {
    UI_POSITION_RELATIVE = 0,
    UI_POSITION_ABSOLUTE = 1
} UiPositionType;

typedef enum UiAlignSelf {
    UI_ALIGN_SELF_AUTO = 0,
    UI_ALIGN_SELF_START = 1,
    UI_ALIGN_SELF_CENTER = 2,
    UI_ALIGN_SELF_END = 3,
    UI_ALIGN_SELF_STRETCH = 4
} UiAlignSelf;

typedef enum UiAlignItems {
    UI_ALIGN_ITEMS_START = 0,
    UI_ALIGN_ITEMS_CENTER = 1,
    UI_ALIGN_ITEMS_END = 2,
    UI_ALIGN_ITEMS_STRETCH = 3,
    UI_ALIGN_ITEMS_NONE = 4
} UiAlignItems;

typedef enum UiFlexDirection {
    UI_FLEX_DIRECTION_COLUMN = 0,
    UI_FLEX_DIRECTION_ROW = 1
} UiFlexDirection;

typedef enum UiFlexWrap {
    UI_FLEX_WRAP_NO_WRAP = 0,
    UI_FLEX_WRAP_WRAP = 1,
    UI_FLEX_WRAP_WRAP_REVERSE = 2
} UiFlexWrap;

typedef enum UiJustifyContent {
    UI_JUSTIFY_START = 0,
    UI_JUSTIFY_CENTER = 1,
    UI_JUSTIFY_END = 2
} UiJustifyContent;

typedef enum UiTextAlign {
    UI_TEXT_ALIGN_LEFT = 0,
    UI_TEXT_ALIGN_CENTER = 1,
    UI_TEXT_ALIGN_RIGHT = 2
} UiTextAlign;

typedef enum UiTextVerticalAlign {
    UI_TEXT_VERTICAL_ALIGN_TOP = 0,
    UI_TEXT_VERTICAL_ALIGN_CENTER = 1,
    UI_TEXT_VERTICAL_ALIGN_BOTTOM = 2
} UiTextVerticalAlign;

typedef enum UiTextOverflow {
    UI_TEXT_OVERFLOW_CLIP = 0,
    UI_TEXT_OVERFLOW_ELLIPSIS = 1,
    UI_TEXT_OVERFLOW_FADE = 2
} UiTextOverflow;

typedef enum UiVisibility {
    UI_VISIBILITY_NORMAL = 0,
    UI_VISIBILITY_HIDDEN = 1,
    UI_VISIBILITY_COLLAPSED = 2
} UiVisibility;

typedef enum UiSemanticRole {
    UI_SEMANTIC_NONE = 0,
    UI_SEMANTIC_BUTTON = 1,
    UI_SEMANTIC_TEXTBOX = 2,
    UI_SEMANTIC_LINK = 3,
    UI_SEMANTIC_HEADING = 4,
    UI_SEMANTIC_FORM = 5,
    UI_SEMANTIC_LIST = 6,
    UI_SEMANTIC_LIST_ITEM = 7,
    UI_SEMANTIC_IMAGE = 8,
    UI_SEMANTIC_DIALOG = 9,
    UI_SEMANTIC_STATIC_TEXT = 10,
    UI_SEMANTIC_CHECKBOX = 11,
    UI_SEMANTIC_RADIO = 12,
    UI_SEMANTIC_RADIO_GROUP = 13,
    UI_SEMANTIC_SWITCH = 14,
    UI_SEMANTIC_SLIDER = 15,
    UI_SEMANTIC_COMBOBOX = 16
} UiSemanticRole;

typedef enum UiSemanticCheckedState {
    UI_SEMANTIC_CHECKED_NONE = 0,
    UI_SEMANTIC_CHECKED_FALSE = 1,
    UI_SEMANTIC_CHECKED_TRUE = 2,
    UI_SEMANTIC_CHECKED_MIXED = 3
} UiSemanticCheckedState;

typedef enum UiOrientation {
    UI_ORIENTATION_NONE = 0,
    UI_ORIENTATION_HORIZONTAL = 1,
    UI_ORIENTATION_VERTICAL = 2
} UiOrientation;

typedef enum UiMissingFontCoverageKind {
    UI_MISSING_FONT_COVERAGE_UNKNOWN = 0,
    UI_MISSING_FONT_COVERAGE_ARABIC = 1,
    UI_MISSING_FONT_COVERAGE_THAI = 2,
    UI_MISSING_FONT_COVERAGE_CJK = 3,
    UI_MISSING_FONT_COVERAGE_SUPPLEMENTAL = 4
} UiMissingFontCoverageKind;

uint32_t ui_get_abi_version(void);

uintptr_t ui_arena_alloc(uint32_t size);

void ui_reset(void);

ui_handle_t ui_create_node(UiNodeType type);
void ui_delete_node(ui_handle_t handle);

void ui_set_node_id(ui_handle_t handle, const uint8_t* utf8_id, uint32_t len);
void ui_set_semantic_role(ui_handle_t handle, UiSemanticRole role_enum);
void ui_set_semantic_label(ui_handle_t handle, const uint8_t* utf8_label, uint32_t len);
void ui_set_semantic_checked(ui_handle_t handle, UiSemanticCheckedState checked_state_enum);
void ui_set_semantic_selected(ui_handle_t handle, bool has_selected, bool is_selected);
void ui_set_semantic_expanded(ui_handle_t handle, bool has_expanded, bool is_expanded);
void ui_set_semantic_disabled(ui_handle_t handle, bool has_disabled, bool is_disabled);
void ui_set_semantic_value_range(ui_handle_t handle, bool has_value_range, float value_now, float value_min, float value_max);
void ui_set_semantic_orientation(ui_handle_t handle, UiOrientation orientation_enum);
void ui_request_semantic_announcement(ui_handle_t handle);

uint32_t ui_push_semantic_scope(ui_handle_t handle);
void ui_remove_semantic_scope(uint32_t token);
void ui_node_add_child(ui_handle_t parent, ui_handle_t child);
void ui_node_remove_child(ui_handle_t parent, ui_handle_t child);
void ui_set_is_portal(ui_handle_t handle, bool is_portal);
void ui_set_visibility(ui_handle_t handle, UiVisibility visibility_enum);

void ui_set_width(ui_handle_t handle, float value, UiSizeUnit unit_enum);
void ui_set_height(ui_handle_t handle, float value, UiSizeUnit unit_enum);
void ui_set_fill_width(ui_handle_t handle, bool fill);
void ui_set_fill_height(ui_handle_t handle, bool fill);
void ui_set_fill_width_percent(ui_handle_t handle, float percent);
void ui_set_fill_height_percent(ui_handle_t handle, float percent);
void ui_set_min_width(ui_handle_t handle, float value, UiSizeUnit unit_enum);
void ui_set_max_width(ui_handle_t handle, float value, UiSizeUnit unit_enum);
void ui_set_min_height(ui_handle_t handle, float value, UiSizeUnit unit_enum);
void ui_set_max_height(ui_handle_t handle, float value, UiSizeUnit unit_enum);
void ui_set_flex_direction(ui_handle_t handle, UiFlexDirection dir_enum);
void ui_set_flex_basis(ui_handle_t handle, float basis);
void ui_set_justify_content(ui_handle_t handle, UiJustifyContent justify_enum);
void ui_set_align_items(ui_handle_t handle, UiAlignItems align_enum);
void ui_set_align_self(ui_handle_t handle, UiAlignSelf align_enum);
void ui_set_padding(ui_handle_t handle, float left, float top, float right, float bottom);
void ui_set_margin(ui_handle_t handle, float left, float top, float right, float bottom);
void ui_set_position_type(ui_handle_t handle, UiPositionType pos_enum);
void ui_set_position(ui_handle_t handle, float left, float top, float right, float bottom);
void ui_set_is_shared_size_scope(ui_handle_t handle, bool is_scope);
void ui_set_custom_drawable(ui_handle_t handle, bool is_custom_drawable);
void ui_set_flex_wrap(ui_handle_t handle, UiFlexWrap wrap_enum);

// Prepares a node for off‑screen rendering. Runs paragraph layout and text
// shaping for a text node, then appends SetBounds + SetGlyphRun commands to
// the pending‑prepare buffer. The next CommitFrame merges them into the
// scene command buffer, after which ed_render_node_to_rgba can render the node.
// Returns 1 on success, 0 if the node is not a text node.
uint32_t ui_prepare_node(ui_handle_t handle);
void ui_set_dynamic_text_charset(ui_handle_t handle, const uint8_t* utf8_charset, uint32_t len);
bool ui_get_text_metrics(
    ui_handle_t handle,
    float* out_width,
    float* out_height,
    float* out_baseline,
    uint32_t* out_line_count,
    float* out_max_line_width);

void ui_grid_set_columns(ui_handle_t handle, uint32_t count, const float* values, const uint8_t* types);
void ui_grid_set_rows(ui_handle_t handle, uint32_t count, const float* values, const uint8_t* types);
void ui_grid_set_column_shared_size_group(
    ui_handle_t handle,
    uint32_t index,
    const uint8_t* utf8_group,
    uint32_t len);
void ui_grid_set_row_shared_size_group(
    ui_handle_t handle,
    uint32_t index,
    const uint8_t* utf8_group,
    uint32_t len);
void ui_node_set_grid_placement(ui_handle_t child, uint32_t row, uint32_t col, uint32_t row_span, uint32_t col_span);

void ui_set_bg_color(ui_handle_t handle, ui_color_t color);
void ui_set_box_style(
    ui_handle_t handle,
    ui_color_t bg_color,
    float radius_tl,
    float radius_tr,
    float radius_br,
    float radius_bl,
    float border_width,
    ui_color_t border_color,
    EdBorderStyle border_style_enum,
    float border_dash_on,
    float border_dash_off);
void ui_set_clip_to_bounds(ui_handle_t handle, bool clip);
void ui_set_linear_gradient(
    ui_handle_t handle,
    float sx,
    float sy,
    float ex,
    float ey,
    uint32_t stop_count,
    const float* offsets,
    const ui_color_t* colors);
void ui_set_drop_shadow(ui_handle_t handle, ui_color_t color, float offset_x, float offset_y, float blur_sigma, float spread);
void ui_set_layer_effect(ui_handle_t handle, float opacity, float blur_sigma, EdBlendMode blend_mode_enum);
void ui_set_background_blur(ui_handle_t handle, float blur_sigma);

void ui_set_image(
    ui_handle_t handle,
    uint32_t texture_id,
    EdObjectFit object_fit_enum,
    EdImageSampling sampling_kind,
    uint32_t max_aniso);
void ui_set_image_nine(
    ui_handle_t handle,
    uint32_t texture_id,
    float inset_l,
    float inset_t,
    float inset_r,
    float inset_b,
    EdImageSampling sampling_kind,
    uint32_t max_aniso);
void ui_set_svg(
    ui_handle_t handle,
    uint32_t svg_id,
    ui_color_t tint_color,
    EdImageSampling sampling_kind,
    uint32_t max_aniso);

void ui_set_text(ui_handle_t handle, const uint8_t* utf8_str, uint32_t len);
void ui_set_text_style_runs(ui_handle_t handle, uint32_t run_count, const uint32_t* runs_words);
void ui_set_font(ui_handle_t handle, uint32_t font_id, float size);
void ui_set_line_height(ui_handle_t handle, float line_height);
void ui_set_text_color(ui_handle_t handle, ui_color_t color);
void ui_set_text_align(ui_handle_t handle, UiTextAlign align_enum);
void ui_set_text_vertical_align(ui_handle_t handle, UiTextVerticalAlign align_enum);
void ui_set_text_limits(ui_handle_t handle, int32_t max_chars, int32_t max_lines);
void ui_set_text_wrapping(ui_handle_t handle, bool wrap);
void ui_set_text_overflow(ui_handle_t handle, UiTextOverflow overflow_enum);
void ui_set_text_overflow_fade(ui_handle_t handle, bool horizontal, bool vertical);
void ui_set_text_obscured(ui_handle_t handle, bool is_password);
void ui_measure_text(
    const uint8_t* utf8_str,
    uint32_t len,
    uint32_t font_id,
    float size,
    float max_width,
    float* out_width,
    float* out_height);

void ui_set_interactive(ui_handle_t handle, bool interactive);
void ui_set_preserve_selection_on_pointer_down(ui_handle_t handle, bool preserve);
void ui_set_editor_command_keys(ui_handle_t handle, bool enabled);
void ui_set_editor_accepts_tab(ui_handle_t handle, bool enabled);
void ui_set_scroll_proxy_target(ui_handle_t handle, ui_handle_t scroll_handle);
void ui_set_scroll_enabled(ui_handle_t handle, bool enabled_x, bool enabled_y);
void ui_set_show_scrollbars(ui_handle_t handle, bool show_scrollbars);
void ui_set_scroll_friction(ui_handle_t handle, float friction);
void ui_set_smooth_scrolling(ui_handle_t handle, bool smooth_scrolling);
void ui_set_focusable(ui_handle_t handle, bool focusable, int32_t tab_index);
void ui_request_focus(ui_handle_t handle);
void ui_set_scroll_offset(ui_handle_t handle, float offset_x, float offset_y);
void ui_set_scroll_content_size(ui_handle_t handle, float content_width, float content_height);
void ui_set_selectable(ui_handle_t handle, bool selectable, ui_color_t selection_color);
void ui_set_selection_area(ui_handle_t handle, bool is_area);
void ui_set_selection_area_barrier(ui_handle_t handle, bool is_barrier);
void ui_clear_selection(ui_handle_t text_node_handle);
void ui_retarget_selection(ui_handle_t from_text_node_handle, ui_handle_t to_text_node_handle);
bool ui_is_point_in_selection(float logical_x, float logical_y);
void ui_set_text_selection_range(ui_handle_t handle, uint32_t selection_start, uint32_t selection_end);
bool ui_select_word_at(ui_handle_t handle, float logical_x, float logical_y);
uint32_t ui_get_text_snapshot_handle_count(void);
uint32_t ui_copy_text_snapshot_handles(uint32_t* out_handle_words, uint32_t max_handle_count);
bool ui_set_text_find_match(ui_handle_t handle, uint32_t start, uint32_t end);
void ui_clear_text_find_match(void);
bool ui_push_text_find_highlight(ui_handle_t handle, uint32_t start, uint32_t end, ui_color_t color);
void ui_clear_text_find_highlights(void);
uint32_t ui_get_text_document_utf8_length(ui_handle_t handle);
bool ui_copy_text_document_utf8(ui_handle_t handle, uint8_t* out_utf8, uint32_t buffer_length);
bool ui_get_text_visible_bounds(
    ui_handle_t handle,
    float* out_x,
    float* out_y,
    float* out_width,
    float* out_height);
uint32_t ui_get_text_range_rect_count(ui_handle_t handle, uint32_t start, uint32_t end);
uint32_t ui_copy_text_range_rects(
    ui_handle_t handle,
    uint32_t start,
    uint32_t end,
    float* out_rect_words,
    uint32_t max_rect_count);
bool ui_copy_cross_selection_endpoint_rects(ui_handle_t area_handle, float* out_rect_words);
bool ui_begin_selection_endpoint_drag(ui_handle_t handle, uint32_t endpoint);
bool ui_preserves_selection_on_pointer_down(ui_handle_t handle);
bool ui_reveal_text_range(ui_handle_t handle, uint32_t start, uint32_t end);
void ui_clear_current_selection(void);
void ui_copy_current_selection(void);
bool ui_can_undo_text_edit(ui_handle_t handle);
bool ui_can_redo_text_edit(ui_handle_t handle);
bool ui_has_text_selection(ui_handle_t handle);
void ui_undo_text_edit(ui_handle_t handle);
void ui_redo_text_edit(ui_handle_t handle);
void ui_copy_text_selection(ui_handle_t handle);
void ui_cut_text_selection(ui_handle_t handle);
void ui_paste_text(ui_handle_t handle);
void ui_select_all_text(ui_handle_t handle);
void ui_set_editable(ui_handle_t handle, bool editable);
void ui_set_caret_color(ui_handle_t handle, ui_color_t color);

#ifdef __cplusplus
void ui_commit_frame(double timestamp_ms = -1.0);
#else
void ui_commit_frame(double timestamp_ms);
#endif
bool ui_has_pending_visual_work(void);
bool ui_needs_animation_frame(void);
bool ui_has_pointer_autoscroll(void);
ui_handle_t ui_selection_autoscroll(float logical_x, float logical_y, float edge_threshold);

void ui_on_pointer_event(
    UiEvent event_enum,
    ui_handle_t handle,
    float logical_x,
    float logical_y,
    int32_t pointer_id,
    UiPointerType pointer_type,
    int32_t button,
    uint32_t buttons,
    float pressure,
    float width,
    float height,
    int32_t click_count,
    uint32_t modifiers);
void ui_on_wheel_event(float delta_x, float delta_y);
#ifdef __cplusplus
void ui_touch_scroll_begin(ui_handle_t handle, float logical_x, float logical_y, double timestamp_ms = -1.0);
void ui_touch_scroll_update(float delta_x, float delta_y, double timestamp_ms = -1.0);
#else
void ui_touch_scroll_begin(ui_handle_t handle, float logical_x, float logical_y, double timestamp_ms);
void ui_touch_scroll_update(float delta_x, float delta_y, double timestamp_ms);
#endif
bool ui_wheel_scroll_can_consume(float delta_x, float delta_y);
bool ui_touch_scroll_can_consume(float delta_x, float delta_y);
#ifdef __cplusplus
void ui_touch_scroll_end(double timestamp_ms = -1.0);
#else
void ui_touch_scroll_end(double timestamp_ms);
#endif
void ui_clear_momentum_scroll(void);
bool ui_touch_scroll_allows_pull_to_refresh(void);
void ui_set_coarse_pointer_mode(bool coarse_pointer_mode);
void ui_set_platform_family(uint32_t platform_family);
bool ui_on_key_event(UiKeyEventType type_enum, const uint8_t* key_utf8, uint32_t len, uint32_t modifiers);
void ui_set_interaction_time(uint64_t interaction_time_ms);
void ui_on_ime_update(ui_handle_t handle, const uint8_t* utf8_str, uint32_t len, uint32_t caret_idx);
void ui_replace_text_range(
    ui_handle_t handle,
    uint32_t start_idx,
    uint32_t end_idx,
    const uint8_t* utf8_str,
    uint32_t len,
    uint32_t caret_idx);
void ui_on_paste_text(ui_handle_t handle, const uint8_t* utf8_str, uint32_t len);
void ui_font_loaded(uint32_t font_id);
void ui_register_icu_data(const uint8_t* bytes, uint32_t len);
void ui_register_font_fallback(uint32_t font_id, uint32_t fallback_font_id);
bool ui_unregister_font_fallback(uint32_t font_id, uint32_t fallback_font_id);
bool ui_unregister_font(uint32_t font_id);

void ui_set_root(ui_handle_t handle);
void ui_resize_window(float logical_w, float logical_h);
bool ui_register_font(uint32_t font_id, const uint8_t* bytes, uint32_t len);

const uint32_t* ui_get_command_buffer(uint32_t* out_length);
const uint32_t* ui_get_semantic_buffer(uint32_t* out_length);
const uint32_t* ui_get_debug_tree_buffer(uint32_t* out_length);
const uint32_t* ui_get_live_fallback_font_buffer(uint32_t* out_length);
bool ui_get_bounds(
    ui_handle_t handle,
    float* out_x,
    float* out_y,
    float* out_width,
    float* out_height);
bool ui_get_visible_bounds(
    ui_handle_t handle,
    float* out_x,
    float* out_y,
    float* out_width,
    float* out_height);

extern void as_on_focus_changed(ui_handle_t handle, bool is_focused);
extern void as_on_pointer_event(ui_handle_t handle, UiEvent event_enum);
extern void as_on_text_changed(ui_handle_t handle, const uint8_t* utf8_str, uint32_t len);
extern void as_on_text_replaced(
    ui_handle_t handle,
    uint32_t start_idx,
    uint32_t end_idx,
    const uint8_t* utf8_str,
    uint32_t len);
extern void as_on_scroll(
    ui_handle_t handle,
    float offset_x,
    float offset_y,
    float content_width,
    float content_height,
    float viewport_width,
    float viewport_height);
extern void as_on_selection_changed(ui_handle_t handle, uint32_t start_idx, uint32_t end_idx);
extern void as_on_cross_selection_changed(ui_handle_t area_handle, const uint8_t* utf8_str, uint32_t len);
extern void as_on_clipboard_write(
    const uint8_t* utf8_plain_text,
    uint32_t plain_text_len,
    const uint8_t* utf8_rich_json,
    uint32_t rich_json_len);
extern void as_on_request_clipboard_read(ui_handle_t handle);
extern void as_on_request_font_load(uint32_t font_id, const uint8_t* utf8_url, uint32_t len);
extern void as_on_missing_font_coverage(uint32_t font_id, uint32_t coverage_kind, const uint8_t* utf8_sample, uint32_t len);
extern void as_on_request_semantic_announcement(ui_handle_t handle);

#ifdef __cplusplus
}
#endif
