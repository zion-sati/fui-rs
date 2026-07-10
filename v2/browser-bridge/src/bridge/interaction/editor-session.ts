import type {
BridgeLogs,
BridgeRuntime,
FocusEventLog,
SelectionChangeLog,
WasmHandleLike,
} from '../../core-types';
import type { BridgeInteractionState, EditorDomTarget } from '../local-types';
import { handleToBigInt } from '../utils/encoding';
import { writeUtf8ToHeap } from '../utils/heap';
import {
applyUtf8ByteReplacementEdit,
buildClampedTextboxEdit,
buildHiddenEditorWindow,
createSingleHiddenEditorTarget,
type HiddenEditorWindow,
type HiddenTextEditor,
type PendingLocalReplacementEcho,
type PendingLocalSelectionEcho,
summarizeTextChange,
utf8ByteOffsetToCodeUnitIndex,
} from './editor-model';
import { createEditorMutationController } from './editor-mutations';
import { codeUnitIndexToUtf8ByteOffset,utf8ByteLength } from './text-encoding';

export interface EditorSession extends BridgeInteractionState {
  handleClipboardRead(handle: WasmHandleLike): void;
  handleFocusChanged(handle: WasmHandleLike, isFocused: boolean): void;
  handleRequestSemanticAnnouncement(handle: WasmHandleLike): void;
  handleSelectionChanged(handle: WasmHandleLike, start: number, end: number): void;
  handleTextChanged(handle: WasmHandleLike, text: string): void;
  handleTextReplaced(handle: WasmHandleLike, start: number, end: number, text: string): void;
}

const CARET_BLINK_INTERVAL_MS = 500;

function currentInteractionTimeMs(): bigint {
  return BigInt(Math.floor(performance.now()));
}

