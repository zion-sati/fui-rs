import * as fs from 'node:fs';
import * as path from 'node:path';

import { expect, test } from '@playwright/test';

import {
  setupServer,
  teardownServer,
  getBaseUrl,
  PUBLIC_DIR,
  screenshotPath,
  gotoBridgePage,
  readActiveRenderer,
  readScenePixel,
  CMD_COMMIT_PAINT_ORDER,
  CMD_COMMIT_SCENE,
  CMD_CREATE_NODE,
  CMD_SET_BOUNDS,
  CMD_SET_BOX_STYLE,
} from './test-utils';

test.beforeAll(async () => {
  await setupServer();
});

test.afterAll(async () => {
  await teardownServer();
});

test('browser bridge boots core/ui wasm and renders a red canvas through the bridge', async ({ page }) => {
  await gotoBridgePage(page);

  const bridgeState = await page.evaluate(() => ({
    ready: window.__bridgeReady === true,
    error: window.__bridgeError ?? null,
    state: window.__bridgeState ?? null,
  }));

  expect(bridgeState.error).toBeNull();
  expect(bridgeState.ready).toBe(true);
  expect(BigInt(bridgeState.state?.rootHandle ?? '0')).toBeGreaterThan(0n);
  expect(bridgeState.state?.commandWordCount).toBeGreaterThan(0);
  expect(bridgeState.state?.commandWords).toContain(CMD_CREATE_NODE);
  expect(bridgeState.state?.commandWords).toContain(CMD_SET_BOUNDS);
  expect(bridgeState.state?.commandWords).toContain(CMD_SET_BOX_STYLE);
  expect(bridgeState.state?.commandWords).toContain(CMD_COMMIT_PAINT_ORDER);
  expect(bridgeState.state?.commandWords).toContain(CMD_COMMIT_SCENE);
  const loaderInfo = await page.evaluate(() => window.__bridgeLoaderInfo ?? null);
  expect(loaderInfo?.activeRenderer).not.toBe('none');

  await expect.poll(async () => (await readScenePixel(page, 160, 110)).red).toBeGreaterThan(220);
  const renderedPixel = await readScenePixel(page, 160, 110);

  expect(renderedPixel.red).toBeGreaterThan(220);
  expect(renderedPixel.green).toBeLessThan(40);
  expect(renderedPixel.blue).toBeLessThan(40);
  expect(renderedPixel.alpha).toBeGreaterThan(220);

  const shot = screenshotPath('chromium-browser-bridge-smoke.png');
  await page.locator('#fui-canvas').screenshot({ path: shot });
  expect(fs.existsSync(shot)).toBe(true);
});

test('browser bridge reports runtime and built-in font startup progress', async ({ page }) => {
  await page.addInitScript(() => {
    const progress: { label: string; completed: number; total: number }[] = [];
    (window as unknown as { __runtimeLoadingProgress: typeof progress }).__runtimeLoadingProgress = progress;
    window.addEventListener('effindom-loading-progress', (event) => {
      if (!(event instanceof CustomEvent)) {
        return;
      }
      const detail = event.detail as { label?: unknown; completed?: unknown; total?: unknown };
      if (
        typeof detail.label === 'string' &&
        typeof detail.completed === 'number' &&
        typeof detail.total === 'number'
      ) {
        progress.push({
          label: detail.label,
          completed: detail.completed,
          total: detail.total,
        });
      }
    });
  });

  await gotoBridgePage(page);

  const progress = await page.evaluate(() => (
    window as unknown as {
      __runtimeLoadingProgress: { label: string; completed: number; total: number }[];
    }
  ).__runtimeLoadingProgress);
  const runtimeProgress = progress.filter((entry) => entry.label === 'Runtime assets');
  const fontProgress = progress.filter((entry) => entry.label === 'Built-in fonts');

  expect(runtimeProgress[0]).toEqual({ label: 'Runtime assets', completed: 0, total: 6 });
  expect(runtimeProgress[runtimeProgress.length - 1]).toEqual({
    label: 'Runtime assets',
    completed: 6,
    total: 6,
  });
  expect(fontProgress[0]?.completed).toBe(0);
  expect(fontProgress[0]?.total).toBeGreaterThan(0);
  expect(fontProgress[fontProgress.length - 1]?.completed).toBe(fontProgress[0]?.total);
  expect(fontProgress[fontProgress.length - 1]?.total).toBe(fontProgress[0]?.total);
});

