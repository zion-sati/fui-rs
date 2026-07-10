import type { BuildMode, DevToolsDomMirrorMode, PageZoomMode } from './runtime-config';
import type { DebugTreeSnapshot } from './debug-tree';
import type {
  OpenCanvasApi,
  OpenCanvasEditableTextKind,
  OpenCanvasFindMatch,
  OpenCanvasFindState,
  OpenCanvasTextDocument,
  SemanticNode,
} from './open-canvas';

export type {
  DebugTreeBehaviorFlags,
  DebugTreeBounds,
  DebugTreeFlags,
  DebugTreeInsets,
  DebugTreeNode,
  DebugTreeScrollMetrics,
  DebugTreeSnapshot,
  DebugTreeStyle,
} from './debug-tree';

export type {
  OpenCanvasApi,
  OpenCanvasAutofillHint,
  OpenCanvasEditableTextDocument,
  OpenCanvasEditableTextKind,
  OpenCanvasForm,
  OpenCanvasFormPurpose,
  OpenCanvasFindMatch,
  OpenCanvasFindOptions,
  OpenCanvasFindResults,
  OpenCanvasFindState,
  OpenCanvasHandle,
  OpenCanvasResolvedFindOptions,
  OpenCanvasTextDocument,
  SemanticBounds,
  SemanticNode,
  SemanticState,
} from './open-canvas';

export type WasmHandleLike =
  | number
  | bigint
  | string
  | {
    valueOf(): unknown;
    toString(): string;
  };

export interface AssetLoadResult {
  readonly width: number;
  readonly height: number;
}

export interface BridgeFontRegistration {
  readonly id: number;
  readonly url: string;
  readonly fallbackIds?: readonly number[];
}

export interface BridgeFontStackRegistration {
  readonly primary: BridgeFontRegistration;
  readonly fallbacks?: readonly BridgeFontRegistration[];
}

export interface CoreModule {
  HEAPU8: Uint8Array;
  HEAPU32: Uint32Array;
  wasmMemory?: WebAssembly.Memory;
  usesMemory64?: boolean;
  refreshHeapViews?(): void;
  locateFile?(path: string, prefix?: string): string;
  instantiateWasm?(
    imports: WebAssembly.Imports,
    successCallback: (instance: WebAssembly.Instance, module?: WebAssembly.Module) => void,
  ): object;
  onAbort?(what?: unknown): void;
  canvas?: HTMLCanvasElement;
  onRuntimeInitialized?: () => void;
  _malloc(size: number): WasmHandleLike;
  _free(ptr: WasmHandleLike): void;
  _ed_get_abi_version?(): number;
  _ed_init(width: number, height: number, dpr: number): void;
  _ed_init_webgl(width: number, height: number, dpr: number): void;
  _ed_init_sw(width: number, height: number, dpr: number): void;
  _ed_resize(width: number, height: number, dpr: number): void;
  _ed_set_viewport_size(logicalWidth: number, logicalHeight: number): void;
  _ed_set_viewport_transform(scale: number, offsetX: number, offsetY: number): void;
  _ed_get_viewport_scale(): number;
  _ed_get_viewport_offset_x(): number;
  _ed_get_viewport_offset_y(): number;
  _ed_set_viewport_zoom_from_scene_anchor(
    scale: number,
    anchorSceneX: number,
    anchorSceneY: number,
    screenX: number,
    screenY: number,
  ): void;
  _ed_pan_viewport_by(deltaX: number, deltaY: number): void;
  _ed_begin_viewport_pan(timestampMs: number): void;
  _ed_update_viewport_pan(deltaX: number, deltaY: number, timestampMs: number): void;
  _ed_end_viewport_pan(timestampMs: number): void;
  _ed_tick_viewport_pan_momentum(timestampMs: number): number;
  _ed_clear_viewport_pan_momentum(): void;
  _ed_register_font(fontId: number, bytesPtr: WasmHandleLike, len: number): void;
  _ed_unregister_font(fontId: number): void;
  _ed_register_svg(svgId: number, bytesPtr: WasmHandleLike, len: number): void;
  _ed_execute_command_buffer(ptr: WasmHandleLike, length: number): void;
  _ed_register_texture_rgba(
    textureId: number,
    rgbaPtr: WasmHandleLike,
    width: number,
    height: number,
    byteLength: number,
  ): void;
  _ed_register_texture_sub_rgba(
    textureId: number,
    subRgbaPtr: WasmHandleLike,
    subX: number,
    subY: number,
    subW: number,
    subH: number,
    fullW: number,
    fullH: number,
  ): void;
  _ed_unregister_texture(textureId: number): void;
  _ed_reset_scene(): void;
  _ed_render_frame(currentTimeMs: number): void;
  _ed_clear_focus_state?(): void;
  _ed_clear_text_input_state?(): void;
  _ed_recover_device(): void;
  _ed_hit_test(x: number, y: number): WasmHandleLike;
  _ed_get_sw_framebuffer(): WasmHandleLike;
  _ed_get_backend_type(): number;
  _ed_get_device_state(): number;
  _ed_notify_webgl_context_lost?(): void;
  _ed_debug_simulate_device_lost?(): void;

