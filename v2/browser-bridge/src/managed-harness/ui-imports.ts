import { copyBytesFromHeap, withHeapAllocation, withHeapBytes, type BridgeRuntime,type WasmHandleLike } from '@effindomv2/runtime';

import { addUiPointer,toBigIntHandle,type AppHandleLike } from './interop';

export interface UiImportDeps {
  getRuntime(): BridgeRuntime;
  readAppUtf8(ptr: number, len: number): string;
  readAppFloats(ptr: number, count: number): Float32Array;
  readAppBytes(ptr: number, len: number): Uint8Array;
  withUiUtf8(text: string, callback: (ptr: WasmHandleLike | number, len: number) => void): void;
  withUiGridData(
    values: Float32Array,
    types: Uint8Array,
    callback: (valuesPtr: WasmHandleLike | number, typesPtr: WasmHandleLike | number) => void,
  ): void;
  withUiGradientData(
    offsets: Float32Array,
    colors: Uint32Array,
    callback: (offsetsPtr: WasmHandleLike | number, colorsPtr: WasmHandleLike | number) => void,
  ): void;
  zeroPointer(): WasmHandleLike | number;
  normalizePointer(ptr: WasmHandleLike | number): WasmHandleLike | number;
  getCurrentMemory(): WebAssembly.Memory;
  setLatestRootHandle(rootHandle: string | null): void;
  updateState(): void;
  queueHarnessFrame(): void;
  syncUiHostCapabilities(): void;
  resetUiState(): void;
  recordTextChangedFromAppSet?(handle: AppHandleLike, text: string): void;
}

