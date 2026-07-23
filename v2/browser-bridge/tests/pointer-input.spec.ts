import { expect,test,type Page } from '@playwright/test';

import {
buildInteractiveBoxScene,
buildNestedProxyScrollScene,
buildScrollScene,
CMD_COMMIT_PAINT_ORDER,
gotoBridgePage,
setupServer,
teardownServer
} from './test-utils';

test.beforeAll(async () => {
  await setupServer();
});

test.afterAll(async () => {
  await teardownServer();
});

async function buildNestedInteractiveBoxScene(page: Page): Promise<{ parentHandle: string; childHandle: string }> {
  return await page.evaluate(() => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    if (runtime === null || runtime === undefined) {
      throw new Error('Bridge runtime is not ready.');
    }
    const ui = runtime.ui;
    const toHandle = (handle: unknown): string => {
      if (typeof handle === 'bigint') {
        return handle.toString();
      }
      if (typeof handle === 'number') {
        return BigInt(handle).toString();
      }
      if (typeof handle === 'string') {
        return BigInt(handle).toString();
      }
      if (handle !== null && typeof handle === 'object' && 'valueOf' in handle && typeof handle.valueOf === 'function') {
        return toHandle(handle.valueOf());
      }
      throw new TypeError(`Cannot convert ${String(handle)} to a handle string.`);
    };

    runtime.resetLogs();
    runtime.resetAppSession();
    ui._ui_reset();

    const root = toHandle(ui._ui_create_node(0));
    const parent = toHandle(ui._ui_create_node(0));
    const child = toHandle(ui._ui_create_node(0));
    ui._ui_set_root(root);
    ui._ui_node_add_child(root, parent);
    ui._ui_node_add_child(parent, child);
    ui._ui_set_width(root, 320, 0);
    ui._ui_set_height(root, 220, 0);
    ui._ui_set_bg_color(root, 0x111827ff);
    ui._ui_set_width(parent, 180, 0);
    ui._ui_set_height(parent, 160, 0);
    ui._ui_set_bg_color(parent, 0x2563ebff);
    ui._ui_set_interactive(parent, 1);
    ui._ui_set_width(child, 90, 0);
    ui._ui_set_height(child, 90, 0);
    ui._ui_set_bg_color(child, 0x22c55eff);
    ui._ui_set_interactive(child, 1);
    runtime.commitFrame();
    runtime.flushPendingCommit();
    runtime.resetLogs();

    return { parentHandle: parent, childHandle: child };
  });
}

test('pointer events use Core hit-testing and surface the real handle back through callbacks', async ({ page }) => {
  await gotoBridgePage(page);
  const scene = await buildInteractiveBoxScene(page);

  const result = await page.evaluate(() => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    if (runtime === null || runtime === undefined) {
      throw new Error('Bridge runtime is not ready.');
    }
    const hitHandle = runtime.getHandleFromPoint(40, 40);
    runtime.ui._ui_set_interaction_time(BigInt(Math.floor(performance.now())));
    runtime.ui._ui_on_pointer_event(1, hitHandle, 40, 40, -1, 1, 0, 0, 0, 0, 0, 0, 0);
    runtime.ui._ui_on_pointer_event(2, hitHandle, 40, 40, -1, 1, 0, 0, 0, 0, 0, 0, 0);
    runtime.commitFrame();
    runtime.flushPendingCommit();
    return {
      hitHandle: hitHandle.toString(),
      pointerEvents: window.__bridgeLogs?.pointerEvents ?? [],
    };
  });

  expect(result.hitHandle).toBe(scene.boxHandle);
  expect(result.pointerEvents.some((entry) => entry.handle === scene.boxHandle)).toBe(true);
});

test('canvas owns touch input by default', async ({ page }) => {
  await gotoBridgePage(page);
  const touchAction = await page.evaluate(() => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const canvas = document.getElementById('fui-canvas');
    if (runtime === null || runtime === undefined || !(canvas instanceof HTMLCanvasElement)) {
      throw new Error('Bridge runtime is not ready.');
    }
    return canvas.style.touchAction;
  });

  expect(touchAction).toBe('none');
});

test('touch long press fires through the long-press owner callback', async ({ page }) => {
  await gotoBridgePage(page);
  const scene = await buildInteractiveBoxScene(page);

  const result = await page.evaluate(async (handles) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const canvas = document.getElementById('fui-canvas');
    if (runtime === null || runtime === undefined || !(canvas instanceof HTMLCanvasElement)) {
      throw new Error('Bridge runtime is not ready.');
    }
    runtime.resetLogs();

    const longPressEvents: {
      handle: string;
      x: number;
      y: number;
      pointerId: number;
      pointerType: number;
      modifiers: number;
      durationMs: number;
    }[] = [];
    const callbacks = window.__effindomCallbacks ?? {};
    const previousResolveLongPressOwner = callbacks.resolveLongPressOwner;
    const previousLongPressEvent = callbacks.onLongPressEventWithCoords;
    const previousVibrateDescriptor = Object.getOwnPropertyDescriptor(navigator, 'vibrate');
    const vibrationRequests: VibratePattern[] = [];
    Object.defineProperty(navigator, 'vibrate', {
      configurable: true,
      value(pattern: VibratePattern): boolean {
        vibrationRequests.push(pattern);
        return true;
      },
    });
    callbacks.resolveLongPressOwner = (handle): bigint | null => {
      return BigInt(handle.toString()) === BigInt(handles.boxHandle) ? BigInt(handles.boxHandle) : null;
    };
    callbacks.onLongPressEventWithCoords = (
      handle,
      x,
      y,
      pointerId,
      pointerType,
      modifiers,
      durationMs,
    ): boolean => {
      longPressEvents.push({
        handle: handle.toString(),
        x,
        y,
        pointerId,
        pointerType,
        modifiers,
        durationMs,
      });
      return true;
    };
    window.__effindomCallbacks = callbacks;

    const rect = canvas.getBoundingClientRect();
    canvas.dispatchEvent(new PointerEvent('pointerdown', {
      bubbles: true,
      cancelable: true,
      pointerId: 201,
      pointerType: 'touch',
      isPrimary: true,
      button: 0,
      buttons: 1,
      clientX: rect.left + 40,
      clientY: rect.top + 40,
      shiftKey: true,
    }));
    await new Promise((resolve) => window.setTimeout(resolve, 560));
    canvas.dispatchEvent(new PointerEvent('pointerup', {
      bubbles: true,
      cancelable: true,
      pointerId: 201,
      pointerType: 'touch',
      isPrimary: true,
      button: 0,
      buttons: 0,
      clientX: rect.left + 40,
      clientY: rect.top + 40,
      shiftKey: true,
    }));

    if (previousResolveLongPressOwner === undefined) {
      delete callbacks.resolveLongPressOwner;
    } else {
      callbacks.resolveLongPressOwner = previousResolveLongPressOwner;
    }
    if (previousLongPressEvent === undefined) {
      delete callbacks.onLongPressEventWithCoords;
    } else {
      callbacks.onLongPressEventWithCoords = previousLongPressEvent;
    }
    if (previousVibrateDescriptor === undefined) {
      delete (navigator as { vibrate?: Navigator['vibrate'] }).vibrate;
    } else {
      Object.defineProperty(navigator, 'vibrate', previousVibrateDescriptor);
    }

    return {
      longPressEvents,
      vibrationRequests,
      pointerEvents: window.__bridgeLogs?.pointerEvents ?? [],
    };
  }, scene);

  expect(result.longPressEvents).toHaveLength(1);
  expect(result.longPressEvents[0]?.handle).toBe(scene.boxHandle);
  expect(Math.abs((result.longPressEvents[0]?.x ?? 0) - 40)).toBeLessThanOrEqual(1);
  expect(Math.abs((result.longPressEvents[0]?.y ?? 0) - 40)).toBeLessThanOrEqual(1);
  expect(result.longPressEvents[0]?.pointerId).toBe(201);
  expect(result.vibrationRequests).toEqual([25]);
  expect(result.longPressEvents[0]?.pointerType).toBe(2);
  expect(result.longPressEvents[0]?.modifiers).toBe(1);
  expect(result.longPressEvents[0]?.durationMs).toBe(500);
  expect(result.pointerEvents.some((entry) => entry.eventType === 1)).toBe(true);
  expect(result.pointerEvents.some((entry) => entry.eventType === 2)).toBe(false);
});

test('native contextmenu during touch long press fires the active long-press gesture once', async ({ page }) => {
  await gotoBridgePage(page);
  const scene = await buildInteractiveBoxScene(page);

  const result = await page.evaluate((handles) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const canvas = document.getElementById('fui-canvas');
    if (runtime === null || runtime === undefined || !(canvas instanceof HTMLCanvasElement)) {
      throw new Error('Bridge runtime is not ready.');
    }
    runtime.resetLogs();

    const callbacks = window.__effindomCallbacks ?? {};
    const previousResolveLongPressOwner = callbacks.resolveLongPressOwner;
    const previousLongPressEvent = callbacks.onLongPressEventWithCoords;
    const longPressEvents: string[] = [];
    callbacks.resolveLongPressOwner = (handle): bigint | null => {
      return BigInt(handle.toString()) === BigInt(handles.boxHandle) ? BigInt(handles.boxHandle) : null;
    };
    callbacks.onLongPressEventWithCoords = (handle): boolean => {
      longPressEvents.push(handle.toString());
      return true;
    };
    window.__effindomCallbacks = callbacks;

    const rect = canvas.getBoundingClientRect();
    const down = new PointerEvent('pointerdown', {
      bubbles: true,
      cancelable: true,
      pointerId: 207,
      pointerType: 'touch',
      isPrimary: true,
      button: 0,
      buttons: 1,
      clientX: rect.left + 40,
      clientY: rect.top + 40,
    });
    canvas.dispatchEvent(down);
    const contextMenu = new MouseEvent('contextmenu', {
      bubbles: true,
      cancelable: true,
      button: 0,
      clientX: rect.left + 40,
      clientY: rect.top + 40,
    });
    canvas.dispatchEvent(contextMenu);
    const up = new PointerEvent('pointerup', {
      bubbles: true,
      cancelable: true,
      pointerId: 207,
      pointerType: 'touch',
      isPrimary: true,
      button: 0,
      buttons: 0,
      clientX: rect.left + 40,
      clientY: rect.top + 40,
    });
    canvas.dispatchEvent(up);

    if (previousResolveLongPressOwner === undefined) {
      delete callbacks.resolveLongPressOwner;
    } else {
      callbacks.resolveLongPressOwner = previousResolveLongPressOwner;
    }
    if (previousLongPressEvent === undefined) {
      delete callbacks.onLongPressEventWithCoords;
    } else {
      callbacks.onLongPressEventWithCoords = previousLongPressEvent;
    }

    return {
      contextMenuPrevented: contextMenu.defaultPrevented,
      longPressEvents,
      pointerEvents: window.__bridgeLogs?.pointerEvents ?? [],
    };
  }, scene);

  expect(result.contextMenuPrevented).toBe(true);
  expect(result.longPressEvents).toEqual([scene.boxHandle]);
  expect(result.pointerEvents.some((entry) => entry.eventType === 1)).toBe(true);
  expect(result.pointerEvents.some((entry) => entry.eventType === 2)).toBe(false);
});

test('touch long press cancels on movement, early pointerup, and second touch', async ({ page }) => {
  await gotoBridgePage(page);
  const scene = await buildInteractiveBoxScene(page);

  const result = await page.evaluate(async (handles) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const canvas = document.getElementById('fui-canvas');
    if (runtime === null || runtime === undefined || !(canvas instanceof HTMLCanvasElement)) {
      throw new Error('Bridge runtime is not ready.');
    }

    const callbacks = window.__effindomCallbacks ?? {};
    const previousResolveLongPressOwner = callbacks.resolveLongPressOwner;
    const previousLongPressEvent = callbacks.onLongPressEventWithCoords;
    const longPressEvents: string[] = [];
    callbacks.resolveLongPressOwner = (handle): bigint | null => {
      return BigInt(handle.toString()) === BigInt(handles.boxHandle) ? BigInt(handles.boxHandle) : null;
    };
    callbacks.onLongPressEventWithCoords = (_handle, _x, _y, pointerId): boolean => {
      longPressEvents.push(String(pointerId));
      return true;
    };
    window.__effindomCallbacks = callbacks;

    const rect = canvas.getBoundingClientRect();
    const emit = (type: string, pointerId: number, x: number, y: number): void => {
      canvas.dispatchEvent(new PointerEvent(type, {
        bubbles: true,
        cancelable: true,
        pointerId,
        pointerType: 'touch',
        isPrimary: pointerId === 211 || pointerId === 221 || pointerId === 231,
        button: 0,
        buttons: type === 'pointerup' ? 0 : 1,
        clientX: rect.left + x,
        clientY: rect.top + y,
      }));
    };

    emit('pointerdown', 211, 40, 40);
    emit('pointermove', 211, 56, 40);
    await new Promise((resolve) => window.setTimeout(resolve, 560));
    emit('pointerup', 211, 56, 40);

    emit('pointerdown', 221, 40, 40);
    await new Promise((resolve) => window.setTimeout(resolve, 120));
    emit('pointerup', 221, 40, 40);
    await new Promise((resolve) => window.setTimeout(resolve, 460));

    emit('pointerdown', 231, 40, 40);
    emit('pointerdown', 232, 80, 40);
    await new Promise((resolve) => window.setTimeout(resolve, 560));
    emit('pointerup', 232, 80, 40);
    emit('pointerup', 231, 40, 40);

    if (previousResolveLongPressOwner === undefined) {
      delete callbacks.resolveLongPressOwner;
    } else {
      callbacks.resolveLongPressOwner = previousResolveLongPressOwner;
    }
    if (previousLongPressEvent === undefined) {
      delete callbacks.onLongPressEventWithCoords;
    } else {
      callbacks.onLongPressEventWithCoords = previousLongPressEvent;
    }

    return { longPressEvents };
  }, scene);

  expect(result.longPressEvents).toEqual([]);
});