  /* Canvas drawing API */
  _ed_canvas_save(canvasPtr: WasmHandleLike): void;
  _ed_canvas_restore(canvasPtr: WasmHandleLike): void;
  _ed_canvas_translate(canvasPtr: WasmHandleLike, x: number, y: number): void;
  _ed_canvas_scale(canvasPtr: WasmHandleLike, sx: number, sy: number): void;
  _ed_canvas_rotate(canvasPtr: WasmHandleLike, degrees: number): void;
  _ed_canvas_clip_rect(canvasPtr: WasmHandleLike, x: number, y: number, w: number, h: number): void;
  _ed_canvas_clip_round_rect(canvasPtr: WasmHandleLike, x: number, y: number, w: number, h: number,
    topLeftRadius: number, topRightRadius: number, bottomRightRadius: number, bottomLeftRadius: number): void;
  _ed_canvas_draw_rect(canvasPtr: WasmHandleLike, x: number, y: number, w: number, h: number,
    fillColor: number, strokeColor: number, strokeWidth: number): void;
  _ed_canvas_draw_circle(canvasPtr: WasmHandleLike, cx: number, cy: number, radius: number,
    fillColor: number, strokeColor: number, strokeWidth: number): void;
  _ed_canvas_draw_line(canvasPtr: WasmHandleLike, x1: number, y1: number, x2: number, y2: number,
    color: number, strokeWidth: number): void;
  _ed_canvas_draw_round_rect(canvasPtr: WasmHandleLike, x: number, y: number, w: number, h: number,
    rx: number, ry: number, fillColor: number, strokeColor: number, strokeWidth: number): void;
  _ed_path_create(): number;
  _ed_path_destroy(pathId: number): void;
  _ed_path_move_to(pathId: number, x: number, y: number): void;
  _ed_path_line_to(pathId: number, x: number, y: number): void;
  _ed_path_quad_to(pathId: number, cx: number, cy: number, x: number, y: number): void;
  _ed_path_cubic_to(pathId: number, cx1: number, cy1: number, cx2: number, cy2: number,
    x: number, y: number): void;
  _ed_path_close(pathId: number): void;
  _ed_path_add_rect(pathId: number, x: number, y: number, w: number, h: number): void;
  _ed_path_add_circle(pathId: number, cx: number, cy: number, r: number): void;
  _ed_canvas_draw_path(canvasPtr: WasmHandleLike, pathId: number,
    fillColor: number, strokeColor: number, strokeWidth: number): void;
  _ed_canvas_draw_text_node(canvasPtr: WasmHandleLike, handleLo: number, handleHi: number, x: number, y: number): void;
  _ed_canvas_draw_image(canvasPtr: WasmHandleLike, textureId: number,
    x: number, y: number, w: number, h: number, samplingKind: number, maxAniso: number): void;
  _ed_canvas_draw_svg(canvasPtr: WasmHandleLike, svgId: number,
    x: number, y: number, w: number, h: number): void;
  _ed_canvas_draw_batch(canvasPtr: WasmHandleLike, wordsPtr: WasmHandleLike, wordCount: number): void;
  _ed_canvas_create_offscreen(width: number, height: number): number;
  _ed_canvas_get_offscreen_canvas(offscreenId: number): WasmHandleLike;
  _ed_canvas_read_offscreen_pixels(offscreenId: number, outRgbaPtr: WasmHandleLike): void;
  _ed_canvas_destroy_offscreen(offscreenId: number): void;
  _ed_render_node_to_rgba(handle: WasmHandleLike, width: number, height: number,
    outPixelsPtr: number | bigint, outCapacity: number, scale: number, x: number, y: number): number;
}

export const EdBackendType = {
  NONE: 0,
  WEBGPU: 1,
  WEBGL2: 2,
  CPU: 3,
} as const;

export type EdBackendType = (typeof EdBackendType)[keyof typeof EdBackendType];

export const EdDeviceState = {
  OK: 0,
  LOST: 1,
  RECOVERING: 2,
} as const;

export type EdDeviceState = (typeof EdDeviceState)[keyof typeof EdDeviceState];