test('browser bridge falls back to WebGL2 when WebGPU is unavailable', async ({ page }) => {
  await page.addInitScript(() => {
    Object.defineProperty(globalThis.navigator, 'gpu', {
      configurable: true,
      get: () => undefined,
    });
  });

  await gotoBridgePage(page);

  await expect.poll(async () => await readActiveRenderer(page)).toBe('webgl2');
  await expect.poll(async () => (await readScenePixel(page, 160, 110)).red).toBeGreaterThan(220);
});

test('browser bridge falls back to CPU when WebGPU and WebGL2 are unavailable', async ({ page }) => {
  await page.addInitScript(() => {
    Object.defineProperty(globalThis.navigator, 'gpu', {
      configurable: true,
      get: () => undefined,
    });

    const originalGetContext: (
      this: HTMLCanvasElement,
      contextId: string,
      options?: unknown,
    ) => RenderingContext | null = Reflect.get(HTMLCanvasElement.prototype, 'getContext') as (
      this: HTMLCanvasElement,
      contextId: string,
      options?: unknown,
    ) => RenderingContext | null;
    function patchedGetContext(
      this: HTMLCanvasElement,
      contextId: '2d',
      options?: CanvasRenderingContext2DSettings,
    ): CanvasRenderingContext2D | null;
    function patchedGetContext(
      this: HTMLCanvasElement,
      contextId: 'bitmaprenderer',
      options?: ImageBitmapRenderingContextSettings,
    ): ImageBitmapRenderingContext | null;
    function patchedGetContext(
      this: HTMLCanvasElement,
      contextId: 'webgl',
      options?: WebGLContextAttributes,
    ): WebGLRenderingContext | null;
    function patchedGetContext(
      this: HTMLCanvasElement,
      contextId: 'webgl2',
      options?: WebGLContextAttributes,
    ): WebGL2RenderingContext | null;
    function patchedGetContext(
      this: HTMLCanvasElement,
      contextId: string,
      options?: unknown,
    ): RenderingContext | null {
      if (contextId === 'webgl2' || contextId === 'webgl') {
        return null;
      }
      return originalGetContext.call(this, contextId as never, options as never);
    }
    HTMLCanvasElement.prototype.getContext = patchedGetContext;
  });

  await gotoBridgePage(page);

  await expect.poll(async () => await readActiveRenderer(page)).toBe('cpu');
  await expect.poll(async () => (await readScenePixel(page, 160, 110)).red).toBeGreaterThan(220);
});

test('browser bridge honors ?backend=webgl2', async ({ page }) => {
  await gotoBridgePage(page, '?backend=webgl2');

  await expect.poll(async () => await readActiveRenderer(page)).toBe('webgl2');
  const loaderInfo = await page.evaluate(() => window.__bridgeLoaderInfo ?? null);
  expect(loaderInfo?.requestedRendererBackend).toBe('webgl2');
  await expect.poll(async () => (await readScenePixel(page, 160, 110)).red).toBeGreaterThan(220);
});

test('browser bridge honors ?backend=software', async ({ page }) => {
  await gotoBridgePage(page, '?backend=software');

  await expect.poll(async () => await readActiveRenderer(page)).toBe('cpu');
  const loaderInfo = await page.evaluate(() => window.__bridgeLoaderInfo ?? null);
  expect(loaderInfo?.requestedRendererBackend).toBe('cpu');
  await expect.poll(async () => (await readScenePixel(page, 160, 110)).red).toBeGreaterThan(220);
});