test('nested touch pan ownership resolves child-only, parent-only, and both recognizers', async ({ page }) => {
  await gotoBridgePage(page);
  const scene = await buildNestedInteractiveBoxScene(page);

  const result = await page.evaluate((handles) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const canvas = document.getElementById('fui-canvas');
    if (runtime === null || runtime === undefined || !(canvas instanceof HTMLCanvasElement)) {
      throw new Error('Bridge runtime is not ready.');
    }
    runtime.resetPageZoom();
    runtime.resetLogs();

    const callbacks = window.__effindomCallbacks ?? {};
    const previousResolveGestureOwner = callbacks.resolveGestureOwner;
    const previousGetGestureIntent = callbacks.getGestureIntent;
    const previousGestureEvent = callbacks.onGestureEventWithCoords;
    let childRecognizer = true;
    let parentRecognizer = false;
    const gestureOwners: string[] = [];
    callbacks.resolveGestureOwner = (handle): bigint | null => {
      const hit = BigInt(handle.toString());
      const child = BigInt(handles.childHandle);
      const parent = BigInt(handles.parentHandle);
      if (hit === child && childRecognizer) {
        return child;
      }
      if ((hit === child || hit === parent) && parentRecognizer) {
        return parent;
      }
      return null;
    };
    callbacks.getGestureIntent = (): number => 1;
    callbacks.onGestureEventWithCoords = (handle, phase): boolean => {
      if (phase === 1) {
        gestureOwners.push(handle.toString());
      }
      return true;
    };
    window.__effindomCallbacks = callbacks;

    const rect = canvas.getBoundingClientRect();
    const emit = (type: string, pointerId: number, x: number, y: number): void => {
      canvas.dispatchEvent(new PointerEvent(type, {
        bubbles: true,
        cancelable: true,
        pointerId,
        pointerType: 'touch',
        isPrimary: pointerId % 10 === 1,
        button: 0,
        buttons: type === 'pointerup' ? 0 : 1,
        clientX: rect.left + x,
        clientY: rect.top + y,
      }));
    };
    const runPan = (baseId: number): void => {
      emit('pointerdown', baseId + 1, 30, 30);
      emit('pointerdown', baseId + 2, 70, 30);
      emit('pointermove', baseId + 1, 48, 30);
      emit('pointermove', baseId + 2, 88, 30);
      emit('pointerup', baseId + 1, 48, 30);
      emit('pointerup', baseId + 2, 88, 30);
    };

    childRecognizer = true;
    parentRecognizer = false;
    runPan(300);
    childRecognizer = false;
    parentRecognizer = true;
    runPan(310);
    childRecognizer = true;
    parentRecognizer = true;
    runPan(320);

    if (previousResolveGestureOwner === undefined) {
      delete callbacks.resolveGestureOwner;
    } else {
      callbacks.resolveGestureOwner = previousResolveGestureOwner;
    }
    if (previousGetGestureIntent === undefined) {
      delete callbacks.getGestureIntent;
    } else {
      callbacks.getGestureIntent = previousGetGestureIntent;
    }
    if (previousGestureEvent === undefined) {
      delete callbacks.onGestureEventWithCoords;
    } else {
      callbacks.onGestureEventWithCoords = previousGestureEvent;
    }

    return {
      gestureOwners,
      childHit: runtime.getHandleFromPoint(40, 40).toString(),
    };
  }, scene);

  expect(result.childHit).toBe(scene.childHandle);
  expect(result.gestureOwners).toEqual([
    scene.childHandle,
    scene.parentHandle,
    scene.childHandle,
  ]);
});

test('touch long press uses owner-specific duration and movement tolerance', async ({ page }) => {
  await gotoBridgePage(page);
  const scene = await buildInteractiveBoxScene(page);

  const result = await page.evaluate(async (handles) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const canvas = document.getElementById('fui-canvas');
    if (runtime === null || runtime === undefined || !(canvas instanceof HTMLCanvasElement)) {
      throw new Error('Bridge runtime is not ready.');
    }

    const longPressEvents: { handle: string; durationMs: number }[] = [];
    const callbacks = window.__effindomCallbacks ?? {};
    const previousResolveLongPressOwner = callbacks.resolveLongPressOwner;
    const previousGetLongPressMinimumDurationMs = callbacks.getLongPressMinimumDurationMs;
    const previousGetLongPressMovementTolerance = callbacks.getLongPressMovementTolerance;
    const previousLongPressEvent = callbacks.onLongPressEventWithCoords;
    let movementTolerance = 30;
    callbacks.resolveLongPressOwner = (handle): bigint | null => {
      return BigInt(handle.toString()) === BigInt(handles.boxHandle) ? BigInt(handles.boxHandle) : null;
    };
    callbacks.getLongPressMinimumDurationMs = (): number => 80;
    callbacks.getLongPressMovementTolerance = (): number => movementTolerance;
    callbacks.onLongPressEventWithCoords = (handle, _x, _y, _pointerId, _pointerType, _modifiers, durationMs): boolean => {
      longPressEvents.push({ handle: handle.toString(), durationMs });
      return true;
    };
    window.__effindomCallbacks = callbacks;

    const rect = canvas.getBoundingClientRect();
    const emit = (type: string, pointerId: number, x: number, y: number): void => {
      canvas.dispatchEvent(new PointerEvent(type, {
        bubbles: true,
        cancelable: true,
        pointerId,
        pointerType: 'touch',
        isPrimary: true,
        button: 0,
        buttons: type === 'pointerup' ? 0 : 1,
        clientX: rect.left + x,
        clientY: rect.top + y,
      }));
    };

    emit('pointerdown', 250, 40, 40);
    await new Promise((resolve) => window.setTimeout(resolve, 110));
    emit('pointerup', 250, 40, 40);

    movementTolerance = 2;
    emit('pointerdown', 251, 40, 40);
    emit('pointermove', 251, 44, 40);
    await new Promise((resolve) => window.setTimeout(resolve, 110));
    emit('pointerup', 251, 44, 40);

    if (previousResolveLongPressOwner === undefined) {
      delete callbacks.resolveLongPressOwner;
    } else {
      callbacks.resolveLongPressOwner = previousResolveLongPressOwner;
    }
    if (previousGetLongPressMinimumDurationMs === undefined) {
      delete callbacks.getLongPressMinimumDurationMs;
    } else {
      callbacks.getLongPressMinimumDurationMs = previousGetLongPressMinimumDurationMs;
    }
    if (previousGetLongPressMovementTolerance === undefined) {
      delete callbacks.getLongPressMovementTolerance;
    } else {
      callbacks.getLongPressMovementTolerance = previousGetLongPressMovementTolerance;
    }
    if (previousLongPressEvent === undefined) {
      delete callbacks.onLongPressEventWithCoords;
    } else {
      callbacks.onLongPressEventWithCoords = previousLongPressEvent;
    }

    return longPressEvents;
  }, scene);

  expect(result).toEqual([
    {
      handle: scene.boxHandle,
      durationMs: 80,
    },
  ]);
});

test('nested long press ownership resolves child-only, parent-only, and both recognizers', async ({ page }) => {
  await gotoBridgePage(page);
  const scene = await buildNestedInteractiveBoxScene(page);

  const result = await page.evaluate(async (handles) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const canvas = document.getElementById('fui-canvas');
    if (runtime === null || runtime === undefined || !(canvas instanceof HTMLCanvasElement)) {
      throw new Error('Bridge runtime is not ready.');
    }
    runtime.resetLogs();

    const callbacks = window.__effindomCallbacks ?? {};
    const previousResolveLongPressOwner = callbacks.resolveLongPressOwner;
    const previousLongPressEvent = callbacks.onLongPressEventWithCoords;
    let childRecognizer = true;
    let parentRecognizer = false;
    const longPressOwners: string[] = [];
    callbacks.resolveLongPressOwner = (handle): bigint | null => {
      const hit = BigInt(handle.toString());
      const child = BigInt(handles.childHandle);
      const parent = BigInt(handles.parentHandle);
      if (hit === child && childRecognizer) {
        return child;
      }
      if ((hit === child || hit === parent) && parentRecognizer) {
        return parent;
      }
      return null;
    };
    callbacks.onLongPressEventWithCoords = (handle): boolean => {
      longPressOwners.push(handle.toString());
      return true;
    };
    window.__effindomCallbacks = callbacks;

    const rect = canvas.getBoundingClientRect();
    const emit = (type: string, pointerId: number, x: number, y: number): void => {
      canvas.dispatchEvent(new PointerEvent(type, {
        bubbles: true,
        cancelable: true,
        pointerId,
        pointerType: 'touch',
        isPrimary: true,
        button: 0,
        buttons: type === 'pointerup' ? 0 : 1,
        clientX: rect.left + x,
        clientY: rect.top + y,
      }));
    };
    const runLongPress = async (pointerId: number): Promise<void> => {
      emit('pointerdown', pointerId, 40, 40);
      await new Promise((resolve) => window.setTimeout(resolve, 540));
      emit('pointerup', pointerId, 40, 40);
    };

    childRecognizer = true;
    parentRecognizer = false;
    await runLongPress(401);
    childRecognizer = false;
    parentRecognizer = true;
    await runLongPress(402);
    childRecognizer = true;
    parentRecognizer = true;
    await runLongPress(403);

    if (previousResolveLongPressOwner === undefined) {
      delete callbacks.resolveLongPressOwner;
    } else {
      callbacks.resolveLongPressOwner = previousResolveLongPressOwner;
    }
    if (previousLongPressEvent === undefined) {
      delete callbacks.onLongPressEventWithCoords;
    } else {
      callbacks.onLongPressEventWithCoords = previousLongPressEvent;
    }

    return {
      longPressOwners,
      childHit: runtime.getHandleFromPoint(40, 40).toString(),
    };
  }, scene);

  expect(result.childHit).toBe(scene.childHandle);
  expect(result.longPressOwners).toEqual([
    scene.childHandle,
    scene.parentHandle,
    scene.childHandle,
  ]);
});

test('second touch cancels app touch routing without emitting stale pointer up', async ({ page }) => {
  await gotoBridgePage(page);
  await buildInteractiveBoxScene(page);

  const result = await page.evaluate(() => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const canvas = document.getElementById('fui-canvas');
    if (runtime === null || runtime === undefined || !(canvas instanceof HTMLCanvasElement)) {
      throw new Error('Bridge runtime is not ready.');
    }
    runtime.resetLogs();
    const rect = canvas.getBoundingClientRect();
    const emit = (type: string, pointerId: number, x: number, y: number): boolean => {
      const event = new PointerEvent(type, {
        bubbles: true,
        cancelable: true,
        pointerId,
        pointerType: 'touch',
        isPrimary: pointerId === 31,
        button: 0,
        buttons: type === 'pointerup' ? 0 : 1,
        clientX: rect.left + x,
        clientY: rect.top + y,
      });
      canvas.dispatchEvent(event);
      return event.defaultPrevented;
    };

    const firstDownPrevented = emit('pointerdown', 31, 40, 40);
    const secondDownPrevented = emit('pointerdown', 32, 80, 80);
    const secondUpPrevented = emit('pointerup', 32, 80, 80);
    const firstUpPrevented = emit('pointerup', 31, 40, 40);

    return {
      firstDownPrevented,
      secondDownPrevented,
      secondUpPrevented,
      firstUpPrevented,
      capturedHandle: runtime.getCapturedPointerHandle()?.toString() ?? null,
      pointerEvents: window.__bridgeLogs?.pointerEvents ?? [],
    };
  });

  expect(result.firstDownPrevented).toBe(true);
  expect(result.secondDownPrevented).toBe(true);
  expect(result.secondUpPrevented).toBe(true);
  expect(result.firstUpPrevented).toBe(true);
  expect(result.capturedHandle).toBeNull();
  expect(result.pointerEvents.some((entry) => entry.eventType === 1)).toBe(true);
  expect(result.pointerEvents.some((entry) => entry.eventType === 2)).toBe(false);
});

