import { expect,test } from '@playwright/test';

import {
buildEditableTextScene,
buildInteractiveBoxScene,
buildStaticTextScene,
gotoBridgePage,
setupServer,
teardownServer
} from './test-utils';
import type { WasmHandleLike } from '../src/core-types';

test.beforeAll(async () => {
  await setupServer();
});

test.afterAll(async () => {
  await teardownServer();
});

test('focused multiline textbox page keys stay owned by the bridge instead of scrolling the page', async ({ page }) => {
  await gotoBridgePage(page);
  const scene = await buildEditableTextScene(
    page,
    Array.from({ length: 20 }, (_, index) => `Line ${index.toString().padStart(2, '0')}`).join('\n'),
    0,
    { multiline: true, wrapping: false, nodeHeight: 160 },
  );

  await page.evaluate((textHandle) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    if (runtime === null || runtime === undefined) {
      throw new Error('Bridge runtime is not ready.');
    }
    const filler = document.createElement('div');
    filler.style.height = '2400px';
    document.body.appendChild(filler);
    window.scrollTo(0, 600);
    const bridge = window.EffinDomBrowserBridge;
    if (bridge === undefined) {
      throw new Error('Bridge state is not ready.');
    }
    const handleArg = bridge.handleToBigInt(textHandle);
    runtime.ui._ui_set_interaction_time(BigInt(Math.floor(performance.now())));
    runtime.ui._ui_on_pointer_event(1, handleArg, 1, 12, -1, 1, 0, 0, 0, 0, 0, 0, 0);
    runtime.ui._ui_on_pointer_event(2, handleArg, 1, 12, -1, 1, 0, 0, 0, 0, 0, 0, 0);
    runtime.commitFrame();
    runtime.flushPendingCommit();
  }, scene.textHandle);

  await expect.poll(async () => {
    return await page.evaluate(() => {
      const hiddenTextarea = document.querySelector('textarea[data-effindom-hidden-editor="true"]');
      return hiddenTextarea instanceof HTMLTextAreaElement && document.activeElement === hiddenTextarea;
    });
  }).toBe(true);

  const initialScrollY = await page.evaluate(() => window.scrollY);
  await page.keyboard.press('PageDown');
  await page.keyboard.press('PageDown');

  await expect.poll(async () => {
    return await page.evaluate(() => {
      const hiddenTextarea = document.querySelector('textarea[data-effindom-hidden-editor="true"]');
      return {
        scrollY: window.scrollY,
        hiddenFocused: hiddenTextarea instanceof HTMLTextAreaElement && document.activeElement === hiddenTextarea,
      };
    });
  }).toEqual({
    scrollY: initialScrollY,
    hiddenFocused: true,
  });
});


test('non-text focus does not attach bridge text input state', async ({ page }) => {
  await gotoBridgePage(page);
  const scene = await buildInteractiveBoxScene(page);

  const focusState = await page.evaluate((boxHandle) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const canvas = document.getElementById('fui-canvas');
    const hiddenInput = document.querySelector('input[data-effindom-hidden-editor="true"]');
    if (
      runtime === null ||
      runtime === undefined ||
      !(canvas instanceof HTMLCanvasElement) ||
      !(hiddenInput instanceof HTMLInputElement)
    ) {
      throw new Error('Bridge runtime is not ready.');
    }

    const bridge = window.EffinDomBrowserBridge;
    if (bridge === undefined) {
      throw new Error('Bridge state is not ready.');
    }
    const handleArg = bridge.handleToBigInt(boxHandle);
    runtime.ui._ui_set_interaction_time(BigInt(Math.floor(performance.now())));
    runtime.ui._ui_on_pointer_event(1, handleArg, 10, 10, -1, 1, 0, 0, 0, 0, 0, 0, 0);
    runtime.ui._ui_on_pointer_event(2, handleArg, 10, 10, -1, 1, 0, 0, 0, 0, 0, 0, 0);
    runtime.commitFrame();
    runtime.flushPendingCommit();

    return {
      activeTextHandle: runtime.getActiveTextHandle()?.toString() ?? null,
      hiddenInputFocused: document.activeElement === hiddenInput,
      hiddenInputValue: hiddenInput.value,
    };
  }, scene.boxHandle);

  expect(focusState.activeTextHandle).toBeNull();
  expect(focusState.hiddenInputFocused).toBe(false);
  expect(focusState.hiddenInputValue).toBe('');
});

