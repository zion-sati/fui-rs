import type { BridgeRuntime } from '@effindomv2/runtime';

import type { WorkerManager } from './worker-manager';
import type { HarnessExports } from './types';
import type { PersistedUiStateController } from './persisted-ui-state-controller';
import type { TextSessionBridge } from './text-session-bridge';
import type { HarnessUiChrome } from './ui-chrome';
import { toBigIntHandle, type AppHandleLike } from './interop';

interface HostImportSessionLike {
  readonly exports: HarnessExports;
  readonly memory: WebAssembly.Memory;
  readonly textBufferPtr: number;
  readonly textBufferSize: number;
}

export interface HostImportDeps {
  getRuntime(): BridgeRuntime;
  getCurrentSession(): HostImportSessionLike;
  getCurrentSessionOrNull(): HostImportSessionLike | null;
  setAppFlushRequested(value: boolean): void;
  queueHarnessFrame(): void;
  uiChrome: HarnessUiChrome;
  readAppUtf8(ptr: number, len: number): string;
  writeAppFloat32(ptr: number, value: number): void;
  writeAppUint32(ptr: number, value: number): void;
  writeAppUtf8(ptr: number, capacity: number, text: string, context: string): number;
  textBridge: TextSessionBridge;
  persistedUiStateController: PersistedUiStateController;
  navigateWithinDocument(target: string, openInNewTab: boolean): void;
  canBrowserNavigateBack(): boolean;
  canBrowserNavigateForward(): boolean;
  navigateBrowserBack(): void;
  navigateBrowserForward(): void;
  cancelHostTimer(timerId: number): void;
  getHostTimer(timerId: number): number | undefined;
  setHostTimer(timerId: number, timeoutId: number): void;
  deleteHostTimer(timerId: number): void;
  workerManager: WorkerManager;
  debugLogsEnabled: boolean;
  notifySvgLoaded(session: HostImportSessionLike | null, svgId: number, width: number, height: number): void;
  notifySvgFailed(session: HostImportSessionLike | null, svgId: number, error: string): void;
  notifyTextureLoaded(session: HostImportSessionLike | null, textureId: number, width: number, height: number): void;
  notifyTextureFailed(session: HostImportSessionLike | null, textureId: number, error: string): void;
}

