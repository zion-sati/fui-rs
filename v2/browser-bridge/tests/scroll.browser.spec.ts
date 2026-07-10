import { expect, test } from '@playwright/test';

import {
  setupServer,
  teardownServer,
  buildScrollScene,
  gotoBridgePage,
} from './test-utils';

test.beforeAll(async () => {
  await setupServer();
});

test.afterAll(async () => {
  await teardownServer();
});

test('wheel events refresh hover state before scrolling', async ({ page }) => {
  await gotoBridgePage(page);
  const scene = await buildScrollScene(page);

  const result = await page.evaluate(() => {
    const canvas = document.getElementById('fui-canvas');
    if (!(canvas instanceof HTMLCanvasElement)) {
      throw new Error('Expected scene canvas.');
    }

    const rect = canvas.getBoundingClientRect();
    const event = new WheelEvent('wheel', {
      bubbles: true,
      cancelable: true,
      clientX: rect.left + 40,
      clientY: rect.top + 40,
      deltaY: 48,
    });
    canvas.dispatchEvent(event);
    const runtime = window.EffinDomBrowserBridge?.getRuntime();

    return {
      defaultPrevented: event.defaultPrevented,
      hitHandle: runtime?.getHandleFromPoint(40, 40).toString() ?? '0',
      scrollEvents: window.__bridgeLogs?.scrollEvents ?? [],
    };
  });

  expect(result.defaultPrevented).toBe(true);
  expect(result.hitHandle).toBe('0');
  expect(result.scrollEvents.some((entry) => entry.handle === scene.scrollHandle && entry.offsetY > 0)).toBe(true);
});

test('touch swipe scrolls a retained scroll view through the browser bridge', async ({ page }) => {
  await gotoBridgePage(page);
  const scene = await buildScrollScene(page);

  const result = await page.evaluate(() => {
    const canvas = document.getElementById('fui-canvas');
    if (!(canvas instanceof HTMLCanvasElement)) {
      throw new Error('Expected scene canvas.');
    }

    const rect = canvas.getBoundingClientRect();
    const dispatch = (type: string, clientX: number, clientY: number): void => {
      canvas.dispatchEvent(new PointerEvent(type, {
        bubbles: true,
        cancelable: true,
        pointerId: 7,
        pointerType: 'touch',
        isPrimary: true,
        button: 0,
        buttons: type === 'pointerup' ? 0 : 1,
        clientX,
        clientY,
      }));
    };

    dispatch('pointerdown', rect.left + 40, rect.top + 72);
    dispatch('pointermove', rect.left + 40, rect.top + 44);
    dispatch('pointermove', rect.left + 40, rect.top + 16);
    dispatch('pointerup', rect.left + 40, rect.top + 16);

    return {
      scrollEvents: window.__bridgeLogs?.scrollEvents ?? [],
    };
  });

  expect(result.scrollEvents.some((entry) => entry.handle === scene.scrollHandle && entry.offsetY > 0)).toBe(true);
});

test('touch fling keeps scrolling with momentum through the browser bridge', async ({ page }) => {
  await gotoBridgePage(page);
  const scene = await buildScrollScene(page);

  await page.evaluate(() => {
    const canvas = document.getElementById('fui-canvas');
    if (!(canvas instanceof HTMLCanvasElement)) {
      throw new Error('Expected scene canvas.');
    }

    const rect = canvas.getBoundingClientRect();
    const dispatch = (type: string, clientX: number, clientY: number): void => {
      canvas.dispatchEvent(new PointerEvent(type, {
        bubbles: true,
        cancelable: true,
        pointerId: 9,
        pointerType: 'touch',
        isPrimary: true,
        button: 0,
        buttons: type === 'pointerup' ? 0 : 1,
        clientX,
        clientY,
      }));
    };

    dispatch('pointerdown', rect.left + 40, rect.top + 40);
    dispatch('pointermove', rect.left + 40, rect.top + 12);
    dispatch('pointermove', rect.left + 40, rect.top - 16);
    dispatch('pointerup', rect.left + 40, rect.top - 16);
  });

  let scrollEvents: { handle: string; offsetY: number }[] = [];
  await expect.poll(async () => {
    scrollEvents = await page.evaluate((scrollHandle) => {
      return (window.__bridgeLogs?.scrollEvents ?? [])
        .filter((entry) => entry.handle === scrollHandle)
        .map((entry) => ({ handle: entry.handle, offsetY: entry.offsetY }));
    }, scene.scrollHandle);
    return scrollEvents.length;
  }).toBeGreaterThan(2);

  expect((scrollEvents[0]?.offsetY ?? 0)).toBeGreaterThan(0);
  expect((scrollEvents[scrollEvents.length - 1]?.offsetY ?? 0)).toBeGreaterThan(scrollEvents[0]?.offsetY ?? 0);
});