test('editor pointer activation uses explicit editor behavior instead of textbox semantics', async ({ page }) => {
  await gotoBridgePage(page);

  const state = await page.evaluate(async () => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const bridge = window.EffinDomBrowserBridge;
    const canvas = document.getElementById('fui-canvas');
    const hiddenInput = document.querySelector('input[data-effindom-hidden-editor="true"]');
    if (
      runtime === null ||
      runtime === undefined ||
      bridge === undefined ||
      !(canvas instanceof HTMLCanvasElement) ||
      !(hiddenInput instanceof HTMLInputElement)
    ) {
      throw new Error('Bridge runtime is not ready.');
    }

    const ui = runtime.ui;
    runtime.resetAppSession();
    ui._ui_reset();

    const root = ui._ui_create_node(0);
    const semanticOnly = ui._ui_create_node(1);
    const editorOnly = ui._ui_create_node(1);
    ui._ui_set_root(root);
    ui._ui_node_add_child(root, semanticOnly);
    ui._ui_node_add_child(root, editorOnly);
    ui._ui_set_width(root, 260, 0);
    ui._ui_set_height(root, 140, 0);

    const configureText = (handle: WasmHandleLike, top: number, value: string): void => {
      const text = new TextEncoder().encode(value);
      const ptr = ui._malloc(text.length);
      const offset = Number(ptr);
      const ptrArg = ui.usesMemory64 === true ? BigInt(offset) : offset;
      ui.HEAPU8.set(text, offset);
      ui._ui_set_position_type(handle, 1);
      ui._ui_set_position(handle, 16, top, 0, 0);
      ui._ui_set_width(handle, 180, 0);
      ui._ui_set_height(handle, 28, 0);
      ui._ui_set_font(handle, 1, 16);
      ui._ui_set_text(handle, ptrArg, text.length);
      ui._free(ptrArg);
      ui._ui_set_selectable(handle, 1, 0x40007AFF);
      ui._ui_set_interactive(handle, 1);
      ui._ui_set_focusable(handle, 1, 0);
    };

    configureText(semanticOnly, 16, 'semantic only');
    configureText(editorOnly, 58, 'editor only');
    ui._ui_set_semantic_role(semanticOnly, 2);
    ui._ui_set_editable(editorOnly, 1);
    ui._ui_set_editable(editorOnly, 0);
    runtime.commitFrame();
    runtime.flushPendingCommit();

    const debug = runtime.getDebugTree();
    const semanticOnlyKey = semanticOnly.toString();
    const editorOnlyKey = editorOnly.toString();
    const before = {
      semanticOnlyTextEditor: debug.nodesByHandle[semanticOnlyKey]?.behavior.textEditor ?? null,
      semanticOnlyRole: debug.nodesByHandle[semanticOnlyKey]?.semanticRole ?? null,
      editorOnlyTextEditor: debug.nodesByHandle[editorOnlyKey]?.behavior.textEditor ?? null,
      editorOnlyRole: debug.nodesByHandle[editorOnlyKey]?.semanticRole ?? null,
    };

    const rect = canvas.getBoundingClientRect();
    const tap = async (x: number, y: number): Promise<void> => {
      canvas.dispatchEvent(new PointerEvent('pointerdown', {
        bubbles: true,
        cancelable: true,
        pointerId: 91,
        pointerType: 'mouse',
        isPrimary: true,
        button: 0,
        buttons: 1,
        clientX: rect.left + x,
        clientY: rect.top + y,
      }));
      canvas.dispatchEvent(new PointerEvent('pointerup', {
        bubbles: true,
        cancelable: true,
        pointerId: 91,
        pointerType: 'mouse',
        isPrimary: true,
        button: 0,
        buttons: 0,
        clientX: rect.left + x,
        clientY: rect.top + y,
      }));
      runtime.commitFrame();
      runtime.flushPendingCommit();
      await new Promise((resolve) => requestAnimationFrame(resolve));
    };

    await tap(24, 24);
    const afterSemanticTap = {
      activeTextHandle: runtime.getActiveTextHandle()?.toString() ?? null,
      hiddenFocused: document.activeElement === hiddenInput,
    };

    await tap(24, 66);
    const afterEditorTap = {
      activeTextHandle: runtime.getActiveTextHandle()?.toString() ?? null,
      hiddenFocused: document.activeElement === hiddenInput,
    };

    return { before, afterSemanticTap, afterEditorTap, editorOnlyKey };
  });

  expect(state.before).toEqual({
    semanticOnlyTextEditor: false,
    semanticOnlyRole: 2,
    editorOnlyTextEditor: true,
    editorOnlyRole: 0,
  });
  expect(state.afterSemanticTap).toEqual({
    activeTextHandle: null,
    hiddenFocused: false,
  });
  expect(state.afterEditorTap).toEqual({
    activeTextHandle: state.editorOnlyKey,
    hiddenFocused: true,
  });
});


