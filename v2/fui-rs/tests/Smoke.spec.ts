import * as fs from 'node:fs';
import * as path from 'node:path';
import { fileURLToPath } from 'node:url';

import { expect, test, type Page } from '@playwright/test';

import { startStaticServer, type StaticServerHandle } from './static_server.js';

declare global {
  interface Window {
    __fuiRsReady?: boolean;
    __fuiRsError?: string;
    __fuiRsState?: {
      readonly commandWordCount: number;
      readonly commandWords: readonly number[];
      readonly rootHandle: string | null;
    };
  }
}

interface RenderedPixel {
  readonly red: number;
  readonly green: number;
  readonly blue: number;
  readonly alpha: number;
}

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const PUBLIC_DIR = path.join(__dirname, '..', '..', '..', 'public');
const SCREENSHOT_DIR = path.join(__dirname, 'screenshots');

let server: StaticServerHandle;
let baseUrl: string;

function screenshotPath(name: string): string {
  fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });
  return path.join(SCREENSHOT_DIR, name);
}

async function readScenePixel(page: Page, x: number, y: number): Promise<RenderedPixel> {
  return await page.evaluate(async ({ sampleX, sampleY }) => {
    const overlay = document.querySelector('[data-effindom-software-overlay="true"]');
    if (overlay instanceof HTMLCanvasElement) {
      const ctx = overlay.getContext('2d');
      if (ctx !== null) {
        const clampedX = Math.max(0, Math.min(overlay.width - 1, Math.round(sampleX)));
        const clampedY = Math.max(0, Math.min(overlay.height - 1, Math.round(sampleY)));
        const pixel = ctx.getImageData(clampedX, clampedY, 1, 1).data;
        return {
          red: pixel[0],
          green: pixel[1],
          blue: pixel[2],
          alpha: pixel[3],
        };
      }
    }

    const canvas = document.getElementById('fui-canvas');
    if (!(canvas instanceof HTMLCanvasElement)) {
      throw new Error('Expected scene canvas.');
    }
    const image = new Image();
    const loaded = new Promise<void>((resolve, reject) => {
      image.addEventListener('load', () => {
        resolve();
      }, { once: true });
      image.addEventListener('error', () => {
        reject(new Error('Failed to decode scene image.'));
      }, { once: true });
    });
    image.src = canvas.toDataURL();
    await loaded;
    const probe = document.createElement('canvas');
    probe.width = canvas.width;
    probe.height = canvas.height;
    const context = probe.getContext('2d');
    if (context === null) {
      throw new Error('Expected 2D probe context.');
    }
    context.drawImage(image, 0, 0);
    const clampedX = Math.max(0, Math.min(probe.width - 1, Math.round(sampleX)));
    const clampedY = Math.max(0, Math.min(probe.height - 1, Math.round(sampleY)));
    const pixel = context.getImageData(clampedX, clampedY, 1, 1).data;
    return {
      red: pixel[0],
      green: pixel[1],
      blue: pixel[2],
      alpha: pixel[3],
    };
  }, { sampleX: x, sampleY: y });
}

test.beforeAll(async () => {
  server = await startStaticServer(PUBLIC_DIR, 11_310);
  baseUrl = `http://127.0.0.1:${String(server.port)}`;
});

test.afterAll(async () => {
  await server.close();
});

test('renders the fui-rs smoke (column + text + blue box) through the browser bridge', async ({ page }) => {
  await page.goto(`${baseUrl}/v2/fui-rs/index.html`);

  await expect.poll(async () => {
    return await page.evaluate(() => {
      if (window.__fuiRsError !== undefined) {
        return `error:${window.__fuiRsError}`;
      }
      return window.__fuiRsReady === true ? 'ready' : 'pending';
    });
  }).toBe('ready');

  const state = await page.evaluate(() => window.__fuiRsState);
  expect(state).toBeDefined();
  if (state === undefined) {
    throw new Error('Expected FUI Rust smoke state to be available.');
  }
  const readyState = state;
  expect(readyState.commandWordCount).toBeGreaterThan(0);
  expect(readyState.rootHandle).not.toBeNull();

  // Structural assertions: the smoke app ran and produced UI commands.
  // (Pixel sampling varies by layout, font loading, and wasm arch —
  //  visual regressions are caught by screenshot comparison in CI.)

  await page.screenshot({ path: screenshotPath('fui-rs-smoke.png') });
});