export interface UiModule {
  HEAPU8: Uint8Array;
  HEAPU32: Uint32Array;
  HEAPF32: Float32Array;
  wasmMemory?: WebAssembly.Memory;
  usesMemory64?: boolean;
  refreshHeapViews?(): void;
  _malloc(size: number): WasmHandleLike;
  _free(ptr: WasmHandleLike): void;
  _ui_get_abi_version?(): number;
  _ui_reset(): void;
  _ui_arena_alloc(size: number): WasmHandleLike;
  _ui_register_icu_data(ptr: WasmHandleLike, len: number): void;
  _ui_create_node(type: number): WasmHandleLike;
  _ui_delete_node(handle: WasmHandleLike): void;
  _ui_node_add_child(parent: WasmHandleLike, child: WasmHandleLike): void;
  _ui_set_root(handle: WasmHandleLike): void;
  _ui_set_node_id(handle: WasmHandleLike, strPtr: WasmHandleLike, len: number): void;
  _ui_set_semantic_role(handle: WasmHandleLike, role: number): void;
  _ui_set_semantic_label(handle: WasmHandleLike, strPtr: WasmHandleLike, len: number): void;
  _ui_set_semantic_checked(handle: WasmHandleLike, checkedState: number): void;
  _ui_set_semantic_selected(handle: WasmHandleLike, hasSelected: number, selected: number): void;
  _ui_set_semantic_expanded(handle: WasmHandleLike, hasExpanded: number, expanded: number): void;
  _ui_set_semantic_disabled(handle: WasmHandleLike, hasDisabled: number, disabled: number): void;
  _ui_set_semantic_value_range(
    handle: WasmHandleLike,
    hasValueRange: number,
    valueNow: number,
    valueMin: number,
    valueMax: number,
  ): void;
  _ui_set_semantic_orientation(handle: WasmHandleLike, orientation: number): void;
  _ui_request_semantic_announcement(handle: WasmHandleLike): void;
  _ui_push_semantic_scope(handle: WasmHandleLike): number;
  _ui_remove_semantic_scope(token: number): void;
  _ui_set_width(handle: WasmHandleLike, value: number, unit: number): void;
  _ui_set_height(handle: WasmHandleLike, value: number, unit: number): void;
  _ui_set_fill_width(handle: WasmHandleLike, fill: number): void;
  _ui_set_fill_height(handle: WasmHandleLike, fill: number): void;
  _ui_set_fill_width_percent(handle: WasmHandleLike, percent: number): void;
  _ui_set_fill_height_percent(handle: WasmHandleLike, percent: number): void;
  _ui_set_min_width(handle: WasmHandleLike, value: number, unit: number): void;
  _ui_set_max_width(handle: WasmHandleLike, value: number, unit: number): void;
  _ui_set_min_height(handle: WasmHandleLike, value: number, unit: number): void;
  _ui_set_max_height(handle: WasmHandleLike, value: number, unit: number): void;
  _ui_set_flex_direction(handle: WasmHandleLike, direction: number): void;
  _ui_set_flex_basis(handle: WasmHandleLike, basis: number): void;
  _ui_set_justify_content(handle: WasmHandleLike, justify: number): void;
  _ui_set_align_items(handle: WasmHandleLike, align: number): void;
  _ui_set_align_self(handle: WasmHandleLike, align: number): void;
  _ui_set_padding(handle: WasmHandleLike, left: number, top: number, right: number, bottom: number): void;
  _ui_set_margin(handle: WasmHandleLike, left: number, top: number, right: number, bottom: number): void;
  _ui_set_position_type(handle: WasmHandleLike, positionType: number): void;
  _ui_set_position(handle: WasmHandleLike, left: number, top: number, right: number, bottom: number): void;
  _ui_node_remove_child(parent: WasmHandleLike, child: WasmHandleLike): void;
  _ui_set_is_portal(handle: WasmHandleLike, portal: number): void;
  _ui_set_is_shared_size_scope(handle: WasmHandleLike, isScope: number): void;
  _ui_set_custom_drawable(handle: WasmHandleLike, flag: number): void;
  _ui_set_flex_wrap(handle: WasmHandleLike, wrap: number): void;
  _ui_prepare_node(handle: WasmHandleLike): number;
  _ui_set_dynamic_text_charset(handle: WasmHandleLike, strPtr: WasmHandleLike, len: number): void;
  _ui_grid_set_columns(handle: WasmHandleLike, count: number, valuesPtr: WasmHandleLike, typesPtr: WasmHandleLike): void;
  _ui_grid_set_rows(handle: WasmHandleLike, count: number, valuesPtr: WasmHandleLike, typesPtr: WasmHandleLike): void;
  _ui_grid_set_column_shared_size_group(handle: WasmHandleLike, index: number, strPtr: WasmHandleLike, len: number): void;
  _ui_grid_set_row_shared_size_group(handle: WasmHandleLike, index: number, strPtr: WasmHandleLike, len: number): void;
  _ui_node_set_grid_placement(handle: WasmHandleLike, row: number, col: number, rowSpan: number, colSpan: number): void;
  _ui_set_bg_color(handle: WasmHandleLike, color: number): void;
  _ui_set_box_style(
    handle: WasmHandleLike,
    bgColor: number,
    topLeftRadius: number,
    topRightRadius: number,
    bottomRightRadius: number,
    bottomLeftRadius: number,
    borderWidth: number,
    borderColor: number,
    borderStyle: number,
    borderDashOn: number,
    borderDashOff: number,
  ): void;
  _ui_set_drop_shadow(
    handle: WasmHandleLike,
    color: number,
    offsetX: number,
    offsetY: number,
    blurSigma: number,
    spread: number,
  ): void;
  _ui_set_layer_effect(handle: WasmHandleLike, opacity: number, blurSigma: number, blendMode: number): void;
  _ui_set_background_blur(handle: WasmHandleLike, blurSigma: number): void;
  _ui_set_image(handle: WasmHandleLike, textureId: number, objectFit: number, samplingKind: number, maxAniso: number): void;
  _ui_set_image_nine(
    handle: WasmHandleLike,
    textureId: number,
    insetLeft: number,
    insetTop: number,
    insetRight: number,
    insetBottom: number,
    samplingKind: number,
    maxAniso: number,
  ): void;
  _ui_set_svg(handle: WasmHandleLike, svgId: number, tintColor: number, samplingKind: number, maxAniso: number): void;
  _ui_set_linear_gradient(
    handle: WasmHandleLike,
    startX: number,
    startY: number,
    endX: number,
    endY: number,
    stopCount: number,
    offsetsPtr: WasmHandleLike,
    colorsPtr: WasmHandleLike,
  ): void;
  _ui_set_clip_to_bounds(handle: WasmHandleLike, clip: number): void;
  _ui_set_visibility(handle: WasmHandleLike, visibility: number): void;
  _ui_set_font(handle: WasmHandleLike, fontId: number, size: number): void;
  _ui_set_line_height(handle: WasmHandleLike, lineHeight: number): void;
  _ui_set_text(handle: WasmHandleLike, strPtr: WasmHandleLike, len: number): void;
  _ui_set_text_style_runs(handle: WasmHandleLike, runCount: number, runsWordsPtr: WasmHandleLike): void;
  _ui_set_text_color(handle: WasmHandleLike, color: number): void;
  _ui_set_text_align(handle: WasmHandleLike, align: number): void;
  _ui_set_text_vertical_align(handle: WasmHandleLike, align: number): void;
  _ui_set_text_limits(handle: WasmHandleLike, maxChars: number, maxLines: number): void;
  _ui_set_text_wrapping(handle: WasmHandleLike, wrap: number): void;
  _ui_set_text_overflow(handle: WasmHandleLike, overflow: number): void;
  _ui_set_text_overflow_fade(handle: WasmHandleLike, horizontal: number, vertical: number): void;
  _ui_set_text_obscured(handle: WasmHandleLike, obscured: number): void;
  _ui_set_scroll_offset(handle: WasmHandleLike, x: number, y: number): void;
  _ui_has_pending_visual_work(): number;
  _ui_needs_animation_frame(): number;
  _ui_has_pointer_autoscroll(): number;
  _ui_selection_autoscroll(x: number, y: number, edgeThreshold: number): WasmHandleLike;
  _ui_get_bounds(
    handle: WasmHandleLike,
    outX: WasmHandleLike,
    outY: WasmHandleLike,
    outWidth: WasmHandleLike,
    outHeight: WasmHandleLike,
  ): number;
  _ui_get_visible_bounds(
    handle: WasmHandleLike,
    outX: WasmHandleLike,
    outY: WasmHandleLike,
    outWidth: WasmHandleLike,
    outHeight: WasmHandleLike,
  ): number;
  _ui_get_text_metrics(
    handle: WasmHandleLike,
    outWidth: WasmHandleLike,
    outHeight: WasmHandleLike,
    outBaseline: WasmHandleLike,
    outLineCount: WasmHandleLike,
    outMaxLineWidth: WasmHandleLike,
  ): number;
  _ui_set_selectable(handle: WasmHandleLike, selectable: number, color: number): void;
  _ui_set_selection_area(handle: WasmHandleLike, isArea: number): void;
  _ui_set_selection_area_barrier(handle: WasmHandleLike, isBarrier: number): void;
  _ui_clear_selection(handle: WasmHandleLike): void;
  _ui_retarget_selection(fromHandle: WasmHandleLike, toHandle: WasmHandleLike): void;
  _ui_is_point_in_selection(x: number, y: number): number;
  _ui_set_text_selection_range(handle: WasmHandleLike, selectionStart: number, selectionEnd: number): void;
  _ui_select_word_at(this: void, handle: WasmHandleLike, x: number, y: number): number;
  _ui_begin_selection_endpoint_drag(this: void, handle: WasmHandleLike, endpoint: number): number;
  _ui_preserves_selection_on_pointer_down?(this: void, handle: WasmHandleLike): number;
  _ui_get_text_snapshot_handle_count(): number;
  _ui_copy_text_snapshot_handles(outPtr: WasmHandleLike, maxHandleCount: number): number;
  _ui_set_text_find_match(handle: WasmHandleLike, start: number, end: number): number;
  _ui_clear_text_find_match(): void;
  _ui_push_text_find_highlight(handle: WasmHandleLike, start: number, end: number, color: number): number;
  _ui_clear_text_find_highlights(): void;
  _ui_get_text_document_utf8_length(handle: WasmHandleLike): number;
  _ui_copy_text_document_utf8(handle: WasmHandleLike, outPtr: WasmHandleLike, bufferLength: number): number;
  _ui_get_text_visible_bounds(
    handle: WasmHandleLike,
    outXPtr: WasmHandleLike,
    outYPtr: WasmHandleLike,
    outWidthPtr: WasmHandleLike,
    outHeightPtr: WasmHandleLike,
  ): number;
  _ui_get_text_range_rect_count(handle: WasmHandleLike, start: number, end: number): number;
  _ui_copy_text_range_rects(
    handle: WasmHandleLike,
    start: number,
    end: number,
    outPtr: WasmHandleLike,
    maxRectCount: number,
  ): number;
  _ui_copy_cross_selection_endpoint_rects(this: void, areaHandle: WasmHandleLike, outPtr: WasmHandleLike): number;
  _ui_reveal_text_range(handle: WasmHandleLike, start: number, end: number): number;
  _ui_clear_current_selection(): void;
  _ui_copy_current_selection(): void;
  _ui_can_undo_text_edit(handle: WasmHandleLike): number;
  _ui_can_redo_text_edit(handle: WasmHandleLike): number;
  _ui_has_text_selection(handle: WasmHandleLike): number;
  _ui_undo_text_edit(handle: WasmHandleLike): void;
  _ui_redo_text_edit(handle: WasmHandleLike): void;
  _ui_copy_text_selection(handle: WasmHandleLike): void;
  _ui_cut_text_selection(handle: WasmHandleLike): void;
  _ui_paste_text(handle: WasmHandleLike): void;
  _ui_select_all_text(handle: WasmHandleLike): void;
  _ui_set_interactive(handle: WasmHandleLike, interactive: number): void;
  _ui_set_preserve_selection_on_pointer_down(this: void, handle: WasmHandleLike, preserve: number): void;
  _ui_set_editor_command_keys(this: void, handle: WasmHandleLike, enabled: number): void;
  _ui_set_editor_accepts_tab(this: void, handle: WasmHandleLike, enabled: number): void;
  _ui_set_scroll_proxy_target(handle: WasmHandleLike, scrollHandle: WasmHandleLike): void;
  _ui_set_scroll_enabled(handle: WasmHandleLike, enabledX: number, enabledY: number): void;
  _ui_set_show_scrollbars(handle: WasmHandleLike, showScrollbars: number): void;
  _ui_set_scroll_friction(handle: WasmHandleLike, friction: number): void;
  _ui_set_smooth_scrolling(handle: WasmHandleLike, smoothScrolling: number): void;
  _ui_set_scroll_content_size(handle: WasmHandleLike, contentWidth: number, contentHeight: number): void;
  _ui_set_editable(handle: WasmHandleLike, editable: number): void;
  _ui_set_caret_color(handle: WasmHandleLike, color: number): void;
  _ui_set_focusable(handle: WasmHandleLike, focusable: number, tabIndex: number): void;
  _ui_request_focus(handle: WasmHandleLike): void;
  _ui_commit_frame(timestampMs?: number): void;
  _ui_get_command_buffer(outLenPtr: WasmHandleLike): WasmHandleLike;
  _ui_get_semantic_buffer(outLenPtr: WasmHandleLike): WasmHandleLike;
  _ui_get_debug_tree_buffer(outLenPtr: WasmHandleLike): WasmHandleLike;
  _ui_get_live_fallback_font_buffer(outLenPtr: WasmHandleLike): WasmHandleLike;
  _ui_resize_window(w: number, h: number): void;
  _ui_on_pointer_event(
    event: number,
    handle: WasmHandleLike,
    x: number,
    y: number,
    pointerId: number,
    pointerType: number,
    button: number,
    buttons: number,
    pressure: number,
    width: number,
    height: number,
    clickCount: number,
    modifiers: number,
  ): void;
  _ui_on_wheel_event(deltaX: number, deltaY: number): void;
  _ui_touch_scroll_begin(handle: WasmHandleLike, x: number, y: number, timestampMs?: number): void;
  _ui_touch_scroll_update(deltaX: number, deltaY: number, timestampMs?: number): void;
  _ui_wheel_scroll_can_consume(deltaX: number, deltaY: number): number;
  _ui_touch_scroll_can_consume(deltaX: number, deltaY: number): number;
  _ui_touch_scroll_end(timestampMs?: number): void;
  _ui_clear_momentum_scroll(): void;
  _ui_touch_scroll_allows_pull_to_refresh(): number;
  _ui_set_coarse_pointer_mode(coarsePointerMode: number): void;
  _ui_set_platform_family(platformFamily: number): void;
  _ui_on_key_event(type: number, strPtr: WasmHandleLike, len: number, mods: number): number;
  _ui_on_ime_update(handle: WasmHandleLike, strPtr: WasmHandleLike, len: number, caretIdx: number): void;
  _ui_replace_text_range(
    handle: WasmHandleLike,
    startIdx: number,
    endIdx: number,
    strPtr: WasmHandleLike,
    len: number,
    caretIdx: number,
  ): void;
  _ui_on_paste_text(handle: WasmHandleLike, strPtr: WasmHandleLike, len: number): void;
  _ui_set_interaction_time(ms: WasmHandleLike): void;
  _ui_measure_text(
    strPtr: WasmHandleLike,
    len: number,
    fontId: number,
    size: number,
    maxWidth: number,
    outWidthPtr: WasmHandleLike,
    outHeightPtr: WasmHandleLike,
  ): void;
  _ui_register_font(id: number, bytesPtr: WasmHandleLike, len: number): number;
  _ui_register_font_fallback(fontId: number, fallbackFontId: number): void;
  _ui_unregister_font_fallback(fontId: number, fallbackFontId: number): number;
  _ui_unregister_font(fontId: number): number;
  _ui_font_loaded(fontId: number): void;
}

