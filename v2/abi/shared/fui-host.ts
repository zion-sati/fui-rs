export type AbiScalarType = "void" | "bool" | "i32" | "u32" | "u64" | "f32" | "usize";

export interface AbiParam {
  readonly name: string;
  readonly type: AbiScalarType;
}

export interface FuiHostImport {
  readonly name: string;
  readonly importName: string;
  readonly args: readonly AbiParam[];
  readonly returns: AbiScalarType;
}

export interface FuiHostEnumMember {
  readonly name: string;
  readonly value: string;
}

export interface FuiHostEnum {
  readonly name: string;
  readonly members: readonly FuiHostEnumMember[];
}

export const fuiHostEnums = [
  {
    name: "FuiCursorStyle",
    members: [
      { name: "FUI_CURSOR_DEFAULT", value: "0" },
      { name: "FUI_CURSOR_POINTER", value: "1" },
      { name: "FUI_CURSOR_TEXT", value: "2" },
      { name: "FUI_CURSOR_MOVE", value: "3" },
      { name: "FUI_CURSOR_GRAB", value: "4" },
      { name: "FUI_CURSOR_GRABBING", value: "5" },
      { name: "FUI_CURSOR_RESIZE_NS", value: "6" },
      { name: "FUI_CURSOR_RESIZE_EW", value: "7" },
    ],
  },
  {
    name: "FuiPlatformFamily",
    members: [
      { name: "FUI_PLATFORM_UNKNOWN", value: "0" },
      { name: "FUI_PLATFORM_APPLE", value: "1" },
      { name: "FUI_PLATFORM_WINDOWS", value: "2" },
      { name: "FUI_PLATFORM_LINUX", value: "3" },
    ],
  },
] as const satisfies readonly FuiHostEnum[];

