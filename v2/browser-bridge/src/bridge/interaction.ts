import { enrichClipboardPayload,writeClipboardPayload } from '../clipboard';
import type {
  BridgeRuntime,
  ClipboardWritePayload,
  EffinDomCallbacks,
  PointerEventLog,
  ScrollEventLog,
  WasmHandleLike,
} from '../core-types';
import { createEditorSession, type EditorSession } from './interaction/editor-session';
import { createBridgeLogs } from './interaction/logs';
import { handleToString } from './utils/encoding';

function isHandledResult(value: unknown): boolean {
  return value === true || value === 1;
}

export function installCallbacks(runtimeRef: { current: BridgeRuntime | null }): EditorSession {
  const logs = createBridgeLogs();
  const editorSession = createEditorSession(runtimeRef, logs);

  const callbacks: EffinDomCallbacks = {
    onPointerEvent: (handle, eventType) => {
      const { x, y } = editorSession.getLastPointerPosition();
      const modifiers = editorSession.getLastPointerModifiers();
      const pending = window.__effindomPendingPointerMetadata;
      delete window.__effindomPendingPointerMetadata;
      window.__effindomLastPointerEventHandled = isHandledResult(window.__effindomCallbacks?.onPointerEventWithMetadata?.(
        eventType,
        handle,
        x,
        y,
        pending?.modifiers ?? modifiers,
        pending?.pointerId ?? -1,
        pending?.pointerType ?? 0,
        pending?.button ?? 0,
        pending?.buttons ?? 0,
        pending?.pressure ?? 0,
        pending?.width ?? 0,
        pending?.height ?? 0,
        pending?.clickCount ?? 0,
      ));
    },
    onPointerEventWithMetadata: (
      eventType,
      handle,
      x,
      y,
      modifiers,
      pointerId,
      pointerType,
      button,
      buttons,
      pressure,
      width,
      height,
      clickCount,
    ) => {
      const entry: PointerEventLog = {
        handle: handleToString(handle),
        eventType,
        x,
        y,
        modifiers,
        pointerId,
        pointerType,
        button,
        buttons,
        pressure,
        width,
        height,
        clickCount,
      };
      logs.pointerEvents.push(entry);
      return false;
    },
    onFocusChanged: (handle: WasmHandleLike, isFocused: boolean) => {
      editorSession.handleFocusChanged(handle, isFocused);
    },
    onTextChanged: (handle: WasmHandleLike, text: string) => {
      editorSession.handleTextChanged(handle, text);
    },
    onRequestSemanticAnnouncement: (handle: WasmHandleLike) => {
      editorSession.handleRequestSemanticAnnouncement(handle);
    },
    onTextReplaced: (handle: WasmHandleLike, start: number, end: number, text: string) => {
      editorSession.handleTextReplaced(handle, start, end, text);
    },
    onSelectionChanged: (handle: WasmHandleLike, start: number, end: number) => {
      editorSession.handleSelectionChanged(handle, start, end);
    },
    onScroll: (handle, offsetX, offsetY, contentWidth, contentHeight, viewportWidth, viewportHeight) => {
      const entry: ScrollEventLog = {
        handle: handleToString(handle),
        offsetX,
        offsetY,
        contentWidth,
        contentHeight,
        viewportWidth,
        viewportHeight,
      };
      logs.scrollEvents.push(entry);
    },
    onCrossSelectionChanged: (areaHandle, text) => {
      logs.crossSelectionChanges.push({ areaHandle: handleToString(areaHandle), text });
    },
    onClipboardWrite: (payload: ClipboardWritePayload) => {
      logs.clipboardWrites.push(payload.plainText);
      const runtime = runtimeRef.current;
      const enrichedPayload =
        runtime === null
          ? payload
          : enrichClipboardPayload(payload, (fontId) => runtime.getClipboardFontUrl(fontId));
      void writeClipboardPayload(enrichedPayload).catch(() => undefined);
    },
    onClipboardRead: (handle: WasmHandleLike) => {
      editorSession.handleClipboardRead(handle);
    },
    onRequestFontLoad: (fontId, url) => {
      const runtime = runtimeRef.current;
      if (runtime === null || url.length === 0) {
        return;
      }
      void runtime.loadFont(fontId, url).catch((error: unknown) => {
        const message = error instanceof Error ? error.message : String(error);
        console.error(`[fui_host] font ${String(fontId)} failed lazy load from ${url}: ${message}`);
      });
    },
    onMissingFontCoverage: (fontId, coverageKind, sampleText) => {
      const runtime = runtimeRef.current;
      if (runtime === null) {
        return;
      }
      logs.missingFontCoverageRequests.push({
        fontId,
        coverageKind,
        sampleText,
      });
      runtime.handleMissingFontCoverage(fontId, coverageKind, sampleText);
    },
  };

  window.__effindomCallbacks = callbacks;
  return editorSession;
}