test('pointer callbacks include browser pointer metadata', async ({ page }) => {
  await gotoBridgePage(page);
  await buildInteractiveBoxScene(page);

  const result = await page.evaluate(() => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const canvas = document.getElementById('fui-canvas');
    if (runtime === null || runtime === undefined || !(canvas instanceof HTMLCanvasElement)) {
      throw new Error('Bridge runtime is not ready.');
    }

    runtime.resetLogs();
    const rect = canvas.getBoundingClientRect();
    const event = new PointerEvent('pointerdown', {
      bubbles: true,
      cancelable: true,
      pointerId: 47,
      pointerType: 'pen',
      button: 1,
      buttons: 3,
      pressure: 0.625,
      width: 11,
      height: 13,
      shiftKey: true,
      clientX: rect.left + 40,
      clientY: rect.top + 40,
    });
    canvas.dispatchEvent(event);
    canvas.dispatchEvent(new PointerEvent('pointerdown', {
      bubbles: true,
      cancelable: true,
      pointerId: 48,
      pointerType: 'pen',
      button: 1,
      buttons: 3,
      pressure: 0.5,
      width: 11,
      height: 13,
      clientX: rect.left + 42,
      clientY: rect.top + 40,
    }));
    canvas.dispatchEvent(new PointerEvent('pointerdown', {
      bubbles: true,
      cancelable: true,
      pointerId: 51,
      pointerType: 'pen',
      button: 1,
      buttons: 3,
      pressure: 0.5,
      width: 11,
      height: 13,
      detail: 7,
      clientX: rect.left + 42,
      clientY: rect.top + 40,
    }));
    canvas.dispatchEvent(new PointerEvent('pointerdown', {
      bubbles: true,
      cancelable: true,
      pointerId: 49,
      pointerType: 'pen',
      button: 1,
      buttons: 3,
      pressure: 0.5,
      width: 11,
      height: 13,
      clientX: rect.left + 42,
      clientY: rect.top + 40,
    }));
    canvas.dispatchEvent(new PointerEvent('pointerdown', {
      bubbles: true,
      cancelable: true,
      pointerId: 50,
      pointerType: 'pen',
      button: 1,
      buttons: 3,
      pressure: 0.5,
      width: 11,
      height: 13,
      clientX: rect.left + 42,
      clientY: rect.top + 40,
    }));
    return {
      prevented: event.defaultPrevented,
      pointerEvents: window.__bridgeLogs?.pointerEvents ?? [],
    };
  });

  expect(result.prevented).toBe(false);
  const down = result.pointerEvents.find((entry) => entry.eventType === 1);
  expect(down).toBeDefined();
  expect(down?.pointerId).toBe(47);
  expect(down?.pointerType).toBe(3);
  expect(down?.button).toBe(1);
  expect(down?.buttons).toBe(3);
  expect(down?.modifiers).toBe(1);
  expect(down?.pressure).toBeCloseTo(0.625, 3);
  expect(down?.width).toBe(11);
  expect(down?.height).toBe(13);
  expect(down?.clickCount).toBe(1);
  const downEvents = result.pointerEvents.filter((entry) => entry.eventType === 1);
  expect(downEvents[1]?.clickCount).toBe(2);
  expect(downEvents[2]?.clickCount).toBe(7);
  expect(downEvents[3]?.clickCount).toBe(3);
  expect(downEvents[4]?.clickCount).toBe(4);
});

test('right-click pointer delivery runs before context menu fallback', async ({ page }) => {
  await gotoBridgePage(page);
  await buildInteractiveBoxScene(page);

  const result = await page.evaluate(() => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const canvas = document.getElementById('fui-canvas');
    const callbacks = window.__effindomCallbacks;
    if (runtime === null || runtime === undefined || !(canvas instanceof HTMLCanvasElement) || callbacks === undefined) {
      throw new Error('Bridge runtime is not ready.');
    }

    runtime.resetLogs();
    const calls: string[] = [];
    const previousPointer = callbacks.onPointerEventWithMetadata;
    const previousContextMenu = callbacks.onContextMenu;
    callbacks.onPointerEventWithMetadata = (...args) => {
      const handled = previousPointer?.(...args) === true;
      calls.push(`pointer:${String(args[0])}:${String(args[7])}`);
      return handled;
    };
    callbacks.onContextMenu = (handle, x, y) => {
      calls.push('context');
      previousContextMenu?.(handle, x, y);
    };

    const rect = canvas.getBoundingClientRect();
    const event = new PointerEvent('pointerdown', {
      bubbles: true,
      cancelable: true,
      pointerId: 57,
      pointerType: 'mouse',
      button: 2,
      buttons: 2,
      clientX: rect.left + 40,
      clientY: rect.top + 40,
    });
    canvas.dispatchEvent(event);
    return {
      prevented: event.defaultPrevented,
      calls,
      pointerEvents: window.__bridgeLogs?.pointerEvents ?? [],
    };
  });

  expect(result.prevented).toBe(true);
  expect(result.calls.slice(0, 2)).toEqual(['pointer:1:2', 'context']);
  const down = result.pointerEvents.find((entry) => entry.eventType === 1);
  expect(down?.button).toBe(2);
});

test('handled right-click pointer delivery prevents context menu fallback', async ({ page }) => {
  await gotoBridgePage(page);
  await buildInteractiveBoxScene(page);

  const result = await page.evaluate(() => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const canvas = document.getElementById('fui-canvas');
    const callbacks = window.__effindomCallbacks;
    if (runtime === null || runtime === undefined || !(canvas instanceof HTMLCanvasElement) || callbacks === undefined) {
      throw new Error('Bridge runtime is not ready.');
    }

    runtime.resetLogs();
    let contextMenus = 0;
    const previousPointer = callbacks.onPointerEventWithMetadata;
    const previousContextMenu = callbacks.onContextMenu;
    callbacks.onPointerEventWithMetadata = (...args) => {
      previousPointer?.(...args);
      return args[0] === 1 && args[7] === 2 ? 1 as unknown as boolean : 0 as unknown as boolean;
    };
    callbacks.onContextMenu = (handle, x, y) => {
      contextMenus += 1;
      previousContextMenu?.(handle, x, y);
    };

    const rect = canvas.getBoundingClientRect();
    const event = new PointerEvent('pointerdown', {
      bubbles: true,
      cancelable: true,
      pointerId: 58,
      pointerType: 'mouse',
      button: 2,
      buttons: 2,
      clientX: rect.left + 40,
      clientY: rect.top + 40,
    });
    canvas.dispatchEvent(event);
    return {
      prevented: event.defaultPrevented,
      contextMenus,
      pointerEvents: window.__bridgeLogs?.pointerEvents ?? [],
    };
  });

  expect(result.prevented).toBe(true);
  expect(result.contextMenus).toBe(0);
  const down = result.pointerEvents.find((entry) => entry.eventType === 1);
  expect(down?.button).toBe(2);
});

test('pointercancel is delivered as cancel metadata and clears captured pointer state', async ({ page }) => {
  await gotoBridgePage(page);
  await buildInteractiveBoxScene(page);

  const result = await page.evaluate(() => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const canvas = document.getElementById('fui-canvas');
    if (runtime === null || runtime === undefined || !(canvas instanceof HTMLCanvasElement)) {
      throw new Error('Bridge runtime is not ready.');
    }

    runtime.resetLogs();
    const rect = canvas.getBoundingClientRect();
    const emit = (type: string, x: number, y: number): boolean => {
      const event = new PointerEvent(type, {
        bubbles: true,
        cancelable: true,
        pointerId: 51,
        pointerType: 'touch',
        isPrimary: true,
        button: 0,
        buttons: type === 'pointercancel' ? 0 : 1,
        pressure: type === 'pointercancel' ? 0 : 0.5,
        clientX: rect.left + x,
        clientY: rect.top + y,
      });
      canvas.dispatchEvent(event);
      return event.defaultPrevented;
    };

    const downPrevented = emit('pointerdown', 40, 40);
    const cancelPrevented = emit('pointercancel', 42, 42);

    return {
      downPrevented,
      cancelPrevented,
      capturedHandle: runtime.getCapturedPointerHandle()?.toString() ?? null,
      pointerEvents: window.__bridgeLogs?.pointerEvents ?? [],
    };
  });

  expect(result.downPrevented).toBe(true);
  expect(result.cancelPrevented).toBe(true);
  expect(result.capturedHandle).toBeNull();
  const cancel = result.pointerEvents.find((entry) => entry.eventType === 6);
  expect(cancel).toBeDefined();
  expect(cancel?.pointerId).toBe(51);
  expect(cancel?.pointerType).toBe(2);
  expect(result.pointerEvents.some((entry) => entry.eventType === 2)).toBe(false);
});

test('page zoom API maps screen coordinates through inverse scene transform', async ({ page }) => {
  await gotoBridgePage(page);
  const scene = await buildInteractiveBoxScene(page);

  const result = await page.evaluate(() => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const bridge = window.EffinDomBrowserBridge;
    if (runtime === null || runtime === undefined || bridge === undefined) {
      throw new Error('Bridge runtime is not ready.');
    }
    bridge.setPageZoom(2.0, -20.0, -10.0);
    runtime.core._ed_render_frame(performance.now());
    const zoom = bridge.getPageZoom();
    return {
      zoom,
      scenePoint: runtime.screenToScenePoint(100, 90),
      hitHandle: runtime.getHandleFromPoint(100, 90).toString(),
    };
  });

  expect(result.zoom).toEqual({ scale: 2, offsetX: -20, offsetY: -10 });
  expect(result.scenePoint.x).toBe(60);
  expect(result.scenePoint.y).toBe(50);
  expect(result.hitHandle).toBe(scene.boxHandle);
});

test('pointer event hit testing matches page zoom visual transform', async ({ page }) => {
  await gotoBridgePage(page);

  const result = await page.evaluate(() => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const canvas = document.getElementById('fui-canvas');
    if (runtime === null || runtime === undefined || !(canvas instanceof HTMLCanvasElement)) {
      throw new Error('Bridge runtime is not ready.');
    }

    const ui = runtime.ui;
    const toHandle = (handle: unknown): string => {
      if (typeof handle === 'bigint') return handle.toString();
      if (typeof handle === 'number') return BigInt(handle).toString();
      if (typeof handle === 'string') return BigInt(handle).toString();
      throw new TypeError(`Cannot convert ${String(handle)} to a handle string.`);
    };

    runtime.resetLogs();
    runtime.resetAppSession();
    ui._ui_reset();

    const root = ui._ui_create_node(0);
    const target = ui._ui_create_node(0);
    ui._ui_set_root(root);
    ui._ui_node_add_child(root, target);
    ui._ui_set_width(root, 320, 0);
    ui._ui_set_height(root, 220, 0);
    ui._ui_set_width(target, 40, 0);
    ui._ui_set_height(target, 40, 0);
    ui._ui_set_position_type(target, 1);
    ui._ui_set_position(target, 90, 70, 0, 0);
    ui._ui_set_interactive(target, 1);
    runtime.commitFrame();
    runtime.flushPendingCommit();
    runtime.resetLogs();

    runtime.setPageZoom(2.0, -20.0, -10.0);
    const directHit = runtime.getHandleFromPoint(190, 160).toString();

    const rect = canvas.getBoundingClientRect();
    canvas.dispatchEvent(new PointerEvent('pointerdown', {
      bubbles: true,
      cancelable: true,
      pointerId: 301,
      pointerType: 'mouse',
      button: 0,
      buttons: 1,
      clientX: rect.left + 190,
      clientY: rect.top + 160,
    }));
    canvas.dispatchEvent(new PointerEvent('pointerup', {
      bubbles: true,
      cancelable: true,
      pointerId: 301,
      pointerType: 'mouse',
      button: 0,
      buttons: 0,
      clientX: rect.left + 190,
      clientY: rect.top + 160,
    }));

    return {
      targetHandle: toHandle(target),
      directHit,
      pointerEvents: window.__bridgeLogs?.pointerEvents ?? [],
    };
  });

  expect(result.directHit).toBe(result.targetHandle);
  expect(result.pointerEvents.some((entry) => entry.eventType === 1 && entry.handle === result.targetHandle)).toBe(true);
});

