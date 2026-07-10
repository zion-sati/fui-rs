#pragma once

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef uint64_t ed_handle_t;
typedef uintptr_t ed_ptr_t;

enum {
    ED_INVALID_HANDLE = 0,
    ED_ABI_VERSION = 2
};

typedef enum EdCommand {
    CMD_CREATE_NODE = 1,
    CMD_DELETE_NODE = 2,

    CMD_SET_BOUNDS = 10,

    CMD_SET_BOX_STYLE = 20,
    CMD_SET_LAYER_EFFECT = 21,
    CMD_SET_LINEAR_GRADIENT = 22,
    CMD_SET_BACKGROUND_BLUR = 23,
    CMD_SET_DROP_SHADOW = 24,

    CMD_SET_IMAGE = 30,
    CMD_SET_IMAGE_NINE = 31,
    CMD_SET_PATH = 32,
    CMD_SET_SVG = 33,

    CMD_SET_GLYPH_RUN = 40,
    CMD_SET_TEXT_FADE = 41,
    CMD_SET_CARET = 42,
    CMD_SET_HIGHLIGHTS = 43,
    CMD_SET_GLYPH_RUN_COLORED = 44,
    CMD_SET_HIGHLIGHTS_COLORED = 45,
    CMD_SET_GLYPH_RUN_STYLED = 46,

    CMD_COMMIT_PAINT_ORDER = 98,
    CMD_COMMIT_SCENE = 99
} EdCommand;

typedef enum SceneOpcode {
    OP_DRAW_NODE = 1,
    OP_PUSH_CLIP = 2,
    OP_PUSH_LAYER = 3,
    OP_POP = 4,
    OP_PUSH_TRANSLATE = 5,
    OP_DRAW_CUSTOM = 6
} SceneOpcode;

typedef enum EdClipMode {
    ED_CLIP_MODE_RASTER_SAFE_VISUAL = 0,
    ED_CLIP_MODE_STRICT_CONTENT = 1
} EdClipMode;

enum {
    ED_BOUNDS_FLAG_INTERACTIVE = 1 << 0,
    ED_BOUNDS_CLIP_MODE_SHIFT = 1,
    ED_BOUNDS_CLIP_MODE_MASK = 0x3 << ED_BOUNDS_CLIP_MODE_SHIFT
};

typedef enum EdBorderStyle {
    ED_BORDER_SOLID = 0,
    ED_BORDER_DASHED = 1,
    ED_BORDER_DOTTED = 2
} EdBorderStyle;

typedef enum EdObjectFit {
    ED_OBJECT_FIT_FILL = 0,
    ED_OBJECT_FIT_CONTAIN = 1,
    ED_OBJECT_FIT_COVER = 2,
    ED_OBJECT_FIT_NONE = 3,
    ED_OBJECT_FIT_SCALE_DOWN = 4
} EdObjectFit;

typedef enum EdImageSampling {
    ED_IMAGE_SAMPLING_LINEAR = 0,
    ED_IMAGE_SAMPLING_NEAREST = 1,
    ED_IMAGE_SAMPLING_LINEAR_MIPMAP_NEAREST = 2,
    ED_IMAGE_SAMPLING_LINEAR_MIPMAP_LINEAR = 3,
    ED_IMAGE_SAMPLING_CUBIC_MITCHELL = 4,
    ED_IMAGE_SAMPLING_CUBIC_CATMULL_ROM = 5,
    ED_IMAGE_SAMPLING_ANISOTROPIC = 6
} EdImageSampling;

typedef enum EdBlendMode {
    ED_BLEND_SRC_OVER = 0,
    ED_BLEND_MULTIPLY = 1,
    ED_BLEND_SCREEN = 2,
    ED_BLEND_OVERLAY = 3,
    ED_BLEND_DARKEN = 4,
    ED_BLEND_LIGHTEN = 5
} EdBlendMode;

typedef enum EdPathVerb {
    ED_PATH_MOVE_TO = 0,
    ED_PATH_LINE_TO = 1,
    ED_PATH_QUAD_TO = 2,
    ED_PATH_CUBIC_TO = 3,
    ED_PATH_CLOSE = 4
} EdPathVerb;

typedef enum EdFadeEdge {
    ED_FADE_NONE = 0,
    ED_FADE_LEFT = 1 << 0,
    ED_FADE_TOP = 1 << 1,
    ED_FADE_RIGHT = 1 << 2,
    ED_FADE_BOTTOM = 1 << 3
} EdFadeEdge;

enum {
    ED_FADE_ALL_MASK = ED_FADE_LEFT | ED_FADE_TOP | ED_FADE_RIGHT | ED_FADE_BOTTOM
};

typedef enum EdBackendType {
    ED_BACKEND_NONE = 0,
    ED_BACKEND_WEBGPU = 1,
    ED_BACKEND_WEBGL2 = 2,
    ED_BACKEND_CPU = 3
} EdBackendType;

typedef enum EdDeviceState {
    ED_DEVICE_OK = 0,
    ED_DEVICE_LOST = 1,
    ED_DEVICE_RECOVERING = 2
} EdDeviceState;

uint32_t ed_get_abi_version(void);