export const fuiHostImports = [
  {
    name: "request_render",
    importName: "request_render",
    args: [],
    returns: "void",
  },
  {
    name: "get_viewport_width",
    importName: "get_viewport_width",
    args: [],
    returns: "f32",
  },
  {
    name: "get_viewport_height",
    importName: "get_viewport_height",
    args: [],
    returns: "f32",
  },
  {
    name: "get_device_pixel_ratio",
    importName: "get_device_pixel_ratio",
    args: [],
    returns: "f32",
  },
  {
    name: "fui_set_pointer_capture",
    importName: "fui_set_pointer_capture",
    args: [{ name: "handle", type: "u64" }],
    returns: "void",
  },
  {
    name: "fui_release_pointer_capture",
    importName: "fui_release_pointer_capture",
    args: [],
    returns: "void",
  },
  {
    name: "fui_reload_page",
    importName: "fui_reload_page",
    args: [],
    returns: "void",
  },
  {
    name: "fui_can_navigate_back",
    importName: "fui_can_navigate_back",
    args: [],
    returns: "bool",
  },
  {
    name: "fui_can_navigate_forward",
    importName: "fui_can_navigate_forward",
    args: [],
    returns: "bool",
  },
  {
    name: "fui_navigate_back",
    importName: "fui_navigate_back",
    args: [],
    returns: "void",
  },
  {
    name: "fui_navigate_forward",
    importName: "fui_navigate_forward",
    args: [],
    returns: "void",
  },
  {
    name: "fui_copy_text",
    importName: "fui_copy_text",
    args: [{ name: "ptr", type: "usize" }, { name: "len", type: "u32" }],
    returns: "void",
  },
  {
    name: "fui_register_text_input_metadata",
    importName: "fui_register_text_input_metadata",
    args: [
      { name: "handle", type: "u64" },
      { name: "isPassword", type: "bool" },
      { name: "hintPtr", type: "usize" },
      { name: "hintLen", type: "u32" },
    ],
    returns: "void",
  },
  {
    name: "fui_has_text_selection_snapshot",
    importName: "fui_has_text_selection_snapshot",
    args: [{ name: "handle", type: "u64" }],
    returns: "bool",
  },
  {
    name: "fui_freeze_text_selection_snapshot",
    importName: "fui_freeze_text_selection_snapshot",
    args: [{ name: "handle", type: "u64" }],
    returns: "void",
  },
  {
    name: "fui_copy_text_selection_snapshot",
    importName: "fui_copy_text_selection_snapshot",
    args: [{ name: "handle", type: "u64" }],
    returns: "bool",
  },
  {
    name: "fui_cut_focused_text_selection",
    importName: "fui_cut_focused_text_selection",
    args: [],
    returns: "bool",
  },
  {
    name: "fui_cut_text_selection_snapshot",
    importName: "fui_cut_text_selection_snapshot",
    args: [{ name: "handle", type: "u64" }],
    returns: "bool",
  },
  {
    name: "fui_cut_text_range_snapshot",
    importName: "fui_cut_text_range_snapshot",
    args: [{ name: "handle", type: "u64" }, { name: "start", type: "u32" }, { name: "end", type: "u32" }],
    returns: "bool",
  },
  {
    name: "fui_delete_focused_text_range",
    importName: "fui_delete_focused_text_range",
    args: [{ name: "start", type: "u32" }, { name: "end", type: "u32" }],
    returns: "bool",
  },
  {
    name: "fui_commit_text_action_focus",
    importName: "fui_commit_text_action_focus",
    args: [{ name: "handle", type: "u64" }],
    returns: "void",
  },
  {
    name: "fui_load_svg",
    importName: "fui_load_svg",
    args: [{ name: "svgId", type: "u32" }, { name: "ptr", type: "usize" }, { name: "len", type: "u32" }],
    returns: "void",
  },
  {
    name: "fui_load_texture",
    importName: "fui_load_texture",
    args: [{ name: "textureId", type: "u32" }, { name: "ptr", type: "usize" }, { name: "len", type: "u32" }],
    returns: "void",
  },
  {
    name: "fui_release_svg",
    importName: "fui_release_svg",
    args: [{ name: "svgId", type: "u32" }],
    returns: "void",
  },
  {
    name: "fui_release_texture",
    importName: "fui_release_texture",
    args: [{ name: "textureId", type: "u32" }],
    returns: "void",
  },
  {
    name: "fui_bitmap_commit",
    importName: "fui_bitmap_commit",
    args: [{ name: "textureId", type: "u32" }, { name: "bytesPtr", type: "usize" }, { name: "bytesLen", type: "u32" }, { name: "width", type: "u32" }, { name: "height", type: "u32" }],
    returns: "void",
  },
  {
    name: "fui_bitmap_commit_dirty",
    importName: "fui_bitmap_commit_dirty",
    args: [{ name: "textureId", type: "u32" }, { name: "bytesPtr", type: "usize" }, { name: "bytesLen", type: "u32" }, { name: "fullW", type: "u32" }, { name: "fullH", type: "u32" }, { name: "subX", type: "u32" }, { name: "subY", type: "u32" }, { name: "subW", type: "u32" }, { name: "subH", type: "u32" }],
    returns: "void",
  },
  {
    name: "fui_bitmap_release",
    importName: "fui_bitmap_release",
    args: [{ name: "textureId", type: "u32" }],
    returns: "void",
  },
  {
    name: "fui_render_node_to_rgba",
    importName: "fui_render_node_to_rgba",
    args: [{ name: "handle", type: "u64" }, { name: "width", type: "u32" }, { name: "height", type: "u32" }, { name: "outPtr", type: "usize" }, { name: "outCapacity", type: "u32" }, { name: "scale", type: "f32" }, { name: "x", type: "f32" }, { name: "y", type: "f32" }],
    returns: "u32",
  },
  {
    name: "fui_load_font",
    importName: "fui_load_font",
    args: [{ name: "fontId", type: "u32" }, { name: "ptr", type: "usize" }, { name: "len", type: "u32" }],
    returns: "void",
  },
  {
    name: "fui_start_timer",
    importName: "fui_start_timer",
    args: [{ name: "timerId", type: "u32" }, { name: "delayMs", type: "i32" }],
    returns: "void",
  },
  {
    name: "fui_cancel_timer",
    importName: "fui_cancel_timer",
    args: [{ name: "timerId", type: "u32" }],
    returns: "void",
  },
  {
    name: "fui_set_cursor",
    importName: "fui_set_cursor",
    args: [{ name: "style", type: "u32" }],
    returns: "void",
  },
  {
    name: "fui_is_dark_mode",
    importName: "fui_is_dark_mode",
    args: [],
    returns: "bool",
  },
  {
    name: "fui_get_accent_color",
    importName: "fui_get_accent_color",
    args: [],
    returns: "u32",
  },
  {
    name: "fui_get_platform_family",
    importName: "fui_get_platform_family",
    args: [],
    returns: "u32",
  },
  {
    name: "fui_is_coarse_pointer",
    importName: "fui_is_coarse_pointer",
    args: [],
    returns: "bool",
  },
  {
    name: "fui_show_url_preview",
    importName: "fui_show_url_preview",
    args: [{ name: "ptr", type: "usize" }, { name: "len", type: "u32" }],
    returns: "void",
  },
  {
    name: "fui_hide_url_preview",
    importName: "fui_hide_url_preview",
    args: [],
    returns: "void",
  },
  {
    name: "fui_navigate_to",
    importName: "fui_navigate_to",
    args: [{ name: "ptr", type: "usize" }, { name: "len", type: "u32" }, { name: "openInNewTab", type: "bool" }],
    returns: "void",
  },
  {
    name: "fui_set_persisted_scroll_offset",
    importName: "fui_set_persisted_scroll_offset",
    args: [{ name: "nodeIdPtr", type: "usize" }, { name: "nodeIdLen", type: "u32" }, { name: "x", type: "f32" }, { name: "y", type: "f32" }],
    returns: "void",
  },
  {
    name: "fui_try_get_persisted_scroll_offset",
    importName: "fui_try_get_persisted_scroll_offset",
    args: [{ name: "nodeIdPtr", type: "usize" }, { name: "nodeIdLen", type: "u32" }, { name: "outX", type: "usize" }, { name: "outY", type: "usize" }],
    returns: "bool",
  },
  {
    name: "fui_set_persisted_state",
    importName: "fui_set_persisted_state",
    args: [{ name: "nodeIdPtr", type: "usize" }, { name: "nodeIdLen", type: "u32" }, { name: "kindPtr", type: "usize" }, { name: "kindLen", type: "u32" }, { name: "version", type: "u32" }, { name: "payloadPtr", type: "usize" }, { name: "payloadLen", type: "u32" }],
    returns: "void",
  },
  {
    name: "fui_copy_persisted_state",
    importName: "fui_copy_persisted_state",
    args: [{ name: "nodeIdPtr", type: "usize" }, { name: "nodeIdLen", type: "u32" }, { name: "kindPtr", type: "usize" }, { name: "kindLen", type: "u32" }, { name: "outVersionPtr", type: "usize" }, { name: "payloadPtr", type: "usize" }, { name: "payloadCapacity", type: "u32" }],
    returns: "i32",
  },
  {
    name: "fui_log",
    importName: "fui_log",
    args: [{ name: "categoryPtr", type: "usize" }, { name: "catLen", type: "u32" }, { name: "msgPtr", type: "usize" }, { name: "msgLen", type: "u32" }],
    returns: "void",
  },
  {
    name: "fui_logs_enabled",
    importName: "fui_logs_enabled",
    args: [],
    returns: "bool",
  },
  {
    name: "fui_worker_start_string",
    importName: "fui_worker_start_string",
    args: [{ name: "workerId", type: "u32" }, { name: "wasmPathPtr", type: "usize" }, { name: "wasmPathLen", type: "u32" }, { name: "entryPtr", type: "usize" }, { name: "entryLen", type: "u32" }, { name: "inputPtr", type: "usize" }, { name: "inputLen", type: "u32" }],
    returns: "void",
  },
  {
    name: "fui_worker_cancel",
    importName: "fui_worker_cancel",
    args: [{ name: "workerId", type: "u32" }],
    returns: "void",
  },
  {
    name: "fui_file_capabilities",
    importName: "fui_file_capabilities",
    args: [],
    returns: "u32",
  },
  {
    name: "fui_file_pick",
    importName: "fui_file_pick",
    args: [{ name: "requestId", type: "u32" }, { name: "acceptPtr", type: "usize" }, { name: "acceptLen", type: "u32" }, { name: "multiple", type: "bool" }],
    returns: "void",
  },
  {
    name: "fui_file_read_chunk",
    importName: "fui_file_read_chunk",
    args: [{ name: "requestId", type: "u32" }, { name: "fileIdPtr", type: "usize" }, { name: "fileIdLen", type: "u32" }, { name: "offsetBytes", type: "u64" }, { name: "maxBytes", type: "u32" }],
    returns: "void",
  },
  {
    name: "fui_file_save_text",
    importName: "fui_file_save_text",
    args: [{ name: "requestId", type: "u32" }, { name: "suggestedNamePtr", type: "usize" }, { name: "suggestedNameLen", type: "u32" }, { name: "mimeTypePtr", type: "usize" }, { name: "mimeTypeLen", type: "u32" }, { name: "fileExtensionPtr", type: "usize" }, { name: "fileExtensionLen", type: "u32" }, { name: "textPtr", type: "usize" }, { name: "textLen", type: "u32" }],
    returns: "void",
  },
  {
    name: "fui_file_save_bytes",
    importName: "fui_file_save_bytes",
    args: [{ name: "requestId", type: "u32" }, { name: "suggestedNamePtr", type: "usize" }, { name: "suggestedNameLen", type: "u32" }, { name: "mimeTypePtr", type: "usize" }, { name: "mimeTypeLen", type: "u32" }, { name: "fileExtensionPtr", type: "usize" }, { name: "fileExtensionLen", type: "u32" }, { name: "bytesPtr", type: "usize" }, { name: "bytesLen", type: "u32" }],
    returns: "void",
  },
  {
    name: "fui_file_create_writer",
    importName: "fui_file_create_writer",
    args: [{ name: "requestId", type: "u32" }, { name: "suggestedNamePtr", type: "usize" }, { name: "suggestedNameLen", type: "u32" }, { name: "mimeTypePtr", type: "usize" }, { name: "mimeTypeLen", type: "u32" }, { name: "fileExtensionPtr", type: "usize" }, { name: "fileExtensionLen", type: "u32" }],
    returns: "void",
  },
  {
    name: "fui_file_writer_write_text",
    importName: "fui_file_writer_write_text",
    args: [{ name: "requestId", type: "u32" }, { name: "writerIdPtr", type: "usize" }, { name: "writerIdLen", type: "u32" }, { name: "textPtr", type: "usize" }, { name: "textLen", type: "u32" }],
    returns: "void",
  },
  {
    name: "fui_file_writer_write_bytes",
    importName: "fui_file_writer_write_bytes",
    args: [{ name: "requestId", type: "u32" }, { name: "writerIdPtr", type: "usize" }, { name: "writerIdLen", type: "u32" }, { name: "bytesPtr", type: "usize" }, { name: "bytesLen", type: "u32" }],
    returns: "void",
  },
  {
    name: "fui_file_writer_finish",
    importName: "fui_file_writer_finish",
    args: [{ name: "requestId", type: "u32" }, { name: "writerIdPtr", type: "usize" }, { name: "writerIdLen", type: "u32" }],
    returns: "void",
  },
  {
    name: "fui_file_process_worker_start",
    importName: "fui_file_process_worker_start",
    args: [{ name: "requestId", type: "u32" }, { name: "workerWasmPathPtr", type: "usize" }, { name: "workerWasmPathLen", type: "u32" }, { name: "workerEntryPtr", type: "usize" }, { name: "workerEntryLen", type: "u32" }, { name: "fileIdPtr", type: "usize" }, { name: "fileIdLen", type: "u32" }, { name: "suggestedNamePtr", type: "usize" }, { name: "suggestedNameLen", type: "u32" }, { name: "chunkBytes", type: "u32" }, { name: "saveToPickedFile", type: "bool" }],
    returns: "void",
  },
  {
    name: "fui_file_process_worker_cancel",
    importName: "fui_file_process_worker_cancel",
    args: [{ name: "requestId", type: "u32" }],
    returns: "void",
  },
  {
    name: "fui_canvas_save",
    importName: "fui_canvas_save",
    args: [{ name: "canvasPtr", type: "usize" }],
    returns: "void",
  },
  {
    name: "fui_canvas_restore",
    importName: "fui_canvas_restore",
    args: [{ name: "canvasPtr", type: "usize" }],
    returns: "void",
  },
  {
    name: "fui_canvas_translate",
    importName: "fui_canvas_translate",
    args: [{ name: "canvasPtr", type: "usize" }, { name: "x", type: "f32" }, { name: "y", type: "f32" }],
    returns: "void",
  },
  {
    name: "fui_canvas_scale",
    importName: "fui_canvas_scale",
    args: [{ name: "canvasPtr", type: "usize" }, { name: "sx", type: "f32" }, { name: "sy", type: "f32" }],
    returns: "void",
  },
  {
    name: "fui_canvas_rotate",
    importName: "fui_canvas_rotate",
    args: [{ name: "canvasPtr", type: "usize" }, { name: "degrees", type: "f32" }],
    returns: "void",
  },
  {
    name: "fui_canvas_clip_rect",
    importName: "fui_canvas_clip_rect",
    args: [{ name: "canvasPtr", type: "usize" }, { name: "x", type: "f32" }, { name: "y", type: "f32" }, { name: "w", type: "f32" }, { name: "h", type: "f32" }],
    returns: "void",
  },
  {
    name: "fui_canvas_clip_round_rect",
    importName: "fui_canvas_clip_round_rect",
    args: [{ name: "canvasPtr", type: "usize" }, { name: "x", type: "f32" }, { name: "y", type: "f32" }, { name: "w", type: "f32" }, { name: "h", type: "f32" }, { name: "topLeft", type: "f32" }, { name: "topRight", type: "f32" }, { name: "bottomRight", type: "f32" }, { name: "bottomLeft", type: "f32" }],
    returns: "void",
  },
  {
    name: "fui_canvas_draw_rect",
    importName: "fui_canvas_draw_rect",
    args: [{ name: "canvasPtr", type: "usize" }, { name: "x", type: "f32" }, { name: "y", type: "f32" }, { name: "w", type: "f32" }, { name: "h", type: "f32" }, { name: "fillColor", type: "u32" }, { name: "strokeColor", type: "u32" }, { name: "strokeWidth", type: "f32" }],
    returns: "void",
  },
  {
    name: "fui_canvas_draw_circle",
    importName: "fui_canvas_draw_circle",
    args: [{ name: "canvasPtr", type: "usize" }, { name: "cx", type: "f32" }, { name: "cy", type: "f32" }, { name: "radius", type: "f32" }, { name: "fillColor", type: "u32" }, { name: "strokeColor", type: "u32" }, { name: "strokeWidth", type: "f32" }],
    returns: "void",
  },
  {
    name: "fui_canvas_draw_line",
    importName: "fui_canvas_draw_line",
    args: [{ name: "canvasPtr", type: "usize" }, { name: "x1", type: "f32" }, { name: "y1", type: "f32" }, { name: "x2", type: "f32" }, { name: "y2", type: "f32" }, { name: "color", type: "u32" }, { name: "strokeWidth", type: "f32" }],
    returns: "void",
  },
  {
    name: "fui_canvas_draw_round_rect",
    importName: "fui_canvas_draw_round_rect",
    args: [{ name: "canvasPtr", type: "usize" }, { name: "x", type: "f32" }, { name: "y", type: "f32" }, { name: "w", type: "f32" }, { name: "h", type: "f32" }, { name: "rx", type: "f32" }, { name: "ry", type: "f32" }, { name: "fillColor", type: "u32" }, { name: "strokeColor", type: "u32" }, { name: "strokeWidth", type: "f32" }],
    returns: "void",
  },
  {
    name: "fui_path_create",
    importName: "fui_path_create",
    args: [],
    returns: "u32",
  },
  {
    name: "fui_path_destroy",
    importName: "fui_path_destroy",
    args: [{ name: "pathId", type: "u32" }],
    returns: "void",
  },
  {
    name: "fui_path_move_to",
    importName: "fui_path_move_to",
    args: [{ name: "pathId", type: "u32" }, { name: "x", type: "f32" }, { name: "y", type: "f32" }],
    returns: "void",
  },
  {
    name: "fui_path_line_to",
    importName: "fui_path_line_to",
    args: [{ name: "pathId", type: "u32" }, { name: "x", type: "f32" }, { name: "y", type: "f32" }],
    returns: "void",
  },
  {
    name: "fui_path_quad_to",
    importName: "fui_path_quad_to",
    args: [{ name: "pathId", type: "u32" }, { name: "cx", type: "f32" }, { name: "cy", type: "f32" }, { name: "x", type: "f32" }, { name: "y", type: "f32" }],
    returns: "void",
  },
  {
    name: "fui_path_cubic_to",
    importName: "fui_path_cubic_to",
    args: [{ name: "pathId", type: "u32" }, { name: "cx1", type: "f32" }, { name: "cy1", type: "f32" }, { name: "cx2", type: "f32" }, { name: "cy2", type: "f32" }, { name: "x", type: "f32" }, { name: "y", type: "f32" }],
    returns: "void",
  },
  {
    name: "fui_path_close",
    importName: "fui_path_close",
    args: [{ name: "pathId", type: "u32" }],
    returns: "void",
  },
  {
    name: "fui_path_add_rect",
    importName: "fui_path_add_rect",
    args: [{ name: "pathId", type: "u32" }, { name: "x", type: "f32" }, { name: "y", type: "f32" }, { name: "w", type: "f32" }, { name: "h", type: "f32" }],
    returns: "void",
  },
  {
    name: "fui_path_add_circle",
    importName: "fui_path_add_circle",
    args: [{ name: "pathId", type: "u32" }, { name: "cx", type: "f32" }, { name: "cy", type: "f32" }, { name: "r", type: "f32" }],
    returns: "void",
  },
  {
    name: "fui_canvas_draw_path",
    importName: "fui_canvas_draw_path",
    args: [{ name: "canvasPtr", type: "usize" }, { name: "pathId", type: "u32" }, { name: "fillColor", type: "u32" }, { name: "strokeColor", type: "u32" }, { name: "strokeWidth", type: "f32" }],
    returns: "void",
  },
  {
    name: "fui_canvas_draw_text_node",
    importName: "fui_canvas_draw_text_node",
    args: [{ name: "canvasPtr", type: "usize" }, { name: "handleLo", type: "u32" }, { name: "handleHi", type: "u32" }, { name: "x", type: "f32" }, { name: "y", type: "f32" }],
    returns: "void",
  },
  {
    name: "fui_canvas_draw_image",
    importName: "fui_canvas_draw_image",
    args: [{ name: "canvasPtr", type: "usize" }, { name: "textureId", type: "u32" }, { name: "x", type: "f32" }, { name: "y", type: "f32" }, { name: "w", type: "f32" }, { name: "h", type: "f32" }, { name: "samplingKind", type: "u32" }, { name: "maxAniso", type: "u32" }],
    returns: "void",
  },
  {
    name: "fui_canvas_draw_svg",
    importName: "fui_canvas_draw_svg",
    args: [{ name: "canvasPtr", type: "usize" }, { name: "svgId", type: "u32" }, { name: "x", type: "f32" }, { name: "y", type: "f32" }, { name: "w", type: "f32" }, { name: "h", type: "f32" }],
    returns: "void",
  },
  {
    name: "fui_canvas_draw_batch",
    importName: "fui_canvas_draw_batch",
    args: [{ name: "canvasPtr", type: "usize" }, { name: "wordsPtr", type: "usize" }, { name: "wordCount", type: "u32" }],
    returns: "void",
  },
  {
    name: "fui_canvas_create_offscreen",
    importName: "fui_canvas_create_offscreen",
    args: [{ name: "width", type: "u32" }, { name: "height", type: "u32" }],
    returns: "u32",
  },
  {
    name: "fui_canvas_get_offscreen_ptr",
    importName: "fui_canvas_get_offscreen_ptr",
    args: [{ name: "offscreenId", type: "u32" }],
    returns: "usize",
  },
  {
    name: "fui_canvas_read_offscreen_pixels",
    importName: "fui_canvas_read_offscreen_pixels",
    args: [{ name: "offscreenId", type: "u32" }, { name: "outPtr", type: "usize" }, { name: "width", type: "u32" }, { name: "height", type: "u32" }],
    returns: "void",
  },
  {
    name: "fui_canvas_destroy_offscreen",
    importName: "fui_canvas_destroy_offscreen",
    args: [{ name: "offscreenId", type: "u32" }],
    returns: "void",
  },
] as const satisfies readonly FuiHostImport[];