test('page zoom clamps to identity and viewport bounds', async ({ page }) => {
  await gotoBridgePage(page);

  const result = await page.evaluate(() => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    if (runtime === null || runtime === undefined) {
      throw new Error('Bridge runtime is not ready.');
    }

    runtime.setPageZoom(0.5, 40.0, 30.0);
    const belowIdentity = runtime.getPageZoom();

    runtime.setPageZoom(2.0, -999.0, 999.0);
    const bounded = runtime.getPageZoom();

    return {
      belowIdentity,
      bounded,
    };
  });

  expect(result.belowIdentity).toEqual({ scale: 1, offsetX: 0, offsetY: 0 });
  expect(result.bounded.scale).toBe(2);
  expect(result.bounded.offsetX).toBe(-320);
  expect(result.bounded.offsetY).toBe(0);
});

test('two-finger touch updates framework-owned page zoom', async ({ page }) => {
  await gotoBridgePage(page);
  await buildInteractiveBoxScene(page);

  const result = await page.evaluate(() => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const canvas = document.getElementById('fui-canvas');
    if (runtime === null || runtime === undefined || !(canvas instanceof HTMLCanvasElement)) {
      throw new Error('Bridge runtime is not ready.');
    }
    runtime.resetPageZoom();
    const rect = canvas.getBoundingClientRect();
    const emit = (type: string, pointerId: number, x: number, y: number): boolean => {
      const event = new PointerEvent(type, {
        bubbles: true,
        cancelable: true,
        pointerId,
        pointerType: 'touch',
        isPrimary: pointerId === 41,
        button: 0,
        buttons: type === 'pointerup' ? 0 : 1,
        clientX: rect.left + x,
        clientY: rect.top + y,
      });
      canvas.dispatchEvent(event);
      return event.defaultPrevented;
    };

    const firstDownPrevented = emit('pointerdown', 41, 80, 80);
    const secondDownPrevented = emit('pointerdown', 42, 120, 80);
    const firstMovePrevented = emit('pointermove', 41, 60, 80);
    const secondMovePrevented = emit('pointermove', 42, 140, 80);
    const zoom = runtime.getPageZoom();
    const anchor = runtime.screenToScenePoint(100, 80);
    emit('pointerup', 41, 60, 80);
    emit('pointerup', 42, 140, 80);

    return {
      firstDownPrevented,
      secondDownPrevented,
      firstMovePrevented,
      secondMovePrevented,
      zoom,
      anchor,
      pointerEvents: window.__bridgeLogs?.pointerEvents ?? [],
    };
  });

  expect(result.firstDownPrevented).toBe(true);
  expect(result.secondDownPrevented).toBe(true);
  expect(result.firstMovePrevented).toBe(true);
  expect(result.secondMovePrevented).toBe(true);
  expect(result.zoom.scale).toBeCloseTo(2.0, 4);
  expect(Math.abs(result.anchor.x - 100.0)).toBeLessThanOrEqual(1.0);
  expect(Math.abs(result.anchor.y - 80.0)).toBeLessThanOrEqual(1.0);
  expect(result.pointerEvents.some((entry) => entry.eventType === 2)).toBe(false);
});

test('two-finger pan routes to a pan-intent control gesture owner', async ({ page }) => {
  await gotoBridgePage(page);
  const scene = await buildInteractiveBoxScene(page);

  const result = await page.evaluate((handles) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const canvas = document.getElementById('fui-canvas');
    if (runtime === null || runtime === undefined || !(canvas instanceof HTMLCanvasElement)) {
      throw new Error('Bridge runtime is not ready.');
    }
    runtime.resetPageZoom();
    runtime.resetLogs();

    const gestureEvents: {
      phase: number;
      kind: number;
      handle: string;
      x: number;
      y: number;
      deltaX: number;
      deltaY: number;
      scale: number;
      pointerCount: number;
    }[] = [];
    const callbacks = window.__effindomCallbacks ?? {};
    const previousResolveGestureOwner = callbacks.resolveGestureOwner;
    const previousGetGestureIntent = callbacks.getGestureIntent;
    const previousGestureEvent = callbacks.onGestureEventWithCoords;
    callbacks.resolveGestureOwner = (handle): bigint | null => {
      return BigInt(handle.toString()) === BigInt(handles.boxHandle) ? BigInt(handles.boxHandle) : null;
    };
    callbacks.getGestureIntent = (): number => 1;
    callbacks.onGestureEventWithCoords = (
      handle,
      phase,
      kind,
      x,
      y,
      deltaX,
      deltaY,
      scale,
      pointerCount,
    ): boolean => {
      gestureEvents.push({
        phase,
        kind,
        handle: handle.toString(),
        x,
        y,
        deltaX,
        deltaY,
        scale,
        pointerCount,
      });
      return true;
    };
    window.__effindomCallbacks = callbacks;

    const rect = canvas.getBoundingClientRect();
    const emit = (type: string, pointerId: number, x: number, y: number): boolean => {
      const event = new PointerEvent(type, {
        bubbles: true,
        cancelable: true,
        pointerId,
        pointerType: 'touch',
        isPrimary: pointerId === 101,
        button: 0,
        buttons: type === 'pointerup' ? 0 : 1,
        clientX: rect.left + x,
        clientY: rect.top + y,
      });
      canvas.dispatchEvent(event);
      return event.defaultPrevented;
    };

    emit('pointerdown', 101, 40, 40);
    emit('pointerdown', 102, 80, 40);
    const firstMovePrevented = emit('pointermove', 101, 60, 40);
    const secondMovePrevented = emit('pointermove', 102, 100, 40);
    emit('pointerup', 101, 60, 40);
    emit('pointerup', 102, 100, 40);

    if (previousResolveGestureOwner === undefined) {
      delete callbacks.resolveGestureOwner;
    } else {
      callbacks.resolveGestureOwner = previousResolveGestureOwner;
    }
    if (previousGetGestureIntent === undefined) {
      delete callbacks.getGestureIntent;
    } else {
      callbacks.getGestureIntent = previousGetGestureIntent;
    }
    if (previousGestureEvent === undefined) {
      delete callbacks.onGestureEventWithCoords;
    } else {
      callbacks.onGestureEventWithCoords = previousGestureEvent;
    }

    return {
      firstMovePrevented,
      secondMovePrevented,
      gestureEvents,
      zoom: runtime.getPageZoom(),
      pointerEvents: window.__bridgeLogs?.pointerEvents ?? [],
    };
  }, scene);

  expect(result.firstMovePrevented).toBe(true);
  expect(result.secondMovePrevented).toBe(true);
  expect(result.gestureEvents.map((entry) => entry.phase)).toEqual([1, 2, 3]);
  expect(result.gestureEvents.every((entry) => entry.kind === 1)).toBe(true);
  expect(result.gestureEvents.every((entry) => entry.handle === scene.boxHandle)).toBe(true);
  expect(result.gestureEvents[1]?.deltaX).toBeGreaterThan(0);
  expect(Math.abs((result.gestureEvents[1]?.scale ?? 0) - 1)).toBeLessThan(0.01);
  expect(result.gestureEvents[0]?.pointerCount).toBe(2);
  expect(result.zoom).toEqual({ scale: 1, offsetX: 0, offsetY: 0 });
  expect(result.pointerEvents.some((entry) => entry.eventType === 2)).toBe(false);
});

test('two-finger pinch routes to a pinch-intent control gesture owner', async ({ page }) => {
  await gotoBridgePage(page);
  const scene = await buildInteractiveBoxScene(page);

  const result = await page.evaluate((handles) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const canvas = document.getElementById('fui-canvas');
    if (runtime === null || runtime === undefined || !(canvas instanceof HTMLCanvasElement)) {
      throw new Error('Bridge runtime is not ready.');
    }
    runtime.resetPageZoom();
    runtime.resetLogs();

    const gestureEvents: {
      phase: number;
      kind: number;
      handle: string;
      x: number;
      y: number;
      scale: number;
    }[] = [];
    const callbacks = window.__effindomCallbacks ?? {};
    const previousResolveGestureOwner = callbacks.resolveGestureOwner;
    const previousGetGestureIntent = callbacks.getGestureIntent;
    const previousGestureEvent = callbacks.onGestureEventWithCoords;
    callbacks.resolveGestureOwner = (handle): bigint | null => {
      return BigInt(handle.toString()) === BigInt(handles.boxHandle) ? BigInt(handles.boxHandle) : null;
    };
    callbacks.getGestureIntent = (): number => 2;
    callbacks.onGestureEventWithCoords = (handle, phase, kind, x, y, _deltaX, _deltaY, scale): boolean => {
      gestureEvents.push({
        phase,
        kind,
        handle: handle.toString(),
        x,
        y,
        scale,
      });
      return true;
    };
    window.__effindomCallbacks = callbacks;

    const rect = canvas.getBoundingClientRect();
    const emit = (type: string, pointerId: number, x: number, y: number): void => {
      canvas.dispatchEvent(new PointerEvent(type, {
        bubbles: true,
        cancelable: true,
        pointerId,
        pointerType: 'touch',
        isPrimary: pointerId === 111,
        button: 0,
        buttons: type === 'pointerup' ? 0 : 1,
        clientX: rect.left + x,
        clientY: rect.top + y,
      }));
    };

    emit('pointerdown', 111, 40, 40);
    emit('pointerdown', 112, 80, 40);
    emit('pointermove', 111, 30, 40);
    emit('pointermove', 112, 90, 40);
    emit('pointerup', 111, 30, 40);
    emit('pointerup', 112, 90, 40);

    if (previousResolveGestureOwner === undefined) {
      delete callbacks.resolveGestureOwner;
    } else {
      callbacks.resolveGestureOwner = previousResolveGestureOwner;
    }
    if (previousGetGestureIntent === undefined) {
      delete callbacks.getGestureIntent;
    } else {
      callbacks.getGestureIntent = previousGetGestureIntent;
    }
    if (previousGestureEvent === undefined) {
      delete callbacks.onGestureEventWithCoords;
    } else {
      callbacks.onGestureEventWithCoords = previousGestureEvent;
    }

    return {
      gestureEvents,
      zoom: runtime.getPageZoom(),
      pointerEvents: window.__bridgeLogs?.pointerEvents ?? [],
    };
  }, scene);

  expect(result.gestureEvents[0]?.phase).toBe(1);
  expect(result.gestureEvents[result.gestureEvents.length - 1]?.phase).toBe(3);
  expect(result.gestureEvents.slice(1, -1).every((entry) => entry.phase === 2)).toBe(true);
  expect(result.gestureEvents.every((entry) => entry.kind === 2)).toBe(true);
  expect(result.gestureEvents.every((entry) => entry.handle === scene.boxHandle)).toBe(true);
  expect(Math.max(...result.gestureEvents.map((entry) => entry.scale))).toBeGreaterThan(1.25);
  expect(result.zoom).toEqual({ scale: 1, offsetX: 0, offsetY: 0 });
  expect(result.pointerEvents.some((entry) => entry.eventType === 2)).toBe(false);
});

test('two-finger pinch can start when only one touch pointer reports movement', async ({ page }) => {
  await gotoBridgePage(page);
  const scene = await buildInteractiveBoxScene(page);

  const result = await page.evaluate((handles) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const canvas = document.getElementById('fui-canvas');
    if (runtime === null || runtime === undefined || !(canvas instanceof HTMLCanvasElement)) {
      throw new Error('Bridge runtime is not ready.');
    }
    runtime.resetPageZoom();
    runtime.resetLogs();

    const gestureEvents: { phase: number; kind: number; scale: number }[] = [];
    const callbacks = window.__effindomCallbacks ?? {};
    const previousResolveGestureOwner = callbacks.resolveGestureOwner;
    const previousGetGestureIntent = callbacks.getGestureIntent;
    const previousGestureEvent = callbacks.onGestureEventWithCoords;
    callbacks.resolveGestureOwner = (handle): bigint | null => {
      return BigInt(handle.toString()) === BigInt(handles.boxHandle) ? BigInt(handles.boxHandle) : null;
    };
    callbacks.getGestureIntent = (): number => 2;
    callbacks.onGestureEventWithCoords = (_handle, phase, kind, _x, _y, _deltaX, _deltaY, scale): boolean => {
      gestureEvents.push({ phase, kind, scale });
      return true;
    };
    window.__effindomCallbacks = callbacks;

    const rect = canvas.getBoundingClientRect();
    const emit = (type: string, pointerId: number, x: number, y: number): void => {
      canvas.dispatchEvent(new PointerEvent(type, {
        bubbles: true,
        cancelable: true,
        pointerId,
        pointerType: 'touch',
        isPrimary: pointerId === 131,
        button: 0,
        buttons: type === 'pointerup' ? 0 : 1,
        clientX: rect.left + x,
        clientY: rect.top + y,
      }));
    };

    emit('pointerdown', 131, 40, 40);
    emit('pointerdown', 132, 80, 40);
    emit('pointermove', 132, 110, 40);
    emit('pointerup', 131, 40, 40);
    emit('pointerup', 132, 110, 40);

    if (previousResolveGestureOwner === undefined) {
      delete callbacks.resolveGestureOwner;
    } else {
      callbacks.resolveGestureOwner = previousResolveGestureOwner;
    }
    if (previousGetGestureIntent === undefined) {
      delete callbacks.getGestureIntent;
    } else {
      callbacks.getGestureIntent = previousGetGestureIntent;
    }
    if (previousGestureEvent === undefined) {
      delete callbacks.onGestureEventWithCoords;
    } else {
      callbacks.onGestureEventWithCoords = previousGestureEvent;
    }

    return {
      gestureEvents,
      zoom: runtime.getPageZoom(),
    };
  }, scene);

  expect(result.gestureEvents.map((entry) => entry.phase)).toEqual([1, 2, 3]);
  expect(result.gestureEvents.every((entry) => entry.kind === 2)).toBe(true);
  expect(result.gestureEvents[1]?.scale).toBeGreaterThan(1.5);
  expect(result.zoom).toEqual({ scale: 1, offsetX: 0, offsetY: 0 });
});

