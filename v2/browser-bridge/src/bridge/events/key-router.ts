import type { BridgeRuntime } from '../../core-types';
import type { BridgeInteractionState } from '../local-types';
import type { DesktopFindDialogController } from '../find-dialog';
import { computeModifiers } from '../utils/encoding';
import { writeUtf8ToHeap } from '../utils/heap';

const UI_KEY_EVENT_DOWN = 1;
const UI_KEY_EVENT_UP = 2;

function currentInteractionTimeMs(): bigint {
  return BigInt(Math.floor(performance.now()));
}

export function installKeyAndWindowHandlers(
  runtime: BridgeRuntime,
  interactionState: BridgeInteractionState,
  desktopFindDialog: DesktopFindDialogController,
): () => void {
  const { ui } = runtime;

  const activeTextUsesEditorCommandKeys = (handle: bigint | null): boolean => {
    if (handle === null) {
      return false;
    }
    const debugTree = runtime.getDebugTree();
    const node = debugTree.nodesByHandle[handle.toString()];
    return node?.behavior.editorCommandKeys === true;
  };

  const activeTextAcceptsTab = (handle: bigint | null): boolean => {
    if (handle === null) {
      return false;
    }
    const debugTree = runtime.getDebugTree();
    const node = debugTree.nodesByHandle[handle.toString()];
    return node?.behavior.editorAcceptsTab === true;
  };

  const dispatchKeyToRuntime = (
    type: number,
    event: KeyboardEvent,
    modifiers: number,
  ): boolean => {
    ui._ui_set_interaction_time(currentInteractionTimeMs());
    const heapString = writeUtf8ToHeap(ui, event.key);
    try {
      const callbackHandled = window.__effindomCallbacks?.onKeyEventWithKey?.(type, event.key, modifiers) === true;
      if (callbackHandled) {
        return true;
      }
      const runtimeHandled = ui._ui_on_key_event(type, heapString.ptr, heapString.len, modifiers) !== 0;
      return runtimeHandled || callbackHandled;
    } finally {
      heapString.dispose();
    }
  };

  const forwardKeyEvent = (type: number) => (event: KeyboardEvent): void => {
    if (desktopFindDialog.consumeGlobalKeyEvent(event, type === UI_KEY_EVENT_DOWN ? 'down' : 'up')) {
      return;
    }
    const modifiers = computeModifiers(event);
    const activeTextHandle = interactionState.getActiveTextHandle();
    const activeTextEditable = interactionState.getActiveTextEditable();
    const activeTextMultiline = interactionState.getActiveTextMultiline();
    const activeTextInputFocused = interactionState.isActiveTextInputFocused();
    const activeElement = document.activeElement;
    const hiddenTextEditorFocused =
      (activeElement instanceof HTMLInputElement || activeElement instanceof HTMLTextAreaElement) &&
      activeElement.dataset.effindomHiddenEditor === 'true';
    const activeEditorWindowHandle = window.__bridgeActiveEditorWindow?.handle ?? null;
    const recoveredActiveTextHandle =
      hiddenTextEditorFocused && activeEditorWindowHandle !== null
        ? BigInt(activeEditorWindowHandle)
        : activeTextHandle;
    const recoveredActiveTextEditable =
      hiddenTextEditorFocused
        ? !activeElement.readOnly
        : activeTextEditable;
    const activeEditableTextMayOwnPlainTab =
      recoveredActiveTextHandle !== null &&
      recoveredActiveTextEditable &&
      hiddenTextEditorFocused &&
      !event.ctrlKey &&
      !event.metaKey &&
      !event.altKey &&
      !event.shiftKey &&
      event.key === 'Tab';
    const activeEditableTextMayOwnSelectAll =
      recoveredActiveTextHandle !== null &&
      recoveredActiveTextEditable &&
      hiddenTextEditorFocused &&
      (event.ctrlKey || event.metaKey) &&
      !event.altKey &&
      !event.shiftKey &&
      event.key.toLowerCase() === 'a';
    const activeEditableTextOwnsNativePasteShortcut =
      recoveredActiveTextHandle !== null &&
      recoveredActiveTextEditable &&
      hiddenTextEditorFocused &&
      (event.ctrlKey || event.metaKey) &&
      !event.altKey &&
      !event.shiftKey &&
      event.key.toLowerCase() === 'v';
    const activeTextComboBoxCommandKey =
      activeTextUsesEditorCommandKeys(recoveredActiveTextHandle) &&
      !event.ctrlKey &&
      !event.metaKey &&
      !event.altKey &&
      !event.shiftKey &&
      (event.key === 'ArrowUp' || event.key === 'ArrowDown');
    const activeTextOwnsNativeNavigationKey =
      recoveredActiveTextHandle !== null &&
      (activeTextInputFocused || hiddenTextEditorFocused) &&
      !activeTextComboBoxCommandKey &&
      !event.ctrlKey &&
      !event.metaKey &&
      !event.altKey &&
      (
        event.key === 'ArrowLeft' ||
        event.key === 'ArrowRight' ||
        event.key === 'ArrowUp' ||
        event.key === 'ArrowDown' ||
        event.key === 'Home' ||
        event.key === 'End' ||
        event.key === 'PageUp' ||
        event.key === 'PageDown'
      );
    const activeEditableTextOwnsNativeEditingKey =
      recoveredActiveTextHandle !== null &&
      recoveredActiveTextEditable &&
      (activeTextInputFocused || hiddenTextEditorFocused) &&
      !event.ctrlKey &&
      !event.metaKey &&
      !event.altKey &&
      (
        event.key.length === 1 ||
        (activeTextMultiline && event.key === 'Enter') ||
        event.key === 'Backspace' ||
        event.key === 'Delete'
      );
    if (activeTextOwnsNativeNavigationKey) {
      interactionState.flushPendingTextMutationsToRuntime();
      runtime.flushPendingCommit();
      ui._ui_set_interaction_time(currentInteractionTimeMs());
      const handled = (() => {
        const heapString = writeUtf8ToHeap(ui, event.key);
        try {
          return ui._ui_on_key_event(type, heapString.ptr, heapString.len, modifiers) !== 0;
        } finally {
          heapString.dispose();
        }
      })();
      if (handled) {
        event.preventDefault();
      }
      runtime.commitFrame();
      return;
    }
    if (activeEditableTextMayOwnPlainTab) {
      if (
        type === UI_KEY_EVENT_DOWN &&
        activeTextAcceptsTab(recoveredActiveTextHandle) &&
        interactionState.replaceActiveTextSelectionWithText('\t')
      ) {
        event.preventDefault();
        return;
      }
      ui._ui_request_focus(recoveredActiveTextHandle);
      window.__effindomCallbacks?.onFocusChanged?.(recoveredActiveTextHandle, true);
      const handled = dispatchKeyToRuntime(type, event, modifiers);
      if (handled) {
        event.preventDefault();
        runtime.commitFrame();
        return;
      }
      runtime.commitFrame();
      return;
    }
    if (activeEditableTextMayOwnSelectAll) {
      if (
        type === UI_KEY_EVENT_DOWN &&
        interactionState.selectAllActiveText()
      ) {
        event.preventDefault();
        return;
      }
      return;
    }
    if (activeEditableTextOwnsNativePasteShortcut) {
      return;
    }
    if (activeEditableTextOwnsNativeEditingKey) {
      interactionState.flushPendingTextMutationsToRuntime();
      runtime.flushPendingCommit();
      if (
        type === UI_KEY_EVENT_DOWN &&
        (event.key === 'Backspace' || event.key === 'Delete') &&
        interactionState.applyActiveTextDeletion(event.key === 'Delete')
      ) {
        event.preventDefault();
      }
      return;
    }
    interactionState.flushPendingTextMutationsToRuntime();
    runtime.flushPendingCommit();
    const handled = dispatchKeyToRuntime(type, event, modifiers);
    if (handled) {
      event.preventDefault();
    }
    runtime.commitFrame();
    if (interactionState.getActiveTextHandle() !== null && !interactionState.isActiveTextInputFocused()) {
      interactionState.refocusActiveTextInput();
    }
  };

  const handleKeyDown = forwardKeyEvent(UI_KEY_EVENT_DOWN);
  const handleKeyUp = forwardKeyEvent(UI_KEY_EVENT_UP);
  const keyListenerCapture = true;
  const reconcileFindSelection = (): void => {
    requestAnimationFrame(() => {
      runtime.syncFindSelection(true);
    });
  };
  const handleWindowBlur = (): void => {
    if (interactionState.getActiveTextHandle() === null) {
      reconcileFindSelection();
      return;
    }
    ui._ui_request_focus(0n);
    runtime.commitFrame();
    reconcileFindSelection();
  };
  const handleWindowFocus = (): void => {
    reconcileFindSelection();
  };
  const handleResize = (): void => {
    runtime.updateCanvasSize();
    runtime.commitFrame();
  };

  window.addEventListener('keydown', handleKeyDown, keyListenerCapture);
  window.addEventListener('keyup', handleKeyUp, keyListenerCapture);
  window.addEventListener('blur', handleWindowBlur);
  window.addEventListener('focus', handleWindowFocus);
  window.addEventListener('resize', handleResize);

  return () => {
    window.removeEventListener('keydown', handleKeyDown, keyListenerCapture);
    window.removeEventListener('keyup', handleKeyUp, keyListenerCapture);
    window.removeEventListener('blur', handleWindowBlur);
    window.removeEventListener('focus', handleWindowFocus);
    window.removeEventListener('resize', handleResize);
  };
}