export interface PointerEventLog {
  readonly handle: string;
  readonly eventType: number;
  readonly x?: number;
  readonly y?: number;
  readonly modifiers?: number;
  readonly pointerId?: number;
  readonly pointerType?: number;
  readonly button?: number;
  readonly buttons?: number;
  readonly pressure?: number;
  readonly width?: number;
  readonly height?: number;
  readonly clickCount?: number;
}

export interface PendingPointerMetadata {
  readonly eventType: number;
  readonly handle: WasmHandleLike;
  readonly x: number;
  readonly y: number;
  readonly modifiers: number;
  readonly pointerId: number;
  readonly pointerType: number;
  readonly button: number;
  readonly buttons: number;
  readonly pressure: number;
  readonly width: number;
  readonly height: number;
  readonly clickCount: number;
}

export interface FocusEventLog {
  readonly handle: string;
  readonly isFocused: boolean;
}

export interface TextChangeLog {
  readonly handle: string;
  readonly text: string;
  readonly textLength?: number;
  readonly truncated?: boolean;
}

export interface SelectionChangeLog {
  readonly handle: string;
  readonly start: number;
  readonly end: number;
}

export interface CrossSelectionChangeLog {
  readonly areaHandle: string;
  readonly text: string;
}

export interface ScrollEventLog {
  readonly handle: string;
  readonly offsetX: number;
  readonly offsetY: number;
  readonly contentWidth: number;
  readonly contentHeight: number;
  readonly viewportWidth: number;
  readonly viewportHeight: number;
}