test('non-matching control gesture intent falls back to framework page zoom', async ({ page }) => {
  await gotoBridgePage(page);
  const scene = await buildInteractiveBoxScene(page);

  const result = await page.evaluate((handles) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const canvas = document.getElementById('fui-canvas');
    if (runtime === null || runtime === undefined || !(canvas instanceof HTMLCanvasElement)) {
      throw new Error('Bridge runtime is not ready.');
    }
    runtime.resetPageZoom();
    runtime.resetLogs();

    let gestureCalls = 0;
    const callbacks = window.__effindomCallbacks ?? {};
    const previousResolveGestureOwner = callbacks.resolveGestureOwner;
    const previousGetGestureIntent = callbacks.getGestureIntent;
    const previousGestureEvent = callbacks.onGestureEventWithCoords;
    callbacks.resolveGestureOwner = (handle): bigint | null => {
      return BigInt(handle.toString()) === BigInt(handles.boxHandle) ? BigInt(handles.boxHandle) : null;
    };
    callbacks.getGestureIntent = (): number => 1;
    callbacks.onGestureEventWithCoords = (): boolean => {
      gestureCalls += 1;
      return true;
    };
    window.__effindomCallbacks = callbacks;

    const rect = canvas.getBoundingClientRect();
    const emit = (type: string, pointerId: number, x: number, y: number): void => {
      canvas.dispatchEvent(new PointerEvent(type, {
        bubbles: true,
        cancelable: true,
        pointerId,
        pointerType: 'touch',
        isPrimary: pointerId === 121,
        button: 0,
        buttons: type === 'pointerup' ? 0 : 1,
        clientX: rect.left + x,
        clientY: rect.top + y,
      }));
    };

    emit('pointerdown', 121, 80, 80);
    emit('pointerdown', 122, 120, 80);
    emit('pointermove', 121, 60, 80);
    emit('pointermove', 122, 140, 80);
    const zoom = runtime.getPageZoom();
    emit('pointerup', 121, 60, 80);
    emit('pointerup', 122, 140, 80);

    if (previousResolveGestureOwner === undefined) {
      delete callbacks.resolveGestureOwner;
    } else {
      callbacks.resolveGestureOwner = previousResolveGestureOwner;
    }
    if (previousGetGestureIntent === undefined) {
      delete callbacks.getGestureIntent;
    } else {
      callbacks.getGestureIntent = previousGetGestureIntent;
    }
    if (previousGestureEvent === undefined) {
      delete callbacks.onGestureEventWithCoords;
    } else {
      callbacks.onGestureEventWithCoords = previousGestureEvent;
    }

    return { gestureCalls, zoom };
  }, scene);

  expect(result.gestureCalls).toBe(0);
  expect(result.zoom.scale).toBeCloseTo(2.0, 4);
});

test('unhandled pinch gesture falls back to framework page zoom while app events continue', async ({ page }) => {
  await gotoBridgePage(page);
  const scene = await buildInteractiveBoxScene(page);

  const result = await page.evaluate((handles) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const canvas = document.getElementById('fui-canvas');
    if (runtime === null || runtime === undefined || !(canvas instanceof HTMLCanvasElement)) {
      throw new Error('Bridge runtime is not ready.');
    }
    runtime.resetPageZoom();

    const gesturePhases: number[] = [];
    const callbacks = window.__effindomCallbacks ?? {};
    const previousResolveGestureOwner = callbacks.resolveGestureOwner;
    const previousGetGestureIntent = callbacks.getGestureIntent;
    const previousGestureEvent = callbacks.onGestureEventWithCoords;
    callbacks.resolveGestureOwner = (handle): bigint | null => {
      return BigInt(handle.toString()) === BigInt(handles.boxHandle) ? BigInt(handles.boxHandle) : null;
    };
    callbacks.getGestureIntent = (): number => 2;
    callbacks.onGestureEventWithCoords = (_handle, phase): boolean => {
      gesturePhases.push(phase);
      return false;
    };
    window.__effindomCallbacks = callbacks;

    const rect = canvas.getBoundingClientRect();
    const emit = (type: string, pointerId: number, x: number, y: number): void => {
      canvas.dispatchEvent(new PointerEvent(type, {
        bubbles: true,
        cancelable: true,
        pointerId,
        pointerType: 'touch',
        isPrimary: pointerId === 151,
        button: 0,
        buttons: type === 'pointerup' ? 0 : 1,
        clientX: rect.left + x,
        clientY: rect.top + y,
      }));
    };

    emit('pointerdown', 151, 40, 40);
    emit('pointerdown', 152, 80, 40);
    emit('pointermove', 151, 20, 40);
    emit('pointermove', 152, 100, 40);
    emit('pointermove', 151, 10, 40);
    emit('pointermove', 152, 110, 40);
    const zoom = runtime.getPageZoom();
    emit('pointerup', 151, 20, 40);
    emit('pointerup', 152, 100, 40);

    if (previousResolveGestureOwner === undefined) {
      delete callbacks.resolveGestureOwner;
    } else {
      callbacks.resolveGestureOwner = previousResolveGestureOwner;
    }
    if (previousGetGestureIntent === undefined) {
      delete callbacks.getGestureIntent;
    } else {
      callbacks.getGestureIntent = previousGetGestureIntent;
    }
    if (previousGestureEvent === undefined) {
      delete callbacks.onGestureEventWithCoords;
    } else {
      callbacks.onGestureEventWithCoords = previousGestureEvent;
    }

    return { gesturePhases, zoom };
  }, scene);

  expect(result.gesturePhases[0]).toBe(1);
  expect(result.gesturePhases.filter((phase) => phase === 2).length).toBeGreaterThan(1);
  expect(result.gesturePhases[result.gesturePhases.length - 1]).toBe(3);
  expect(result.zoom.scale).toBeGreaterThan(1);
});

test('two-finger touch cannot zoom below identity', async ({ page }) => {
  await gotoBridgePage(page);
  await buildInteractiveBoxScene(page);

  const result = await page.evaluate(() => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const canvas = document.getElementById('fui-canvas');
    if (runtime === null || runtime === undefined || !(canvas instanceof HTMLCanvasElement)) {
      throw new Error('Bridge runtime is not ready.');
    }
    runtime.resetPageZoom();
    const rect = canvas.getBoundingClientRect();
    const emit = (type: string, pointerId: number, x: number, y: number): void => {
      canvas.dispatchEvent(new PointerEvent(type, {
        bubbles: true,
        cancelable: true,
        pointerId,
        pointerType: 'touch',
        isPrimary: pointerId === 51,
        button: 0,
        buttons: type === 'pointerup' ? 0 : 1,
        clientX: rect.left + x,
        clientY: rect.top + y,
      }));
    };

    emit('pointerdown', 51, 60, 80);
    emit('pointerdown', 52, 140, 80);
    emit('pointermove', 51, 90, 80);
    emit('pointermove', 52, 110, 80);
    const zoom = runtime.getPageZoom();
    const hitHandle = runtime.getHandleFromPoint(80, 80).toString();
    emit('pointerup', 51, 90, 80);
    emit('pointerup', 52, 110, 80);

    return { zoom, hitHandle };
  });

  expect(result.zoom).toEqual({ scale: 1, offsetX: 0, offsetY: 0 });
  expect(result.hitHandle).not.toBe('0');
});

test('two-finger touch past max zoom keeps the midpoint anchored', async ({ page }) => {
  await gotoBridgePage(page);
  await buildInteractiveBoxScene(page);

  const result = await page.evaluate(() => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const canvas = document.getElementById('fui-canvas');
    if (runtime === null || runtime === undefined || !(canvas instanceof HTMLCanvasElement)) {
      throw new Error('Bridge runtime is not ready.');
    }
    runtime.setPageZoom(4.0, 0.0, 0.0);
    const rect = canvas.getBoundingClientRect();
    const emit = (type: string, pointerId: number, x: number, y: number): void => {
      canvas.dispatchEvent(new PointerEvent(type, {
        bubbles: true,
        cancelable: true,
        pointerId,
        pointerType: 'touch',
        isPrimary: pointerId === 71,
        button: 0,
        buttons: type === 'pointerup' ? 0 : 1,
        clientX: rect.left + x,
        clientY: rect.top + y,
      }));
    };

    emit('pointerdown', 71, 80, 80);
    emit('pointerdown', 72, 120, 80);
    emit('pointermove', 71, 60, 80);
    emit('pointermove', 72, 140, 80);
    const zoom = runtime.getPageZoom();
    const midpointScene = runtime.screenToScenePoint(100, 80);
    emit('pointerup', 71, 60, 80);
    emit('pointerup', 72, 140, 80);

    return { zoom, midpointScene };
  });

  expect(result.zoom).toEqual({ scale: 4, offsetX: 0, offsetY: 0 });
  expect(result.midpointScene.x).toBe(25);
  expect(result.midpointScene.y).toBe(20);
});

test('two-finger touch reverses immediately after max zoom saturation', async ({ page }) => {
  await gotoBridgePage(page);
  await buildInteractiveBoxScene(page);

  const result = await page.evaluate(() => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const canvas = document.getElementById('fui-canvas');
    if (runtime === null || runtime === undefined || !(canvas instanceof HTMLCanvasElement)) {
      throw new Error('Bridge runtime is not ready.');
    }
    runtime.setPageZoom(4.0, 0.0, 0.0);
    const rect = canvas.getBoundingClientRect();
    const emit = (type: string, pointerId: number, x: number, y: number): void => {
      canvas.dispatchEvent(new PointerEvent(type, {
        bubbles: true,
        cancelable: true,
        pointerId,
        pointerType: 'touch',
        isPrimary: pointerId === 81,
        button: 0,
        buttons: type === 'pointerup' ? 0 : 1,
        clientX: rect.left + x,
        clientY: rect.top + y,
      }));
    };

    emit('pointerdown', 81, 80, 80);
    emit('pointerdown', 82, 120, 80);
    emit('pointermove', 81, 60, 80);
    emit('pointermove', 82, 140, 80);
    const saturatedZoom = runtime.getPageZoom();
    emit('pointermove', 81, 65, 80);
    emit('pointermove', 82, 135, 80);
    const reversedZoom = runtime.getPageZoom();
    emit('pointerup', 81, 65, 80);
    emit('pointerup', 82, 135, 80);

    return { saturatedZoom, reversedZoom };
  });

  expect(result.saturatedZoom.scale).toBe(4);
  expect(result.reversedZoom.scale).toBeLessThan(4);
  expect(result.reversedZoom.scale).toBeGreaterThan(1);
});

test('runtime config can disable framework-owned page zoom', async ({ page }) => {
  await gotoBridgePage(page, '', { pageZoom: 'disabled' });
  await buildInteractiveBoxScene(page);

  const result = await page.evaluate(() => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const canvas = document.getElementById('fui-canvas');
    if (runtime === null || runtime === undefined || !(canvas instanceof HTMLCanvasElement)) {
      throw new Error('Bridge runtime is not ready.');
    }
    const rect = canvas.getBoundingClientRect();
    const emit = (type: string, pointerId: number, x: number, y: number): void => {
      canvas.dispatchEvent(new PointerEvent(type, {
        bubbles: true,
        cancelable: true,
        pointerId,
        pointerType: 'touch',
        isPrimary: pointerId === 61,
        button: 0,
        buttons: type === 'pointerup' ? 0 : 1,
        clientX: rect.left + x,
        clientY: rect.top + y,
      }));
    };

    emit('pointerdown', 61, 80, 80);
    emit('pointerdown', 62, 120, 80);
    emit('pointermove', 61, 60, 80);
    emit('pointermove', 62, 140, 80);
    const zoom = runtime.getPageZoom();
    emit('pointerup', 61, 60, 80);
    emit('pointerup', 62, 140, 80);

    return {
      pageZoomMode: runtime.pageZoomMode,
      enabled: runtime.isPageZoomEnabled(),
      zoom,
    };
  });

  expect(result.pageZoomMode).toBe('disabled');
  expect(result.enabled).toBe(false);
  expect(result.zoom).toEqual({ scale: 1, offsetX: 0, offsetY: 0 });
});