test('blurring an active textbox clears the hidden editor value', async ({ page }) => {
  await gotoBridgePage(page);
  const scene = await buildEditableTextScene(page, '');

  const state = await page.evaluate((textHandle) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const canvas = document.getElementById('fui-canvas');
    const hiddenInput = document.querySelector('input[data-effindom-hidden-editor="true"]');
    if (
      runtime === null ||
      runtime === undefined ||
      !(canvas instanceof HTMLCanvasElement) ||
      !(hiddenInput instanceof HTMLInputElement)
    ) {
      throw new Error('Bridge runtime is not ready.');
    }

    const bridge = window.EffinDomBrowserBridge;
    if (bridge === undefined) {
      throw new Error('Bridge state is not ready.');
    }
    const handleArg = bridge.handleToBigInt(textHandle);
    runtime.ui._ui_set_interaction_time(BigInt(Math.floor(performance.now())));
    runtime.ui._ui_on_pointer_event(1, handleArg, 12, 12, -1, 1, 0, 0, 0, 0, 0, 0, 0);
    runtime.ui._ui_on_pointer_event(2, handleArg, 12, 12, -1, 1, 0, 0, 0, 0, 0, 0, 0);
    runtime.commitFrame();
    runtime.flushPendingCommit();

    hiddenInput.value = 'focused text';
    hiddenInput.setSelectionRange(hiddenInput.value.length, hiddenInput.value.length, 'none');
    runtime.ui._ui_request_focus(0n);
    runtime.commitFrame();
    runtime.flushPendingCommit();

    return {
      activeTextHandle: runtime.getActiveTextHandle()?.toString() ?? null,
      hiddenInputFocused: document.activeElement === hiddenInput,
      hiddenInputValue: hiddenInput.value,
    };
  }, scene.textHandle);

  expect(state).toEqual({
    activeTextHandle: null,
    hiddenInputFocused: false,
    hiddenInputValue: '',
  });
});

test('programmatic textbox text hydrates hidden editor before first focus selection clamp', async ({ page }) => {
  await gotoBridgePage(page);
  const scene = await buildEditableTextScene(page, 'Melbourne');

  await page.evaluate((textHandle) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const hiddenInput = document.querySelector('input[data-effindom-hidden-editor="true"]');
    const bridge = window.EffinDomBrowserBridge;
    if (
      runtime === null ||
      runtime === undefined ||
      !(hiddenInput instanceof HTMLInputElement) ||
      bridge === undefined ||
      window.__bridgeTextByHandle === undefined
    ) {
      throw new Error('Bridge runtime is not ready.');
    }

    const handleArg = bridge.handleToBigInt(textHandle);
    const ui = runtime.ui;
    const toPointer = (pointer: unknown): { ptr: number | bigint; offset: number } =>
      bridge.toHeapPointer(ui, pointer as string | number | bigint | { valueOf(): unknown; toString(): string });
    const writeText = (value: string): { ptr: number | bigint; offset: number; len: number } => {
      const bytes = new TextEncoder().encode(value);
      const pointer = bytes.length === 0 ? { ptr: ui.usesMemory64 === true ? 0n : 0, offset: 0 } : toPointer(ui._malloc(bytes.length));
      if (bytes.length > 0 && pointer.offset === 0) {
        throw new Error('ui malloc failed.');
      }
      if (bytes.length > 0) {
        ui.HEAPU8.set(bytes, pointer.offset);
      }
      return { ptr: pointer.ptr, offset: pointer.offset, len: bytes.length };
    };

    window.__bridgeTextByHandle[textHandle] = '';
    const heapText = writeText('Melbourne');
    try {
      runtime.ui._ui_set_text(handleArg, heapText.ptr, heapText.len);
    } finally {
      if (heapText.offset !== 0) {
        runtime.ui._free(heapText.ptr);
      }
    }
    runtime.ui._ui_set_text_selection_range(handleArg, 9, 9);
    runtime.ui._ui_request_focus(handleArg);
    runtime.commitFrame();
    runtime.flushPendingCommit();
  }, scene.textHandle);

  await expect.poll(async () => await page.evaluate((textHandle) => {
    const hiddenInput = document.querySelector('input[data-effindom-hidden-editor="true"]');
    return {
      bridgeText: window.__bridgeTextByHandle?.[textHandle] ?? '',
      hiddenFocused: hiddenInput instanceof HTMLInputElement && document.activeElement === hiddenInput,
      hiddenValue: hiddenInput instanceof HTMLInputElement ? hiddenInput.value : '',
      selectionStart: hiddenInput instanceof HTMLInputElement ? hiddenInput.selectionStart : null,
      selectionEnd: hiddenInput instanceof HTMLInputElement ? hiddenInput.selectionEnd : null,
    };
  }, scene.textHandle)).toEqual({
    bridgeText: 'Melbourne',
    hiddenFocused: true,
    hiddenValue: 'Melbourne',
    selectionStart: 9,
    selectionEnd: 9,
  });
});