export interface MissingFontCoverageLog {
  readonly fontId: number;
  readonly coverageKind: number;
  readonly sampleText: string;
}

export interface IncrementalFontPackageRequestLog {
  readonly primaryFontId: number;
  readonly coverageKind: number;
  readonly packageId: string;
  readonly segmentIds: readonly string[];
  readonly sampleText: string;
}

export type IncrementalFontAutoGrowBlockReason =
  | 'auto-grow-disabled'
  | 'font-not-allowed'
  | 'package-blocked';

export interface IncrementalFontPolicy {
  readonly autoGrow: boolean;
  readonly maxCachedShardFonts: number;
  readonly allowedFontIds: readonly number[] | null;
  readonly blockedPackageIds: readonly string[] | null;
}

export interface IncrementalFontCacheState {
  readonly maxCachedShardFonts: number;
  readonly cachedShardCount: number;
  readonly cachedShardKeys: readonly string[];
  readonly evictedShardKeys: readonly string[];
}

export interface IncrementalFontRuntimeState {
  readonly fontId: number;
  readonly sourceUrl: string | null;
  readonly sourceState: 'unknown' | 'known' | 'loading' | 'loaded' | 'failed';
  readonly loaded: boolean;
  readonly requestedSegmentIds: readonly string[];
  readonly pendingSegmentIds: readonly string[];
  readonly appliedSegmentIds: readonly string[];
  readonly evictedSegmentIds: readonly string[];
  readonly revision: number;
  readonly autoGrowAllowed: boolean;
  readonly blockedPackageIds: readonly string[];
  readonly lastBlockedReason: IncrementalFontAutoGrowBlockReason | null;
}