test('ctrl wheel trackpad pinch zooms around the cursor anchor', async ({ page }) => {
  await gotoBridgePage(page);
  await buildInteractiveBoxScene(page);

  const result = await page.evaluate(() => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const canvas = document.getElementById('fui-canvas');
    if (runtime === null || runtime === undefined || !(canvas instanceof HTMLCanvasElement)) {
      throw new Error('Bridge runtime is not ready.');
    }
    runtime.resetPageZoom();
    runtime.resetLogs();
    const rect = canvas.getBoundingClientRect();
    const event = new WheelEvent('wheel', {
      bubbles: true,
      cancelable: true,
      ctrlKey: true,
      clientX: rect.left + 100,
      clientY: rect.top + 80,
      deltaY: -70,
    });
    canvas.dispatchEvent(event);
    return {
      prevented: event.defaultPrevented,
      zoom: runtime.getPageZoom(),
      anchor: runtime.screenToScenePoint(100, 80),
      scrollEvents: window.__bridgeLogs?.scrollEvents ?? [],
    };
  });

  expect(result.prevented).toBe(true);
  expect(result.zoom.scale).toBeGreaterThan(1);
  expect(Math.abs(result.anchor.x - 100.0)).toBeLessThanOrEqual(1.0);
  expect(Math.abs(result.anchor.y - 80.0)).toBeLessThanOrEqual(1.0);
  expect(result.scrollEvents.length).toBe(0);
});

test('ctrl wheel trackpad pinch routes to a pinch-intent control gesture owner before page zoom', async ({ page }) => {
  await gotoBridgePage(page);
  const scene = await buildInteractiveBoxScene(page);

  const result = await page.evaluate((handles) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const canvas = document.getElementById('fui-canvas');
    if (runtime === null || runtime === undefined || !(canvas instanceof HTMLCanvasElement)) {
      throw new Error('Bridge runtime is not ready.');
    }
    runtime.resetPageZoom();
    runtime.resetLogs();

    const gestureEvents: { handle: string; phase: number; kind: number; scale: number; pointerCount: number }[] = [];
    const callbacks = window.__effindomCallbacks ?? {};
    const previousResolveGestureOwner = callbacks.resolveGestureOwner;
    const previousGetGestureIntent = callbacks.getGestureIntent;
    const previousGestureEvent = callbacks.onGestureEventWithCoords;
    callbacks.resolveGestureOwner = (handle): bigint | null => {
      return BigInt(handle.toString()) === BigInt(handles.boxHandle) ? BigInt(handles.boxHandle) : null;
    };
    callbacks.getGestureIntent = (): number => 2;
    callbacks.onGestureEventWithCoords = (handle, phase, kind, _x, _y, _deltaX, _deltaY, scale, pointerCount): boolean => {
      gestureEvents.push({
        handle: handle.toString(),
        phase,
        kind,
        scale,
        pointerCount,
      });
      return true;
    };
    window.__effindomCallbacks = callbacks;

    const rect = canvas.getBoundingClientRect();
    const event = new WheelEvent('wheel', {
      bubbles: true,
      cancelable: true,
      ctrlKey: true,
      clientX: rect.left + 100,
      clientY: rect.top + 80,
      deltaY: -70,
    });
    canvas.dispatchEvent(event);

    if (previousResolveGestureOwner === undefined) {
      delete callbacks.resolveGestureOwner;
    } else {
      callbacks.resolveGestureOwner = previousResolveGestureOwner;
    }
    if (previousGetGestureIntent === undefined) {
      delete callbacks.getGestureIntent;
    } else {
      callbacks.getGestureIntent = previousGetGestureIntent;
    }
    if (previousGestureEvent === undefined) {
      delete callbacks.onGestureEventWithCoords;
    } else {
      callbacks.onGestureEventWithCoords = previousGestureEvent;
    }

    return {
      prevented: event.defaultPrevented,
      gestureEvents,
      zoom: runtime.getPageZoom(),
    };
  }, scene);

  expect(result.prevented).toBe(true);
  expect(result.gestureEvents).toHaveLength(1);
  expect(result.gestureEvents[0]?.handle).toBe(scene.boxHandle);
  expect(result.gestureEvents[0]?.phase).toBe(2);
  expect(result.gestureEvents[0]?.kind).toBe(2);
  expect(result.gestureEvents[0]?.scale).toBeGreaterThan(1);
  expect(result.gestureEvents[0]?.pointerCount).toBe(2);
  expect(result.zoom).toEqual({ scale: 1, offsetX: 0, offsetY: 0 });
});

test('ctrl wheel uses normal wheel routing when page zoom is disabled', async ({ page }) => {
  await gotoBridgePage(page, '', { pageZoom: 'disabled' });
  const scene = await buildNestedProxyScrollScene(page);

  const result = await page.evaluate((handles) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const canvas = document.getElementById('fui-canvas');
    if (runtime === null || runtime === undefined || !(canvas instanceof HTMLCanvasElement)) {
      throw new Error('Bridge runtime is not ready.');
    }
    runtime.ui._ui_set_scroll_offset(handles.outerScrollHandle, 0, 20);
    runtime.commitFrame();
    runtime.flushPendingCommit();
    runtime.resetLogs();
    let wheelCalls = 0;
    const originalWheel = runtime.ui._ui_on_wheel_event.bind(runtime.ui);
    runtime.ui._ui_on_wheel_event = (deltaX: number, deltaY: number): void => {
      wheelCalls += 1;
      originalWheel(deltaX, deltaY);
    };
    const rect = canvas.getBoundingClientRect();
    const event = new WheelEvent('wheel', {
      bubbles: true,
      cancelable: true,
      ctrlKey: true,
      clientX: rect.left + 40,
      clientY: rect.top + 30,
      deltaY: 24,
    });
    canvas.dispatchEvent(event);
    return {
      prevented: event.defaultPrevented,
      zoom: runtime.getPageZoom(),
      wheelCalls,
      scrollEvents: window.__bridgeLogs?.scrollEvents ?? [],
    };
  }, scene);

  expect(result.prevented).toBe(true);
  expect(result.zoom).toEqual({ scale: 1, offsetX: 0, offsetY: 0 });
  expect(result.wheelCalls).toBe(1);
});

test('wheel pans the page zoom viewport when nested scroll content is already at the edge', async ({ page }) => {
  await gotoBridgePage(page);
  const scene = await buildNestedProxyScrollScene(page);

  const result = await page.evaluate((handles) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const canvas = document.getElementById('fui-canvas');
    if (runtime === null || runtime === undefined || !(canvas instanceof HTMLCanvasElement)) {
      throw new Error('Bridge runtime is not ready.');
    }

    const ui = runtime.ui;
    const rect = canvas.getBoundingClientRect();
    const resetScrolledToBottomRight = (): void => {
      ui._ui_set_scroll_offset(handles.outerScrollHandle, 10000, 10000);
      ui._ui_set_scroll_offset(handles.innerScrollHandle, 10000, 10000);
      runtime.commitFrame();
      runtime.flushPendingCommit();
      runtime.resetLogs();
      runtime.setPageZoom(2.0, 0.0, 0.0);
    };
    const dispatchWheel = (deltaX: number, deltaY: number): boolean => {
      const event = new WheelEvent('wheel', {
        bubbles: true,
        cancelable: true,
        clientX: rect.left + 40,
        clientY: rect.top + 30,
        deltaX,
        deltaY,
      });
      canvas.dispatchEvent(event);
      return event.defaultPrevented;
    };

    resetScrolledToBottomRight();
    const verticalPrevented = dispatchWheel(0, 24);
    const verticalZoom = runtime.getPageZoom();
    const verticalScrollEvents = window.__bridgeLogs?.scrollEvents ?? [];

    resetScrolledToBottomRight();
    const horizontalPrevented = dispatchWheel(24, 0);
    const horizontalZoom = runtime.getPageZoom();
    const horizontalScrollEvents = window.__bridgeLogs?.scrollEvents ?? [];

    return {
      verticalPrevented,
      verticalZoom,
      verticalScrollEvents,
      horizontalPrevented,
      horizontalZoom,
      horizontalScrollEvents,
    };
  }, scene);

  expect(result.verticalPrevented).toBe(true);
  expect(result.verticalZoom.offsetY).toBeLessThan(0);
  expect(result.verticalScrollEvents.length).toBe(0);
  expect(result.horizontalPrevented).toBe(true);
  expect(result.horizontalZoom.offsetX).toBeLessThan(0);
  expect(result.horizontalScrollEvents.length).toBe(0);
});

test('handled app wheel callbacks take precedence over scrollable wheel routing', async ({ page }) => {
  await gotoBridgePage(page);
  const scene = await buildNestedProxyScrollScene(page);

  const result = await page.evaluate((handles) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const canvas = document.getElementById('fui-canvas');
    if (runtime === null || runtime === undefined || !(canvas instanceof HTMLCanvasElement)) {
      throw new Error('Bridge runtime is not ready.');
    }

    runtime.ui._ui_set_scroll_offset(handles.outerScrollHandle, 0, 20);
    runtime.ui._ui_set_scroll_offset(handles.innerScrollHandle, 0, 0);
    runtime.commitFrame();
    runtime.flushPendingCommit();
    runtime.resetLogs();

    const callbacks = window.__effindomCallbacks ?? {};
    const previousWheel = callbacks.onWheelEventWithCoords;
    let wheelCalls = 0;
    callbacks.onWheelEventWithCoords = (handle, x, y, deltaX, deltaY, deltaMode, modifiers): boolean => {
      wheelCalls += 1;
      previousWheel?.(handle, x, y, deltaX, deltaY, deltaMode, modifiers);
      return true;
    };
    window.__effindomCallbacks = callbacks;

    const rect = canvas.getBoundingClientRect();
    const event = new WheelEvent('wheel', {
      bubbles: true,
      cancelable: true,
      clientX: rect.left + 40,
      clientY: rect.top + 30,
      deltaY: 24,
    });
    canvas.dispatchEvent(event);
    if (previousWheel === undefined) {
      delete callbacks.onWheelEventWithCoords;
    } else {
      callbacks.onWheelEventWithCoords = previousWheel;
    }

    return {
      prevented: event.defaultPrevented,
      wheelCalls,
      scrollEvents: window.__bridgeLogs?.scrollEvents ?? [],
      zoom: runtime.getPageZoom(),
    };
  }, scene);

  expect(result.prevented).toBe(true);
  expect(result.wheelCalls).toBe(1);
  expect(result.scrollEvents.length).toBe(0);
  expect(result.zoom).toEqual({ scale: 1, offsetX: 0, offsetY: 0 });
});

test('unhandled app wheel callbacks fall through to scrollable wheel routing', async ({ page }) => {
  await gotoBridgePage(page);
  const scene = await buildNestedProxyScrollScene(page);

  const result = await page.evaluate((handles) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const canvas = document.getElementById('fui-canvas');
    if (runtime === null || runtime === undefined || !(canvas instanceof HTMLCanvasElement)) {
      throw new Error('Bridge runtime is not ready.');
    }

    runtime.ui._ui_set_scroll_offset(handles.outerScrollHandle, 0, 20);
    runtime.ui._ui_set_scroll_offset(handles.innerScrollHandle, 0, 0);
    runtime.commitFrame();
    runtime.flushPendingCommit();
    runtime.resetLogs();

    const callbacks = window.__effindomCallbacks ?? {};
    const previousWheel = callbacks.onWheelEventWithCoords;
    let wheelCalls = 0;
    callbacks.onWheelEventWithCoords = (handle, x, y, deltaX, deltaY, deltaMode, modifiers): boolean => {
      wheelCalls += 1;
      previousWheel?.(handle, x, y, deltaX, deltaY, deltaMode, modifiers);
      return false;
    };
    window.__effindomCallbacks = callbacks;

    const rect = canvas.getBoundingClientRect();
    const event = new WheelEvent('wheel', {
      bubbles: true,
      cancelable: true,
      clientX: rect.left + 40,
      clientY: rect.top + 30,
      deltaY: 24,
    });
    canvas.dispatchEvent(event);
    if (previousWheel === undefined) {
      delete callbacks.onWheelEventWithCoords;
    } else {
      callbacks.onWheelEventWithCoords = previousWheel;
    }

    return {
      prevented: event.defaultPrevented,
      wheelCalls,
      scrollEvents: window.__bridgeLogs?.scrollEvents ?? [],
      zoom: runtime.getPageZoom(),
    };
  }, scene);

  expect(result.prevented).toBe(true);
  expect(result.wheelCalls).toBe(1);
  expect(result.scrollEvents.length).toBeGreaterThan(0);
  expect(result.zoom).toEqual({ scale: 1, offsetX: 0, offsetY: 0 });
});

