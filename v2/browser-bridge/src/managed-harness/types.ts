import type {
  BridgeRuntime,
  BuildMode,
  DevToolsDomMirrorMode,
  PageZoomMode,
  WasmHandleLike,
} from '@effindomv2/runtime';

import type { DebugTreeSnapshot } from '../debug-tree';
import type { HostEventsDefinition } from './host-events';
import type { HostServicesDefinition } from './host-services';
import type { WorkerHostServicesBundleConfig } from './worker-types';

export interface HarnessState {
  readonly commandWordCount: number;
  readonly commandWords: readonly number[];
  readonly rootHandle: string | null;
}

export interface HarnessExports {
  readonly memory: WebAssembly.Memory;
  __flushRenders(this: void): void;
  __fui_capture_persisted_ui_state?(this: void): void;
  __fui_debug_pointer_event?(this: void, eventType: number, handle: bigint, x: number, y: number, modifiers: number): void;
  __fui_debug_focus_changed?(this: void, handle: bigint, focused: boolean): void;
  __fui_debug_key_event?(this: void, eventType: number, keyPtr: number, keyLen: number, modifiers: number): void;
  __fui_debug_scroll?(this: void, 
    handle: bigint,
    offsetX: number,
    offsetY: number,
    contentWidth: number,
    contentHeight: number,
    viewportWidth: number,
    viewportHeight: number,
  ): void;
  __fui_on_pointer_event_with_metadata(this: void, 
    eventType: number,
    handle: bigint,
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
  ): boolean;
  __fui_on_wheel_event(this: void, 
    handle: bigint,
    x: number,
    y: number,
    deltaX: number,
    deltaY: number,
    deltaMode: number,
    modifiers: number,
  ): number;
  __fui_resolve_gesture_owner?(this: void, handle: bigint): bigint;
  __fui_get_gesture_intent?(this: void, handle: bigint): number;
  __fui_on_gesture_event?(this: void, 
    handle: bigint,
    phase: number,
    kind: number,
    x: number,
    y: number,
    deltaX: number,
    deltaY: number,
    scale: number,
    pointerCount: number,
  ): boolean | number;
  __fui_resolve_long_press_owner?(this: void, handle: bigint): bigint;
  __fui_get_long_press_minimum_duration_ms?(this: void, handle: bigint): number;
  __fui_get_long_press_movement_tolerance?(this: void, handle: bigint): number;
  __fui_on_long_press_event?(this: void, 
    handle: bigint,
    x: number,
    y: number,
    pointerId: number,
    pointerType: number,
    modifiers: number,
    durationMs: number,
  ): boolean | number;
  __fui_on_external_drag_event(this: void, 
    eventType: number,
    handle: bigint,
    x: number,
    y: number,
    modifiers: number,
    payloadPtr: number,
    payloadLen: number,
  ): number;
  __fui_on_fetch_complete(this: void, 
    requestId: number,
    ok: boolean,
    status: number,
    payloadPtr: number,
    payloadLen: number,
  ): void;
  __fui_on_fetch_error(this: void, requestId: number, payloadPtr: number, payloadLen: number): void;
  __fui_on_file_pick_result(this: void, requestId: number, status: number, payloadPtr: number, payloadLen: number): void;
  __fui_on_file_read_result(this: void, 
    requestId: number,
    status: number,
    offsetBytes: bigint,
    fileSizeBytes: bigint,
    payloadPtr: number,
    payloadLen: number,
  ): void;
  __fui_on_file_save_result(this: void, 
    requestId: number,
    status: number,
    writtenBytes: bigint,
    payloadPtr: number,
    payloadLen: number,
  ): void;
  __fui_on_file_writer_created(this: void, requestId: number, status: number, payloadPtr: number, payloadLen: number): void;
  __fui_on_file_write_result(this: void, 
    requestId: number,
    status: number,
    writtenBytes: bigint,
    totalWrittenBytes: bigint,
    payloadPtr: number,
    payloadLen: number,
  ): void;
  __fui_on_file_finish_result(this: void, 
    requestId: number,
    status: number,
    writtenBytes: bigint,
    payloadPtr: number,
    payloadLen: number,
  ): void;
  __fui_on_file_worker_process_progress(this: void, 
    requestId: number,
    copiedBytes: bigint,
    totalBytes: bigint,
    payloadPtr: number,
    payloadLen: number,
  ): void;
  __fui_on_file_worker_process_chunk(this: void, 
    requestId: number,
    offsetBytes: bigint,
    fileSizeBytes: bigint,
    payloadPtr: number,
    payloadLen: number,
  ): void;
  __fui_on_file_worker_process_complete(this: void, 
    requestId: number,
    writtenBytes: bigint,
    payloadPtr: number,
    payloadLen: number,
  ): void;
  __fui_on_file_worker_process_error(this: void, requestId: number, status: number, payloadPtr: number, payloadLen: number): void;
  __fui_on_context_menu(this: void, handle: bigint, x: number, y: number): void;
  __fui_can_show_context_menu?(this: void, handle: bigint): boolean;
  __fui_hide_active_context_menu(this: void): void;
  __fui_key_buffer(this: void): number;
  __fui_text_buffer(this: void): number;
  __fui_text_buffer_size(this: void): number;
  __fui_on_focus_changed(this: void, handle: bigint, focused: boolean): void;
  __fui_on_text_changed(this: void, handle: bigint, textPtr: number, textLen: number): void;
  __fui_on_text_replaced(this: void, handle: bigint, start: number, end: number, textPtr: number, textLen: number): void;
  __fui_on_selection_changed(this: void, handle: bigint, start: number, end: number): void;
  __fui_on_key_event(this: void, eventType: number, keyPtr: number, keyLen: number, modifiers: number): number;
  __fui_on_scroll(this: void, 
    handle: bigint,
    offsetX: number,
    offsetY: number,
    contentWidth: number,
    contentHeight: number,
    viewportWidth: number,
    viewportHeight: number,
  ): void;
  __fui_on_cross_selection_changed(this: void, handle: bigint, textPtr: number, textLen: number): void;
  __fui_on_route_changed(this: void, routePtr: number, routeLen: number): void;
  __fui_on_viewport_changed(this: void, width: number, height: number): void;
  __fui_on_system_dark_mode_changed(this: void, isDark: boolean): void;
  __fui_on_system_accent_color_changed(this: void, color: number): void;
  __fui_on_svg_loaded(this: void, svgId: number, width: number, height: number): void;
  __fui_on_svg_failed(this: void, svgId: number, errorPtr: number, errorLen: number): void;
  __fui_on_texture_loaded(this: void, textureId: number, width: number, height: number): void;
  __fui_on_texture_failed(this: void, textureId: number, errorPtr: number, errorLen: number): void;
  __fui_on_frame(this: void, timestampMs: number): void;
  __fui_on_timer(this: void, timerId: number): void;
  __fui_on_font_loaded(this: void, fontId: number): void;
  __fui_on_worker_progress(this: void, workerId: number, textPtr: number, textLen: number): void;
  __fui_on_worker_complete(this: void, workerId: number, textPtr: number, textLen: number): void;
  __fui_on_worker_error(this: void, workerId: number, textPtr: number, textLen: number): void;
  __fui_restore_persisted_ui_state?(this: void): void;
  fui_dispatch_custom_draw?(this: void, handle: bigint, canvasPtr: number): void;
}