export interface ClipboardRichTextPart {
  readonly text: string;
  readonly fontId?: number;
  readonly fontSize?: number;
  readonly color?: number;
  readonly bgColor?: number;
  readonly decorationFlags?: number;
  readonly fontUrl?: string;
}

export interface ClipboardRichTextPayload {
  readonly version: 1;
  readonly parts: readonly ClipboardRichTextPart[];
}

export interface ClipboardWritePayload {
  readonly plainText: string;
  readonly richText?: ClipboardRichTextPayload;
}

export interface BridgeLogs {
  readonly pointerEvents: PointerEventLog[];
  readonly focusEvents: FocusEventLog[];
  readonly textChanges: TextChangeLog[];
  readonly selectionChanges: SelectionChangeLog[];
  readonly crossSelectionChanges: CrossSelectionChangeLog[];
  readonly clipboardWrites: string[];
  readonly clipboardReadRequests: string[];
  readonly scrollEvents: ScrollEventLog[];
  readonly missingFontCoverageRequests: MissingFontCoverageLog[];
  readonly incrementalFontPackageRequests: IncrementalFontPackageRequestLog[];
}

export interface BridgeRuntime {
  readonly core: CoreModule;
  readonly ui: UiModule;
  readonly canvas: HTMLCanvasElement;
  readonly buildMode: BuildMode;
  readonly devToolsDomMirrorMode: DevToolsDomMirrorMode;
  readonly pageZoomMode: PageZoomMode;
  readonly devTools: BridgeDevToolsApi;
  readonly openCanvasApi: OpenCanvasApi;
  readonly logs: BridgeLogs;
  updateCanvasSize(): void;
  extractCommandBuffer(): Uint32Array;
  executeCommandBuffer(words: Uint32Array): void;
  syncCommandBufferToCore(): Uint32Array;
  flushPendingCommit(): Uint32Array | null;
  hasPendingCommit(): boolean;
  commitFrame(timestampMs?: number): void;
  requestFrame(): void;
  setFrameRequester(requester: (() => void) | null): void;
  getSemanticTree(): readonly SemanticNode[];
  getDebugTree(): DebugTreeSnapshot;
  setTextInputMetadata(
    handle: string,
    metadata: {
      readonly kind: OpenCanvasEditableTextKind;
      readonly hostAutofillHint: string | null;
    },
  ): void;
  getTextInputMetadata(handle: string): {
    readonly kind: OpenCanvasEditableTextKind;
    readonly hostAutofillHint: string | null;
  } | null;
  clearTextInputMetadata(): void;
  getActiveTextHandle(): bigint | null;
  getCapturedPointerHandle(): bigint | null;
  setCapturedPointerHandle(handle: bigint | null): void;
  getPageZoom(): { readonly scale: number; readonly offsetX: number; readonly offsetY: number };
  isPageZoomEnabled(): boolean;
  setPageZoom(scale: number, offsetX: number, offsetY: number): void;
  setPageZoomFromSceneAnchor(
    scale: number,
    anchorSceneX: number,
    anchorSceneY: number,
    screenX: number,
    screenY: number,
  ): { readonly scale: number; readonly offsetX: number; readonly offsetY: number };
  panPageZoomBy(deltaX: number, deltaY: number): { readonly scale: number; readonly offsetX: number; readonly offsetY: number };
  beginPageZoomPan(timestampMs: number): void;
  updatePageZoomPan(deltaX: number, deltaY: number, timestampMs: number): { readonly scale: number; readonly offsetX: number; readonly offsetY: number };
  endPageZoomPan(timestampMs: number): void;
  clearPageZoomPanMomentum(): void;
  resetPageZoom(): void;
  screenToScenePoint(x: number, y: number): { readonly x: number; readonly y: number };
  setAppFrameHandler(handler: ((timestampMs: number) => void) | null): void;
  runAppFrameHandler(timestampMs: number): void;
  uiHasPendingVisualWork(): boolean;
  uiNeedsAnimationFrame(): boolean;
  getHandleFromPoint(x: number, y: number): bigint;
  clearPointerHover(): void;
  refreshPointerHover(): void;
  getFindDocuments(): readonly OpenCanvasTextDocument[];
  activateFindMatch(match: OpenCanvasFindMatch | null, reveal?: boolean): boolean;
  syncFindSelection(clearOnMissing?: boolean): boolean;
  clearFindMatch(): boolean;
  ensureFont(fontId: number): Promise<void>;
  ensureBuiltInFont(fontId: number): Promise<void>;
  isFontLoaded(fontId: number, url?: string): boolean;
  getIncrementalFontState(fontId: number): IncrementalFontRuntimeState | null;
  getIncrementalFontCacheState(): IncrementalFontCacheState;
  getIncrementalFontPolicy(): IncrementalFontPolicy;
  setIncrementalFontPolicy(policy: Partial<IncrementalFontPolicy>): void;
  getClipboardFontUrl(fontId: number): string | null;
  registerLazyFont(fontId: number, url: string): void;
  registerFontFallback(fontId: number, fallbackFontId: number): void;
  handleMissingFontCoverage(fontId: number, coverageKind: number, sampleText: string): void;
  loadFont(fontId: number, url: string): Promise<void>;
  registerFont(font: BridgeFontRegistration): Promise<void>;
  registerFontStack(stack: BridgeFontStackRegistration): Promise<void>;
  loadSvg(svgId: number, url: string): Promise<AssetLoadResult>;
  loadTexture(textureId: number, url: string): Promise<AssetLoadResult>;
  releaseSvg(svgId: number): void;
  releaseTexture(textureId: number): void;
  replayLoadedAssets(): Promise<void>;
  resetLogs(): void;
  resetAppSession(): void;
}