test('handled wheel callback prevents page pan when no scrollview can consume', async ({ page }) => {
  await gotoBridgePage(page);
  const scene = await buildNestedProxyScrollScene(page);

  const result = await page.evaluate((handles) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const canvas = document.getElementById('fui-canvas');
    if (runtime === null || runtime === undefined || !(canvas instanceof HTMLCanvasElement)) {
      throw new Error('Bridge runtime is not ready.');
    }

    runtime.ui._ui_set_scroll_offset(handles.outerScrollHandle, 10000, 10000);
    runtime.ui._ui_set_scroll_offset(handles.innerScrollHandle, 10000, 10000);
    runtime.commitFrame();
    runtime.flushPendingCommit();
    runtime.resetLogs();

    const callbacks = window.__effindomCallbacks ?? {};
    const previousWheel = callbacks.onWheelEventWithCoords;
    let wheelCalls = 0;
    callbacks.onWheelEventWithCoords = (handle, x, y, deltaX, deltaY, deltaMode, modifiers): boolean => {
      wheelCalls += 1;
      previousWheel?.(handle, x, y, deltaX, deltaY, deltaMode, modifiers);
      return true;
    };
    window.__effindomCallbacks = callbacks;

    const rect = canvas.getBoundingClientRect();
    const event = new WheelEvent('wheel', {
      bubbles: true,
      cancelable: true,
      clientX: rect.left + 40,
      clientY: rect.top + 30,
      deltaY: 24,
    });
    canvas.dispatchEvent(event);
    if (previousWheel === undefined) {
      delete callbacks.onWheelEventWithCoords;
    } else {
      callbacks.onWheelEventWithCoords = previousWheel;
    }

    return {
      prevented: event.defaultPrevented,
      wheelCalls,
      scrollEvents: window.__bridgeLogs?.scrollEvents ?? [],
      zoom: runtime.getPageZoom(),
    };
  }, scene);

  expect(result.prevented).toBe(true);
  expect(result.wheelCalls).toBe(1);
  expect(result.scrollEvents.length).toBe(0);
  expect(result.zoom).toEqual({ scale: 1, offsetX: 0, offsetY: 0 });
});

test('handled touch pointer move prevents framework touch scrolling', async ({ page }) => {
  await gotoBridgePage(page);
  const scene = await buildScrollScene(page);

  const result = await page.evaluate((handles) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const canvas = document.getElementById('fui-canvas');
    if (runtime === null || runtime === undefined || !(canvas instanceof HTMLCanvasElement)) {
      throw new Error('Bridge runtime is not ready.');
    }

    runtime.ui._ui_set_scroll_offset(handles.scrollHandle, 0, 30);
    runtime.ui._ui_set_interactive(handles.scrollHandle, 1);
    runtime.commitFrame();
    runtime.flushPendingCommit();
    runtime.resetLogs();

    const callbacks = window.__effindomCallbacks ?? {};
    const previousPointer = callbacks.onPointerEventWithMetadata;
    let handledMoveCalls = 0;
    callbacks.onPointerEventWithMetadata = (
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
    ): boolean => {
      previousPointer?.(
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
      );
      if (eventType === 3) {
        handledMoveCalls += 1;
        return true;
      }
      return false;
    };
    window.__effindomCallbacks = callbacks;

    const rect = canvas.getBoundingClientRect();
    const emit = (type: string, y: number): void => {
      canvas.dispatchEvent(new PointerEvent(type, {
        bubbles: true,
        cancelable: true,
        pointerId: 141,
        pointerType: 'touch',
        isPrimary: true,
        button: 0,
        buttons: type === 'pointerup' ? 0 : 1,
        clientX: rect.left + 40,
        clientY: rect.top + y,
      }));
    };

    emit('pointerdown', 50);
    emit('pointermove', 78);
    emit('pointermove', 106);
    emit('pointerup', 106);

    if (previousPointer === undefined) {
      delete callbacks.onPointerEventWithMetadata;
    } else {
      callbacks.onPointerEventWithMetadata = previousPointer;
    }

    return {
      handledMoveCalls,
      scrollEvents: window.__bridgeLogs?.scrollEvents ?? [],
    };
  }, scene);

  expect(result.handledMoveCalls).toBeGreaterThan(0);
  expect(result.scrollEvents.some((entry) => entry.handle === scene.scrollHandle && entry.offsetY !== 30)).toBe(false);
});

test('handled touch selection drag keeps edge autoscroll ticking', async ({ page }) => {
  await gotoBridgePage(page);
  const scene = await buildInteractiveBoxScene(page);

  const result = await page.evaluate(async (handles) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const canvas = document.getElementById('fui-canvas');
    if (runtime === null || runtime === undefined || !(canvas instanceof HTMLCanvasElement)) {
      throw new Error('Bridge runtime is not ready.');
    }

    const ui = runtime.ui;
    const rect = canvas.getBoundingClientRect();
    const originalPointerEvent = ui._ui_on_pointer_event.bind(ui);
    const originalSelectionAutoScroll = ui._ui_selection_autoscroll.bind(ui);
    const originalHasPointerAutoScroll = ui._ui_has_pointer_autoscroll.bind(ui);
    const originalCommitFrame = runtime.commitFrame.bind(runtime);
    const originalFlushPendingCommit = runtime.flushPendingCommit.bind(runtime);
    const originalRequestFrame = runtime.requestFrame.bind(runtime);
    const autoScrollCalls: { x: number; y: number; edgeThreshold: number }[] = [];
    let handledMoveCalls = 0;
    let commitFrames = 0;
    let flushes = 0;
    let frameRequests = 0;

    try {
      runtime.resetLogs();
      ui._ui_on_pointer_event = (
        eventType: number,
        handle: string | number | bigint | { valueOf(): unknown; toString(): string },
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
      ): number => {
        const result = originalPointerEvent(
          eventType,
          handle,
          x,
          y,
          pointerId,
          pointerType,
          button,
          buttons,
          pressure,
          width,
          height,
          clickCount,
          modifiers,
        );
        if (eventType === 3 && pointerType === 2) {
          handledMoveCalls += 1;
          window.__effindomLastPointerEventHandled = true;
        }
        return result;
      };
      ui._ui_selection_autoscroll = (x: number, y: number, edgeThreshold: number): bigint => {
        autoScrollCalls.push({ x, y, edgeThreshold });
        return BigInt(handles.boxHandle);
      };
      ui._ui_has_pointer_autoscroll = (): number => autoScrollCalls.length > 0 ? 1 : 0;
      runtime.commitFrame = (): void => {
        commitFrames += 1;
      };
      runtime.flushPendingCommit = (): Uint32Array | null => {
        flushes += 1;
        return null;
      };
      runtime.requestFrame = (): void => {
        frameRequests += 1;
      };

      const emit = (type: string, x: number, y: number): boolean => {
        const event = new PointerEvent(type, {
          bubbles: true,
          cancelable: true,
          pointerId: 171,
          pointerType: 'touch',
          isPrimary: true,
          button: 0,
          buttons: type === 'pointerup' ? 0 : 1,
          clientX: rect.left + x,
          clientY: rect.top + y,
        });
        canvas.dispatchEvent(event);
        return event.defaultPrevented;
      };

      const downPrevented = emit('pointerdown', 40, 40);
      const movePrevented = emit('pointermove', 42, 72);
      await new Promise<void>((resolve) => {
        requestAnimationFrame(() => {
          resolve();
        });
      });
      const callsBeforeUp = autoScrollCalls.length;
      emit('pointerup', 42, 72);
      await new Promise<void>((resolve) => {
        requestAnimationFrame(() => {
          resolve();
        });
      });

      return {
        downPrevented,
        movePrevented,
        handledMoveCalls,
        callsBeforeUp,
        callsAfterUp: autoScrollCalls.length,
        commitFrames,
        flushes,
        frameRequests,
        firstCall: autoScrollCalls[0] ?? null,
      };
    } finally {
      ui._ui_on_pointer_event = originalPointerEvent;
      ui._ui_selection_autoscroll = originalSelectionAutoScroll;
      ui._ui_has_pointer_autoscroll = originalHasPointerAutoScroll;
      runtime.commitFrame = originalCommitFrame;
      runtime.flushPendingCommit = originalFlushPendingCommit;
      runtime.requestFrame = originalRequestFrame;
    }
  }, scene);

  expect(result.downPrevented).toBe(true);
  expect(result.movePrevented).toBe(true);
  expect(result.handledMoveCalls).toBeGreaterThan(0);
  expect(result.callsBeforeUp).toBeGreaterThanOrEqual(2);
  expect(result.callsAfterUp).toBe(result.callsBeforeUp);
  expect(result.commitFrames).toBeGreaterThan(0);
  expect(result.flushes).toBeGreaterThan(0);
  expect(result.frameRequests).toBeGreaterThan(0);
  expect(result.firstCall?.x).toBeGreaterThanOrEqual(40);
  expect(result.firstCall?.x).toBeLessThanOrEqual(43);
  expect(result.firstCall?.y).toBeGreaterThanOrEqual(70);
  expect(result.firstCall?.y).toBeLessThanOrEqual(73);
  expect(result.firstCall?.edgeThreshold).toBe(30);
});

test('one-finger touch pans the page zoom viewport when nested scroll content is already at the edge', async ({ page }) => {
  await gotoBridgePage(page);
  const scene = await buildNestedProxyScrollScene(page);

  const result = await page.evaluate((handles) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const canvas = document.getElementById('fui-canvas');
    if (runtime === null || runtime === undefined || !(canvas instanceof HTMLCanvasElement)) {
      throw new Error('Bridge runtime is not ready.');
    }

    const ui = runtime.ui;
    const rect = canvas.getBoundingClientRect();
    ui._ui_set_scroll_offset(handles.outerScrollHandle, 10000, 10000);
    ui._ui_set_scroll_offset(handles.innerScrollHandle, 10000, 10000);
    runtime.commitFrame();
    runtime.flushPendingCommit();
    runtime.resetLogs();
    runtime.setPageZoom(2.0, 0.0, 0.0);

    const emit = (type: string, x: number, y: number): boolean => {
      const event = new PointerEvent(type, {
        bubbles: true,
        cancelable: true,
        pointerId: 91,
        pointerType: 'touch',
        isPrimary: true,
        button: 0,
        buttons: type === 'pointerup' ? 0 : 1,
        clientX: rect.left + x,
        clientY: rect.top + y,
      });
      canvas.dispatchEvent(event);
      return event.defaultPrevented;
    };

    const downPrevented = emit('pointerdown', 40, 78);
    const firstMovePrevented = emit('pointermove', 40, 64);
    const secondMovePrevented = emit('pointermove', 40, 50);
    const zoom = runtime.getPageZoom();
    const scrollEvents = window.__bridgeLogs?.scrollEvents ?? [];
    emit('pointerup', 40, 50);

    return {
      downPrevented,
      firstMovePrevented,
      secondMovePrevented,
      zoom,
      scrollEvents,
    };
  }, scene);

  expect(result.downPrevented).toBe(true);
  expect(result.firstMovePrevented).toBe(true);
  expect(result.secondMovePrevented).toBe(true);
  expect(result.zoom.offsetY).toBeCloseTo(-28.0, 4);
  expect(result.scrollEvents.length).toBe(0);
});

test('one-finger touch page zoom pan continues with native momentum after release', async ({ page }) => {
  await gotoBridgePage(page);
  const scene = await buildNestedProxyScrollScene(page);

  const result = await page.evaluate(async (handles) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const canvas = document.getElementById('fui-canvas');
    if (runtime === null || runtime === undefined || !(canvas instanceof HTMLCanvasElement)) {
      throw new Error('Bridge runtime is not ready.');
    }

    const ui = runtime.ui;
    const rect = canvas.getBoundingClientRect();
    ui._ui_set_scroll_offset(handles.outerScrollHandle, 10000, 10000);
    ui._ui_set_scroll_offset(handles.innerScrollHandle, 10000, 10000);
    runtime.commitFrame();
    runtime.flushPendingCommit();
    runtime.resetLogs();
    runtime.setPageZoom(2.0, 0.0, 0.0);

    const emit = (type: string, x: number, y: number): boolean => {
      const event = new PointerEvent(type, {
        bubbles: true,
        cancelable: true,
        pointerId: 92,
        pointerType: 'touch',
        isPrimary: true,
        button: 0,
        buttons: type === 'pointerup' ? 0 : 1,
        clientX: rect.left + x,
        clientY: rect.top + y,
      });
      canvas.dispatchEvent(event);
      return event.defaultPrevented;
    };

    emit('pointerdown', 40, 92);
    emit('pointermove', 40, 72);
    emit('pointermove', 40, 52);
    const beforeRelease = runtime.getPageZoom();
    emit('pointerup', 40, 52);
    const afterRelease = runtime.getPageZoom();
    await new Promise<void>((resolve) => {
      requestAnimationFrame(() => {
        requestAnimationFrame(() => {
          requestAnimationFrame(() => {
            resolve();
          });
        });
      });
    });
    const afterMomentum = runtime.getPageZoom();
    const scrollEvents = window.__bridgeLogs?.scrollEvents ?? [];

    return {
      beforeRelease,
      afterRelease,
      afterMomentum,
      scrollEvents,
    };
  }, scene);

  expect(result.beforeRelease.offsetY).toBeCloseTo(-40.0, 4);
  expect(result.afterRelease.offsetY).toBe(result.beforeRelease.offsetY);
  expect(result.afterMomentum.offsetY).toBeLessThan(result.afterRelease.offsetY);
  expect(result.scrollEvents.length).toBe(0);
});