test('browser undo and redo shortcuts pass through when unhandled', async ({ page }) => {
  await gotoBridgePage(page);

  const keyState = await page.evaluate(() => {
    const undoEvent = new KeyboardEvent('keydown', { key: 'z', ctrlKey: true, cancelable: true });
    const redoEvent = new KeyboardEvent('keydown', { key: 'y', ctrlKey: true, cancelable: true });
    window.dispatchEvent(undoEvent);
    window.dispatchEvent(redoEvent);
    return {
      undoPrevented: undoEvent.defaultPrevented,
      redoPrevented: redoEvent.defaultPrevented,
    };
  });

  expect(keyState.undoPrevented).toBe(false);
  expect(keyState.redoPrevented).toBe(false);
});

test('browser undo and redo shortcuts can be consumed by app key handler', async ({ page }) => {
  await gotoBridgePage(page);

  const keyState = await page.evaluate(() => {
    const callbacks = window.__effindomCallbacks;
    if (callbacks === undefined) {
      throw new Error('Bridge callbacks are not ready.');
    }
    const previousKeyEvent = callbacks.onKeyEventWithKey;
    callbacks.onKeyEventWithKey = (type, key) => {
      previousKeyEvent?.(type, key, 0);
      return type === 1 && (key === 'z' || key === 'y');
    };
    try {
      const undoEvent = new KeyboardEvent('keydown', { key: 'z', ctrlKey: true, cancelable: true });
      const redoEvent = new KeyboardEvent('keydown', { key: 'y', ctrlKey: true, cancelable: true });
      window.dispatchEvent(undoEvent);
      window.dispatchEvent(redoEvent);
      return {
        undoPrevented: undoEvent.defaultPrevented,
        redoPrevented: redoEvent.defaultPrevented,
      };
    } finally {
      if (previousKeyEvent === undefined) {
        delete callbacks.onKeyEventWithKey;
      } else {
        callbacks.onKeyEventWithKey = previousKeyEvent;
      }
    }
  });

  expect(keyState.undoPrevented).toBe(true);
  expect(keyState.redoPrevented).toBe(true);
});

test('browser reload shortcut is not swallowed after selecting static text', async ({ page }) => {
  await gotoBridgePage(page);
  const scene = await buildStaticTextScene(page, 'Selectable text');

  const keyState = await page.evaluate((textHandle) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const bridge = window.EffinDomBrowserBridge;
    if (runtime === null || runtime === undefined || bridge === undefined) {
      throw new Error('Bridge runtime is not ready.');
    }
    const handleArg = bridge.handleToBigInt(textHandle);
    runtime.ui._ui_set_interaction_time(BigInt(Math.floor(performance.now())));
    runtime.ui._ui_on_pointer_event(1, handleArg, 12, 12, -1, 1, 0, 0, 0, 0, 0, 0, 0);
    runtime.ui._ui_on_pointer_event(2, handleArg, 12, 12, -1, 1, 0, 0, 0, 0, 0, 0, 0);
    runtime.commitFrame();
    runtime.flushPendingCommit();

    const reloadEvent = new KeyboardEvent('keydown', {
      key: '®',
      code: 'KeyR',
      metaKey: true,
      cancelable: true,
    });
    window.dispatchEvent(reloadEvent);
    return {
      reloadPrevented: reloadEvent.defaultPrevented,
    };
  }, scene.textHandle);

  expect(keyState.reloadPrevented).toBe(false);
});