void ed_init(uint32_t physical_w, uint32_t physical_h, float dpr);
void ed_init_webgl(uint32_t physical_w, uint32_t physical_h, float dpr);
void ed_init_sw(uint32_t physical_w, uint32_t physical_h, float dpr);
void ed_resize(uint32_t physical_w, uint32_t physical_h, float dpr);
void ed_set_viewport_size(float logical_w, float logical_h);
void ed_set_viewport_transform(float scale, float offset_x, float offset_y);
float ed_get_viewport_scale(void);
float ed_get_viewport_offset_x(void);
float ed_get_viewport_offset_y(void);
void ed_set_viewport_zoom_from_scene_anchor(float scale, float anchor_scene_x, float anchor_scene_y, float screen_x, float screen_y);
void ed_pan_viewport_by(float delta_x, float delta_y);
void ed_begin_viewport_pan(double timestamp_ms);
void ed_update_viewport_pan(float delta_x, float delta_y, double timestamp_ms);
void ed_end_viewport_pan(double timestamp_ms);
bool ed_tick_viewport_pan_momentum(double timestamp_ms);
void ed_clear_viewport_pan_momentum(void);

void ed_register_font(uint32_t font_id, const uint8_t* bytes, uint32_t len);
void ed_unregister_font(uint32_t font_id);
void ed_register_svg(uint32_t svg_id, const uint8_t* bytes, uint32_t len);
void ed_register_texture_rgba(uint32_t texture_id, const uint8_t* rgba, uint32_t w, uint32_t h, uint32_t byte_length);
void ed_register_texture_sub_rgba(uint32_t texture_id, const uint8_t* sub_rgba, uint32_t sub_x, uint32_t sub_y, uint32_t sub_w, uint32_t sub_h, uint32_t full_w, uint32_t full_h);
void ed_unregister_texture(uint32_t texture_id);

void ed_execute_command_buffer(const uint32_t* buffer, uint32_t length);
void ed_render_frame(double current_time_ms);
void ed_recover_device(void);

uint64_t ed_hit_test(float logical_x, float logical_y);
ed_ptr_t ed_get_sw_framebuffer(void);
EdBackendType ed_get_backend_type(void);
EdDeviceState ed_get_device_state(void);

/* Debug / test only – simulates a device loss without destroying the GPU context. */
void ed_debug_simulate_device_lost(void);

/*
 * Immediate-mode canvas drawing API.
 *
 * These functions operate on an opaque canvas pointer obtained during
 * the render pass (OP_DRAW_CUSTOM callback) or from an offscreen surface.
 * The canvas pointer must NOT be dereferenced outside of these functions.
 *
 * Color values are 0xRRGGBBAA packed into uint32_t (matching the Tier 3 Color type).
 * A fill_color or stroke_color of 0 means “no fill” / “no stroke.”
 */

/* ── Canvas state ─────────────────────────────────────────────── */

void ed_canvas_save(void* canvas);
void ed_canvas_restore(void* canvas);
void ed_canvas_translate(void* canvas, float x, float y);
void ed_canvas_scale(void* canvas, float sx, float sy);
void ed_canvas_rotate(void* canvas, float degrees);
void ed_canvas_clip_rect(void* canvas, float x, float y, float w, float h);
void ed_canvas_clip_round_rect(void* canvas, float x, float y, float w, float h,
                               float top_left, float top_right, float bottom_right, float bottom_left);

/* ── Drawing primitives ───────────────────────────────────────── */

void ed_canvas_draw_rect(void* canvas, float x, float y, float w, float h,
                         uint32_t fill_color, uint32_t stroke_color, float stroke_width);

void ed_canvas_draw_circle(void* canvas, float cx, float cy, float radius,
                           uint32_t fill_color, uint32_t stroke_color, float stroke_width);

void ed_canvas_draw_line(void* canvas, float x1, float y1, float x2, float y2,
                         uint32_t color, float stroke_width);

void ed_canvas_draw_round_rect(void* canvas, float x, float y, float w, float h,
                               float rx, float ry,
                               uint32_t fill_color, uint32_t stroke_color, float stroke_width);

/* ── Path API ──────────────────────────────────────────────────── */

uint32_t ed_path_create(void);
void ed_path_destroy(uint32_t path_id);

void ed_path_move_to(uint32_t path_id, float x, float y);
void ed_path_line_to(uint32_t path_id, float x, float y);
void ed_path_quad_to(uint32_t path_id, float cx, float cy, float x, float y);
void ed_path_cubic_to(uint32_t path_id, float cx1, float cy1, float cx2, float cy2, float x, float y);
void ed_path_close(uint32_t path_id);
void ed_path_add_rect(uint32_t path_id, float x, float y, float w, float h);
void ed_path_add_circle(uint32_t path_id, float cx, float cy, float r);

void ed_canvas_draw_path(void* canvas, uint32_t path_id,
                         uint32_t fill_color, uint32_t stroke_color, float stroke_width);

/* ── Text ──────────────────────────────────────────────────────── */

void ed_canvas_draw_text_node(void* canvas, uint32_t handle_lo, uint32_t handle_hi, float x, float y);

/* ── Image / SVG ───────────────────────────────────────────────── */

void ed_canvas_draw_image(void* canvas, uint32_t texture_id,
                          float x, float y, float w, float h,
                          uint32_t sampling_kind, uint32_t max_aniso);

void ed_canvas_draw_svg(void* canvas, uint32_t svg_id,
                        float x, float y, float w, float h);

void ed_canvas_draw_batch(void* canvas, const uint32_t* words, uint32_t word_count);

/* ── Offscreen surfaces ────────────────────────────────────────── */

uint32_t ed_canvas_create_offscreen(uint32_t width, uint32_t height);
void*    ed_canvas_get_offscreen_canvas(uint32_t offscreen_id);
void     ed_canvas_read_offscreen_pixels(uint32_t offscreen_id, uint8_t* out_rgba);
void     ed_canvas_destroy_offscreen(uint32_t offscreen_id);

#ifdef __cplusplus
}
#endif