export function createHostImportModule(deps: HostImportDeps) {
  function isPasswordTextInput(handle: AppHandleLike): boolean {
    const metadata = deps.getRuntime().getTextInputMetadata(toBigIntHandle(handle).toString());
    return metadata?.kind === 'password';
  }

  function isActivePasswordTextInput(): boolean {
    const activeHandle = deps.getRuntime().getActiveTextHandle();
    return activeHandle !== null && deps.getRuntime().getTextInputMetadata(activeHandle.toString())?.kind === 'password';
  }

  function safeNotify(label: string, notify: () => void): void {
    try {
      notify();
    } catch (error: unknown) {
      const message = error instanceof Error ? error.message : String(error);
      console.error(`[fui_host] ${label}: ${message}`);
    }
  }

  return {
    request_render(): void {
      const runtime = deps.getRuntime();
      deps.setAppFlushRequested(true);
      runtime.requestFrame();
      deps.queueHarnessFrame();
    },
    get_viewport_width(): number {
      const runtime = deps.getRuntime();
      const sizeSource = deps.uiChrome.getCanvasSizeSource(runtime.canvas);
      const rect = sizeSource.getBoundingClientRect();
      return sizeSource.clientWidth > 0 ? sizeSource.clientWidth : (rect.width > 0 ? rect.width : runtime.canvas.width);
    },
    get_viewport_height(): number {
      const runtime = deps.getRuntime();
      const sizeSource = deps.uiChrome.getCanvasSizeSource(runtime.canvas);
      const rect = sizeSource.getBoundingClientRect();
      return sizeSource.clientHeight > 0 ? sizeSource.clientHeight : (rect.height > 0 ? rect.height : runtime.canvas.height);
    },
    get_device_pixel_ratio(): number {
      return window.devicePixelRatio > 0 ? window.devicePixelRatio : 1;
    },
    fui_set_pointer_capture(handle: AppHandleLike): void {
      deps.getRuntime().setCapturedPointerHandle(toBigIntHandle(handle));
    },
    fui_release_pointer_capture(): void {
      deps.getRuntime().setCapturedPointerHandle(null);
    },
    fui_reload_page(): void {
      window.location.reload();
    },
    fui_can_navigate_back(): number {
      return deps.canBrowserNavigateBack() ? 1 : 0;
    },
    fui_can_navigate_forward(): number {
      return deps.canBrowserNavigateForward() ? 1 : 0;
    },
    fui_navigate_back(): void {
      deps.navigateBrowserBack();
    },
    fui_navigate_forward(): void {
      deps.navigateBrowserForward();
    },
    fui_copy_text(ptr: number, len: number): void {
      const text = deps.readAppUtf8(ptr, len);
      window.__effindomCallbacks?.onClipboardWrite?.({ plainText: text });
    },
    fui_has_text_selection_snapshot(handle: AppHandleLike): number {
      if (isPasswordTextInput(handle)) {
        return 0;
      }
      return deps.textBridge.resolveFrozenOrLiveTextSelection(handle) !== null ? 1 : 0;
    },
    fui_freeze_text_selection_snapshot(handle: AppHandleLike): void {
      if (isPasswordTextInput(handle)) {
        deps.textBridge.clearFrozenTextSelectionSnapshot();
        return;
      }
      deps.textBridge.freezeTextSelectionSnapshot(handle);
    },
    fui_copy_text_selection_snapshot(handle: AppHandleLike): number {
      if (isPasswordTextInput(handle)) {
        return 0;
      }
      const snapshot = deps.textBridge.resolveFrozenOrLiveTextSelection(handle);
      if (snapshot === null) {
        return 0;
      }
      window.__effindomCallbacks?.onClipboardWrite?.({
        plainText: snapshot.text.slice(snapshot.start, snapshot.end),
      });
      return 1;
    },
    fui_cut_focused_text_selection(): number {
      if (isActivePasswordTextInput()) {
        return 0;
      }
      const editor = deps.textBridge.getHiddenTextEditor();
      if (editor === null) {
        return 0;
      }
      const selectionStart = editor.selectionStart ?? 0;
      const selectionEnd = editor.selectionEnd ?? 0;
      if (selectionStart === selectionEnd) {
        return 0;
      }
      const start = Math.min(selectionStart, selectionEnd);
      const end = Math.max(selectionStart, selectionEnd);
      window.__effindomCallbacks?.onClipboardWrite?.({
        plainText: editor.value.slice(start, end),
      });
      editor.focus({ preventScroll: true });
      editor.setRangeText('', start, end, 'start');
      editor.dispatchEvent(new InputEvent('input', { bubbles: true, inputType: 'deleteByCut', data: null }));
      return 1;
    },
    fui_cut_text_selection_snapshot(handle: AppHandleLike): number {
      if (isPasswordTextInput(handle)) {
        deps.textBridge.clearFrozenTextSelectionSnapshot();
        return 0;
      }
      const runtime = deps.getRuntime();
      const snapshot = deps.textBridge.resolveFrozenOrLiveTextSelection(handle);
      if (snapshot === null) {
        return 0;
      }
      const { handleKey, text, start, end } = snapshot;
      window.__effindomCallbacks?.onClipboardWrite?.({
        plainText: text.slice(start, end),
      });
      const updatedText = text.slice(0, start) + text.slice(end);
      const editor = deps.textBridge.getHiddenTextEditor();
      if (editor !== null) {
        editor.focus({ preventScroll: true });
        editor.value = updatedText;
        editor.setSelectionRange(start, start, 'none');
      }
      window.setTimeout(() => {
        runtime.ui._ui_request_focus(toBigIntHandle(handle));
        deps.textBridge.withUiUtf8('', (uiPtr, uiLen) => {
          runtime.ui._ui_replace_text_range(toBigIntHandle(handle), start, end, uiPtr, uiLen, start);
        });
        runtime.commitFrame();
        deps.queueHarnessFrame();
        deps.textBridge.updateLiveTextAfterCut(handleKey, updatedText, start);
        const activeEditor = deps.textBridge.getHiddenTextEditor();
        if (activeEditor !== null) {
          activeEditor.focus({ preventScroll: true });
          activeEditor.setSelectionRange(start, start, 'none');
        }
      }, 0);
      deps.textBridge.clearFrozenTextSelectionSnapshot();
      return 1;
    },
    fui_cut_text_range_snapshot(handle: AppHandleLike, start: number, end: number): number {
      if (isPasswordTextInput(handle)) {
        return 0;
      }
      const textSnapshot = deps.textBridge.resolveFrozenOrLiveTextSelection(handle);
      const handleKey = toBigIntHandle(handle).toString();
      const text = textSnapshot?.handleKey === handleKey
        ? textSnapshot.text
        : deps.textBridge.getLatestText(handle);
      const resolvedText = text.length > 0 ? text : '';
      if (resolvedText.length === 0) {
        return 0;
      }
      const rangeStart = Math.max(0, Math.min(start, end));
      const rangeEnd = Math.max(rangeStart, Math.min(resolvedText.length, Math.max(start, end)));
      if (rangeStart === rangeEnd) {
        return 0;
      }
      window.__effindomCallbacks?.onClipboardWrite?.({
        plainText: resolvedText.slice(rangeStart, rangeEnd),
      });
      const updatedText = resolvedText.slice(0, rangeStart) + resolvedText.slice(rangeEnd);
      const editor = deps.textBridge.getHiddenTextEditor();
      if (editor !== null) {
        editor.focus({ preventScroll: true });
        editor.value = updatedText;
        editor.setSelectionRange(rangeStart, rangeStart, 'none');
      }
      window.setTimeout(() => {
        deps.getRuntime().ui._ui_request_focus(toBigIntHandle(handle));
        deps.textBridge.syncEditableTextToRuntime(handle, updatedText, rangeStart);
        deps.textBridge.updateLiveTextAfterCut(handleKey, updatedText, rangeStart);
        const activeEditor = deps.textBridge.getHiddenTextEditor();
        if (activeEditor !== null) {
          activeEditor.focus({ preventScroll: true });
          activeEditor.setSelectionRange(rangeStart, rangeStart, 'none');
        }
      }, 0);
      return 1;
    },
    fui_delete_focused_text_range(start: number, end: number): number {
      const editor = deps.textBridge.getHiddenTextEditor();
      if (editor === null) {
        return 0;
      }
      const rangeStart = Math.max(0, Math.min(start, end));
      const rangeEnd = Math.max(rangeStart, Math.max(start, end));
      editor.focus({ preventScroll: true });
      editor.setSelectionRange(rangeStart, rangeEnd);
      editor.setRangeText('', rangeStart, rangeEnd, 'start');
      editor.dispatchEvent(new InputEvent('input', { bubbles: true, inputType: 'deleteByCut', data: null }));
      return 1;
    },
    fui_commit_text_action_focus(handle: AppHandleLike): void {
      const runtime = deps.getRuntime();
      window.setTimeout(() => {
        runtime.ui._ui_request_focus(toBigIntHandle(handle));
        runtime.commitFrame();
        deps.queueHarnessFrame();
        const editor = deps.textBridge.getHiddenTextEditor();
        if (editor !== null) {
          editor.focus({ preventScroll: true });
        }
      }, 0);
    },
    fui_register_text_input_metadata(handle: AppHandleLike, isPassword: boolean, hintPtr: number, hintLen: number): void {
      const hostAutofillHint = hintLen > 0 ? deps.readAppUtf8(hintPtr, hintLen) : null;
      deps.getRuntime().setTextInputMetadata(toBigIntHandle(handle).toString(), {
        kind: isPassword ? 'password' : (hostAutofillHint === 'email' ? 'email' : 'text'),
        hostAutofillHint,
      });
    },
    fui_load_svg(svgId: number, ptr: number, len: number): void {
      const runtime = deps.getRuntime();
      const session = deps.getCurrentSessionOrNull();
      const url = deps.readAppUtf8(ptr, len);
      void runtime.loadSvg(svgId, url).then((result) => {
        if (deps.getCurrentSessionOrNull() !== session) {
          return;
        }
        safeNotify(`failed to deliver SVG ${String(svgId)} success callback`, () => {
          deps.notifySvgLoaded(session, svgId, result.width, result.height);
        });
      }).catch((error: unknown) => {
        const message = error instanceof Error ? error.message : String(error);
        console.error(`[fui_host] SVG ${String(svgId)} failed to load from ${url}: ${message}`);
        if (deps.getCurrentSessionOrNull() !== session) {
          return;
        }
        safeNotify(`failed to deliver SVG ${String(svgId)} failure callback`, () => {
          deps.notifySvgFailed(session, svgId, message);
        });
      });
    },
    fui_load_texture(textureId: number, ptr: number, len: number): void {
      const runtime = deps.getRuntime();
      const session = deps.getCurrentSessionOrNull();
      const url = deps.readAppUtf8(ptr, len);
      void runtime.loadTexture(textureId, url).then((result) => {
        if (deps.getCurrentSessionOrNull() !== session) {
          return;
        }
        safeNotify(`failed to deliver texture ${String(textureId)} success callback`, () => {
          deps.notifyTextureLoaded(session, textureId, result.width, result.height);
        });
      }).catch((error: unknown) => {
        const message = error instanceof Error ? error.message : String(error);
        console.error(`[fui_host] texture ${String(textureId)} failed to load from ${url}: ${message}`);
        if (deps.getCurrentSessionOrNull() !== session) {
          return;
        }
        safeNotify(`failed to deliver texture ${String(textureId)} failure callback`, () => {
          deps.notifyTextureFailed(session, textureId, message);
        });
      });
    },
    fui_release_svg(svgId: number): void {
      deps.getRuntime().releaseSvg(svgId);
    },
    fui_release_texture(textureId: number): void {
      deps.getRuntime().releaseTexture(textureId);
    },
    fui_load_font(fontId: number, ptr: number, len: number): void {
      const url = deps.readAppUtf8(ptr, len);
      deps.getRuntime().registerLazyFont(fontId, url);
    },
    fui_start_timer(timerId: number, delayMs: number): void {
      deps.cancelHostTimer(timerId);
      const session = deps.getCurrentSessionOrNull();
      const clampedDelayMs = Math.max(0, Math.ceil(delayMs));
      const timeoutId = window.setTimeout(() => {
        if (deps.getHostTimer(timerId) !== timeoutId) {
          return;
        }
        deps.deleteHostTimer(timerId);
        if (session === null || deps.getCurrentSessionOrNull() !== session) {
          return;
        }
        session.exports.__fui_on_timer(timerId);
      }, clampedDelayMs);
      deps.setHostTimer(timerId, timeoutId);
    },
    fui_cancel_timer(timerId: number): void {
      deps.cancelHostTimer(timerId);
    },
    fui_now_ms(): number {
      return performance.now();
    },
    fui_worker_start_string(workerId: number, wasmPathPtr: number, wasmPathLen: number, entryPtr: number, entryLen: number, inputPtr: number, inputLen: number): void {
      deps.workerManager.startString(
        workerId,
        deps.readAppUtf8(wasmPathPtr, wasmPathLen),
        deps.readAppUtf8(entryPtr, entryLen),
        deps.readAppUtf8(inputPtr, inputLen),
      );
    },
    fui_worker_cancel(workerId: number): void {
      deps.workerManager.cancel(workerId);
    },
    fui_set_cursor(style: number): void {
      if (deps.uiChrome.detectCoarsePointer()) {
        return;
      }
      const cursor =
        style === 1 ? 'pointer' :
        style === 2 ? 'text' :
        style === 3 ? 'move' :
        style === 4 ? 'grab' :
        style === 5 ? 'grabbing' :
        style === 6 ? 'ns-resize' :
        style === 7 ? 'ew-resize' :
        'default';
      deps.getRuntime().canvas.style.cursor = cursor;
    },
    fui_is_dark_mode(): number {
      return window.matchMedia('(prefers-color-scheme: dark)').matches ? 1 : 0;
    },
    fui_get_accent_color(): number {
      return deps.uiChrome.readHostAccentColor();
    },
    fui_get_platform_family(): number {
      return deps.uiChrome.detectPlatformFamily();
    },
    fui_is_coarse_pointer(): number {
      return deps.uiChrome.detectCoarsePointer() ? 1 : 0;
    },
    fui_show_url_preview(ptr: number, len: number): void {
      const rawTarget = deps.readAppUtf8(ptr, len);
      try {
        const resolvedTarget = new URL(rawTarget, window.location.href);
        deps.uiChrome.setUrlPreviewText(resolvedTarget.href);
      } catch {
        deps.uiChrome.setUrlPreviewText(rawTarget);
      }
    },
    fui_hide_url_preview(): void {
      deps.uiChrome.setUrlPreviewText('');
    },
    fui_navigate_to(ptr: number, len: number, openInNewTab: number): void {
      deps.navigateWithinDocument(deps.readAppUtf8(ptr, len), openInNewTab !== 0);
    },
    fui_set_persisted_scroll_offset(nodeIdPtr: number, nodeIdLen: number, x: number, y: number): void {
      const nodeId = deps.readAppUtf8(nodeIdPtr, nodeIdLen);
      if (nodeId.length === 0) {
        return;
      }
      deps.persistedUiStateController.setCurrentPersistedScrollEntry(nodeId, x, y);
    },
    fui_try_get_persisted_scroll_offset(nodeIdPtr: number, nodeIdLen: number, outX: number, outY: number): number {
      const nodeId = deps.readAppUtf8(nodeIdPtr, nodeIdLen);
      if (nodeId.length === 0) {
        return 0;
      }
      const payload = deps.persistedUiStateController.getCurrentPersistedScrollEntry(nodeId);
      if (payload === null) {
        return 0;
      }
      deps.writeAppFloat32(outX, payload.x);
      deps.writeAppFloat32(outY, payload.y);
      return 1;
    },
    fui_set_persisted_state(
      nodeIdPtr: number,
      nodeIdLen: number,
      kindPtr: number,
      kindLen: number,
      version: number,
      payloadPtr: number,
      payloadLen: number,
    ): void {
      const nodeId = deps.readAppUtf8(nodeIdPtr, nodeIdLen);
      const kind = deps.readAppUtf8(kindPtr, kindLen);
      if (nodeId.length === 0 || kind.length === 0) {
        return;
      }
      deps.persistedUiStateController.setCurrentPersistedTextEntry(
        nodeId,
        kind,
        version >>> 0,
        deps.readAppUtf8(payloadPtr, payloadLen),
      );
    },
    fui_copy_persisted_state(
      nodeIdPtr: number,
      nodeIdLen: number,
      kindPtr: number,
      kindLen: number,
      outVersionPtr: number,
      payloadPtr: number,
      payloadCapacity: number,
    ): number {
      const nodeId = deps.readAppUtf8(nodeIdPtr, nodeIdLen);
      const kind = deps.readAppUtf8(kindPtr, kindLen);
      if (nodeId.length === 0 || kind.length === 0) {
        return -1;
      }
      const entry = deps.persistedUiStateController.getCurrentPersistedTextEntry(nodeId, kind);
      if (entry === null) {
        return -1;
      }
      deps.writeAppUint32(outVersionPtr, entry.version >>> 0);
      const payloadBytes = new TextEncoder().encode(entry.payload);
      if (payloadBytes.length > payloadCapacity) {
        return payloadBytes.length;
      }
      deps.writeAppUtf8(payloadPtr, payloadCapacity, entry.payload, `Persisted state ${kind}`);
      return payloadBytes.length;
    },
    fui_log(catPtr: number, catLen: number, msgPtr: number, msgLen: number): void {
      if (deps.getCurrentSessionOrNull() === null) {
        return;
      }
      const category = deps.readAppUtf8(catPtr, catLen);
      const message = deps.readAppUtf8(msgPtr, msgLen);
      const formatted = `[fui:${category}] ${message}`;
      if (category.startsWith('Warning/')) {
        console.warn(formatted);
        return;
      }
      if (category.startsWith('Error/')) {
        console.error(formatted);
        return;
      }
      if (deps.debugLogsEnabled) {
        console.debug(formatted);
      }
    },
    fui_logs_enabled(): number {
      return deps.debugLogsEnabled ? 1 : 0;
    },
  };
}
