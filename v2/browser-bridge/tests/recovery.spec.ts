import { expect,test } from '@playwright/test';

import {
gotoBridgePage,
readActiveRenderer,
readScenePixel,
setupServer,
teardownServer
} from './test-utils';

test.beforeAll(async () => {
  await setupServer();
});

test.afterAll(async () => {
  await teardownServer();
});

test('simulated device loss triggers recovery and red rectangle is still rendered afterwards', async ({ page }) => {
  await gotoBridgePage(page);

  // Confirm initial render is healthy
  const initialRenderer = await readActiveRenderer(page);
  expect(initialRenderer).not.toBe('none');
  expect(initialRenderer).not.toBeNull();
  await expect.poll(async () => (await readScenePixel(page, 160, 110)).red).toBeGreaterThan(220);

  // Simulate device loss via the debug API
  const initialRecoveryCount = await page.evaluate(() => window.__bridgeLoaderInfo?.deviceRecoveryCount ?? 0);
  await page.evaluate(() => { window.__bridgeDebug?.forceDeviceLost(); });

  // Wait for recovery to complete (first retry fires after 500 ms; give generous headroom)
  await page.waitForFunction(
    (initial: number) => (window.__bridgeLoaderInfo?.deviceRecoveryCount ?? 0) > initial,
    initialRecoveryCount,
    { timeout: 10_000 },
  );

  // Red rectangle must still be present after recovery
  await expect.poll(async () => (await readScenePixel(page, 160, 110)).red).toBeGreaterThan(220);
  const pixel = await readScenePixel(page, 160, 110);
  expect(pixel.red).toBeGreaterThan(220);
  expect(pixel.green).toBeLessThan(40);
  expect(pixel.blue).toBeLessThan(40);
});

test('webgl context loss event recovers without wasm64 BigInt callback errors', async ({ page }) => {
  const pageErrors: string[] = [];
  page.on('pageerror', (error) => {
    pageErrors.push(error.message);
  });

  await gotoBridgePage(page, '?backend=webgl2');

  const contextLossTriggered = await page.evaluate(() => {
    const canvas = document.getElementById('fui-canvas') ?? document.querySelector('canvas');
    if (!(canvas instanceof HTMLCanvasElement)) {
      return false;
    }

    const gl = canvas.getContext('webgl2') ?? canvas.getContext('webgl');
    if (gl === null) {
      return false;
    }

    const loseContext = gl.getExtension('WEBGL_lose_context');
    if (loseContext === null) {
      return false;
    }

    loseContext.loseContext();
    return true;
  });

  expect(contextLossTriggered).toBe(true);
  await page.waitForTimeout(1_000);

  const bridgeError = await page.evaluate(() => window.__bridgeError ?? null);
  expect(bridgeError).toBeNull();
  const renderer = await readActiveRenderer(page);
  expect(renderer).not.toBeNull();
  expect(pageErrors.some((message) => message.includes('Cannot convert 0 to a BigInt'))).toBe(false);
});