test('browser reload shortcut can be consumed by app key handler after selecting static text', async ({ page }) => {
  await gotoBridgePage(page);
  const scene = await buildStaticTextScene(page, 'Selectable text');

  const keyState = await page.evaluate((textHandle) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const bridge = window.EffinDomBrowserBridge;
    const callbacks = window.__effindomCallbacks;
    if (runtime === null || runtime === undefined || bridge === undefined || callbacks === undefined) {
      throw new Error('Bridge runtime is not ready.');
    }
    const previousKeyEvent = callbacks.onKeyEventWithKey;
    callbacks.onKeyEventWithKey = (type, key, modifiers) => {
      previousKeyEvent?.(type, key, modifiers);
      return type === 1 && key === '®' && modifiers !== 0;
    };
    try {
      const handleArg = bridge.handleToBigInt(textHandle);
      runtime.ui._ui_set_interaction_time(BigInt(Math.floor(performance.now())));
      runtime.ui._ui_on_pointer_event(1, handleArg, 12, 12, -1, 1, 0, 0, 0, 0, 0, 0, 0);
      runtime.ui._ui_on_pointer_event(2, handleArg, 12, 12, -1, 1, 0, 0, 0, 0, 0, 0, 0);
      runtime.commitFrame();
      runtime.flushPendingCommit();

      const reloadEvent = new KeyboardEvent('keydown', {
        key: '®',
        code: 'KeyR',
        metaKey: true,
        cancelable: true,
      });
      window.dispatchEvent(reloadEvent);
      return {
        reloadPrevented: reloadEvent.defaultPrevented,
      };
    } finally {
      if (previousKeyEvent === undefined) {
        delete callbacks.onKeyEventWithKey;
      } else {
        callbacks.onKeyEventWithKey = previousKeyEvent;
      }
    }
  }, scene.textHandle);

  expect(keyState.reloadPrevented).toBe(true);
});

test('text navigation key is consumed when native runtime handles selected static text', async ({ page }) => {
  await gotoBridgePage(page);
  const scene = await buildStaticTextScene(page, 'Selectable text');

  const keyState = await page.evaluate((textHandle) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const bridge = window.EffinDomBrowserBridge;
    if (runtime === null || runtime === undefined || bridge === undefined) {
      throw new Error('Bridge runtime is not ready.');
    }
    const handleArg = bridge.handleToBigInt(textHandle);
    runtime.ui._ui_set_interaction_time(BigInt(Math.floor(performance.now())));
    runtime.ui._ui_on_pointer_event(1, handleArg, 12, 12, -1, 1, 0, 0, 0, 0, 0, 0, 0);
    runtime.ui._ui_on_pointer_event(2, handleArg, 12, 12, -1, 1, 0, 0, 0, 0, 0, 0, 0);
    runtime.commitFrame();
    runtime.flushPendingCommit();

    const arrowEvent = new KeyboardEvent('keydown', {
      key: 'ArrowRight',
      code: 'ArrowRight',
      cancelable: true,
    });
    window.dispatchEvent(arrowEvent);
    return {
      arrowPrevented: arrowEvent.defaultPrevented,
    };
  }, scene.textHandle);

  expect(keyState.arrowPrevented).toBe(true);
});