test('browser bridge boots on non-secure origins without SubtleCrypto', async ({ page }) => {
  await page.addInitScript(() => {
    Object.defineProperty(globalThis, 'crypto', {
      configurable: true,
      value: {},
    });
  });

  await gotoBridgePage(page);

  await expect.poll(async () => (await readScenePixel(page, 160, 110)).red).toBeGreaterThan(220);
});

test('browser bridge falls back atomically when CDN runtime assets are unavailable', async ({ page }) => {
  const manifestPath = path.join(PUBLIC_DIR, 'v2', 'browser-bridge', 'effindom.v2.manifest.json');
  const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8')) as {
    readonly runtime_set_hash: string;
    readonly architectures: Record<string, Record<string, {
      readonly js: string;
      readonly js_integrity: string;
      readonly wasm: string;
      readonly wasm_integrity: string;
    }>>;
  };
  let missingAssetRequests = 0;
  await page.route('**/cdn-runtime-manifest.json', async (route) => {
    const brokenArchitectures = Object.fromEntries(
      Object.entries(manifest.architectures).map(([architecture, bundles]) => [
        architecture,
        Object.fromEntries(Object.entries(bundles).map(([bundleName, bundle]) => [
          bundleName,
          {
            ...bundle,
            js: `./missing-${architecture}-${bundleName}.js`,
            js_integrity: null,
          },
        ])),
      ]),
    );
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ ...manifest, architectures: brokenArchitectures }),
    });
  });
  await page.route('**/missing-*.js', async (route) => {
    missingAssetRequests += 1;
    await route.fulfill({ status: 404, body: 'missing' });
  });
  await page.addInitScript((runtimeSetHash) => {
    window.__effindomRuntime = {
      manifestUrls: [
        '/cdn-runtime-manifest.json',
        '/v2/browser-bridge/effindom.v2.manifest.json',
      ],
      expectedRuntimeSetHash: runtimeSetHash,
    };
  }, manifest.runtime_set_hash);

  await page.goto(`${getBaseUrl()}/v2/browser-bridge/index.html`);
  await page.waitForFunction(() => window.__bridgeReady === true || typeof window.__bridgeError === 'string');

  expect(await page.evaluate(() => window.__bridgeError ?? null)).toBeNull();
  expect(missingAssetRequests).toBeGreaterThan(0);
  await expect.poll(async () => (await readScenePixel(page, 160, 110)).red).toBeGreaterThan(220);
});

test('browser bridge rejects a mismatched CDN runtime set before loading its assets', async ({ page }) => {
  const manifestPath = path.join(PUBLIC_DIR, 'v2', 'browser-bridge', 'effindom.v2.manifest.json');
  const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8')) as { readonly runtime_set_hash: string };
  let mismatchedAssetRequests = 0;
  await page.route('**/cdn-runtime-manifest.json', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ ...manifest, runtime_set_hash: 'wrong-runtime-set' }),
    });
  });
  await page.route('**/cdn-assets/**', async (route) => {
    mismatchedAssetRequests += 1;
    await route.abort();
  });
  await page.addInitScript((runtimeSetHash) => {
    window.__effindomRuntime = {
      manifestUrls: [
        '/cdn-runtime-manifest.json',
        '/v2/browser-bridge/effindom.v2.manifest.json',
      ],
      expectedRuntimeSetHash: runtimeSetHash,
    };
  }, manifest.runtime_set_hash);

  await page.goto(`${getBaseUrl()}/v2/browser-bridge/index.html`);
  await page.waitForFunction(() => window.__bridgeReady === true || typeof window.__bridgeError === 'string');

  expect(await page.evaluate(() => window.__bridgeError ?? null)).toBeNull();
  expect(mismatchedAssetRequests).toBe(0);
  await expect.poll(async () => (await readScenePixel(page, 160, 110)).red).toBeGreaterThan(220);
});