export function createUiImportModule(deps: UiImportDeps) {
  return {
    ui_reset(): void {
      const runtime = deps.getRuntime();
      runtime.ui._ui_reset();
      deps.syncUiHostCapabilities();
      deps.resetUiState();
    },
    ui_create_node(type: number): bigint {
      return toBigIntHandle(deps.getRuntime().ui._ui_create_node(type));
    },
    ui_set_node_id(handle: AppHandleLike, ptr: number, len: number): void {
      const runtime = deps.getRuntime();
      const text = deps.readAppUtf8(ptr, len);
      deps.withUiUtf8(text, (uiPtr, uiLen) => {
        runtime.ui._ui_set_node_id(toBigIntHandle(handle), uiPtr, uiLen);
      });
    },
    ui_delete_node(handle: AppHandleLike): void {
      deps.getRuntime().ui._ui_delete_node(toBigIntHandle(handle));
    },
    ui_set_semantic_role(handle: AppHandleLike, role: number): void {
      deps.getRuntime().ui._ui_set_semantic_role(toBigIntHandle(handle), role);
    },
    ui_set_semantic_label(handle: AppHandleLike, ptr: number, len: number): void {
      const runtime = deps.getRuntime();
      const text = deps.readAppUtf8(ptr, len);
      deps.withUiUtf8(text, (uiPtr, uiLen) => {
        runtime.ui._ui_set_semantic_label(toBigIntHandle(handle), uiPtr, uiLen);
      });
    },
    ui_set_semantic_checked(handle: AppHandleLike, checkedState: number): void {
      deps.getRuntime().ui._ui_set_semantic_checked(toBigIntHandle(handle), checkedState);
    },
    ui_set_semantic_selected(handle: AppHandleLike, hasSelected: number, selected: number): void {
      deps.getRuntime().ui._ui_set_semantic_selected(toBigIntHandle(handle), hasSelected, selected);
    },
    ui_set_semantic_expanded(handle: AppHandleLike, hasExpanded: number, expanded: number): void {
      deps.getRuntime().ui._ui_set_semantic_expanded(toBigIntHandle(handle), hasExpanded, expanded);
    },
    ui_set_semantic_disabled(handle: AppHandleLike, hasDisabled: number, disabled: number): void {
      deps.getRuntime().ui._ui_set_semantic_disabled(toBigIntHandle(handle), hasDisabled, disabled);
    },
    ui_set_semantic_value_range(
      handle: AppHandleLike,
      hasValueRange: number,
      valueNow: number,
      valueMin: number,
      valueMax: number,
    ): void {
      deps.getRuntime().ui._ui_set_semantic_value_range(toBigIntHandle(handle), hasValueRange, valueNow, valueMin, valueMax);
    },
    ui_set_semantic_orientation(handle: AppHandleLike, orientation: number): void {
      deps.getRuntime().ui._ui_set_semantic_orientation(toBigIntHandle(handle), orientation);
    },
    ui_request_semantic_announcement(handle: AppHandleLike): void {
      deps.getRuntime().ui._ui_request_semantic_announcement(toBigIntHandle(handle));
    },
    ui_push_semantic_scope(handle: AppHandleLike): number {
      return deps.getRuntime().ui._ui_push_semantic_scope(toBigIntHandle(handle));
    },
    ui_remove_semantic_scope(token: number): void {
      deps.getRuntime().ui._ui_remove_semantic_scope(token);
    },
    ui_node_add_child(parent: AppHandleLike, child: AppHandleLike): void {
      deps.getRuntime().ui._ui_node_add_child(toBigIntHandle(parent), toBigIntHandle(child));
    },
    ui_node_remove_child(parent: AppHandleLike, child: AppHandleLike): void {
      deps.getRuntime().ui._ui_node_remove_child(toBigIntHandle(parent), toBigIntHandle(child));
    },
    ui_set_is_portal(handle: AppHandleLike, flag: number): void {
      deps.getRuntime().ui._ui_set_is_portal(toBigIntHandle(handle), flag);
    },
    ui_set_is_shared_size_scope(handle: AppHandleLike, flag: number): void {
      deps.getRuntime().ui._ui_set_is_shared_size_scope(toBigIntHandle(handle), flag);
    },
    ui_set_custom_drawable(handle: AppHandleLike, flag: number): void {
      deps.getRuntime().ui._ui_set_custom_drawable(toBigIntHandle(handle), flag);
    },
    ui_set_flex_wrap(handle: AppHandleLike, wrap: number): void {
      deps.getRuntime().ui._ui_set_flex_wrap(toBigIntHandle(handle), wrap);
    },
    ui_prepare_node(handle: AppHandleLike): number {
      const resolved = deps.getRuntime().ui._ui_prepare_node(toBigIntHandle(handle));
      return resolved;
    },
    ui_set_dynamic_text_charset(handle: AppHandleLike, ptr: number, len: number): void {
      const runtime = deps.getRuntime();
      const value = len > 0 ? deps.readAppUtf8(ptr, len) : '';
      deps.withUiUtf8(value, (uiPtr, uiLen) => {
        runtime.ui._ui_set_dynamic_text_charset(toBigIntHandle(handle), uiPtr, uiLen);
      });
    },
    ui_set_root(handle: AppHandleLike): void {
      const runtime = deps.getRuntime();
      const rootHandle = toBigIntHandle(handle);
      deps.setLatestRootHandle(rootHandle.toString());
      runtime.ui._ui_set_root(rootHandle);
      deps.updateState();
    },
    ui_set_width(handle: AppHandleLike, value: number, unit: number): void {
      deps.getRuntime().ui._ui_set_width(toBigIntHandle(handle), value, unit);
    },
    ui_set_height(handle: AppHandleLike, value: number, unit: number): void {
      deps.getRuntime().ui._ui_set_height(toBigIntHandle(handle), value, unit);
    },
    ui_set_fill_width(handle: AppHandleLike, fill: number): void {
      deps.getRuntime().ui._ui_set_fill_width(toBigIntHandle(handle), fill);
    },
    ui_set_fill_height(handle: AppHandleLike, fill: number): void {
      deps.getRuntime().ui._ui_set_fill_height(toBigIntHandle(handle), fill);
    },
    ui_set_fill_width_percent(handle: AppHandleLike, percent: number): void {
      deps.getRuntime().ui._ui_set_fill_width_percent(toBigIntHandle(handle), percent);
    },
    ui_set_fill_height_percent(handle: AppHandleLike, percent: number): void {
      deps.getRuntime().ui._ui_set_fill_height_percent(toBigIntHandle(handle), percent);
    },
    ui_set_min_width(handle: AppHandleLike, value: number, unit: number): void {
      deps.getRuntime().ui._ui_set_min_width(toBigIntHandle(handle), value, unit);
    },
    ui_set_max_width(handle: AppHandleLike, value: number, unit: number): void {
      deps.getRuntime().ui._ui_set_max_width(toBigIntHandle(handle), value, unit);
    },
    ui_set_min_height(handle: AppHandleLike, value: number, unit: number): void {
      deps.getRuntime().ui._ui_set_min_height(toBigIntHandle(handle), value, unit);
    },
    ui_set_max_height(handle: AppHandleLike, value: number, unit: number): void {
      deps.getRuntime().ui._ui_set_max_height(toBigIntHandle(handle), value, unit);
    },
    ui_set_flex_direction(handle: AppHandleLike, direction: number): void {
      deps.getRuntime().ui._ui_set_flex_direction(toBigIntHandle(handle), direction);
    },
    ui_set_flex_basis(handle: AppHandleLike, basis: number): void {
      deps.getRuntime().ui._ui_set_flex_basis(toBigIntHandle(handle), basis);
    },
    ui_set_justify_content(handle: AppHandleLike, justify: number): void {
      deps.getRuntime().ui._ui_set_justify_content(toBigIntHandle(handle), justify);
    },
    ui_set_align_items(handle: AppHandleLike, align: number): void {
      deps.getRuntime().ui._ui_set_align_items(toBigIntHandle(handle), align);
    },
    ui_set_align_self(handle: AppHandleLike, align: number): void {
      deps.getRuntime().ui._ui_set_align_self(toBigIntHandle(handle), align);
    },
    ui_set_padding(handle: AppHandleLike, left: number, top: number, right: number, bottom: number): void {
      deps.getRuntime().ui._ui_set_padding(toBigIntHandle(handle), left, top, right, bottom);
    },
    ui_set_margin(handle: AppHandleLike, left: number, top: number, right: number, bottom: number): void {
      deps.getRuntime().ui._ui_set_margin(toBigIntHandle(handle), left, top, right, bottom);
    },
    ui_set_position_type(handle: AppHandleLike, positionType: number): void {
      deps.getRuntime().ui._ui_set_position_type(toBigIntHandle(handle), positionType);
    },
    ui_set_position(handle: AppHandleLike, left: number, top: number, right: number, bottom: number): void {
      deps.getRuntime().ui._ui_set_position(toBigIntHandle(handle), left, top, right, bottom);
    },
    ui_grid_set_columns(handle: AppHandleLike, count: number, valuesPtr: number, typesPtr: number): void {
      const runtime = deps.getRuntime();
      deps.withUiGridData(deps.readAppFloats(valuesPtr, count), deps.readAppBytes(typesPtr, count), (uiValuesPtr, uiTypesPtr) => {
        runtime.ui._ui_grid_set_columns(toBigIntHandle(handle), count, uiValuesPtr, uiTypesPtr);
      });
    },
    ui_grid_set_rows(handle: AppHandleLike, count: number, valuesPtr: number, typesPtr: number): void {
      const runtime = deps.getRuntime();
      deps.withUiGridData(deps.readAppFloats(valuesPtr, count), deps.readAppBytes(typesPtr, count), (uiValuesPtr, uiTypesPtr) => {
        runtime.ui._ui_grid_set_rows(toBigIntHandle(handle), count, uiValuesPtr, uiTypesPtr);
      });
    },
    ui_grid_set_column_shared_size_group(handle: AppHandleLike, index: number, ptr: number, len: number): void {
      const runtime = deps.getRuntime();
      const text = deps.readAppUtf8(ptr, len);
      deps.withUiUtf8(text, (uiPtr, uiLen) => {
        runtime.ui._ui_grid_set_column_shared_size_group(toBigIntHandle(handle), index, uiPtr, uiLen);
      });
    },
    ui_grid_set_row_shared_size_group(handle: AppHandleLike, index: number, ptr: number, len: number): void {
      const runtime = deps.getRuntime();
      const text = deps.readAppUtf8(ptr, len);
      deps.withUiUtf8(text, (uiPtr, uiLen) => {
        runtime.ui._ui_grid_set_row_shared_size_group(toBigIntHandle(handle), index, uiPtr, uiLen);
      });
    },
    ui_node_set_grid_placement(handle: AppHandleLike, row: number, col: number, rowSpan: number, colSpan: number): void {
      deps.getRuntime().ui._ui_node_set_grid_placement(toBigIntHandle(handle), row, col, rowSpan, colSpan);
    },
    ui_set_bg_color(handle: AppHandleLike, color: number): void {
      deps.getRuntime().ui._ui_set_bg_color(toBigIntHandle(handle), color);
    },
    ui_set_box_style(
      handle: AppHandleLike,
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
    ): void {
      deps.getRuntime().ui._ui_set_box_style(
        toBigIntHandle(handle),
        bgColor,
        topLeftRadius,
        topRightRadius,
        bottomRightRadius,
        bottomLeftRadius,
        borderWidth,
        borderColor,
        borderStyle,
        borderDashOn,
        borderDashOff,
      );
    },
    ui_set_layer_effect(handle: AppHandleLike, opacity: number, blurSigma: number, blendMode: number): void {
      deps.getRuntime().ui._ui_set_layer_effect(toBigIntHandle(handle), opacity, blurSigma, blendMode);
    },
    ui_set_drop_shadow(
      handle: AppHandleLike,
      color: number,
      offsetX: number,
      offsetY: number,
      blurSigma: number,
      spread: number,
    ): void {
      deps.getRuntime().ui._ui_set_drop_shadow(toBigIntHandle(handle), color, offsetX, offsetY, blurSigma, spread);
    },
    ui_set_background_blur(handle: AppHandleLike, blurSigma: number): void {
      deps.getRuntime().ui._ui_set_background_blur(toBigIntHandle(handle), blurSigma);
    },
    ui_set_image(handle: AppHandleLike, textureId: number, objectFit: number, samplingKind: number, maxAniso: number): void {
      deps.getRuntime().ui._ui_set_image(toBigIntHandle(handle), textureId, objectFit, samplingKind, maxAniso);
    },
    ui_set_image_nine(
      handle: AppHandleLike,
      textureId: number,
      insetLeft: number,
      insetTop: number,
      insetRight: number,
      insetBottom: number,
      samplingKind: number,
      maxAniso: number,
    ): void {
      deps.getRuntime().ui._ui_set_image_nine(
        toBigIntHandle(handle),
        textureId,
        insetLeft,
        insetTop,
        insetRight,
        insetBottom,
        samplingKind,
        maxAniso,
      );
    },
    ui_set_svg(handle: AppHandleLike, svgId: number, tintColor: number, samplingKind: number, maxAniso: number): void {
      deps.getRuntime().ui._ui_set_svg(toBigIntHandle(handle), svgId, tintColor, samplingKind, maxAniso);
    },
    ui_set_linear_gradient(
      handle: AppHandleLike,
      startX: number,
      startY: number,
      endX: number,
      endY: number,
      stopCount: number,
      offsetsPtr: number,
      colorsPtr: number,
    ): void {
      const normalizedStopCount = Math.max(0, stopCount);
      const colorBytes = deps.readAppBytes(colorsPtr, normalizedStopCount * 4);
      const colors = new Uint32Array(colorBytes.buffer, colorBytes.byteOffset, normalizedStopCount);
      deps.withUiGradientData(
        deps.readAppFloats(offsetsPtr, normalizedStopCount),
        new Uint32Array(colors),
        (uiOffsetsPtr, uiColorsPtr) => {
          const runtime = deps.getRuntime();
          runtime.ui._ui_set_linear_gradient(
            toBigIntHandle(handle),
            startX,
            startY,
            endX,
            endY,
            normalizedStopCount,
            uiOffsetsPtr,
            uiColorsPtr,
          );
        },
      );
    },
    ui_set_clip_to_bounds(handle: AppHandleLike, clip: number): void {
      deps.getRuntime().ui._ui_set_clip_to_bounds(toBigIntHandle(handle), clip);
    },
    ui_set_visibility(handle: AppHandleLike, visibility: number): void {
      deps.getRuntime().ui._ui_set_visibility(toBigIntHandle(handle), visibility);
    },
    ui_set_interactive(handle: AppHandleLike, flag: number): void {
      deps.getRuntime().ui._ui_set_interactive(toBigIntHandle(handle), flag);
    },
    ui_set_preserve_selection_on_pointer_down(handle: AppHandleLike, preserve: number): void {
      const runtime = deps.getRuntime();
      const setPreserve = runtime.ui._ui_set_preserve_selection_on_pointer_down;
      if (typeof setPreserve !== 'function') {
        console.error(
          '[fui_host] UI runtime is missing _ui_set_preserve_selection_on_pointer_down; ' +
          'run repo root ./build.sh and refresh the served runtime assets.',
        );
        return;
      }
      setPreserve(toBigIntHandle(handle), preserve);
    },
    ui_set_editor_command_keys(handle: AppHandleLike, enabled: number): void {
      const runtime = deps.getRuntime();
      const setEditorCommandKeys = runtime.ui._ui_set_editor_command_keys;
      if (typeof setEditorCommandKeys !== 'function') {
        console.error(
          '[fui_host] UI runtime is missing _ui_set_editor_command_keys; ' +
          'run repo root ./build.sh and refresh the served runtime assets.',
        );
        return;
      }
      setEditorCommandKeys(toBigIntHandle(handle), enabled);
    },
    ui_set_editor_accepts_tab(handle: AppHandleLike, enabled: number): void {
      const runtime = deps.getRuntime();
      const setEditorAcceptsTab = runtime.ui._ui_set_editor_accepts_tab;
      if (typeof setEditorAcceptsTab !== 'function') {
        console.error(
          '[fui_host] UI runtime is missing _ui_set_editor_accepts_tab; ' +
          'run repo root ./build.sh and refresh the served runtime assets.',
        );
        return;
      }
      setEditorAcceptsTab(toBigIntHandle(handle), enabled);
    },
    ui_set_scroll_proxy_target(handle: AppHandleLike, scrollHandle: AppHandleLike): void {
      deps.getRuntime().ui._ui_set_scroll_proxy_target(toBigIntHandle(handle), toBigIntHandle(scrollHandle));
    },
    ui_set_scroll_enabled(handle: AppHandleLike, enabledX: number, enabledY: number): void {
      deps.getRuntime().ui._ui_set_scroll_enabled(toBigIntHandle(handle), enabledX, enabledY);
    },
    ui_set_scroll_friction(handle: AppHandleLike, friction: number): void {
      deps.getRuntime().ui._ui_set_scroll_friction(toBigIntHandle(handle), friction);
    },
    ui_set_smooth_scrolling(handle: AppHandleLike, smoothScrolling: number): void {
      deps.getRuntime().ui._ui_set_smooth_scrolling(toBigIntHandle(handle), smoothScrolling ? 1 : 0);
    },
    ui_set_scroll_content_size(handle: AppHandleLike, contentWidth: number, contentHeight: number): void {
      deps.getRuntime().ui._ui_set_scroll_content_size(toBigIntHandle(handle), contentWidth, contentHeight);
    },
    ui_set_focusable(handle: AppHandleLike, flag: number, tabIndex: number): void {
      deps.getRuntime().ui._ui_set_focusable(toBigIntHandle(handle), flag, tabIndex);
    },
    ui_request_focus(handle: AppHandleLike): void {
      deps.getRuntime().ui._ui_request_focus(toBigIntHandle(handle));
    },
    ui_set_font(handle: AppHandleLike, fontId: number, size: number): void {
      const runtime = deps.getRuntime();
      void runtime.ensureFont(fontId).catch((error: unknown) => {
        const message = error instanceof Error ? error.message : String(error);
        console.error(`[fui_host] font ${String(fontId)} failed to load on demand: ${message}`);
      });
      runtime.ui._ui_set_font(toBigIntHandle(handle), fontId, size);
    },
    ui_set_line_height(handle: AppHandleLike, lineHeight: number): void {
      deps.getRuntime().ui._ui_set_line_height(toBigIntHandle(handle), lineHeight);
    },
    ui_register_font_fallback(fontId: number, fallbackFontId: number): void {
      deps.getRuntime().registerFontFallback(fontId, fallbackFontId);
    },
    ui_set_text_color(handle: AppHandleLike, color: number): void {
      deps.getRuntime().ui._ui_set_text_color(toBigIntHandle(handle), color);
    },
    ui_set_text_align(handle: AppHandleLike, align: number): void {
      deps.getRuntime().ui._ui_set_text_align(toBigIntHandle(handle), align);
    },
    ui_set_text_vertical_align(handle: AppHandleLike, align: number): void {
      deps.getRuntime().ui._ui_set_text_vertical_align(toBigIntHandle(handle), align);
    },
    ui_set_text_limits(handle: AppHandleLike, maxChars: number, maxLines: number): void {
      deps.getRuntime().ui._ui_set_text_limits(toBigIntHandle(handle), maxChars, maxLines);
    },
    ui_set_text_wrapping(handle: AppHandleLike, wrap: number): void {
      deps.getRuntime().ui._ui_set_text_wrapping(toBigIntHandle(handle), wrap);
    },
    ui_set_text_overflow(handle: AppHandleLike, overflow: number): void {
      deps.getRuntime().ui._ui_set_text_overflow(toBigIntHandle(handle), overflow);
    },
    ui_set_text_overflow_fade(handle: AppHandleLike, horizontal: number, vertical: number): void {
      deps.getRuntime().ui._ui_set_text_overflow_fade(toBigIntHandle(handle), horizontal, vertical);
    },
    ui_set_text_obscured(handle: AppHandleLike, obscured: number): void {
      deps.getRuntime().ui._ui_set_text_obscured(toBigIntHandle(handle), obscured);
    },
    ui_set_editable(handle: AppHandleLike, editable: number): void {
      deps.getRuntime().ui._ui_set_editable(toBigIntHandle(handle), editable);
    },
    ui_set_caret_color(handle: AppHandleLike, color: number): void {
      deps.getRuntime().ui._ui_set_caret_color(toBigIntHandle(handle), color);
    },
    ui_set_selectable(handle: AppHandleLike, selectable: number, selectionColor: number): void {
      deps.getRuntime().ui._ui_set_selectable(toBigIntHandle(handle), selectable, selectionColor);
    },
    ui_set_selection_area(handle: AppHandleLike, isArea: number): void {
      deps.getRuntime().ui._ui_set_selection_area(toBigIntHandle(handle), isArea);
    },
    ui_set_selection_area_barrier(handle: AppHandleLike, isBarrier: number): void {
      deps.getRuntime().ui._ui_set_selection_area_barrier(toBigIntHandle(handle), isBarrier);
    },
    ui_clear_selection(handle: AppHandleLike): void {
      deps.getRuntime().ui._ui_clear_selection(toBigIntHandle(handle));
    },
    ui_retarget_selection(fromHandle: AppHandleLike, toHandle: AppHandleLike): void {
      deps.getRuntime().ui._ui_retarget_selection(toBigIntHandle(fromHandle), toBigIntHandle(toHandle));
    },
    ui_is_point_in_selection(x: number, y: number): number {
      return deps.getRuntime().ui._ui_is_point_in_selection(x, y);
    },
    ui_set_text_selection_range(handle: AppHandleLike, selectionStart: number, selectionEnd: number): void {
      deps.getRuntime().ui._ui_set_text_selection_range(toBigIntHandle(handle), selectionStart, selectionEnd);
    },
    ui_select_word_at(handle: AppHandleLike, x: number, y: number): number {
      const runtime = deps.getRuntime();
      const selectWordAt = runtime.ui._ui_select_word_at;
      if (typeof selectWordAt !== 'function') {
        console.error(
          '[fui_host] UI runtime is missing _ui_select_word_at; ' +
          'run repo root ./build.sh and refresh the served runtime assets.',
        );
        return 0;
      }
      return selectWordAt(toBigIntHandle(handle), x, y);
    },
    ui_begin_selection_endpoint_drag(handle: AppHandleLike, endpoint: number): number {
      const runtime = deps.getRuntime();
      const beginDrag = runtime.ui._ui_begin_selection_endpoint_drag;
      if (typeof beginDrag !== 'function') {
        console.error(
          '[fui_host] UI runtime is missing _ui_begin_selection_endpoint_drag; ' +
          'run repo root ./build.sh and refresh the served runtime assets.',
        );
        return 0;
      }
      return beginDrag(toBigIntHandle(handle), endpoint);
    },
    ui_get_text_range_rect_count(handle: AppHandleLike, start: number, end: number): number {
      return deps.getRuntime().ui._ui_get_text_range_rect_count(toBigIntHandle(handle), start, end);
    },
    ui_copy_text_range_rects(
      handle: AppHandleLike,
      start: number,
      end: number,
      outRectWordsPtr: number,
      maxRectCount: number,
    ): number {
      const runtime = deps.getRuntime();
      const clampedRectCount = Math.max(0, maxRectCount | 0);
      if (clampedRectCount === 0) {
        return 0;
      }
      const byteLength = clampedRectCount * 4 * 4;
      return withHeapAllocation(runtime.ui, byteLength, (heap) => {
        const copiedCount = runtime.ui._ui_copy_text_range_rects(
          toBigIntHandle(handle),
          start,
          end,
          heap.ptr,
          clampedRectCount,
        );
        if (copiedCount === 0) {
          return 0;
        }
        const copiedBytes = copiedCount * 4 * 4;
        const uiBytes = copyBytesFromHeap(runtime.ui, heap.ptr, copiedBytes);
        const appMemory = deps.getCurrentMemory();
        const appBytes = new Uint8Array(appMemory.buffer, outRectWordsPtr, copiedBytes);
        appBytes.set(uiBytes);
        return copiedCount;
      });
    },
    ui_copy_cross_selection_endpoint_rects(areaHandle: AppHandleLike, outRectWordsPtr: number): number {
      const runtime = deps.getRuntime();
      const copyEndpointRects = runtime.ui._ui_copy_cross_selection_endpoint_rects;
      if (typeof copyEndpointRects !== 'function') {
        console.error(
          '[fui_host] UI runtime is missing _ui_copy_cross_selection_endpoint_rects; ' +
          'run repo root ./build.sh and refresh the served runtime assets.',
        );
        return 0;
      }
      const byteLength = 8 * 4;
      return withHeapAllocation(runtime.ui, byteLength, (heap) => {
        const copied = copyEndpointRects(
          toBigIntHandle(areaHandle),
          heap.ptr,
        );
        if (copied === 0) {
          return 0;
        }
        const uiBytes = copyBytesFromHeap(runtime.ui, heap.ptr, byteLength);
        const appMemory = deps.getCurrentMemory();
        const appBytes = new Uint8Array(appMemory.buffer, outRectWordsPtr, byteLength);
        appBytes.set(uiBytes);
        return 1;
      });
    },
    ui_clear_current_selection(): void {
      deps.getRuntime().ui._ui_clear_current_selection();
    },
    ui_copy_current_selection(): void {
      deps.getRuntime().ui._ui_copy_current_selection();
    },
    ui_can_undo_text_edit(handle: AppHandleLike): number {
      return deps.getRuntime().ui._ui_can_undo_text_edit(toBigIntHandle(handle));
    },
    ui_can_redo_text_edit(handle: AppHandleLike): number {
      return deps.getRuntime().ui._ui_can_redo_text_edit(toBigIntHandle(handle));
    },
    ui_has_text_selection(handle: AppHandleLike): number {
      return deps.getRuntime().ui._ui_has_text_selection(toBigIntHandle(handle));
    },
    ui_undo_text_edit(handle: AppHandleLike): void {
      deps.getRuntime().ui._ui_undo_text_edit(toBigIntHandle(handle));
    },
    ui_redo_text_edit(handle: AppHandleLike): void {
      deps.getRuntime().ui._ui_redo_text_edit(toBigIntHandle(handle));
    },
    ui_copy_text_selection(handle: AppHandleLike): void {
      deps.getRuntime().ui._ui_copy_text_selection(toBigIntHandle(handle));
    },
    ui_cut_text_selection(handle: AppHandleLike): void {
      deps.getRuntime().ui._ui_cut_text_selection(toBigIntHandle(handle));
    },
    ui_replace_text_range(handle: AppHandleLike, start: number, end: number, ptr: number, len: number, caret: number): void {
      deps.getRuntime().ui._ui_replace_text_range(toBigIntHandle(handle), start, end, ptr, len, caret);
    },
    ui_paste_text(handle: AppHandleLike): void {
      deps.getRuntime().ui._ui_paste_text(toBigIntHandle(handle));
    },
    ui_select_all_text(handle: AppHandleLike): void {
      deps.getRuntime().ui._ui_select_all_text(toBigIntHandle(handle));
    },
    ui_set_scroll_offset(handle: AppHandleLike, x: number, y: number): void {
      deps.getRuntime().ui._ui_set_scroll_offset(toBigIntHandle(handle), x, y);
    },
    ui_clear_momentum_scroll(): void {
      deps.getRuntime().ui._ui_clear_momentum_scroll();
    },
    ui_get_bounds(
      handle: AppHandleLike,
      outX: number,
      outY: number,
      outWidth: number,
      outHeight: number,
    ): number {
      const runtime = deps.getRuntime();
      const appMemory = deps.getCurrentMemory();
      return withHeapAllocation(runtime.ui, 16, (heap) => {
        const found = runtime.ui._ui_get_bounds(
          toBigIntHandle(handle),
          heap.ptr,
          addUiPointer(runtime, heap.ptr, 4),
          addUiPointer(runtime, heap.ptr, 8),
          addUiPointer(runtime, heap.ptr, 12),
        );
        if (found === 0) {
          return 0;
        }

        const uiView = new DataView(copyBytesFromHeap(runtime.ui, heap.ptr, heap.len).buffer);
        const appView = new DataView(appMemory.buffer);
        appView.setFloat32(outX, uiView.getFloat32(0, true), true);
        appView.setFloat32(outY, uiView.getFloat32(4, true), true);
        appView.setFloat32(outWidth, uiView.getFloat32(8, true), true);
        appView.setFloat32(outHeight, uiView.getFloat32(12, true), true);
        return 1;
      });
    },
    ui_get_visible_bounds(
      handle: AppHandleLike,
      outX: number,
      outY: number,
      outWidth: number,
      outHeight: number,
    ): number {
      const runtime = deps.getRuntime();
      const appMemory = deps.getCurrentMemory();
      return withHeapAllocation(runtime.ui, 16, (heap) => {
        const found = runtime.ui._ui_get_visible_bounds(
          toBigIntHandle(handle),
          heap.ptr,
          addUiPointer(runtime, heap.ptr, 4),
          addUiPointer(runtime, heap.ptr, 8),
          addUiPointer(runtime, heap.ptr, 12),
        );
        if (found === 0) {
          return 0;
        }

        const uiView = new DataView(copyBytesFromHeap(runtime.ui, heap.ptr, heap.len).buffer);
        const appView = new DataView(appMemory.buffer);
        appView.setFloat32(outX, uiView.getFloat32(0, true), true);
        appView.setFloat32(outY, uiView.getFloat32(4, true), true);
        appView.setFloat32(outWidth, uiView.getFloat32(8, true), true);
        appView.setFloat32(outHeight, uiView.getFloat32(12, true), true);
        return 1;
      });
    },
    ui_get_text_metrics(
      handle: AppHandleLike,
      outWidth: number,
      outHeight: number,
      outBaseline: number,
      outLineCount: number,
      outMaxLineWidth: number,
    ): number {
      const runtime = deps.getRuntime();
      const appMemory = deps.getCurrentMemory();
      return withHeapAllocation(runtime.ui, 20, (heap) => {
        const found = runtime.ui._ui_get_text_metrics(
          toBigIntHandle(handle),
          heap.ptr,
          addUiPointer(runtime, heap.ptr, 4),
          addUiPointer(runtime, heap.ptr, 8),
          addUiPointer(runtime, heap.ptr, 12),
          addUiPointer(runtime, heap.ptr, 16),
        );
        if (found === 0) {
          return 0;
        }

        const uiView = new DataView(copyBytesFromHeap(runtime.ui, heap.ptr, heap.len).buffer);
        const appView = new DataView(appMemory.buffer);
        appView.setFloat32(outWidth, uiView.getFloat32(0, true), true);
        appView.setFloat32(outHeight, uiView.getFloat32(4, true), true);
        appView.setFloat32(outBaseline, uiView.getFloat32(8, true), true);
        appView.setUint32(outLineCount, uiView.getUint32(12, true), true);
        appView.setFloat32(outMaxLineWidth, uiView.getFloat32(16, true), true);
        return 1;
      });
    },
    ui_set_text(handle: AppHandleLike, ptr: number, len: number): void {
      const runtime = deps.getRuntime();
      const text = deps.readAppUtf8(ptr, len);
      deps.withUiUtf8(text, (uiPtr, uiLen) => {
        runtime.ui._ui_set_text(toBigIntHandle(handle), uiPtr, uiLen);
      });
      deps.recordTextChangedFromAppSet?.(handle, text);
    },
    ui_set_text_style_runs(handle: AppHandleLike, runCount: number, runsWordsPtr: AppHandleLike): void {
      const runtime = deps.getRuntime();
      const clampedRunCount = Math.max(0, runCount | 0);
      if (clampedRunCount === 0) {
        runtime.ui._ui_set_text_style_runs(toBigIntHandle(handle), 0, deps.zeroPointer());
        return;
      }
      const byteLength = clampedRunCount * 7 * 4;
      const appPtr = Number(runsWordsPtr);
      const appBytes = new Uint8Array(deps.getCurrentMemory().buffer, appPtr, byteLength);
      const runs = new Uint32Array(appBytes.buffer, appBytes.byteOffset, clampedRunCount * 7);
      const requestedFonts = new Set<number>();
      for (let index = 0; index < clampedRunCount; index += 1) {
        const fontId = runs[(index * 7) + 2] ?? 0;
        if (fontId !== 0) {
          requestedFonts.add(fontId);
        }
      }
      for (const fontId of requestedFonts) {
        void runtime.ensureFont(fontId).catch((error: unknown) => {
          const message = error instanceof Error ? error.message : String(error);
          console.error(`[fui_host] rich text font ${String(fontId)} failed to load on demand: ${message}`);
        });
      }
      withHeapBytes(runtime.ui, appBytes, (heap) => {
        runtime.ui._ui_set_text_style_runs(
          toBigIntHandle(handle),
          clampedRunCount,
          heap.ptr,
        );
      });
    },
    ui_commit_frame(): void {
      const runtime = deps.getRuntime();
      runtime.commitFrame();
      deps.queueHarnessFrame();
    },
    ui_resize_window(width: number, height: number): void {
      deps.getRuntime().ui._ui_resize_window(width, height);
    },
  };
}