export function createEditorSession(
  runtimeRef: { current: BridgeRuntime | null },
  logs: BridgeLogs,
): EditorSession {
  const textByHandle = Object.create(null) as Record<string, string>;
  const selectionsByHandle = Object.create(null) as Record<string, { start: number; end: number }>;
  const domTarget: EditorDomTarget = createSingleHiddenEditorTarget();
  let activeTextHandle: bigint | null = null;
  let lastPointerClientX: number | null = null;
  let lastPointerClientY: number | null = null;
  let lastPointerX = 0;
  let lastPointerY = 0;
  let lastPointerModifiers = 0;
  let lastInteractivePointerHandle: bigint | null = null;
  let capturedPointerHandle: bigint | null = null;
  let pointerInsideCanvas = false;
  let appSessionVersion = 0;
  let activeTextEditable = false;
  let activeTextMultiline = false;
  let activeEditorWindow: HiddenEditorWindow = { text: '', docStart: 0, docEnd: 0, textStart: 0, textEnd: 0 };
  const textByteLengthsByHandle = Object.create(null) as Record<string, number>;
  const pendingCaretRevealByHandle = Object.create(null) as Record<string, boolean>;
  let pendingCaretRevealFrame: number | null = null;
  let pendingLocalReplacementEcho: PendingLocalReplacementEcho | null = null;
  let pendingLocalSelectionEcho: PendingLocalSelectionEcho | null = null;
  let pendingProjectedReplacementEcho: (PendingLocalReplacementEcho & { nextText: string }) | null = null;
  let deferredTouchFocusHandle: string | null = null;
  let caretBlinkTimer: ReturnType<typeof setTimeout> | null = null;
  let focusedHandle: string | null = null;
  const pendingSemanticAnnouncements = new Set<string>();
  window.__bridgeLogs = logs;
  window.__bridgeTextByHandle = textByHandle;
  window.__bridgeSelectionsByHandle = selectionsByHandle;
  window.__bridgeActiveEditorWindow = { handle: null, ...activeEditorWindow };

  const getActiveEditor = (): HiddenTextEditor => domTarget.getEditor(activeTextHandle?.toString() ?? null, activeTextMultiline);

  const shouldWaitForProjectedEditor = (): boolean => {
    if (activeTextHandle === null) {
      return false;
    }
    const runtime = runtimeRef.current;
    if (runtime === null) {
      return false;
    }
    const document = runtime.openCanvasApi.getEditableTextDocument(activeTextHandle.toString());
    if (document?.autofillHint === null || document?.autofillHint === undefined || document.formHandle === null) {
      return false;
    }
    return !domTarget.hasSemanticTextEditor(activeTextHandle.toString());
  };

  const isActiveEditorFocused = (): boolean => document.activeElement === getActiveEditor();

  const queueSemanticAnnouncement = (handleKey: string): void => {
    pendingSemanticAnnouncements.add(handleKey);
    runtimeRef.current?.requestFrame();
  };

  const clampSelectionToText = (
    length: number,
    selection: { start: number; end: number },
  ): { start: number; end: number } => ({
    start: Math.max(0, Math.min(selection.start, length)),
    end: Math.max(0, Math.min(selection.end, length)),
  });

  const clearCaretBlinkTimer = (): void => {
    if (caretBlinkTimer !== null) {
      clearTimeout(caretBlinkTimer);
      caretBlinkTimer = null;
    }
  };

  const shouldRunCaretBlinkTimer = (): boolean => {
    if (activeTextHandle === null || !isActiveEditorFocused()) {
      return false;
    }
    const handleKey = activeTextHandle.toString();
    const text = textByHandle[handleKey] ?? '';
    const textByteLength = textByteLengthsByHandle[handleKey] ?? utf8ByteLength(text);
    const selection = selectionsByHandle[handleKey] ?? { start: textByteLength, end: textByteLength };
    const { start, end } = clampSelectionToText(textByteLength, selection);
    return start === end;
  };

  const armCaretBlinkTimer = (): void => {
    if (caretBlinkTimer !== null) {
      return;
    }
    caretBlinkTimer = setTimeout(() => {
      caretBlinkTimer = null;
      if (!shouldRunCaretBlinkTimer()) {
        return;
      }
      runtimeRef.current?.requestFrame();
      armCaretBlinkTimer();
    }, CARET_BLINK_INTERVAL_MS);
  };

  const updateCaretBlinkTimer = (resetPhase = false): void => {
    if (!shouldRunCaretBlinkTimer()) {
      clearCaretBlinkTimer();
      return;
    }
    if (resetPhase) {
      clearCaretBlinkTimer();
    }
    armCaretBlinkTimer();
  };

  const syncActiveEditorWindowDebug = (handle: string | null): void => {
    window.__bridgeActiveEditorWindow = {
      handle,
      text: activeEditorWindow.text,
      docStart: activeEditorWindow.docStart,
      docEnd: activeEditorWindow.docEnd,
    };
  };

  const clearActiveEditorWindow = (): void => {
    activeEditorWindow = { text: '', docStart: 0, docEnd: 0, textStart: 0, textEnd: 0 };
    syncActiveEditorWindowDebug(null);
  };

  const clearPendingCaretReveal = (): void => {
    if (pendingCaretRevealFrame !== null) {
      cancelAnimationFrame(pendingCaretRevealFrame);
      pendingCaretRevealFrame = null;
    }
    for (const key of Object.keys(pendingCaretRevealByHandle)) {
      Reflect.deleteProperty(pendingCaretRevealByHandle, key);
    }
  };

  const updateActiveEditorWindowText = (text: string): void => {
    activeEditorWindow = {
      text,
      docStart: activeEditorWindow.docStart,
      docEnd: activeEditorWindow.docStart + utf8ByteLength(text),
      textStart: activeEditorWindow.textStart,
      textEnd: activeEditorWindow.textStart + text.length,
    };
    syncActiveEditorWindowDebug(activeTextHandle?.toString() ?? null);
  };

  const detachBridgeTextInput = (): void => {
    domTarget.detach();
  };

  const syncActiveTextInputViewport = (): void => {
    void 0;
  };

  const getTextboxState = (
    handleKey: string,
  ): { isTextbox: boolean; isEditable: boolean; isMultiline: boolean } => {
    const runtime = runtimeRef.current;
    if (runtime === null) {
      return { isTextbox: false, isEditable: false, isMultiline: false };
    }
    const document = runtime.openCanvasApi.getEditableTextDocument(handleKey);
    if (document === null) {
      return { isTextbox: false, isEditable: false, isMultiline: false };
    }
    return {
      isTextbox: true,
      isEditable: !document.readOnly && !document.disabled,
      isMultiline: document.multiline,
    };
  };

  const syncFocusedInputState = (): void => {
    if (activeTextHandle === null) {
      clearCaretBlinkTimer();
      clearActiveEditorWindow();
      detachBridgeTextInput();
      domTarget.clearAll();
      return;
    }

    const activeEditor = getActiveEditor();
    const handleKey = activeTextHandle.toString();
    const document = runtimeRef.current?.openCanvasApi.getEditableTextDocument(handleKey) ?? null;
    const text = textByHandle[handleKey] ?? document?.text ?? '';
    const textByteLength = textByteLengthsByHandle[handleKey] ?? utf8ByteLength(text);
    textByHandle[handleKey] ??= text;
    textByteLengthsByHandle[handleKey] = textByteLength;
    const selection = selectionsByHandle[handleKey] ?? { start: textByteLength, end: textByteLength };
    const { start: startByte, end: endByte } = clampSelectionToText(textByteLength, selection);
    const start = utf8ByteOffsetToCodeUnitIndex(text, startByte, textByteLength);
    const end = utf8ByteOffsetToCodeUnitIndex(text, endByte, textByteLength);
    const direction = start === end ? 'none' : (startByte < endByte ? 'forward' : 'backward');
    const normalizedStart = Math.min(start, end);
    const normalizedEnd = Math.max(start, end);
    activeEditorWindow = buildHiddenEditorWindow(text, normalizedStart, normalizedEnd, textByteLength, activeEditorWindow);
    syncActiveEditorWindowDebug(handleKey);
    const localStart = normalizedStart - activeEditorWindow.textStart;
    const localEnd = normalizedEnd - activeEditorWindow.textStart;
    syncActiveEditorDomAttributes();
    if (activeEditor.value !== activeEditorWindow.text) {
      activeEditor.value = activeEditorWindow.text;
    }
    if (activeEditor.readOnly !== !activeTextEditable) {
      activeEditor.readOnly = !activeTextEditable;
    }
    const currentSelectionStart = activeEditor.selectionStart ?? 0;
    const currentSelectionEnd = activeEditor.selectionEnd ?? currentSelectionStart;
    const currentSelectionDirection = activeEditor.selectionDirection ?? 'none';
    if (
      currentSelectionStart !== localStart ||
      currentSelectionEnd !== localEnd ||
      currentSelectionDirection !== direction
    ) {
      activeEditor.setSelectionRange(localStart, localEnd, direction);
    }
    updateCaretBlinkTimer();
  };

  const mutationController = createEditorMutationController({
    runtimeRef,
    textByHandle,
    selectionsByHandle,
    textByteLengthsByHandle,
    getActiveEditor,
    getActiveEditorWindow: () => activeEditorWindow,
    getActiveTextEditable: () => activeTextEditable,
    getActiveTextHandle: () => activeTextHandle,
    isActiveEditorFocused,
    setPendingLocalReplacementEcho: (value) => {
      pendingLocalReplacementEcho = value;
    },
    setPendingLocalSelectionEcho: (value) => {
      pendingLocalSelectionEcho = value;
    },
    setPendingProjectedReplacementEcho: (value) => {
      pendingProjectedReplacementEcho = value;
    },
    syncFocusedInputState,
    updateActiveEditorWindowText,
  });

  const clearPendingTextMutations = (): void => {
    mutationController.clearPendingTextMutations();
  };

  const resetHiddenEditorDomAttributes = (editor: HiddenTextEditor): void => {
    if (editor instanceof HTMLInputElement) {
      editor.type = 'text';
    }
    editor.setAttribute('autocomplete', 'off');
    editor.removeAttribute('name');
    editor.removeAttribute('id');
  };

  const applyHiddenEditorDomAttributes = (
    editor: HiddenTextEditor,
    kind: 'text' | 'password' | 'email',
    autofillHint: string | null,
    stableFieldName: string | null,
  ): void => {
    if (editor instanceof HTMLInputElement && editor.type !== kind) {
      editor.type = kind;
    }
    const autocompleteValue = autofillHint ?? 'off';
    if (editor.getAttribute('autocomplete') !== autocompleteValue) {
      editor.setAttribute('autocomplete', autocompleteValue);
    }
    if (autofillHint !== null && stableFieldName !== null) {
      if (editor.getAttribute('name') !== stableFieldName) {
        editor.setAttribute('name', stableFieldName);
      }
      if (editor.getAttribute('id') !== stableFieldName) {
        editor.setAttribute('id', stableFieldName);
      }
      return;
    }
    if (editor.hasAttribute('name')) {
      editor.removeAttribute('name');
    }
    if (editor.hasAttribute('id')) {
      editor.removeAttribute('id');
    }
  };

  const syncActiveEditorDomAttributes = (): void => {
    const activeEditor = activeTextHandle === null ? null : getActiveEditor();
    if (domTarget.singleLineEditor !== activeEditor) {
      resetHiddenEditorDomAttributes(domTarget.singleLineEditor);
    }
    if (domTarget.multiLineEditor !== activeEditor) {
      resetHiddenEditorDomAttributes(domTarget.multiLineEditor);
    }
    if (activeTextHandle === null) {
      return;
    }
    const runtime = runtimeRef.current;
    if (runtime === null) {
      return;
    }
    const handleKey = activeTextHandle.toString();
    const document = runtime.openCanvasApi.getEditableTextDocument(handleKey);
    if (document === null || activeEditor === null) {
      return;
    }
    applyHiddenEditorDomAttributes(activeEditor, document.kind, document.autofillHint, document.stableFieldName);
  };

  const focusHiddenEditorNow = (): void => {
    if (shouldWaitForProjectedEditor()) {
      return;
    }
    syncFocusedInputState();
    domTarget.focus(activeTextHandle?.toString() ?? null, activeTextMultiline, { preventScroll: true });
    syncFocusedInputState();
    updateCaretBlinkTimer(true);
  };

  const refocusActiveTextInput = (): void => {
    if (activeTextHandle === null) {
      if (deferredTouchFocusHandle === null) {
        return;
      }
      const textboxState = getTextboxState(deferredTouchFocusHandle);
      if (!textboxState.isTextbox) {
        return;
      }
      activeTextHandle = handleToBigInt(deferredTouchFocusHandle);
      activeTextEditable = textboxState.isEditable;
      activeTextMultiline = textboxState.isMultiline;
      syncFocusedInputState();
      focusHiddenEditorNow();
      return;
    }
    focusHiddenEditorNow();
  };

  const beginTouchTextFocusDeferral = (handle: bigint): void => {
    deferredTouchFocusHandle = handle.toString();
  };

  const cancelTouchTextFocusDeferral = (): void => {
    deferredTouchFocusHandle = null;
  };

  const commitTouchTextFocusDeferral = (handle: bigint): void => {
    const handleKey = handle.toString();
    if (deferredTouchFocusHandle !== handleKey) {
      return;
    }
    deferredTouchFocusHandle = null;
    if (activeTextHandle?.toString() !== handleKey) {
      return;
    }
    if (!isActiveEditorFocused()) {
      focusHiddenEditorNow();
    }
  };

  const scheduleCaretRevealReplay = (handleKey: string): void => {
    if (!pendingCaretRevealByHandle[handleKey] || activeTextHandle?.toString() !== handleKey) {
      return;
    }
    if (pendingCaretRevealFrame !== null) {
      return;
    }
    pendingCaretRevealFrame = requestAnimationFrame(() => {
      pendingCaretRevealFrame = null;
      if (!pendingCaretRevealByHandle[handleKey] || activeTextHandle?.toString() !== handleKey) {
        return;
      }
      Reflect.deleteProperty(pendingCaretRevealByHandle, handleKey);
      const runtime = runtimeRef.current;
      if (runtime === null) {
        return;
      }
      const text = textByHandle[handleKey] ?? '';
      const textByteLength = utf8ByteLength(text);
      const selection = selectionsByHandle[handleKey] ?? { start: textByteLength, end: textByteLength };
      const { start, end } = clampSelectionToText(textByteLength, selection);
      runtime.ui._ui_reveal_text_range(handleToBigInt(handleKey), start, end);
    });
  };

  const clearRecordMap = <T>(record: Record<string, T>): void => {
    for (const key of Object.keys(record)) {
      Reflect.deleteProperty(record, key);
    }
  };

  const resetAppSession = (): void => {
    appSessionVersion += 1;
    clearRecordMap(textByHandle);
    clearRecordMap(textByteLengthsByHandle);
    clearRecordMap(selectionsByHandle);
    clearPendingCaretReveal();
    mutationController.reset();
    pendingLocalReplacementEcho = null;
    pendingLocalSelectionEcho = null;
    pendingProjectedReplacementEcho = null;
    deferredTouchFocusHandle = null;
    clearCaretBlinkTimer();
    activeTextHandle = null;
    activeTextEditable = false;
    activeTextMultiline = false;
    clearActiveEditorWindow();
    focusedHandle = null;
    pendingSemanticAnnouncements.clear();
    lastInteractivePointerHandle = null;
    capturedPointerHandle = null;
    domTarget.singleLineEditor.value = '';
    domTarget.singleLineEditor.setSelectionRange(0, 0, 'none');
    domTarget.multiLineEditor.value = '';
    domTarget.multiLineEditor.setSelectionRange(0, 0, 'none');
    resetHiddenEditorDomAttributes(domTarget.singleLineEditor);
    resetHiddenEditorDomAttributes(domTarget.multiLineEditor);
    detachBridgeTextInput();
  };
  domTarget.attachListeners((editor) => {
    mutationController.attachHiddenEditorListeners(editor);
  });

  const handleFocusChanged = (handle: WasmHandleLike, isFocused: boolean): void => {
    const handleKey = handle.toString();
    const entry: FocusEventLog = { handle: handleKey, isFocused };
    logs.focusEvents.push(entry);
    if (isFocused) {
      focusedHandle = handleKey;
    } else if (focusedHandle === handleKey) {
      focusedHandle = null;
    }
    if (isFocused) {
      const textboxState = getTextboxState(handleKey);
      if (!textboxState.isTextbox) {
        deferredTouchFocusHandle = null;
        Reflect.deleteProperty(pendingCaretRevealByHandle, handleKey);
        activeTextHandle = null;
        activeTextEditable = false;
        activeTextMultiline = false;
        syncFocusedInputState();
        updateCaretBlinkTimer();
        queueSemanticAnnouncement(handleKey);
        return;
      }
      activeTextHandle = handleToBigInt(handle);
      activeTextEditable = textboxState.isEditable;
      activeTextMultiline = textboxState.isMultiline;
      syncFocusedInputState();
      if (deferredTouchFocusHandle === handleKey) {
        updateCaretBlinkTimer();
        queueSemanticAnnouncement(handleKey);
        return;
      }
      window.setTimeout(() => {
        if (activeTextHandle !== null && activeTextHandle.toString() === handleKey) {
          focusHiddenEditorNow();
        }
      }, 0);
    } else if (activeTextHandle !== null && activeTextHandle.toString() === handleKey) {
      if (deferredTouchFocusHandle === handleKey) {
        updateCaretBlinkTimer();
        return;
      }
      mutationController.flushPendingTextMutationsToRuntime();
      runtimeRef.current?.flushPendingCommit();
      deferredTouchFocusHandle = null;
      Reflect.deleteProperty(pendingCaretRevealByHandle, handleKey);
      pendingLocalReplacementEcho = null;
      pendingLocalSelectionEcho = null;
      clearPendingTextMutations();
      activeTextHandle = null;
      activeTextEditable = false;
      activeTextMultiline = false;
      syncFocusedInputState();
      runtimeRef.current?.flushPendingCommit();
    }
    updateCaretBlinkTimer();
    if (isFocused) {
      queueSemanticAnnouncement(handleKey);
    }
  };

  const handleTextChanged = (handle: WasmHandleLike, text: string): void => {
    const handleKey = handle.toString();
    if (pendingLocalReplacementEcho !== null && pendingLocalReplacementEcho.handle === handleKey) {
      pendingLocalReplacementEcho = null;
    }
    textByHandle[handleKey] = text;
    textByteLengthsByHandle[handleKey] = utf8ByteLength(text);
    logs.textChanges.push(summarizeTextChange(handleKey, text));
    if (activeTextHandle !== null && activeTextHandle.toString() === handleKey) {
      pendingCaretRevealByHandle[handleKey] = true;
      if (!isActiveEditorFocused()) {
        focusHiddenEditorNow();
        updateCaretBlinkTimer(true);
        return;
      }
      syncFocusedInputState();
      updateCaretBlinkTimer(true);
    }
  };

  const handleRequestSemanticAnnouncement = (handle: WasmHandleLike): void => {
    queueSemanticAnnouncement(handle.toString());
  };

  const reconcileLiveHandles = (handles: readonly string[]): void => {
    const liveHandles = new Set(handles);
    if (focusedHandle !== null && !liveHandles.has(focusedHandle)) {
      focusedHandle = null;
    }
    if (deferredTouchFocusHandle !== null && !liveHandles.has(deferredTouchFocusHandle)) {
      deferredTouchFocusHandle = null;
    }
    if (activeTextHandle === null) {
      return;
    }
    const activeHandleKey = activeTextHandle.toString();
    if (liveHandles.has(activeHandleKey)) {
      return;
    }
    Reflect.deleteProperty(pendingCaretRevealByHandle, activeHandleKey);
    pendingLocalReplacementEcho = null;
    pendingLocalSelectionEcho = null;
    clearPendingTextMutations();
    activeTextHandle = null;
    activeTextEditable = false;
    activeTextMultiline = false;
    syncFocusedInputState();
    updateCaretBlinkTimer();
  };

  const handleTextReplaced = (handle: WasmHandleLike, start: number, end: number, text: string): void => {
    const handleKey = handle.toString();
    const previousText = textByHandle[handleKey] ?? '';
    const previousTextByteLength = textByteLengthsByHandle[handleKey] ?? utf8ByteLength(previousText);
    const isLocalEcho = pendingLocalReplacementEcho !== null
      && pendingLocalReplacementEcho.handle === handleKey
      && pendingLocalReplacementEcho.start === start
      && pendingLocalReplacementEcho.end === end
      && pendingLocalReplacementEcho.text === text
      && activeTextHandle !== null
      && activeTextHandle.toString() === handleKey
      && isActiveEditorFocused();
    const isProjectedEcho = pendingProjectedReplacementEcho !== null
      && pendingProjectedReplacementEcho.handle === handleKey
      && pendingProjectedReplacementEcho.start === start
      && pendingProjectedReplacementEcho.end === end
      && pendingProjectedReplacementEcho.text === text;
    const projectedReplacementEcho = pendingProjectedReplacementEcho;
    const nextText = isLocalEcho
      ? previousText
      : (isProjectedEcho
        ? (projectedReplacementEcho === null ? previousText : projectedReplacementEcho.nextText)
        : applyUtf8ByteReplacementEdit(previousText, start, end, text));
    if (!isLocalEcho && !isProjectedEcho) {
      textByHandle[handleKey] = nextText;
      textByteLengthsByHandle[handleKey] = previousTextByteLength - (end - start) + utf8ByteLength(text);
    }
    if (isLocalEcho) {
      pendingLocalReplacementEcho = null;
    }
    if (isProjectedEcho) {
      pendingProjectedReplacementEcho = null;
    }
    logs.textChanges.push(summarizeTextChange(handleKey, nextText));
    if (activeTextHandle !== null && activeTextHandle.toString() === handleKey) {
      pendingCaretRevealByHandle[handleKey] = true;
      if (!isActiveEditorFocused()) {
        focusHiddenEditorNow();
        updateCaretBlinkTimer(true);
        return;
      }
      if (!isLocalEcho) {
        syncFocusedInputState();
      }
      updateCaretBlinkTimer(true);
    }
  };

  const handleSelectionChanged = (handle: WasmHandleLike, start: number, end: number): void => {
    const handleKey = handle.toString();
    const isLocalEcho = pendingLocalSelectionEcho !== null
      && pendingLocalSelectionEcho.handle === handleKey
      && pendingLocalSelectionEcho.start === start
      && pendingLocalSelectionEcho.end === end
      && activeTextHandle !== null
      && activeTextHandle.toString() === handleKey
      && isActiveEditorFocused();
    selectionsByHandle[handleKey] = { start, end };
    if (pendingLocalSelectionEcho !== null && pendingLocalSelectionEcho.handle === handleKey) {
      pendingLocalSelectionEcho = null;
    }
    const entry: SelectionChangeLog = { handle: handleKey, start, end };
    logs.selectionChanges.push(entry);
    if (activeTextHandle !== null && activeTextHandle.toString() === handleKey) {
      if (!isActiveEditorFocused()) {
        focusHiddenEditorNow();
        updateCaretBlinkTimer(true);
        return;
      }
      if (!isLocalEcho) {
        syncFocusedInputState();
      }
      updateCaretBlinkTimer(true);
      scheduleCaretRevealReplay(handleKey);
    }
  };

  const handleClipboardRead = (handle: WasmHandleLike): void => {
    const runtime = runtimeRef.current;
    if (runtime === null) {
      return;
    }
    const handleValue = handleToBigInt(handle);
    const requestSessionVersion = appSessionVersion;
    logs.clipboardReadRequests.push(handleValue.toString());
    void navigator.clipboard.readText().then((text) => {
      if (requestSessionVersion !== appSessionVersion) {
        return;
      }
      const handleKey = handleValue.toString();
      const currentText = textByHandle[handleKey] ?? '';
      const currentTextByteLength = textByteLengthsByHandle[handleKey] ?? utf8ByteLength(currentText);
      const selection = selectionsByHandle[handleKey] ?? { start: currentTextByteLength, end: currentTextByteLength };
      const rangeStart = Math.max(0, Math.min(selection.start, selection.end));
      const rangeEnd = Math.max(rangeStart, Math.min(currentTextByteLength, Math.max(selection.start, selection.end)));
      const clampedEdit = buildClampedTextboxEdit(
        currentText,
        rangeStart,
        rangeEnd,
        text,
        rangeStart + utf8ByteLength(text),
      );
      textByHandle[handleKey] = clampedEdit.fullNextText;
      textByteLengthsByHandle[handleKey] = utf8ByteLength(clampedEdit.fullNextText);
      selectionsByHandle[handleKey] = { start: clampedEdit.caretByte, end: clampedEdit.caretByte };
      if (activeTextHandle !== null && activeTextHandle.toString() === handleKey) {
        syncFocusedInputState();
      }
      if (clampedEdit.replacement === null) {
        return;
      }
      runtime.ui._ui_set_interaction_time(currentInteractionTimeMs());
      const replacementStartByte = codeUnitIndexToUtf8ByteOffset(currentText, clampedEdit.replacement.start);
      const replacementEndByte = codeUnitIndexToUtf8ByteOffset(currentText, clampedEdit.replacement.end);
      pendingLocalReplacementEcho = {
        handle: handleKey,
        start: replacementStartByte,
        end: replacementEndByte,
        text: clampedEdit.replacement.insertedText,
      };
      pendingLocalSelectionEcho = {
        handle: handleKey,
        start: clampedEdit.caretByte,
        end: clampedEdit.caretByte,
      };
      const heapString = writeUtf8ToHeap(runtime.ui, clampedEdit.replacement.insertedText);
      try {
        runtime.ui._ui_replace_text_range(
          handleValue,
          replacementStartByte,
          replacementEndByte,
          heapString.ptr,
          heapString.len,
          clampedEdit.caretByte,
        );
      } finally {
        heapString.dispose();
      }
      runtime.commitFrame();
    }).catch(() => undefined);
  };

  return {
    logs,
    textByHandle,
    selectionsByHandle,
    flushPendingTextMutationsToRuntime: () => {
      mutationController.flushPendingTextMutationsToRuntime();
    },
    hasPendingTextMutations: () => mutationController.hasPendingTextMutations(),
    materializePendingTextMutations: () => mutationController.materializePendingTextMutations(),
    getActiveTextEditable: () => activeTextEditable,
    getActiveTextHandle: () => activeTextHandle,
    getActiveTextMultiline: () => activeTextMultiline,
    getCapturedPointerHandle: () => capturedPointerHandle,
    getLastPointerClientPosition: () => ({ x: lastPointerClientX, y: lastPointerClientY }),
    getLastPointerPosition: () => ({ x: lastPointerX, y: lastPointerY }),
    getLastPointerModifiers: () => lastPointerModifiers,
    getLastInteractivePointerHandle: () => lastInteractivePointerHandle,
    isActiveTextInputFocused: isActiveEditorFocused,
    isPointerInsideCanvas: () => pointerInsideCanvas,
    applyActiveTextDeletion: (forward) => mutationController.applyActiveTextDeletion(forward),
    replaceActiveTextSelectionWithText: (text) => mutationController.replaceActiveSelectionWithText(text),
    syncActiveTextSelectionFromDom: () => {
      mutationController.syncActiveSelectionFromDom();
    },
    beginTouchTextFocusDeferral,
    cancelTouchTextFocusDeferral,
    commitTouchTextFocusDeferral,
    refocusActiveTextInput,
    resetAppSession,
    reconcileLiveHandles,
    syncActiveTextInputViewport,
    registerSemanticTextEditor: (handle: string, editor: HiddenTextEditor | null): void => {
      domTarget.registerSemanticTextEditor(handle, editor);
      if (editor === null) {
        return;
      }
      const targetHandle = activeTextHandle?.toString() ?? focusedHandle;
      if (targetHandle !== handle) {
        return;
      }
      if (activeTextHandle === null) {
        const textboxState = getTextboxState(handle);
        if (!textboxState.isTextbox) {
          return;
        }
        activeTextHandle = handleToBigInt(handle);
        activeTextEditable = textboxState.isEditable;
        activeTextMultiline = textboxState.isMultiline;
      }
      window.setTimeout(() => {
        if (activeTextHandle?.toString() !== handle) {
          return;
        }
        focusHiddenEditorNow();
      }, 0);
    },
    consumePendingSemanticAnnouncements: () => {
      const handles = Array.from(pendingSemanticAnnouncements.values());
      pendingSemanticAnnouncements.clear();
      return handles;
    },
    getFocusedHandle: () => focusedHandle,
    setCapturedPointerHandle: (handle: bigint | null) => {
      capturedPointerHandle = handle;
    },
    setLastPointerClientPosition: (x: number, y: number) => {
      lastPointerClientX = x;
      lastPointerClientY = y;
    },
    setLastPointerModifiers: (modifiers: number) => {
      lastPointerModifiers = modifiers;
    },
    setLastPointerPosition: (x: number, y: number) => {
      lastPointerX = x;
      lastPointerY = y;
    },
    setLastInteractivePointerHandle: (handle: bigint | null) => {
      lastInteractivePointerHandle = handle;
    },
    setPointerInsideCanvas: (flag: boolean) => {
      pointerInsideCanvas = flag;
    },
    handleClipboardRead,
    handleFocusChanged,
    handleRequestSemanticAnnouncement,
    handleSelectionChanged,
    handleTextChanged,
    handleTextReplaced,
  };
}