test('browser bridge reloads a mutable local fallback manifest', async ({ page }) => {
  const manifestPath = path.join(PUBLIC_DIR, 'v2', 'browser-bridge', 'effindom.v2.manifest.json');
  const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8')) as { readonly runtime_set_hash: string };
  await page.route('**/mutable-runtime-manifest.json', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      headers: { 'Cache-Control': 'public, max-age=31536000' },
      path: manifestPath,
    });
  });
  await page.addInitScript((runtimeSetHash) => {
    const originalFetch = window.fetch.bind(window);
    window.fetch = async (input: RequestInfo | URL, init?: RequestInit): Promise<Response> => {
      const inputUrl = typeof input === 'string'
        ? input
        : (input instanceof URL ? input.href : input.url);
      if (inputUrl.includes('/mutable-runtime-manifest.json')) {
        Reflect.set(window, '__mutableManifestCacheMode', init?.cache ?? null);
      }
      return await originalFetch(input, init);
    };
    window.__effindomRuntime = {
      manifestUrls: ['/v2/browser-bridge/mutable-runtime-manifest.json'],
      expectedRuntimeSetHash: runtimeSetHash,
    };
  }, manifest.runtime_set_hash);

  await page.goto(`${getBaseUrl()}/v2/browser-bridge/index.html`);
  await page.waitForFunction(() => window.__bridgeReady === true || typeof window.__bridgeError === 'string');

  expect(await page.evaluate(() => window.__bridgeError ?? null)).toBeNull();
  expect(await page.evaluate(() => Reflect.get(window, '__mutableManifestCacheMode') as unknown)).toBe('reload');
  await expect.poll(async () => (await readScenePixel(page, 160, 110)).red).toBeGreaterThan(220);
});

test('ICU data fetch retries transient failures before the bridge becomes ready', async ({ page }) => {
  let icuFailures = 0;
  await page.route('**/icudt_minimal*.dat', async (route) => {
    icuFailures += 1;
    if (icuFailures <= 2) {
      await route.fulfill({
        status: 503,
        contentType: 'application/octet-stream',
        body: 'retry',
      });
      return;
    }
    await route.continue();
  });

  await gotoBridgePage(page);
  expect(icuFailures).toBe(3);
});

test('ICU load failure shows the dedicated error UI and prevents rendering', async ({ page }) => {
  await page.route('**/effindom.v2.manifest.json', async (route) => {
    const manifestPath = path.join(PUBLIC_DIR, 'v2', 'browser-bridge', 'effindom.v2.manifest.json');
    const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8')) as {
      readonly version: string;
      readonly manifest_hash: string;
      readonly architectures: Record<string, unknown>;
      readonly assets: Record<string, unknown>;
    };
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        version: manifest.version,
        manifest_hash: manifest.manifest_hash,
        architectures: manifest.architectures,
        assets: {
          ...manifest.assets,
          icu: {
            url: './missing-icu.dat',
            integrity: null,
          },
        },
      }),
    });
  });

  await page.addInitScript(() => {
    window.__effindomRuntime = {
      manifestUrls: ['/v2/browser-bridge/effindom.v2.manifest.json'],
    };
  });
  await page.goto(`${getBaseUrl()}/v2/browser-bridge/index.html`);
  await page.waitForFunction(() => typeof window.__bridgeError === 'string');
  await expect(page.locator('#icu-error')).toBeVisible();

  const failureState = await page.evaluate(() => ({
    ready: window.__bridgeReady === true,
    error: window.__bridgeError ?? '',
    commandStatePresent: window.__bridgeState !== undefined,
  }));
  expect(failureState.ready).toBe(false);
  expect(failureState.error).toContain('Failed to fetch');
  expect(failureState.error).toContain('missing-icu.dat');
  expect(failureState.commandStatePresent).toBe(false);

  const renderedPixel = await readScenePixel(page, 160, 110);
  expect(renderedPixel.red + renderedPixel.green + renderedPixel.blue).toBeLessThan(100);
});