export interface HarnessContext<Exports extends HarnessExports> {
  readonly runtime: BridgeRuntime;
  readonly exports: Exports;
  waitForFrame(this: void): Promise<void>;
}

export interface HarnessOptions<Exports extends HarnessExports> {
  wasmPath: string;
  buildMode?: BuildMode;
  devToolsDomMirror?: DevToolsDomMirrorMode;
  pageZoom?: PageZoomMode;
  run?(this: void, exports: Exports): void;
  onStateUpdated?(this: void, state: HarnessState): void;
  onReady?(this: void, context: HarnessContext<Exports>): void | Promise<void>;
  onDispose?(this: void, exports: Exports): void;
  onError?(this: void, error: unknown): void;
  showLoadingOverlay?: boolean;
  hostEvents?: HostEventsDefinition;
  hostServices?: HostServicesDefinition;
  workerHostServices?: WorkerHostServicesBundleConfig;
  persistedRestoreMode?: 'initial' | 'pop' | 'none';
}

export interface HarnessAppOptions<Exports extends HarnessExports> extends HarnessOptions<Exports> {
  run(this: void, exports: Exports): void;
}

export type HarnessNavigationMode = 'push' | 'replace' | 'pop';

export interface HarnessController {
  readonly runtime: BridgeRuntime;
  waitForFrame(this: void): Promise<void>;
  loadApp<Exports extends HarnessExports>(this: void, options: HarnessAppOptions<Exports>): Promise<HarnessContext<Exports>>;
  unloadApp(this: void): Promise<void>;
  recreateRuntime(this: void): Promise<BridgeRuntime>;
  setSameOriginNavigationHandler(
    handler: ((target: URL, mode: HarnessNavigationMode) => void | Promise<void>) | null,
  ): void;
}

export interface ManagedHarnessOptions {
  buildMode?: BuildMode;
  devToolsDomMirror?: DevToolsDomMirrorMode;
  pageZoom?: PageZoomMode;
  onReady?(this: void, controller: HarnessController): void | Promise<void>;
  onError?(this: void, error: unknown): void;
}

export interface ManagedHistoryState {
  readonly href: string;
  readonly uiSnapshotId?: string;
}

export interface HarnessDebugApi {
  flush(): Promise<void>;
  getDebugTree(): Promise<DebugTreeSnapshot>;
  externalDragEvent(
    type: number,
    handle: WasmHandleLike,
    x: number,
    y: number,
    files: readonly HarnessDebugExternalFile[],
  ): Promise<number>;
  pointerEvent(type: number, handle: WasmHandleLike, x: number, y: number, modifiers?: number): Promise<void>;
  focusChanged(handle: WasmHandleLike, focused: boolean): Promise<void>;
  keyEvent(type: number, key: string, modifiers?: number): Promise<void>;
  navigateTo(target: string): Promise<void>;
  scroll(
    handle: WasmHandleLike,
    offsetX: number,
    offsetY: number,
    contentWidth: number,
    contentHeight: number,
    viewportWidth: number,
    viewportHeight: number,
  ): Promise<void>;
}

export interface HarnessDebugExternalFile {
  readonly name: string;
  readonly type?: string;
  readonly text: string;
}

declare global {
  interface Window {
    __fui_debug?: HarnessDebugApi;
    __fuiUrlPreviewText?: string;
  }
}