export interface BridgeDevToolsApi {
  enableDomMirror(): boolean;
  disableDomMirror(): boolean;
  toggleDomMirror(): boolean;
  isDomMirrorEnabled(): boolean;
  selectHandle(handle: WasmHandleLike): boolean;
  clearSelection(): void;
  getSelectedHandle(): string | null;
  openDebugDialog(): boolean;
  closeDebugDialog(): boolean;
  toggleDebugDialog(): boolean;
  isDebugDialogOpen(): boolean;
}

export interface BridgeState {
  readonly ready: Promise<BridgeRuntime>;
  readonly devTools: BridgeDevToolsApi;
  getRuntime(): BridgeRuntime | null;
  recreateRuntime(): Promise<BridgeRuntime>;
  resetLogs(): void;
  getPageZoom(): { readonly scale: number; readonly offsetX: number; readonly offsetY: number };
  setPageZoom(scale: number, offsetX?: number, offsetY?: number): void;
  resetPageZoom(): void;
  handleToBigInt(handle: WasmHandleLike): bigint;
  handleToString(handle: WasmHandleLike): string;
  pointerToHeapOffset(pointer: WasmHandleLike): number;
  normalizePointerForWasm(
    module: Pick<UiModule | CoreModule, 'usesMemory64'>,
    pointer: WasmHandleLike,
  ): number | bigint;
  toHeapPointer(
    module: Pick<UiModule | CoreModule, 'usesMemory64'>,
    pointer: WasmHandleLike,
  ): { readonly ptr: number | bigint; readonly offset: number };
}