test('one-finger touch page zoom pan cancels when the same gesture returns to content scroll', async ({ page }) => {
  await gotoBridgePage(page);
  const scene = await buildNestedProxyScrollScene(page);

  const result = await page.evaluate(async (handles) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const canvas = document.getElementById('fui-canvas');
    if (runtime === null || runtime === undefined || !(canvas instanceof HTMLCanvasElement)) {
      throw new Error('Bridge runtime is not ready.');
    }

    const ui = runtime.ui;
    const rect = canvas.getBoundingClientRect();
    ui._ui_set_scroll_offset(handles.outerScrollHandle, 10000, 10000);
    ui._ui_set_scroll_offset(handles.innerScrollHandle, 10000, 10000);
    runtime.commitFrame();
    runtime.flushPendingCommit();
    runtime.resetLogs();
    runtime.setPageZoom(2.0, 0.0, 0.0);

    const emit = (type: string, x: number, y: number): boolean => {
      const event = new PointerEvent(type, {
        bubbles: true,
        cancelable: true,
        pointerId: 93,
        pointerType: 'touch',
        isPrimary: true,
        button: 0,
        buttons: type === 'pointerup' ? 0 : 1,
        clientX: rect.left + x,
        clientY: rect.top + y,
      });
      canvas.dispatchEvent(event);
      return event.defaultPrevented;
    };

    emit('pointerdown', 40, 92);
    emit('pointermove', 40, 72);
    const afterViewportPan = runtime.getPageZoom();
    emit('pointermove', 40, 112);
    const afterScrollResume = runtime.getPageZoom();
    emit('pointerup', 40, 112);
    const afterRelease = runtime.getPageZoom();
    await new Promise<void>((resolve) => {
      requestAnimationFrame(() => {
        requestAnimationFrame(() => {
          requestAnimationFrame(() => {
            resolve();
          });
        });
      });
    });
    const afterMomentumWindow = runtime.getPageZoom();
    const scrollEvents = window.__bridgeLogs?.scrollEvents ?? [];

    return {
      afterViewportPan,
      afterScrollResume,
      afterRelease,
      afterMomentumWindow,
      scrollEvents,
    };
  }, scene);

  expect(result.afterViewportPan.offsetY).toBeCloseTo(-20.0, 4);
  expect(result.afterScrollResume.offsetY).toBe(result.afterViewportPan.offsetY);
  expect(result.afterRelease.offsetY).toBe(result.afterViewportPan.offsetY);
  expect(result.afterMomentumWindow.offsetY).toBe(result.afterViewportPan.offsetY);
  expect(result.scrollEvents.length).toBeGreaterThan(0);
});


test('bridge preserves earlier pending UI commits when a later commit overwrites the buffer', async ({ page }) => {
  await gotoBridgePage(page);

  const result = await page.evaluate(async () => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    if (runtime === null || runtime === undefined) {
      throw new Error('Bridge runtime is not ready.');
    }

    const ui = runtime.ui;
    const bridge = window.EffinDomBrowserBridge;
    if (bridge === undefined) {
      throw new Error('Bridge state is not ready.');
    }
    const toHandle = (handle: unknown): string =>
      bridge.handleToString(handle as string | number | bigint | { valueOf(): unknown; toString(): string });

    runtime.resetLogs();
    runtime.resetAppSession();
    ui._ui_reset();

    const root = toHandle(ui._ui_create_node(0));
    const box = toHandle(ui._ui_create_node(0));
    ui._ui_set_root(root);
    ui._ui_node_add_child(root, box);
    ui._ui_set_width(root, 320, 0);
    ui._ui_set_height(root, 220, 0);
    ui._ui_set_bg_color(root, 0x111827ff);
    ui._ui_set_width(box, 120, 0);
    ui._ui_set_height(box, 120, 0);
    ui._ui_set_bg_color(box, 0x2563ebff);
    ui._ui_set_interactive(box, 1);

    runtime.commitFrame();
    runtime.commitFrame();
    await new Promise<void>((resolve) => {
      requestAnimationFrame(() => {
        requestAnimationFrame(() => {
          resolve();
        });
      });
    });

    return {
      boxHandle: box,
      hitHandle: runtime.getHandleFromPoint(40, 40).toString(),
      latestCommandWords: Array.from(runtime.extractCommandBuffer()),
    };
  });

  expect(result.latestCommandWords[0]).toBe(CMD_COMMIT_PAINT_ORDER);
  expect(result.hitHandle).toBe(result.boxHandle);
});


test('wheel scrolling hands proxy-owned nested scroll input off to the outer scrollview at every edge', async ({ page }) => {
  await gotoBridgePage(page);
  const scene = await buildNestedProxyScrollScene(page);

  const result = await page.evaluate((handles) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    if (runtime === null || runtime === undefined) {
      throw new Error('Bridge runtime is not ready.');
    }
    const canvas = document.getElementById('fui-canvas');
    if (!(canvas instanceof HTMLCanvasElement)) {
      throw new Error('Expected scene canvas.');
    }

    const ui = runtime.ui;
    const rect = canvas.getBoundingClientRect();
    const resetOffsets = (outerX: number, outerY: number, innerX: number, innerY: number): void => {
      ui._ui_set_scroll_offset(handles.outerScrollHandle, outerX, outerY);
      ui._ui_set_scroll_offset(handles.innerScrollHandle, innerX, innerY);
      runtime.commitFrame();
      runtime.flushPendingCommit();
      runtime.resetLogs();
    };
    const dispatchWheel = (deltaX: number, deltaY: number): boolean => {
      const event = new WheelEvent('wheel', {
        bubbles: true,
        cancelable: true,
        clientX: rect.left + 40,
        clientY: rect.top + 30,
        deltaX,
        deltaY,
      });
      canvas.dispatchEvent(event);
      return event.defaultPrevented;
    };
    const snapshot = () => {
      return (window.__bridgeLogs?.scrollEvents ?? []).map((entry) => ({
        handle: entry.handle,
        offsetX: entry.offsetX,
        offsetY: entry.offsetY,
      }));
    };

    resetOffsets(0, 20, 0, 0);
    const topPrevented = dispatchWheel(0, -24);
    const topLogs = snapshot();

    resetOffsets(0, 20, 0, 140);
    const bottomPrevented = dispatchWheel(0, 24);
    const bottomLogs = snapshot();

    resetOffsets(20, 0, 0, 0);
    const leftPrevented = dispatchWheel(-24, 0);
    const leftLogs = snapshot();

    resetOffsets(20, 0, 140, 0);
    const rightPrevented = dispatchWheel(24, 0);
    const rightLogs = snapshot();

    return {
      topPrevented,
      topLogs,
      bottomPrevented,
      bottomLogs,
      leftPrevented,
      leftLogs,
      rightPrevented,
      rightLogs,
    };
  }, scene);

  expect(result.topPrevented).toBe(true);
  expect(result.bottomPrevented).toBe(true);
  expect(result.leftPrevented).toBe(true);
  expect(result.rightPrevented).toBe(true);

  expect(result.topLogs.some((entry) => entry.handle === scene.outerScrollHandle && entry.offsetY < 20)).toBe(true);
  expect(result.topLogs.some((entry) => entry.handle === scene.innerScrollHandle && entry.offsetY !== 0)).toBe(false);

  expect(result.bottomLogs.some((entry) => entry.handle === scene.outerScrollHandle && entry.offsetY > 20)).toBe(true);
  expect(result.bottomLogs.some((entry) => entry.handle === scene.innerScrollHandle && entry.offsetY !== 140)).toBe(false);

  expect(result.leftLogs.some((entry) => entry.handle === scene.outerScrollHandle && entry.offsetX < 20)).toBe(true);
  expect(result.leftLogs.some((entry) => entry.handle === scene.innerScrollHandle && entry.offsetX !== 0)).toBe(false);

  expect(result.rightLogs.some((entry) => entry.handle === scene.outerScrollHandle && entry.offsetX > 20)).toBe(true);
  expect(result.rightLogs.some((entry) => entry.handle === scene.innerScrollHandle && entry.offsetX !== 140)).toBe(false);
});


test('touch scrolling hands proxy-owned nested scroll input off to the outer scrollview at every edge', async ({ page }) => {
  await gotoBridgePage(page);
  const scene = await buildNestedProxyScrollScene(page);

  const result = await page.evaluate((handles) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    if (runtime === null || runtime === undefined) {
      throw new Error('Bridge runtime is not ready.');
    }
    const canvas = document.getElementById('fui-canvas');
    if (!(canvas instanceof HTMLCanvasElement)) {
      throw new Error('Expected scene canvas.');
    }

    const ui = runtime.ui;
    const rect = canvas.getBoundingClientRect();
    const resetOffsets = (outerX: number, outerY: number, innerX: number, innerY: number): void => {
      ui._ui_set_scroll_offset(handles.outerScrollHandle, outerX, outerY);
      ui._ui_set_scroll_offset(handles.innerScrollHandle, innerX, innerY);
      runtime.commitFrame();
      runtime.flushPendingCommit();
      runtime.resetLogs();
    };
    const dispatchTouchPath = (points: { x: number; y: number }[], pointerId: number): void => {
      const emit = (type: string, point: { x: number; y: number }): void => {
        canvas.dispatchEvent(new PointerEvent(type, {
          bubbles: true,
          cancelable: true,
          pointerId,
          pointerType: 'touch',
          isPrimary: true,
          button: 0,
          buttons: type === 'pointerup' ? 0 : 1,
          clientX: rect.left + point.x,
          clientY: rect.top + point.y,
        }));
      };

      const firstPoint = points[0];
      const lastPoint = points[points.length - 1];
      if (firstPoint === undefined || lastPoint === undefined) {
        throw new Error('Expected touch points for scroll simulation.');
      }

      emit('pointerdown', firstPoint);
      for (let index = 1; index < points.length; index += 1) {
        const point = points[index];
        if (point === undefined) {
          throw new Error('Expected touch move point.');
        }
        emit('pointermove', point);
      }
      emit('pointerup', lastPoint);
    };
    const snapshot = () => {
      return (window.__bridgeLogs?.scrollEvents ?? []).map((entry) => ({
        handle: entry.handle,
        offsetX: entry.offsetX,
        offsetY: entry.offsetY,
      }));
    };

    resetOffsets(0, 20, 0, 0);
    dispatchTouchPath([{ x: 40, y: 30 }, { x: 40, y: 54 }, { x: 40, y: 78 }], 21);
    const topLogs = snapshot();

    resetOffsets(0, 20, 0, 140);
    dispatchTouchPath([{ x: 40, y: 78 }, { x: 40, y: 54 }, { x: 40, y: 30 }], 22);
    const bottomLogs = snapshot();

    resetOffsets(20, 0, 0, 0);
    dispatchTouchPath([{ x: 40, y: 30 }, { x: 64, y: 30 }, { x: 88, y: 30 }], 23);
    const leftLogs = snapshot();

    resetOffsets(20, 0, 140, 0);
    dispatchTouchPath([{ x: 88, y: 30 }, { x: 64, y: 30 }, { x: 40, y: 30 }], 24);
    const rightLogs = snapshot();

    return { topLogs, bottomLogs, leftLogs, rightLogs };
  }, scene);

  expect(result.topLogs.some((entry) => entry.handle === scene.outerScrollHandle && entry.offsetY < 20)).toBe(true);
  expect(result.topLogs.some((entry) => entry.handle === scene.innerScrollHandle && entry.offsetY > 8)).toBe(false);

  expect(result.bottomLogs.some((entry) => entry.handle === scene.outerScrollHandle && entry.offsetY > 20)).toBe(true);
  expect(result.bottomLogs.some((entry) => entry.handle === scene.innerScrollHandle && entry.offsetY < 132)).toBe(false);

  expect(result.leftLogs.some((entry) => entry.handle === scene.outerScrollHandle && entry.offsetX < 20)).toBe(true);
  expect(result.leftLogs.some((entry) => entry.handle === scene.innerScrollHandle && entry.offsetX > 8)).toBe(false);

  expect(result.rightLogs.some((entry) => entry.handle === scene.outerScrollHandle && entry.offsetX > 20)).toBe(true);
  expect(result.rightLogs.some((entry) => entry.handle === scene.innerScrollHandle && entry.offsetX < 132)).toBe(false);
});