test('textbox input preserves fast typing and keeps only the latest pending paste', async ({ page }) => {
  await gotoBridgePage(page);
  const scene = await buildEditableTextScene(page, '');

  await page.evaluate((textHandle) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const bridge = window.EffinDomBrowserBridge;
    if (runtime === null || runtime === undefined || bridge === undefined) {
      throw new Error('Bridge runtime is not ready.');
    }
    const handleArg = bridge.handleToBigInt(textHandle);
    runtime.ui._ui_set_interaction_time(BigInt(Math.floor(performance.now())));
    runtime.ui._ui_on_pointer_event(1, handleArg, 12, 12, -1, 1, 0, 0, 0, 0, 0, 0, 0);
    runtime.ui._ui_on_pointer_event(2, handleArg, 12, 12, -1, 1, 0, 0, 0, 0, 0, 0, 0);
    runtime.commitFrame();
    runtime.flushPendingCommit();
  }, scene.textHandle);

  await expect.poll(async () => {
    return await page.evaluate(() => {
      const hiddenInput = document.querySelector('input[data-effindom-hidden-editor="true"]');
      return hiddenInput instanceof HTMLInputElement && document.activeElement === hiddenInput;
    });
  }).toBe(true);

  const batchState = await page.evaluate((textHandle) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const hiddenInput = document.querySelector('input[data-effindom-hidden-editor="true"]');
    if (runtime === null || runtime === undefined || !(hiddenInput instanceof HTMLInputElement)) {
      throw new Error('Expected bridge runtime and hidden input.');
    }

    const previousReplaceTextRange = runtime.ui._ui_replace_text_range.bind(runtime.ui);
    let replaceAbiCallCount = 0;
    runtime.ui._ui_replace_text_range = (handle, start, end, ptr, len, caret) => {
      replaceAbiCallCount += 1;
      previousReplaceTextRange(handle, start, end, ptr, len, caret);
    };

    const typeText = (value: string): void => {
      hiddenInput.value = value;
      hiddenInput.setSelectionRange(value.length, value.length, 'none');
      hiddenInput.dispatchEvent(new InputEvent('input', {
        bubbles: true,
        inputType: 'insertText',
        data: value.slice(-1),
      }));
    };

    const pasteText = (text: string): void => {
      const selectionStart = hiddenInput.selectionStart ?? hiddenInput.value.length;
      const selectionEnd = hiddenInput.selectionEnd ?? selectionStart;
      hiddenInput.dispatchEvent(new InputEvent('beforeinput', {
        bubbles: true,
        cancelable: true,
        inputType: 'insertFromPaste',
        data: text,
      }));
      const browserValue =
        `${hiddenInput.value.slice(0, selectionStart)}${text}${hiddenInput.value.slice(selectionEnd)}`;
      const caret = selectionStart + text.length;
      hiddenInput.value = browserValue;
      hiddenInput.setSelectionRange(caret, caret, 'none');
      hiddenInput.dispatchEvent(new InputEvent('input', {
        bubbles: true,
        inputType: 'insertFromPaste',
        data: text,
      }));
    };

    try {
      runtime.resetLogs();
      typeText('a');
      typeText('ab');
      typeText('abc');
      typeText('abcd');
      const afterFourth = {
        replaceAbiCallCount,
        hiddenInputValue: hiddenInput.value,
      };

      typeText('abcde');
      const afterFifth = {
        replaceAbiCallCount,
        hiddenInputValue: hiddenInput.value,
      };

      typeText('abcdef');
      const afterSixth = {
        replaceAbiCallCount,
        hiddenInputValue: hiddenInput.value,
        bridgeText: window.__bridgeTextByHandle?.[textHandle] ?? '',
      };

      pasteText('FIRST');
      pasteText('SECOND');
      const beforeFlush = {
        replaceAbiCallCount,
        hiddenInputValue: hiddenInput.value,
        bridgeText: window.__bridgeTextByHandle?.[textHandle] ?? '',
      };

      runtime.flushPendingCommit();

      return {
        afterFourth,
        afterFifth,
        afterSixth,
        beforeFlush,
        afterFlush: {
          replaceAbiCallCount,
          hiddenInputValue: hiddenInput.value,
          bridgeText: window.__bridgeTextByHandle?.[textHandle] ?? '',
        },
      };
    } finally {
      runtime.ui._ui_replace_text_range = previousReplaceTextRange;
    }
  }, scene.textHandle);

  expect(batchState).toEqual({
    afterFourth: {
      replaceAbiCallCount: 3,
      hiddenInputValue: 'abcd',
    },
    afterFifth: {
      replaceAbiCallCount: 4,
      hiddenInputValue: 'abcde',
    },
    afterSixth: {
      replaceAbiCallCount: 5,
      hiddenInputValue: 'abcdef',
      bridgeText: 'abcdef',
    },
    beforeFlush: {
      replaceAbiCallCount: 6,
      hiddenInputValue: 'abcdefSECOND',
      bridgeText: 'abcdefSECOND',
    },
    afterFlush: {
      replaceAbiCallCount: 7,
      hiddenInputValue: 'abcdefSECOND',
      bridgeText: 'abcdefSECOND',
    },
  });
});