export interface BridgeLoaderInfo {
  manifestHash: string | null;
  requestedWasmArchitecture: string;
  requestedRendererBackend: 'auto' | 'webgpu' | 'webgl2' | 'cpu';
  selectedWasmArchitecture: string;
  availableWasmArchitectures: readonly string[];
  memory64Supported: boolean;
  simdSupported: boolean;
  coreCompileMode: 'streaming' | 'buffer' | 'cached-module';
  uiCompileMode: 'streaming' | 'buffer' | 'cached-module';
  icuDataUrl: string | null;
  activeRenderer: 'none' | 'webgpu' | 'webgl2' | 'cpu';
  /** Incremented each time the renderer recovers from a device loss. Useful for tests. */
  deviceRecoveryCount: number;
}

export interface EffinDomCallbacks {
  onPointerEvent?: (handle: WasmHandleLike, eventType: number) => void;
  onPointerEventWithCoords?: (eventType: number, handle: WasmHandleLike, x: number, y: number, modifiers?: number) => void;
  onPointerEventWithMetadata?: (
    eventType: number,
    handle: WasmHandleLike,
    x: number,
    y: number,
    modifiers: number,
    pointerId: number,
    pointerType: number,
    button: number,
    buttons: number,
    pressure: number,
    width: number,
    height: number,
    clickCount: number,
  ) => boolean | undefined;
  onWheelEventWithCoords?: (
    handle: WasmHandleLike,
    x: number,
    y: number,
    deltaX: number,
    deltaY: number,
    deltaMode: number,
    modifiers: number,
  ) => boolean | undefined;
  resolveGestureOwner?: (handle: WasmHandleLike) => WasmHandleLike | null | undefined;
  getGestureIntent?: (handle: WasmHandleLike) => number | undefined;
  onGestureEventWithCoords?: (
    handle: WasmHandleLike,
    phase: number,
    kind: number,
    x: number,
    y: number,
    deltaX: number,
    deltaY: number,
    scale: number,
    pointerCount: number,
  ) => boolean | undefined;
  resolveLongPressOwner?: (handle: WasmHandleLike) => WasmHandleLike | null | undefined;
  getLongPressMinimumDurationMs?: (handle: WasmHandleLike) => number | undefined;
  getLongPressMovementTolerance?: (handle: WasmHandleLike) => number | undefined;
  onLongPressEventWithCoords?: (
    handle: WasmHandleLike,
    x: number,
    y: number,
    pointerId: number,
    pointerType: number,
    modifiers: number,
    durationMs: number,
  ) => boolean | undefined;
  onBeforeContextMenuHitTest?: () => void;
  onContextMenu?: (handle: WasmHandleLike, x: number, y: number) => void;
  canShowContextMenu?: (handle: WasmHandleLike) => boolean | undefined;
  onKeyEventWithKey?: (eventType: number, key: string, modifiers: number) => boolean | undefined;
  onFocusChanged?: (handle: WasmHandleLike, isFocused: boolean) => void;
  onTextChanged?: (handle: WasmHandleLike, text: string) => void;
  onTextReplaced?: (handle: WasmHandleLike, start: number, end: number, text: string) => void;
  onSelectionChanged?: (handle: WasmHandleLike, start: number, end: number) => void;
  onScroll?: (
    handle: WasmHandleLike,
    offsetX: number,
    offsetY: number,
    contentWidth: number,
    contentHeight: number,
    viewportWidth: number,
    viewportHeight: number,
  ) => void;
  onClipboardWrite?: (payload: ClipboardWritePayload) => void;
  onClipboardRead?: (handle: WasmHandleLike) => void;
  onCrossSelectionChanged?: (areaHandle: WasmHandleLike, text: string) => void;
  onRequestFontLoad?: (fontId: number, url: string) => void;
  onMissingFontCoverage?: (fontId: number, coverageKind: number, sampleText: string) => void;
  onRequestSemanticAnnouncement?: (handle: WasmHandleLike) => void;
  onFontLoaded?: (fontId: number) => void;
}

export type UiFactory = (module?: object) => Promise<UiModule>;

declare global {
  interface Window {
    Module?: CoreModule;
    EffinDomUiV2ModuleFactory?: UiFactory;
    __effindomCallbacks?: EffinDomCallbacks;
    __bridgeReady?: boolean;
    __bridgeError?: string;
    __bridgeState?: {
      readonly commandWordCount: number;
      readonly commandWords: readonly number[];
      readonly rootHandle: string;
    };
    __bridgeLogs?: BridgeLogs;
    __bridgeTextByHandle?: Record<string, string>;
    __bridgeSelectionsByHandle?: Record<string, { start: number; end: number }>;
    __bridgeActiveEditorWindow?: {
      readonly handle: string | null;
      readonly text: string;
      readonly docStart: number;
      readonly docEnd: number;
    };
    __bridgeFindMatch?: OpenCanvasFindMatch | null;
    __bridgeFindState?: OpenCanvasFindState | null;
    __bridgeSemanticTree?: readonly SemanticNode[];
    __bridgeLoaderInfo?: BridgeLoaderInfo;
    __effindomPendingPointerMetadata?: PendingPointerMetadata;
    __effindomLastPointerEventHandled?: boolean;
    __OPEN_CANVAS_API__?: OpenCanvasApi;
    EffinDomBrowserBridge?: BridgeState;
    __bridgeDebug?: { forceDeviceLost(): void };
  }
}

export {};
