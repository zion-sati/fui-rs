import * as path from 'node:path';
import { fileURLToPath } from 'node:url';

import { expect,test,type Locator,type Page } from '@playwright/test';

import { startStaticServer,type StaticServerHandle } from './static_server.js';

declare global {
  interface Window {
    __fuiReady?: boolean;
    __fuiError?: string;
    __fuiState?: {
      readonly commandWordCount: number;
      readonly commandWords: readonly number[];
      readonly rootHandle: string | null;
    };
    __fuiManagerState?: {
      readonly routePath: string;
      readonly activeWasmPath: string;
      readonly routeLoads: Readonly<Record<string, number>>;
    };
    __getFuiHostTick?(): number;
    __getFuiHostDarkMode?(): boolean;
    __startFuiWorker?(): void;
    __startFuiFailingWorker?(): void;
    __getFuiWorkerStatusCode?(): number;
    __getFuiWorkerDetailHasPrimeAndClock?(): boolean;
    __getFuiWorkerDetailHasErrorClock?(): boolean;
    __demoCopiedFileName?: string;
    __demoCopiedFileText?: string;
    __bridgeActiveEditorWindow?: {
      readonly handle: string | null;
      readonly text: string;
      readonly docStart: number;
      readonly docEnd: number;
    };
    __bridgeTextByHandle?: Record<string, string>;
    __tofuSwapCommandBuffers?: number[][];
    __tofuSwapCommandBufferCaptureInstalled?: boolean;
  }
}

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const PUBLIC_DIR = path.join(__dirname, '..', '..', '..', 'public');

let server: StaticServerHandle;
let baseUrl: string;

interface GlyphRunSnapshot {
  readonly handle: string;
  readonly glyphFontIds: readonly number[];
}

async function setHiddenEditorSelection(page: Page, start: number, end = start): Promise<void> {
  await page.evaluate(({ selectionStart, selectionEnd }) => {
    const bridgeWindow = window;
    const activeEditorWindow = bridgeWindow.__bridgeActiveEditorWindow;
    const runtime = bridgeWindow.EffinDomBrowserBridge?.getRuntime();
    const activeHandle = activeEditorWindow?.handle ?? null;
    const documentState = activeHandle === null || runtime === undefined || runtime === null
      ? null
      : runtime.openCanvasApi.getEditableTextDocument(activeHandle);
    const activeElement = document.activeElement;
    const activeElementIsCorrectHiddenEditor =
      ((documentState?.multiline === true && activeElement instanceof HTMLTextAreaElement) ||
        (documentState?.multiline !== true && activeElement instanceof HTMLInputElement)) &&
      activeElement.dataset.effindomHiddenEditor === 'true';
    const editor = activeElementIsCorrectHiddenEditor
      ? activeElement
      : document.querySelector<HTMLInputElement | HTMLTextAreaElement>(
        documentState?.multiline === true
          ? 'textarea[data-effindom-hidden-editor="true"]'
          : 'input[data-effindom-hidden-editor="true"]',
      );
    if (editor === null) {
      throw new Error('Expected hidden bridge editor.');
    }
    if (runtime !== null && runtime !== undefined && activeHandle !== null && bridgeWindow.__effindomCallbacks?.onSelectionChanged !== undefined) {
      const handleArg = runtime.ui.usesMemory64 === true ? BigInt(activeHandle) : Number(activeHandle);
      bridgeWindow.__effindomCallbacks.onSelectionChanged(handleArg, selectionStart, selectionEnd);
      runtime.commitFrame();
      runtime.flushPendingCommit();
    }
    const nextDocStart = bridgeWindow.__bridgeActiveEditorWindow?.docStart ?? 0;
    editor.focus();
    editor.setSelectionRange(selectionStart - nextDocStart, selectionEnd - nextDocStart);
  }, { selectionStart: start, selectionEnd: end });
}

function decodeCommandHandle(low: number | undefined, high: number | undefined): string {
  return ((BigInt(high ?? 0) << 32n) | BigInt(low ?? 0)).toString();
}

function parseGlyphRuns(words: readonly number[]): GlyphRunSnapshot[] {
  const runs: GlyphRunSnapshot[] = [];
  for (let index = 0; index < words.length;) {
    const opcode = words[index];
    if (opcode === 40) {
      const glyphCount = words[index + 6] ?? 0;
      if (glyphCount > 4096 || index + 7 + (glyphCount * 4) > words.length) {
        break;
      }
      const glyphFontIds: number[] = [];
      for (let glyphIndex = 0; glyphIndex < glyphCount; glyphIndex += 1) {
        const base = index + 7 + (glyphIndex * 4);
        glyphFontIds.push(words[base + 3] ?? 0);
      }
      runs.push({
        handle: decodeCommandHandle(words[index + 1], words[index + 2]),
        glyphFontIds,
      });
      index += 7 + (glyphCount * 4);
      continue;
    }
    if (opcode === 44) {
      const glyphCount = words[index + 5] ?? 0;
      if (glyphCount > 4096 || index + 6 + (glyphCount * 5) > words.length) {
        break;
      }
      const glyphFontIds: number[] = [];
      for (let glyphIndex = 0; glyphIndex < glyphCount; glyphIndex += 1) {
        const base = index + 6 + (glyphIndex * 5);
        glyphFontIds.push(words[base + 3] ?? 0);
      }
      runs.push({
        handle: decodeCommandHandle(words[index + 1], words[index + 2]),
        glyphFontIds,
      });
      index += 6 + (glyphCount * 5);
      continue;
    }
    if (opcode === 46) {
      const glyphCount = words[index + 5] ?? 0;
      if (glyphCount > 4096 || index + 6 + (glyphCount * 6) > words.length) {
        break;
      }
      const glyphFontIds: number[] = [];
      for (let glyphIndex = 0; glyphIndex < glyphCount; glyphIndex += 1) {
        const base = index + 6 + (glyphIndex * 6);
        glyphFontIds.push(words[base + 3] ?? 0);
      }
      runs.push({
        handle: decodeCommandHandle(words[index + 1], words[index + 2]),
        glyphFontIds,
      });
      index += 6 + (glyphCount * 6);
      continue;
    }
    if (opcode === 1 || opcode === 2) {
      index += 3;
      continue;
    }
    if (opcode === 10) {
      index += 16;
      continue;
    }
    if (opcode === 20) {
      index += 13;
      continue;
    }
    if (opcode === 21) {
      index += 6;
      continue;
    }
    if (opcode === 22) {
      index += 8 + ((words[index + 7] ?? 0) * 2);
      continue;
    }
    if (opcode === 23) {
      index += 5;
      continue;
    }
    if (opcode === 24) {
      index += 8;
      continue;
    }
    if (opcode === 30 || opcode === 33) {
      index += 7;
      continue;
    }
    if (opcode === 31) {
      index += 10;
      continue;
    }
    if (opcode === 32) {
      const verbCount = words[index + 6] ?? 0;
      let pathIndex = index + 7;
      for (let verbIndex = 0; verbIndex < verbCount && pathIndex < words.length; verbIndex += 1) {
        const verb = words[pathIndex] ?? 4;
        pathIndex += 1;
        pathIndex += verb === 0 || verb === 1 ? 2 : verb === 2 ? 4 : verb === 3 ? 6 : 0;
      }
      index = pathIndex;
      continue;
    }
    if (opcode === 41) {
      index += 4;
      continue;
    }
    if (opcode === 42) {
      index += 8;
      continue;
    }
    if (opcode === 43) {
      index += 5 + ((words[index + 4] ?? 0) * 4);
      continue;
    }
    if (opcode === 45) {
      index += 4 + ((words[index + 3] ?? 0) * 5);
      continue;
    }
    if (opcode === 98) {
      index += 2 + ((words[index + 1] ?? 0) * 2);
      continue;
    }
    if (opcode === 99) {
      index += 2 + ((words[index + 1] ?? 0) * 5);
      continue;
    }
    break;
  }
  return runs;
}

async function findExternalDropPoint(page: Page, sceneSurface: Locator): Promise<{ handle: string; x: number; y: number; }> {
  await expect.poll(async () => {
    return await page.evaluate(() => document.body.innerText.includes('Drop files here'));
  }, { timeout: 10000 }).toBe(true);
  const bounds = await sceneSurface.boundingBox();
  expect(bounds).not.toBeNull();
  if (bounds === null) {
    throw new Error('Expected routed workbench canvas bounds.');
  }
  const targetBounds = await page.evaluate(async () => {
    const debug = window.__fui_debug;
    if (debug === undefined || typeof debug.getDebugTree !== 'function') {
      return null;
    }
    const tree = await debug.getDebugTree();
    const node = tree.nodes.find((entry) => entry.semanticLabel === 'External file drop target');
    const visible = node?.visibleBounds ?? null;
    const bounds = visible !== null && visible.width > 0 && visible.height > 0 ? visible : node?.bounds ?? null;
    return node === undefined || bounds === null ? null : {
      handle: node.handle,
      x: bounds.x,
      y: bounds.y,
      width: bounds.width,
      height: bounds.height,
    };
  });
  expect(targetBounds).not.toBeNull();
  if (targetBounds === null) {
    throw new Error('Expected external file drop target debug bounds.');
  }
  return {
    handle: targetBounds.handle,
    x: Math.floor(bounds.x + targetBounds.x + (targetBounds.width * 0.5)),
    y: Math.floor(bounds.y + targetBounds.y + (targetBounds.height * 0.5)),
  };
}

async function clickDebugNode(page: Page, sceneSurface: Locator, semanticLabel: string): Promise<void> {
  const bounds = await sceneSurface.boundingBox();
  expect(bounds).not.toBeNull();
  if (bounds === null) {
    throw new Error('Expected routed workbench canvas bounds.');
  }
  const targetBounds = await page.evaluate(async (label) => {
    const debug = window.__fui_debug;
    if (debug === undefined || typeof debug.getDebugTree !== 'function') {
      return null;
    }
    const tree = await debug.getDebugTree();
    const candidates = tree.nodes
      .filter((entry) => entry.semanticLabel === label)
      .map((entry) => entry.visibleBounds)
      .filter((candidate) => candidate.width > 0 && candidate.height > 0)
      .sort((a, b) => (b.width * b.height) - (a.width * a.height));
    if (candidates.length === 0) {
      return null;
    }
    const bounds = candidates[0];
    return {
      x: bounds.x,
      y: bounds.y,
      width: bounds.width,
      height: bounds.height,
    };
  }, semanticLabel);
  expect(targetBounds).not.toBeNull();
  if (targetBounds === null) {
    throw new Error(`Expected debug bounds for ${semanticLabel}.`);
  }
  await page.mouse.click(
    Math.floor(bounds.x + targetBounds.x + (targetBounds.width * 0.5)),
    Math.floor(bounds.y + targetBounds.y + (targetBounds.height * 0.5)),
  );
}

async function clickLargestDebugNode(page: Page, sceneSurface: Locator, semanticLabel: string): Promise<void> {
  const canvasBounds = await sceneSurface.boundingBox();
  expect(canvasBounds).not.toBeNull();
  if (canvasBounds === null) {
    throw new Error('Expected routed workbench canvas bounds.');
  }
  const targetBounds = await page.evaluate(async (label) => {
    const debug = window.__fui_debug;
    if (debug === undefined || typeof debug.getDebugTree !== 'function') {
      return null;
    }
    const tree = await debug.getDebugTree();
    const candidates = tree.nodes
      .filter((entry) => entry.semanticLabel === label)
      .map((entry) => entry.visibleBounds)
      .filter((bounds) => bounds.width > 0 && bounds.height > 0)
      .sort((a, b) => (b.width * b.height) - (a.width * a.height));
    return candidates[0] ?? null;
  }, semanticLabel);
  expect(targetBounds).not.toBeNull();
  if (targetBounds === null) {
    throw new Error(`Expected debug bounds for ${semanticLabel}.`);
  }
  await page.mouse.click(
    Math.floor(canvasBounds.x + targetBounds.x + (targetBounds.width * 0.5)),
    Math.floor(canvasBounds.y + targetBounds.y + (targetBounds.height * 0.5)),
  );
}

async function debugNodeCenter(page: Page, sceneSurface: Locator, semanticLabel: string): Promise<{ x: number; y: number; }> {
  const bounds = await debugNodeScreenBounds(page, sceneSurface, semanticLabel);
  return {
    x: Math.floor(bounds.x + (bounds.width * 0.5)),
    y: Math.floor(bounds.y + (bounds.height * 0.5)),
  };
}

async function debugNodeScreenBounds(
  page: Page,
  sceneSurface: Locator,
  semanticLabel: string,
): Promise<{ x: number; y: number; width: number; height: number; }> {
  const canvasBounds = await sceneSurface.boundingBox();
  expect(canvasBounds).not.toBeNull();
  if (canvasBounds === null) {
    throw new Error('Expected routed workbench canvas bounds.');
  }
  const targetBounds = await page.evaluate(async (label) => {
    const debug = window.__fui_debug;
    if (debug === undefined || typeof debug.getDebugTree !== 'function') {
      return null;
    }
    const tree = await debug.getDebugTree();
    const candidates = tree.nodes
      .filter((entry) => entry.semanticLabel === label)
      .map((entry) => entry.visibleBounds)
      .filter((bounds) => bounds.width > 0 && bounds.height > 0)
      .sort((a, b) => (b.width * b.height) - (a.width * a.height));
    if (candidates.length === 0) {
      return null;
    }
    const bounds = candidates[0];
    return {
      x: bounds.x,
      y: bounds.y,
      width: bounds.width,
      height: bounds.height,
    };
  }, semanticLabel);
  expect(targetBounds).not.toBeNull();
  if (targetBounds === null) {
    throw new Error(`Expected debug bounds for ${semanticLabel}.`);
  }
  return {
    x: Math.floor(canvasBounds.x + targetBounds.x),
    y: Math.floor(canvasBounds.y + targetBounds.y),
    width: Math.floor(targetBounds.width),
    height: Math.floor(targetBounds.height),
  };
}

async function debugNodeVisibleHeight(page: Page, semanticLabel: string): Promise<number> {
  return await page.evaluate(async (label) => {
    const debug = window.__fui_debug;
    if (debug === undefined || typeof debug.getDebugTree !== 'function') {
      return 0;
    }
    const tree = await debug.getDebugTree();
    const node = tree.nodes.find((entry) => entry.semanticLabel === label);
    return node === undefined ? 0 : node.visibleBounds.height;
  }, semanticLabel);
}

async function debugNodeVisibleHeightByNodeId(page: Page, nodeId: string): Promise<number> {
  return await page.evaluate(async (id) => {
    const debug = window.__fui_debug;
    if (debug === undefined || typeof debug.getDebugTree !== 'function') {
      return 0;
    }
    const tree = await debug.getDebugTree();
    const node = tree.nodes.find((entry) => entry.nodeId === id);
    return node === undefined ? 0 : node.visibleBounds.height;
  }, nodeId);
}

async function debugNodeVisibleWidthByNodeId(page: Page, nodeId: string): Promise<number> {
  return await page.evaluate(async (id) => {
    const debug = window.__fui_debug;
    if (debug === undefined || typeof debug.getDebugTree !== 'function') {
      return 0;
    }
    const tree = await debug.getDebugTree();
    const node = tree.nodes.find((entry) => entry.nodeId === id);
    return node === undefined ? 0 : node.bounds.width;
  }, nodeId);
}

async function debugNodeSemanticLabelByNodeId(page: Page, nodeId: string): Promise<string> {
  return await page.evaluate(async (id) => {
    const debug = window.__fui_debug;
    if (debug === undefined || typeof debug.getDebugTree !== 'function') {
      return '';
    }
    const tree = await debug.getDebugTree();
    const node = tree.nodes.find((entry) => entry.nodeId === id);
    return node?.semanticLabel ?? '';
  }, nodeId);
}

async function clickLargestDebugNodeByNodeId(page: Page, sceneSurface: Locator, nodeId: string): Promise<void> {
  const canvasBounds = await sceneSurface.boundingBox();
  expect(canvasBounds).not.toBeNull();
  if (canvasBounds === null) {
    throw new Error('Expected routed workbench canvas bounds.');
  }
  const targetBounds = await page.evaluate(async (id) => {
    const debug = window.__fui_debug;
    if (debug === undefined || typeof debug.getDebugTree !== 'function') {
      return null;
    }
    const tree = await debug.getDebugTree();
    const candidates = tree.nodes
      .filter((entry) => entry.nodeId === id)
      .map((entry) => entry.visibleBounds)
      .filter((bounds) => bounds.width > 0 && bounds.height > 0)
      .sort((a, b) => (b.width * b.height) - (a.width * a.height));
    return candidates[0] ?? null;
  }, nodeId);
  expect(targetBounds).not.toBeNull();
  if (targetBounds === null) {
    throw new Error(`Expected debug bounds for ${nodeId}.`);
  }
  await page.mouse.click(
    Math.floor(canvasBounds.x + targetBounds.x + (targetBounds.width * 0.5)),
    Math.floor(canvasBounds.y + targetBounds.y + (targetBounds.height * 0.5)),
  );
}

async function waitForProjectedInput(page: Page, name: string): Promise<void> {
  for (let attempt = 0; attempt < 50; attempt += 1) {
    const found = await page.evaluate((fieldName) =>
      document.querySelector(`form[data-effindom-projected-form="true"] input[name="${fieldName}"]`) !== null,
    name);
    if (found) {
      return;
    }
    const delta = await page.evaluate(async (fieldName) => {
      const tree = await window.__fui_debug?.getDebugTree();
      const target = tree?.nodes.find((entry) => entry.nodeId === fieldName);
      const desired = (target?.bounds.y ?? window.innerHeight) - (window.innerHeight * 0.5);
      return Math.max(-180, Math.min(180, desired));
    }, name);
    await page.mouse.wheel(0, Math.abs(delta) < 1 ? 100 : delta);
    await page.waitForTimeout(150);
  }
  await expect.poll(async () => {
    return await page.evaluate((fieldName) =>
      document.querySelector(`form[data-effindom-projected-form="true"] input[name="${fieldName}"]`) !== null,
    name);
  }, { timeout: 10000 }).toBe(true);
}

async function clickProjectedInput(page: Page, name: string): Promise<void> {
  const rect = await page.evaluate((fieldName) => {
    const input = document.querySelector<HTMLInputElement>(`form[data-effindom-projected-form="true"] input[name="${fieldName}"]`);
    if (input === null) {
      return null;
    }
    const bounds = input.getBoundingClientRect();
    return {
      x: bounds.x,
      y: bounds.y,
      width: bounds.width,
      height: bounds.height,
    };
  }, name);
  expect(rect).not.toBeNull();
  if (rect === null) {
    throw new Error(`Expected projected input for ${name}.`);
  }
  await page.mouse.click(
    Math.floor(rect.x + (rect.width * 0.5)),
    Math.floor(rect.y + (rect.height * 0.5)),
  );
}

async function doubleClickProjectedInput(page: Page, name: string, xFraction = 0.5): Promise<void> {
  await waitForProjectedInput(page, name);
  const rect = await page.evaluate((fieldName) => {
    const input = document.querySelector<HTMLInputElement>(`form[data-effindom-projected-form="true"] input[name="${fieldName}"]`);
    if (input === null) {
      return null;
    }
    const bounds = input.getBoundingClientRect();
    return {
      x: bounds.x,
      y: bounds.y,
      width: bounds.width,
      height: bounds.height,
    };
  }, name);
  expect(rect).not.toBeNull();
  if (rect === null) {
    throw new Error(`Expected projected input for ${name}.`);
  }
  await page.mouse.dblclick(
    Math.floor(rect.x + (rect.width * xFraction)),
    Math.floor(rect.y + (rect.height * 0.5)),
  );
}

async function semanticNodeScreenBounds(
  page: Page,
  sceneSurface: Locator,
  label: string,
): Promise<{ x: number; y: number; width: number; height: number; }> {
  const canvasBounds = await sceneSurface.boundingBox();
  expect(canvasBounds).not.toBeNull();
  if (canvasBounds === null) {
    throw new Error('Expected canvas bounds.');
  }
  const targetBounds = await page.evaluate((targetLabel: string) => {
    const node = (window as Window & {
      __bridgeSemanticTree?: {
        label: string;
        bounds: { x: number; y: number; width: number; height: number; };
      }[];
    }).__bridgeSemanticTree?.find((entry) => entry.label === targetLabel);
    return node?.bounds ?? null;
  }, label);
  expect(targetBounds).not.toBeNull();
  if (targetBounds === null) {
    throw new Error(`Expected semantic bounds for ${label}.`);
  }
  return {
    x: Math.floor(canvasBounds.x + targetBounds.x),
    y: Math.floor(canvasBounds.y + targetBounds.y),
    width: Math.floor(targetBounds.width),
    height: Math.floor(targetBounds.height),
  };
}

test.beforeAll(async () => {
  server = await startStaticServer(PUBLIC_DIR, 0);
  baseUrl = `http://127.0.0.1:${String(server.port)}`;
});

test.afterAll(async () => {
  await server.close();
});

test('shows the styled loading overlay before throttled bridge scripts arrive', async ({ page }) => {
  await page.route(/\/(bridge|harness)\.js(?:\?|$)/, async (route) => {
    await new Promise((resolve) => setTimeout(resolve, 1_000));
    await route.continue();
  });

  await page.goto(`${baseUrl}/v2/fui-rs/demo/workbench/index.html`, { waitUntil: 'commit' });
  await page.waitForTimeout(400);

  const overlay = page.locator('#effindom-loading-overlay');
  await expect(overlay).toBeVisible();
  await expect(overlay).toHaveCSS('display', 'grid');
  await expect(overlay).toHaveCSS('position', 'absolute');
  await expect(overlay).toHaveCSS('cursor', 'default');
  await expect(overlay).toHaveCSS('user-select', 'none');
  await expect(page.locator('#effindom-loading-detail')).toHaveText('Runtime assets');
});

test('shows built-in font replay progress during a warm route load', async ({ page }) => {
  await page.goto(`${baseUrl}/v2/fui-rs/demo/index.html`);
  await page.waitForFunction(() => window.__fuiReady === true);

  let releaseFonts: (() => void) | undefined;
  const fontsBlocked = new Promise<void>((resolve) => {
    releaseFonts = resolve;
  });
  await page.route('**/fonts/*.ttf', async (route) => {
    await fontsBlocked;
    await route.continue();
  });

  await page.evaluate(() => {
    void window.__fui_debug?.navigateTo('/v2/fui-rs/demo/stage5/');
  });

  await expect(page.locator('#effindom-loading-overlay')).toBeVisible();
  await expect(page.locator('#effindom-loading-detail')).toHaveText('Built-in fonts 0 / 6');

  releaseFonts?.();
  await page.waitForFunction(() => window.__fuiReady === true);
  await expect(page).toHaveURL(`${baseUrl}/v2/fui-rs/demo/stage5/`);
  await expect(page.locator('#effindom-loading-overlay')).toBeHidden();
});

test('does not wait for app-authored fonts during a warm route load', async ({ page }) => {
  await page.goto(`${baseUrl}/v2/fui-rs/demo/index.html`);
  await page.waitForFunction(() => window.__fuiReady === true);

  let releaseFonts: (() => void) | undefined;
  const fontsBlocked = new Promise<void>((resolve) => {
    releaseFonts = resolve;
  });
  await page.route('**/v2/fonts/*.ttf', async (route) => {
    await fontsBlocked;
    await route.continue();
  });

  await page.evaluate(() => {
    void window.__fui_debug?.navigateTo('/v2/fui-rs/demo/stage4/');
  });

  await page.waitForFunction(() => window.__fuiReady === true);
  await expect(page).toHaveURL(`${baseUrl}/v2/fui-rs/demo/stage4/`);
  await expect(page.locator('#effindom-loading-overlay')).toBeHidden();
  await expect(page.locator('#effindom-loading-detail')).not.toContainText('Fonts ');
  releaseFonts?.();
});

test('routes the fui-rs demo scaffold through the shared routed harness', async ({ page }) => {
  await page.goto(`${baseUrl}/v2/fui-rs/demo/index.html`);
  await page.waitForFunction(() => window.__fuiReady === true);
  const sceneSurface = page.locator('canvas').first();

  await expect.poll(async () => {
    return await page.evaluate(async () => {
      const response = await fetch('/v2/fui-rs/demo/routes.json');
      const manifest = await response.json() as {
        readonly routes?: readonly {
          readonly routePath?: string;
          readonly wasmPath?: string;
        }[];
      };
      const routes = manifest.routes ?? [];
      return routes.some(route => route.routePath === '/' && route.wasmPath === '/home.wasm')
        && routes.some(route => route.routePath === '/workbench/' && route.wasmPath === '/workbench.wasm')
        && routes.some(route => route.routePath === '/stage4/' && route.wasmPath === '/stage4.wasm')
        && routes.some(route => route.routePath === '/immediate-drawing/' && route.wasmPath === '/immediate-drawing.wasm');
    });
  }).toBe(true);

  await page.goto(`${baseUrl}/v2/fui-rs/demo/immediate-drawing/index.html`);
  await page.waitForFunction(() => window.__fuiReady === true);
  await expect.poll(async () => {
    return await page.evaluate(() => window.__fuiManagerState?.routePath ?? '');
  }).toBe('/v2/fui-rs/demo/immediate-drawing/');
  await expect.poll(async () => {
    return await page.evaluate(() => {
      if (window.__fuiError !== undefined) {
        return `error:${window.__fuiError}`;
      }
      const text = document.body.innerText;
      const labels = (window as Window & {
        __bridgeSemanticTree?: readonly { readonly label: string; }[];
      }).__bridgeSemanticTree?.map(node => node.label) ?? [];
      return text.includes('FUI-RS Immediate Drawing')
        && labels.includes('Animated gauge drawing sample')
        && labels.includes('Animated bar chart drawing sample');
    });
  }, { timeout: 10000 }).toBe(true);
  await page.mouse.move(640, 600);
  await page.mouse.wheel(0, 1400);
  await expect.poll(async () => {
    return await page.evaluate(() => {
      const labels = (window as Window & {
        __bridgeSemanticTree?: readonly { readonly label: string; }[];
      }).__bridgeSemanticTree?.map(node => node.label) ?? [];
      return labels.includes('Dancing yarn interactive noise panel')
        && labels.includes('Paint canvas - drag to draw');
    });
  }, { timeout: 10000 }).toBe(true);

  await page.goto(`${baseUrl}/v2/fui-rs/demo/index.html`);
  await page.waitForFunction(() => window.__fuiReady === true);

  await expect(page).toHaveURL(/\/v2\/fui-rs\/demo\/index\.html$/);
  await expect.poll(async () => {
    return await page.evaluate(() => window.__fuiManagerState?.routePath ?? '');
  }).toBe('/v2/fui-rs/demo/index.html');
  await expect.poll(async () => {
    return await page.evaluate(() => window.__fuiManagerState?.activeWasmPath ?? '');
  }).toContain('/v2/fui-rs/demo/home.wasm');

  await page.goto(`${baseUrl}/v2/fui-rs/demo/workbench/index.html`);
  await expect(page).toHaveURL(/\/v2\/fui-rs\/demo\/workbench\/index\.html$/);
  await page.waitForFunction(() => window.__fuiReady === true);
  await expect.poll(async () => {
    return await page.evaluate(() => window.__fuiManagerState?.routePath ?? '');
  }).toBe('/v2/fui-rs/demo/workbench/');
  await expect.poll(async () => {
    return await page.evaluate(() => window.__fuiManagerState?.activeWasmPath ?? '');
  }).toContain('/v2/fui-rs/demo/workbench.wasm');

  await expect.poll(async () => {
    return await page.evaluate(() => {
      return window.__fuiManagerState?.routeLoads['/v2/fui-rs/demo/workbench/'] ?? 0;
    });
  }).toBeGreaterThan(0);

  await page.goto(`${baseUrl}/v2/fui-rs/demo/index.html`);
  await expect(page).toHaveURL(/\/v2\/fui-rs\/demo\/index\.html$/);
  await page.waitForFunction(() => window.__fuiReady === true);
  await expect.poll(async () => {
    return await page.evaluate(() => window.__fuiManagerState?.routePath ?? '');
  }).toBe('/v2/fui-rs/demo/index.html');

  await page.mouse.click(190, 52);
  await expect(page).toHaveURL(/\/v2\/fui-rs\/demo\/workbench\/$/);
  await expect.poll(async () => {
    return await page.evaluate(() => {
      if (window.__fuiError !== undefined) {
        return `error:${window.__fuiError}`;
      }
      return document.body.innerText.includes('FUI-RS workbench') ? 'workbench' : 'pending';
    });
  }, { timeout: 10000 }).toBe('workbench');
  await expect.poll(async () => {
    return await page.evaluate(() => window.__fuiManagerState?.routePath ?? '');
  }).toBe('/v2/fui-rs/demo/workbench/');
  await expect.poll(async () => {
    return await page.evaluate(() => document.body.innerText.includes('Stage 4 route-relative image state: Ready'));
  }, { timeout: 10000 }).toBe(true);
  await clickDebugNode(page, sceneSurface, 'Stage 4 persisted route switch');
  await expect.poll(async () => {
    return await page.evaluate(() => document.body.innerText.includes('Stage 4 persisted switch: on'));
  }).toBe(true);

  await page.goBack();
  await expect.poll(async () => {
    return await page.evaluate(() => window.__fuiManagerState?.routePath ?? '');
  }, { timeout: 10000 }).toBe('/v2/fui-rs/demo/index.html');
  await expect.poll(async () => {
    return await page.evaluate(() => document.body.innerText.includes('FUI-RS demo dashboard'));
  }, { timeout: 10000 }).toBe(true);

  await page.goForward();
  await expect.poll(async () => {
    return await page.evaluate(() => window.__fuiManagerState?.routePath ?? '');
  }, { timeout: 10000 }).toBe('/v2/fui-rs/demo/workbench/');
  await expect.poll(async () => {
    return await page.evaluate(() => document.body.innerText.includes('FUI-RS workbench'));
  }, { timeout: 10000 }).toBe(true);
  await expect.poll(async () => {
    return await page.evaluate(() => document.body.innerText.includes('Stage 4 persisted switch: on'));
  }, { timeout: 10000 }).toBe(true);

  await page.goto(`${baseUrl}/v2/fui-rs/demo/stage4/index.html`);
  await expect(page).toHaveURL(/\/v2\/fui-rs\/demo\/stage4\/index\.html$/);
  await page.waitForFunction(() => window.__fuiReady === true);
  await expect.poll(async () => {
    return await page.evaluate(() => window.__fuiManagerState?.routePath ?? '');
  }).toBe('/v2/fui-rs/demo/stage4/');
  await expect.poll(async () => {
    return await page.evaluate(() => window.__fuiManagerState?.activeWasmPath ?? '');
  }).toContain('/v2/fui-rs/demo/stage4.wasm');
  await expect.poll(async () => {
    return await page.evaluate(() => {
      const text = document.body.innerText;
      return text.includes('FUI-RS Stage 4 presentation verification')
        && text.includes('App-level ControlTemplateSet')
        && text.includes('Per-instance template precedence')
        && text.includes('Control sizing tokens')
        && text.includes('Presenter color overrides')
        && text.includes('Dropdown presenter contracts')
        && text.includes('Local override checkbox: on')
        && text.includes('Radio sizing selected: compact')
        && text.includes('Switch presenter state: on')
        && text.includes('Slider sizing value: 42');
    });
  }, { timeout: 10000 }).toBe(true);

  await clickDebugNode(page, sceneSurface, 'Stage 4 local override checkbox');
  await expect.poll(async () => {
    return await page.evaluate(() => document.body.innerText.includes('Local override checkbox: off'));
  }, { timeout: 10000 }).toBe(true);

  await clickDebugNode(page, sceneSurface, 'Dashboard');
  await expect.poll(async () => {
    return await page.evaluate(() => window.__fuiManagerState?.routePath ?? '');
  }, { timeout: 10000 }).toBe('/v2/fui-rs/demo/index.html');
  await clickDebugNode(page, sceneSurface, 'Stage 4');
  await expect(page).toHaveURL(/\/v2\/fui-rs\/demo\/stage4\/$/);
  await expect.poll(async () => {
    return await page.evaluate(() => window.__fuiManagerState?.routePath ?? '');
  }, { timeout: 10000 }).toBe('/v2/fui-rs/demo/stage4/');
});

test('generated routed-demo host services and host events flow into the Rust home route', async ({ page }) => {
  await page.emulateMedia({ colorScheme: 'light' });
  await page.goto(`${baseUrl}/v2/fui-rs/demo/index.html`);
  await page.waitForFunction(() => window.__fuiReady === true);

  const initial = await page.evaluate(() => ({
    tick: window.__getFuiHostTick?.() ?? -1,
    darkMode: window.__getFuiHostDarkMode?.(),
  }));
  expect(initial.tick).toBeGreaterThanOrEqual(0);
  expect(initial.darkMode).toBe(false);

  await expect.poll(async () => {
    return await page.evaluate(() => window.__getFuiHostTick?.() ?? -1);
  }, { timeout: 4000 }).toBeGreaterThan(initial.tick);

  await page.emulateMedia({ colorScheme: 'dark' });
  await expect.poll(async () => {
    return await page.evaluate(() => window.__getFuiHostDarkMode?.() ?? false);
  }).toBe(true);
});

test('stage 4 house button is bold before hover and keeps its font across hover', async ({ page }) => {
  await page.goto(`${baseUrl}/v2/fui-rs/demo/stage4/index.html`);
  await page.waitForFunction(() => window.__fuiReady === true);
  const sceneSurface = page.locator('canvas').first();

  const readLabelFont = async () => await page.evaluate(async () => {
    const debug = window.__fui_debug;
    if (debug === undefined || typeof debug.getDebugTree !== 'function') {
      return null;
    }
    const tree = await debug.getDebugTree();
    const button = tree.nodes.find((entry) => entry.nodeId === 'stage4-template-house-button');
    if (button === undefined) {
      return null;
    }
    const pending = [...button.childHandles];
    while (pending.length > 0) {
      const handle = pending.shift();
      if (handle === undefined) {
        continue;
      }
      const node = tree.nodesByHandle[handle];
      if (node.behavior.textNode) {
        return { fontId: node.style.fontId, fontSize: node.style.fontSize };
      }
      pending.push(...node.childHandles);
    }
    return null;
  });

  await expect.poll(readLabelFont).toEqual({ fontId: 2, fontSize: 17 });
  const bounds = await debugNodeScreenBounds(page, sceneSurface, 'Stage 4 house template button');
  await page.mouse.move(bounds.x + (bounds.width / 2), bounds.y + (bounds.height / 2));
  await expect.poll(readLabelFont).toEqual({ fontId: 2, fontSize: 17 });
  await page.mouse.move(0, 0);
  await expect.poll(readLabelFont).toEqual({ fontId: 2, fontSize: 17 });
});

test('workbench renders rich text spans through the inherited text surface', async ({ page }) => {
  await page.goto(`${baseUrl}/v2/fui-rs/demo/workbench/index.html`);
  await page.waitForFunction(() => window.__fuiReady === true);

  await expect.poll(async () => {
    return await page.evaluate(() => document.body.innerText.includes('Rich text underline strike helpers'));
  }, { timeout: 10000 }).toBe(true);
});

test('workbench root follows the active dark system theme', async ({ page }) => {
  await page.emulateMedia({ colorScheme: 'dark' });
  await page.goto(`${baseUrl}/v2/fui-rs/demo/workbench/index.html`);
  await page.waitForFunction(() => window.__fuiReady === true);

  const readBackground = async () => await page.evaluate(async () => {
    const debug = window.__fui_debug;
    if (debug === undefined || typeof debug.getDebugTree !== 'function') {
      return null;
    }
    const tree = await debug.getDebugTree();
    return tree.nodes.find((entry) => entry.nodeId === 'workbench-scroll-root')?.style.bgColor ?? null;
  });

  await expect.poll(readBackground).not.toBeNull();
  const background = await readBackground();
  expect(background).not.toBe(0xF7F4ECFF);
  expect(((background ?? 0) >>> 24) & 0xFF).toBeLessThan(0x80);
  expect(((background ?? 0) >>> 16) & 0xFF).toBeLessThan(0x80);
  expect(((background ?? 0) >>> 8) & 0xFF).toBeLessThan(0x80);
});

test('virtual list on the routed dashboard advances its visible window on scroll', async ({ page }) => {
  await page.goto(`${baseUrl}/v2/fui-rs/demo/index.html`);
  await page.waitForFunction(() => window.__fuiReady === true);
  const sceneSurface = page.locator('canvas').first();
  const canvasBounds = await sceneSurface.boundingBox();
  expect(canvasBounds).not.toBeNull();
  if (canvasBounds === null) {
    throw new Error('Expected dashboard canvas bounds.');
  }

  await expect.poll(async () => {
    return await page.evaluate(() =>
      document.body.innerText.includes('First visible item 0')
        && document.body.innerText.includes('Rendered rows 10'),
    );
  }, { timeout: 10000 }).toBe(true);

  const itemBounds = await semanticNodeScreenBounds(page, sceneSurface, 'Item 0');
  await page.mouse.move(
    Math.floor(itemBounds.x + (itemBounds.width * 0.5)),
    Math.floor(itemBounds.y + (itemBounds.height * 0.5)),
  );
  await page.mouse.wheel(0, 220);

  await expect.poll(async () => {
    return await page.evaluate(() => {
      const match = /First visible item (\d+)/.exec(document.body.innerText);
      return match === null ? -1 : Number.parseInt(match[1], 10);
    });
  }, { timeout: 10000 }).toBeGreaterThan(0);

  await page.mouse.move(Math.floor(canvasBounds.x + 80), Math.floor(canvasBounds.y + 120));
  await page.mouse.wheel(0, 900);
  await expect.poll(async () => {
    return await debugNodeVisibleHeight(page, 'Next phase');
  }, { timeout: 10000 }).toBeGreaterThan(0);
});

test('runs routed-demo workers through the browser worker bridge and generated worker host services', async ({ page }) => {
  await page.goto(`${baseUrl}/v2/fui-rs/demo/workbench/index.html`);
  await page.waitForFunction(() => window.__fuiReady === true);

  await page.evaluate(() => {
    window.__startFuiWorker?.();
  });
  await expect.poll(async () => {
    return await page.evaluate(() => window.__getFuiWorkerStatusCode?.() ?? 0);
  }).toBe(1);
  await expect.poll(async () => {
    return await page.evaluate(() => window.__getFuiWorkerStatusCode?.() ?? 0);
  }, { timeout: 8000 }).toBe(2);
  await expect.poll(async () => {
    return await page.evaluate(() => window.__getFuiWorkerDetailHasPrimeAndClock?.() ?? false);
  }, { timeout: 8000 }).toBe(true);

  await page.evaluate(() => {
    window.__startFuiFailingWorker?.();
  });
  await expect.poll(async () => {
    return await page.evaluate(() => window.__getFuiWorkerStatusCode?.() ?? 0);
  }, { timeout: 4000 }).toBe(3);
  await expect.poll(async () => {
    return await page.evaluate(() => window.__getFuiWorkerDetailHasErrorClock?.() ?? false);
  }, { timeout: 4000 }).toBe(true);
});

test('renders the routed workbench reorder drag section', async ({ page }) => {
  await page.goto(`${baseUrl}/v2/fui-rs/demo/workbench/index.html`);
  await page.waitForFunction(() => window.__fuiReady === true);

  await expect.poll(async () => {
    return await page.evaluate(() => {
      const text = document.body.innerText;
      return text.includes('Retained reorder drag/drop')
        && text.includes('Drop at end of reorder list')
        && text.includes('Reorder order:');
    });
  }, { timeout: 10000 }).toBe(true);
});

test('reorders a routed workbench row by dragging without hanging the app', async ({ page }) => {
  const runtimeErrors: string[] = [];
  page.on('pageerror', error => {
    runtimeErrors.push(error.message);
  });
  page.on('console', message => {
    const text = message.text();
    if (message.type() === 'error' && !text.includes('missing-stage4')) {
      runtimeErrors.push(text);
    }
  });

  await page.goto(`${baseUrl}/v2/fui-rs/demo/workbench/index.html`);
  await page.waitForFunction(() => window.__fuiReady === true);
  const sceneSurface = page.locator('canvas').first();
  for (let attempt = 0; attempt < 8; attempt += 1) {
    const hasVisibleGrip = await page.evaluate(async () => {
      const debug = window.__fui_debug;
      if (debug === undefined || typeof debug.getDebugTree !== 'function') {
        return false;
      }
      const tree = await debug.getDebugTree();
      const grip = tree.nodes.find((entry) => entry.semanticLabel === 'Drag grip for Document Core rename');
      const visible = grip?.visibleBounds;
      return visible !== undefined && visible.width > 0 && visible.height > 0;
    });
    if (hasVisibleGrip) {
      break;
    }
    await page.mouse.wheel(0, 600);
    await page.waitForTimeout(120);
  }
  await page.waitForTimeout(600);

  const sourceBounds = await debugNodeScreenBounds(page, sceneSurface, 'Drag grip for Document Core rename');
  const source = {
    x: sourceBounds.x + 10,
    y: Math.floor(sourceBounds.y + (sourceBounds.height * 0.5)),
  };
  const viewport = await debugNodeCenter(page, sceneSurface, 'Reorder demo viewport');
  await page.mouse.move(source.x, source.y);
  await page.mouse.down();
  await page.mouse.move(source.x + 4, source.y + 58, { steps: 12 });
  await page.mouse.move(viewport.x, viewport.y + 95, { steps: 12 });
  await page.mouse.up();

  await expect.poll(async () => {
    return await page.evaluate(() => {
      return /Reorder drag status:[^\n]*/.exec(document.body.innerText)?.[0] ?? '';
    });
  }, { timeout: 10000 }).toContain('moved Document Core rename');

  await expect.poll(async () => {
    return await page.evaluate(() => {
      return /Reorder order:[^\n]*/.exec(document.body.innerText)?.[0] ?? '';
    });
  }).toContain('Audit font shard cache | Document Core rename');
  expect(runtimeErrors.filter(error => error.includes('Panic') || error.includes('unreachable'))).toEqual([]);
});

test('reorders a routed workbench row by touch long press and release', async ({ page, browserName }) => {
  test.skip(browserName !== 'chromium', 'Native CDP touch injection is Chromium-only.');
  await page.addInitScript(() => {
    Object.defineProperty(navigator, 'maxTouchPoints', { configurable: true, get: () => 5 });
  });
  await page.goto(`${baseUrl}/v2/fui-rs/demo/workbench/index.html`);
  await page.waitForFunction(() => window.__fuiReady === true);
  const sceneSurface = page.locator('canvas').first();
  for (let attempt = 0; attempt < 8; attempt += 1) {
    const visible = await page.evaluate(async () => {
      const tree = await window.__fui_debug?.getDebugTree();
      return tree?.nodes.find((entry) => entry.semanticLabel === 'Drag grip for Document Core rename')?.visibleBounds.height ?? 0;
    });
    if (visible > 0) break;
    await page.mouse.wheel(0, 600);
    await page.waitForTimeout(120);
  }

  const sourceBounds = await debugNodeScreenBounds(page, sceneSurface, 'Drag grip for Document Core rename');
  const viewport = await debugNodeCenter(page, sceneSurface, 'Reorder demo viewport');
  const start = {
    x: sourceBounds.x + sourceBounds.width * 0.5,
    y: sourceBounds.y + sourceBounds.height * 0.5,
  };
  const end = {
    x: viewport.x,
    y: viewport.y + 95,
  };
  const client = await page.context().newCDPSession(page);
  await client.send('Input.dispatchTouchEvent', {
    type: 'touchStart',
    touchPoints: [{ ...start, id: 72, radiusX: 8, radiusY: 8, force: 1 }],
  });
  await page.waitForTimeout(650);
  for (let step = 1; step <= 8; step += 1) {
    const progress = step / 8;
    await client.send('Input.dispatchTouchEvent', {
      type: 'touchMove',
      touchPoints: [{
        x: start.x + (end.x - start.x) * progress,
        y: start.y + (end.y - start.y) * progress,
        id: 72,
        radiusX: 8,
        radiusY: 8,
        force: 1,
      }],
    });
    await page.waitForTimeout(35);
  }
  await client.send('Input.dispatchTouchEvent', { type: 'touchEnd', touchPoints: [] });

  await expect.poll(async () => {
    return await page.evaluate(() => /Reorder drag status:[^\n]*/.exec(document.body.innerText)?.[0] ?? '');
  }).toContain('moved Document Core rename');
});

test('accepts metadata-first external file drops on the routed workbench target', async ({ page }) => {
  await page.goto(`${baseUrl}/v2/fui-rs/demo/workbench/index.html`);
  await page.waitForFunction(() => window.__fuiReady === true);
  await page.mouse.wheel(0, 7000);
  await page.waitForTimeout(300);

  const sceneSurface = page.locator('[data-effindom-canvas-size-source]');
  const dropPoint = await findExternalDropPoint(page, sceneSurface);
  await page.evaluate(async (payload) => {
    await window.__fui_debug?.externalDragEvent(1, payload.handle, payload.x, payload.y, [
      { name: 'todo.txt', type: 'text/plain', text: 'todo: ship' },
    ]);
    await window.__fui_debug?.externalDragEvent(2, payload.handle, payload.x, payload.y, [
      { name: 'todo.txt', type: 'text/plain', text: 'todo: ship' },
    ]);
  }, dropPoint);

  await expect.poll(async () => {
    return await page.evaluate(() => {
      return /External drop status:[^\n]*/.exec(document.body.innerText)?.[0] ?? '';
    });
  }, { timeout: 10000 }).toContain('hovering 1 file');

  await page.evaluate(async (payload) => {
    await window.__fui_debug?.externalDragEvent(4, payload.handle, payload.x, payload.y, [
      { name: 'todo.txt', type: 'text/plain', text: 'todo: ship' },
    ]);
  }, dropPoint);

  await expect.poll(async () => {
    return await page.evaluate(() => {
      return /External drop status:[^\n]*/.exec(document.body.innerText)?.[0] ?? '';
    });
  }, { timeout: 10000 }).toContain('dropped 1 file');
  await expect.poll(async () => {
    return await page.evaluate(() => {
      return /External drop items:[^\n]*/.exec(document.body.innerText)?.[0] ?? '';
    });
  }, { timeout: 10000 }).toContain('todo.txt (file, text/plain, 10 bytes)');
});

test('routed workbench fetch demo completes GET through browser host fetch', async ({ page }) => {
  await page.route('https://jsonplaceholder.typicode.com/posts/1', async route => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ id: 1, title: 'intercepted' }),
    });
  });
  await page.goto(`${baseUrl}/v2/fui-rs/demo/workbench/index.html`);
  await page.waitForFunction(() => window.__fuiReady === true);
  const sceneSurface = page.locator('[data-effindom-canvas-size-source]');
  await expect.poll(async () => {
    return await page.evaluate(() => document.body.innerText.includes('Online Fetch sample'));
  }, { timeout: 10000 }).toBe(true);
  for (let attempt = 0; attempt < 12; attempt += 1) {
    const visible = await page.evaluate(async () => {
      const debug = window.__fui_debug;
      if (debug === undefined) {
        return false;
      }
      const tree = await debug.getDebugTree();
      const node = tree.nodes.find((entry) => entry.semanticLabel === 'GET /posts/1');
      const bounds = node?.visibleBounds ?? null;
      return bounds !== null && bounds.width > 0 && bounds.height > 0;
    });
    if (visible) {
      break;
    }
    await page.mouse.wheel(0, 650);
    await page.waitForTimeout(120);
  }

  await clickLargestDebugNode(page, sceneSurface, 'GET /posts/1');
  await expect.poll(async () => {
    return await page.evaluate(() => document.body.innerText.includes('Fetch status: complete'));
  }, { timeout: 10000 }).toBe(true);
  await expect.poll(async () => {
    return await page.evaluate(() => document.body.innerText.includes('Latest result: GET https://jsonplaceholder.typicode.com/posts/1 -> ok=true • status 200 OK • resolved url https://jsonplaceholder.typicode.com/posts/1'));
  }, { timeout: 10000 }).toBe(true);
});

test('copies dropped files through the worker-backed save demo', async ({ page }) => {
  await page.addInitScript(() => {
    Object.defineProperty(window, 'showSaveFilePicker', {
      configurable: true,
      value: (options?: { suggestedName?: string; }) => {
        const fileName = options?.suggestedName ?? 'copy.bin';
        window.__demoCopiedFileName = fileName;
        const chunks: string[] = [];
        return Promise.resolve({
          name: fileName,
          createWritable() {
            return Promise.resolve({
              async write(data: BufferSource | Blob | string) {
                if (typeof data === 'string') {
                  chunks.push(data);
                  return;
                }
                if (data instanceof Blob) {
                  chunks.push(await data.text());
                  return;
                }
                if (data instanceof ArrayBuffer) {
                  chunks.push(new TextDecoder().decode(data));
                  return;
                }
                chunks.push(new TextDecoder().decode(data.buffer.slice(data.byteOffset, data.byteOffset + data.byteLength)));
              },
              close() {
                window.__demoCopiedFileText = chunks.join('');
                return Promise.resolve();
              },
              abort() {
                chunks.length = 0;
                return Promise.resolve();
              },
            });
          },
        });
      },
    });
  });
  await page.goto(`${baseUrl}/v2/fui-rs/demo/workbench/index.html`);
  await page.waitForFunction(() => window.__fuiReady === true);
  await page.mouse.wheel(0, 7000);
  await page.waitForTimeout(300);

  const sceneSurface = page.locator('[data-effindom-canvas-size-source]');
  const dropPoint = await findExternalDropPoint(page, sceneSurface);
  await page.evaluate(async (payload) => {
    await window.__fui_debug?.externalDragEvent(1, payload.handle, payload.x, payload.y, [
      { name: 'todo.txt', type: 'text/plain', text: 'todo: ship' },
    ]);
    await window.__fui_debug?.externalDragEvent(4, payload.handle, payload.x, payload.y, [
      { name: 'todo.txt', type: 'text/plain', text: 'todo: ship' },
    ]);
  }, dropPoint);
  await expect.poll(async () => {
    return await page.evaluate(() => {
      return /External drop status:[^\n]*/.exec(document.body.innerText)?.[0] ?? '';
    });
  }, { timeout: 10000 }).toContain('dropped 1 file');

  await clickDebugNode(page, sceneSurface, 'Save dropped file copy');

  await expect.poll(async () => {
    return await page.evaluate(() => {
      return /External drop status:[^\n]*/.exec(document.body.innerText)?.[0] ?? '';
    });
  }, { timeout: 12000 }).toContain('worker copied 10 bytes');
  await expect.poll(async () => {
    return await page.evaluate(() => window.__demoCopiedFileName ?? '');
  }, { timeout: 4000 }).toBe('todo-copy.txt');
  await expect.poll(async () => {
    return await page.evaluate(() => window.__demoCopiedFileText ?? '');
  }, { timeout: 4000 }).toBe('todo: ship');
});

test('routes to the stage 5 dropdown page and selects through keyboard interaction', async ({ page }) => {
  await page.goto(`${baseUrl}/v2/fui-rs/demo/stage5/index.html`);
  await page.waitForFunction(() => window.__fuiReady === true);
  await expect.poll(async () => {
    return await page.evaluate(() => document.body.innerText.includes('Phase 5.1 + 5.2 + 5.3 controls'));
  }, { timeout: 10000 }).toBe(true);

  const sceneSurface = page.locator('[data-effindom-canvas-size-source]');
  await clickLargestDebugNode(page, sceneSurface, 'Focused');
  await page.keyboard.press('ArrowDown');
  await page.keyboard.press('Home');
  await page.keyboard.press('Enter');

  await expect.poll(async () => {
    return await page.evaluate(() => document.body.innerText.includes('Normal changed: Calm at index 0'));
  }, { timeout: 10000 }).toBe(true);
});

test('routes to the stage 5 combobox page and selects through popup keyboard interaction', async ({ page }) => {
  await page.goto(`${baseUrl}/v2/fui-rs/demo/stage5/index.html`);
  await page.waitForFunction(() => window.__fuiReady === true);

  const sceneSurface = page.locator('[data-effindom-canvas-size-source]');
  for (let attempt = 0; attempt < 12; attempt += 1) {
    const visibleHeight = await debugNodeVisibleHeightByNodeId(page, 'stage5-combobox-normal');
    if (visibleHeight > 0) {
      break;
    }
    await page.mouse.wheel(0, 500);
    await page.waitForTimeout(100);
  }

  await clickLargestDebugNodeByNodeId(page, sceneSurface, 'stage5-combobox-normal');
  await expect.poll(async () => {
    return await page.evaluate(() => window.__bridgeActiveEditorWindow?.text ?? '');
  }, { timeout: 10000 }).toContain('Focused');
  await page.keyboard.press(process.platform === 'darwin' ? 'Meta+A' : 'Control+A');
  await page.keyboard.type('En');
  await expect.poll(async () => {
    return await page.evaluate(() => window.__bridgeActiveEditorWindow?.text ?? '');
  }, { timeout: 10000 }).toContain('En');
  await page.keyboard.press('Enter');

  await expect.poll(async () => {
    return await page.evaluate(() => document.body.innerText.includes('Combo changed: Energetic at index 2'));
  }, { timeout: 10000 }).toBe(true);

  for (const nodeId of [
    'stage5-combobox-filter',
    'stage5-combobox-autocomplete',
    'stage5-combobox-themed',
    'stage5-combobox-disabled',
    'stage5-combobox-templated',
  ]) {
    await expect.poll(async () => {
      return await debugNodeVisibleWidthByNodeId(page, nodeId);
    }, { timeout: 10000 }).toBeGreaterThanOrEqual(200);
  }

  for (let attempt = 0; attempt < 12; attempt += 1) {
    const visibleHeight = await debugNodeVisibleHeightByNodeId(page, 'stage5-combobox-autocomplete');
    if (visibleHeight > 0) {
      break;
    }
    await page.mouse.wheel(0, 500);
    await page.waitForTimeout(100);
  }
  await clickLargestDebugNodeByNodeId(page, sceneSurface, 'stage5-combobox-autocomplete');
  await page.keyboard.type('Mel');
  await expect.poll(async () => {
    return await debugNodeSemanticLabelByNodeId(page, 'stage5-combobox-autocomplete');
  }, { timeout: 10000 }).toBe('Melbourne');
  await page.keyboard.press('Backspace');
  await expect.poll(async () => {
    return await debugNodeSemanticLabelByNodeId(page, 'stage5-combobox-autocomplete');
  }, { timeout: 10000 }).toBe('Mel');
  await page.keyboard.press('Backspace');
  await expect.poll(async () => {
    return await debugNodeSemanticLabelByNodeId(page, 'stage5-combobox-autocomplete');
  }, { timeout: 10000 }).toBe('Me');
});

test('stage 5 combobox pointer item selection survives clearing and editor blur', async ({ page }) => {
  await page.goto(`${baseUrl}/v2/fui-rs/demo/stage5/index.html`);
  await page.waitForFunction(() => window.__fuiReady === true);

  const sceneSurface = page.locator('[data-effindom-canvas-size-source]');
  for (let attempt = 0; attempt < 12; attempt += 1) {
    const visibleHeight = await debugNodeVisibleHeightByNodeId(page, 'stage5-combobox-normal');
    if (visibleHeight > 0) {
      break;
    }
    await page.mouse.wheel(0, 500);
    await page.waitForTimeout(100);
  }

  await clickLargestDebugNodeByNodeId(page, sceneSurface, 'stage5-combobox-normal');
  await page.keyboard.press(process.platform === 'darwin' ? 'Meta+A' : 'Control+A');
  await page.keyboard.press('Backspace');

  await page.keyboard.press('ArrowDown');
  await expect.poll(async () => {
    return await page.evaluate(() => {
      return (window.__bridgeSemanticTree ?? []).some((entry) => entry.label === 'Energetic');
    });
  }, { timeout: 10000 }).toBe(true);
  await page.waitForTimeout(100);

  const optionBounds = await debugNodeScreenBounds(page, sceneSurface, 'Energetic');
  await page.mouse.move(
    Math.floor(optionBounds.x + (optionBounds.width * 0.5)),
    Math.floor(optionBounds.y + (optionBounds.height * 0.5)),
  );
  await page.waitForTimeout(50);
  await page.mouse.click(
    Math.floor(optionBounds.x + (optionBounds.width * 0.5)),
    Math.floor(optionBounds.y + (optionBounds.height * 0.5)),
  );

  await expect.poll(async () => {
    return await debugNodeSemanticLabelByNodeId(page, 'stage5-combobox-normal');
  }, { timeout: 10000 }).toBe('Energetic');
  await expect.poll(async () => {
    return await page.evaluate(() => document.body.innerText.includes('Combo changed: Energetic at index 2'));
  }, { timeout: 10000 }).toBe(true);

  for (let attempt = 0; attempt < 12; attempt += 1) {
    const visibleHeight = await debugNodeVisibleHeightByNodeId(page, 'stage5-combobox-templated');
    if (visibleHeight > 0) {
      break;
    }
    await page.mouse.wheel(0, 500);
    await page.waitForTimeout(100);
  }
  await clickLargestDebugNodeByNodeId(page, sceneSurface, 'stage5-combobox-templated');
  await page.keyboard.press('ArrowDown');
  await expect.poll(async () => {
    return await page.evaluate(() => {
      return (window.__bridgeSemanticTree ?? []).some((entry) => entry.label === 'Primary');
    });
  }, { timeout: 10000 }).toBe(true);
  await page.waitForTimeout(100);

  const templatedOptionBounds = await debugNodeScreenBounds(page, sceneSurface, 'Primary');
  await page.mouse.move(
    Math.floor(templatedOptionBounds.x + (templatedOptionBounds.width * 0.5)),
    Math.floor(templatedOptionBounds.y + (templatedOptionBounds.height * 0.5)),
  );
  await page.waitForTimeout(50);
  await page.mouse.click(
    Math.floor(templatedOptionBounds.x + (templatedOptionBounds.width * 0.5)),
    Math.floor(templatedOptionBounds.y + (templatedOptionBounds.height * 0.5)),
  );

  await expect.poll(async () => {
    return await debugNodeSemanticLabelByNodeId(page, 'stage5-combobox-templated');
  }, { timeout: 10000 }).toBe('Primary');
});

test('routes to the stage 5 text input page and reports typing plus selection', async ({ page, browserName }) => {
  await page.goto(`${baseUrl}/v2/fui-rs/demo/stage5/index.html`);
  await page.waitForFunction(() => window.__fuiReady === true);

  await waitForProjectedInput(page, 'stage5-username');
  await clickProjectedInput(page, 'stage5-username');
  await page.keyboard.type('hello world');
  await expect.poll(async () => {
    return await page.evaluate(() => document.body.innerText.includes('Username changed: hello world'));
  }, { timeout: 10000 }).toBe(true);
  await page.keyboard.press('Shift+ArrowLeft');
  await expect.poll(async () => {
    return await page.evaluate(() => document.body.innerText.includes('Selection: 11..10'));
  }, { timeout: 10000 }).toBe(true);
  await expect.poll(async () => {
    return await page.evaluate(() => document.body.innerText.includes('Disabled themed value'));
  }, { timeout: 10000 }).toBe(true);

  await page.keyboard.press('Tab');
  await page.keyboard.type('secret');
  await expect.poll(async () => {
    return await page.evaluate(() => document.body.innerText.includes('Password changed: 6 chars'));
  }, { timeout: 10000 }).toBe(true);

  if (browserName === 'chromium') {
    await page.evaluate(() => {
      const runtime = window.EffinDomBrowserBridge?.getRuntime();
      if (runtime === null || runtime === undefined) {
        return;
      }
      if (window.__tofuSwapCommandBufferCaptureInstalled === true) {
        return;
      }
      window.__tofuSwapCommandBufferCaptureInstalled = true;
      window.__tofuSwapCommandBuffers = [];
      const capture = (): void => {
        const buffers = window.__tofuSwapCommandBuffers;
        if (buffers !== undefined) {
          buffers.push(Array.from(runtime.extractCommandBuffer()));
        }
      };
      const originalCommitFrame = runtime.commitFrame.bind(runtime);
      runtime.commitFrame = (...args: Parameters<typeof runtime.commitFrame>) => {
        originalCommitFrame(...args);
        capture();
      };
      const originalFlushPendingCommit = runtime.flushPendingCommit.bind(runtime);
      runtime.flushPendingCommit = (...args: Parameters<typeof runtime.flushPendingCommit>) => {
        const result = originalFlushPendingCommit(...args);
        capture();
        return result;
      };
    });
    await clickProjectedInput(page, 'stage5-username');
    await page.keyboard.press('ControlOrMeta+A');
    await page.keyboard.insertText('我想睡觉');
    const activeEditorHandle = await page.evaluate(() => window.__bridgeActiveEditorWindow?.handle ?? null);
    expect(activeEditorHandle).not.toBeNull();
    await expect.poll(async () => {
      return await page.evaluate(() => {
        const runtime = window.EffinDomBrowserBridge?.getRuntime();
        const state = runtime?.getIncrementalFontState(1);
        return state?.appliedSegmentIds.some((id) => id.startsWith('cjk-sc:')) === true;
      });
    }, { timeout: 15000 }).toBe(true);
    await expect.poll(async () => {
      const buffers = await page.evaluate(() => window.__tofuSwapCommandBuffers ?? []);
      return buffers.some((commandWords) => {
        const glyphFontIds = parseGlyphRuns(commandWords)
          .filter((run) => run.handle === activeEditorHandle)
          .flatMap((run) => run.glyphFontIds);
        return glyphFontIds.some((fontId) => fontId !== 1);
      });
    }, { timeout: 15000 }).toBe(true);
  }

  const cjkText = '我想睡觉'.repeat(24);
  await clickProjectedInput(page, 'stage5-username');
  await page.keyboard.press('ControlOrMeta+A');
  await page.keyboard.insertText(cjkText);
  await page.keyboard.press('Tab');
  await page.keyboard.press('Shift+Tab');
  await page.keyboard.press('Backspace');
  await page.keyboard.press('Backspace');
  await page.keyboard.press('Backspace');
  await page.keyboard.press('Backspace');
  await expect.poll(async () => {
    return await page.evaluate(() => {
      const handle = window.__bridgeActiveEditorWindow?.handle;
      return handle === null || handle === undefined ? null : window.__bridgeTextByHandle?.[handle] ?? null;
    });
  }, { timeout: 10000 }).toBe(cjkText.slice(0, -4));
  await page.keyboard.press('ControlOrMeta+A');
  await page.keyboard.press('Backspace');
  await expect.poll(async () => {
    return await page.evaluate(() => {
      const handle = window.__bridgeActiveEditorWindow?.handle;
      return handle === null || handle === undefined ? null : window.__bridgeTextByHandle?.[handle] ?? null;
    });
  }, { timeout: 10000 }).toBe('');
});

test('stage 5 password input double-click selects the obscured field and blocks copy', async ({ page }) => {
  await page.goto(`${baseUrl}/v2/fui-rs/demo/stage5/index.html`);
  await page.waitForFunction(() => window.__fuiReady === true);

  await waitForProjectedInput(page, 'stage5-password');
  await clickProjectedInput(page, 'stage5-password');
  await page.keyboard.press('ControlOrMeta+A');
  await page.keyboard.type('secret-word');

  await doubleClickProjectedInput(page, 'stage5-password', 0.78);
  await expect.poll(async () => {
    return await page.evaluate(() => {
      const editor = document.activeElement;
      if (!(editor instanceof HTMLInputElement) || editor.dataset.effindomHiddenEditor !== 'true') {
        return null;
      }
      return {
        start: editor.selectionStart ?? 0,
        end: editor.selectionEnd ?? 0,
        focused: true,
      };
    });
  }, { timeout: 10000 }).toEqual({ start: 0, end: 'secret-word'.length, focused: true });

  await page.evaluate(() => {
    if (window.__bridgeLogs !== undefined) {
      window.__bridgeLogs.clipboardWrites.length = 0;
    }
  });
  await page.keyboard.press('ControlOrMeta+C');
  await expect.poll(async () => {
    return await page.evaluate(() => window.__bridgeLogs?.clipboardWrites ?? []);
  }, { timeout: 10000 }).toEqual([]);
});

test('stage 5 form projects grouped host autofill inputs', async ({ page }) => {
  await page.goto(`${baseUrl}/v2/fui-rs/demo/stage5/index.html`);
  await page.waitForFunction(() => window.__fuiReady === true);

  await waitForProjectedInput(page, 'stage5-username');

  await expect.poll(async () => page.locator('form[data-effindom-projected-form="true"]').count(), { timeout: 10000 }).toBe(1);
  await expect.poll(async () => {
    return await page.evaluate(() => {
      const username = document.querySelector<HTMLInputElement>('form[data-effindom-projected-form="true"] input[name="stage5-username"]');
      const password = document.querySelector<HTMLInputElement>('form[data-effindom-projected-form="true"] input[name="stage5-password"]');
      return {
        usernameAutocomplete: username?.getAttribute('autocomplete') ?? null,
        usernameId: username?.id ?? null,
        usernameType: username?.type ?? null,
        passwordAutocomplete: password?.getAttribute('autocomplete') ?? null,
        passwordId: password?.id ?? null,
        passwordType: password?.type ?? null,
      };
    });
  }, { timeout: 10000 }).toEqual({
    usernameAutocomplete: 'username',
    usernameId: 'stage5-username',
    usernameType: 'text',
    passwordAutocomplete: 'current-password',
    passwordId: 'stage5-password',
    passwordType: 'password',
  });

  await clickProjectedInput(page, 'stage5-username');
  await page.keyboard.type('autofill-user');
  await expect.poll(async () => {
    return await page.evaluate(() => {
      return document.querySelector<HTMLInputElement>('form[data-effindom-projected-form="true"] input[name="stage5-username"]')?.value ?? null;
    });
  }, { timeout: 10000 }).toBe('autofill-user');

  await page.keyboard.press('Enter');
  await expect.poll(async () => {
    return await page.evaluate(() => document.body.innerText.includes("Form submit: OK • username 'autofill-user'"));
  }, { timeout: 10000 }).toBe(true);

  await page.keyboard.press('Escape');
  await expect.poll(async () => {
    return await page.evaluate(() => document.body.innerText.includes('Form submit: Cancel'));
  }, { timeout: 10000 }).toBe(true);
});

test('stage 5 text input keeps CJK text stable after blur, refocus, and middle insertion', async ({ page, browserName }) => {
  test.skip(browserName !== 'chromium', 'This regression covers the Chromium/Edge hidden-editor path.');

  const baseText = '我想睡觉'.repeat(24);
  const insertion = '你好朋友再见世界';
  const insertionIndex = 52;
  const expected = `${baseText.slice(0, insertionIndex)}${insertion}${baseText.slice(insertionIndex)}`;

  await page.goto(`${baseUrl}/v2/fui-rs/demo/stage5/index.html`);
  await page.waitForFunction(() => window.__fuiReady === true);
  await page.evaluate(() => {
    window.EffinDomBrowserBridge?.getRuntime()?.setIncrementalFontPolicy({ maxCachedShardFonts: 1 });
  });

  await waitForProjectedInput(page, 'stage5-username');

  await clickProjectedInput(page, 'stage5-username');
  await page.keyboard.press('ControlOrMeta+A');
  await page.keyboard.insertText(baseText);
  await expect.poll(async () => {
    return await page.evaluate(() => {
      const handle = window.__bridgeActiveEditorWindow?.handle;
      return handle === null || handle === undefined ? null : window.__bridgeTextByHandle?.[handle] ?? null;
    });
  }, { timeout: 10000 }).toBe(baseText);

  await clickProjectedInput(page, 'stage5-password');
  await clickProjectedInput(page, 'stage5-username');

  const byteOffset = new TextEncoder().encode(baseText.slice(0, insertionIndex)).length;
  await page.evaluate(({ codeUnitOffset, nativeByteOffset }) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const activeEditorWindow = window.__bridgeActiveEditorWindow;
    const handle = activeEditorWindow?.handle ?? null;
    const editor = document.querySelector<HTMLInputElement>('input[data-effindom-hidden-editor="true"]');
    if (runtime === null || runtime === undefined || handle === null || editor === null) {
      throw new Error('Expected focused hidden text editor.');
    }
    const handleArg = runtime.ui.usesMemory64 === true ? BigInt(handle) : Number(handle);
    runtime.ui._ui_set_text_selection_range(handleArg, nativeByteOffset, nativeByteOffset);
    runtime.commitFrame();
    runtime.flushPendingCommit();
    editor.focus();
    editor.setSelectionRange(codeUnitOffset, codeUnitOffset, 'none');
  }, { codeUnitOffset: insertionIndex, nativeByteOffset: byteOffset });

  for (const char of insertion) {
    await page.keyboard.insertText(char);
  }

  await expect.poll(async () => {
    return await page.evaluate(() => {
      const handle = window.__bridgeActiveEditorWindow?.handle;
      return handle === null || handle === undefined ? null : window.__bridgeTextByHandle?.[handle] ?? null;
    });
  }, { timeout: 10000 }).toBe(expected);
  await expect.poll(async () => {
    return await page.evaluate((value) => document.body.innerText.includes(`Username changed: ${value}`), expected);
  }, { timeout: 10000 }).toBe(true);
  await expect.poll(async () => {
    return await page.evaluate((value) => {
      const state = window.EffinDomBrowserBridge?.getRuntime()?.getIncrementalFontState(1);
      if (state?.pendingSegmentIds.length !== 0) {
        return false;
      }
      const requiredChars = Array.from(new Set(Array.from(value)));
      return state.appliedSegmentIds.some((segmentId) =>
        segmentId.startsWith('cjk-sc:') && requiredChars.every((char) => segmentId.includes(char)));
    }, expected);
  }, { timeout: 15000 }).toBe(true);
  await expect.poll(async () => {
    return await page.evaluate((value) => {
      const runtime = window.EffinDomBrowserBridge?.getRuntime();
      if (runtime === null || runtime === undefined) {
        return false;
      }
      const cache = runtime.getIncrementalFontCacheState();
      const requiredChars = Array.from(new Set(Array.from(value)));
      return cache.cachedShardCount <= 1
        && cache.evictedShardKeys.length > 0
        && cache.cachedShardKeys.some((segmentId) =>
          segmentId.startsWith('cjk-sc:') && requiredChars.every((char) => segmentId.includes(char)));
    }, expected);
  }, { timeout: 15000 }).toBe(true);
});

test('routes to the stage 5 text area page and reports multiline editing plus selection', async ({ page }) => {
  await page.goto(`${baseUrl}/v2/fui-rs/demo/stage5/index.html?debug-logs=1`);
  await page.waitForFunction(() => window.__fuiReady === true);

  const sceneSurface = page.locator('[data-effindom-canvas-size-source]');
  for (let attempt = 0; attempt < 16; attempt += 1) {
    const visibleHeight = await debugNodeVisibleHeightByNodeId(page, 'stage5-text-area');
    if (visibleHeight > 0) {
      break;
    }
    await page.mouse.wheel(0, 500);
    await page.waitForTimeout(100);
  }
  await clickLargestDebugNodeByNodeId(page, sceneSurface, 'stage5-text-area');
  await expect.poll(async () => {
    return await page.evaluate(() => window.__bridgeActiveEditorWindow?.text ?? '');
  }, { timeout: 10000 }).toContain('Line one\nLine two\nLine three\nFallback sample: 你好，你好吗？');

  const textAreaHandle = await page.evaluate(() => window.__bridgeActiveEditorWindow?.handle ?? null);
  expect(textAreaHandle).not.toBeNull();
  await page.keyboard.press('Shift+Tab');
  await expect.poll(async () => {
    return await page.evaluate((handle) => window.__bridgeActiveEditorWindow?.handle !== handle, textAreaHandle);
  }, { timeout: 10000 }).toBe(true);
  await page.keyboard.press('Tab');
  await expect.poll(async () => {
    return await page.evaluate((handle) => {
      const activeElement = document.activeElement;
      return window.__bridgeActiveEditorWindow?.handle === handle &&
        activeElement instanceof HTMLTextAreaElement &&
        activeElement.dataset.effindomHiddenEditor === 'true';
    }, textAreaHandle);
  }, { timeout: 10000 }).toBe(true);

  await setHiddenEditorSelection(page, 'Line one\n'.length);
  await page.keyboard.press('Tab');
  await page.keyboard.press('Tab');
  await page.keyboard.press('Backspace');
  await expect.poll(async () => {
    return await page.evaluate(() => window.__bridgeActiveEditorWindow?.text ?? '');
  }, { timeout: 10000 }).toContain('Line one\n\tLine two\nLine three\nFallback sample: 你好，你好吗？');
  await setHiddenEditorSelection(page, 0);
  await page.keyboard.press('Tab');
  await page.keyboard.press('Tab');
  await page.keyboard.press('Tab');
  await page.keyboard.press('Backspace');
  await page.keyboard.press('Backspace');
  await page.keyboard.press('Backspace');
  await expect.poll(async () => {
    return await page.evaluate(() => window.__bridgeActiveEditorWindow?.text ?? '');
  }, { timeout: 10000 }).toContain('Line one\n\tLine two\nLine three\nFallback sample: 你好，你好吗？');
  await page.goto(`${baseUrl}/v2/fui-rs/demo/stage5/index.html?debug-logs=1`);
  await page.waitForFunction(() => window.__fuiReady === true);
  for (let attempt = 0; attempt < 16; attempt += 1) {
    const visibleHeight = await debugNodeVisibleHeightByNodeId(page, 'stage5-text-area');
    if (visibleHeight > 0) {
      break;
    }
    await page.mouse.wheel(0, 500);
    await page.waitForTimeout(100);
  }
  await clickLargestDebugNodeByNodeId(page, sceneSurface, 'stage5-text-area');
  const initialStage5Text = 'Line one\nLine two\nLine three\nFallback sample: 你好，你好吗？\nLonger content so scrollbar policy is easy to spot.';
  await expect.poll(async () => {
    return await page.evaluate(() => window.__bridgeActiveEditorWindow?.text ?? '');
  }, { timeout: 10000 }).toBe(initialStage5Text);
  await setHiddenEditorSelection(page, 0);
  await page.keyboard.press('Tab');
  await expect.poll(async () => {
    return await page.evaluate(() => window.__bridgeActiveEditorWindow?.text ?? '');
  }, { timeout: 10000 }).toBe(`\t${initialStage5Text}`);
  await page.keyboard.press(process.platform === 'darwin' ? 'Meta+Z' : 'Control+Z');
  await expect.poll(async () => {
    return await page.evaluate(() => window.__bridgeActiveEditorWindow?.text ?? '');
  }, { timeout: 10000 }).toBe(initialStage5Text);
  await page.keyboard.press('Tab');
  await expect.poll(async () => {
    return await page.evaluate(() => window.__bridgeActiveEditorWindow?.text ?? '');
  }, { timeout: 10000 }).toContain('\t');
  await page.keyboard.type('Z');
  await expect.poll(async () => {
    return await page.evaluate(() => window.__bridgeActiveEditorWindow?.text ?? '');
  }, { timeout: 10000 }).toContain('Z');
  await page.keyboard.press('Backspace');
  await expect.poll(async () => {
    return await page.evaluate(() => window.__bridgeActiveEditorWindow?.text ?? '');
  }, { timeout: 10000 }).not.toContain('Z');
  await page.keyboard.press('ControlOrMeta+A');
  await expect.poll(async () => {
    return await page.evaluate(() => {
      const handle = window.__bridgeActiveEditorWindow?.handle;
      if (handle === null || handle === undefined) {
        return false;
      }
      const text = window.__bridgeTextByHandle?.[handle] ?? '';
      const selection = window.__bridgeSelectionsByHandle?.[handle] ?? null;
      if (selection === null) {
        return false;
      }
      return selection.start === 0 && selection.end === new TextEncoder().encode(text).byteLength;
    });
  }, { timeout: 10000 }).toBe(true);
  await page.keyboard.type('omega');
  await page.keyboard.press('Enter');
  await page.keyboard.type('beta!');
  await page.keyboard.press('ArrowLeft');
  const pageScrollBeforeEditorArrow = await page.evaluate(() => document.scrollingElement?.scrollTop ?? window.scrollY);
  await page.keyboard.press('ArrowDown');
  await expect.poll(async () => {
    return await page.evaluate(() => document.scrollingElement?.scrollTop ?? window.scrollY);
  }, { timeout: 10000 }).toBe(pageScrollBeforeEditorArrow);
  await page.keyboard.press('Delete');
  await page.keyboard.insertText('\npasted');

  await expect.poll(async () => {
    return await page.evaluate(() => {
      const handle = window.__bridgeActiveEditorWindow?.handle;
      const text = handle === null || handle === undefined ? '' : window.__bridgeTextByHandle?.[handle] ?? '';
      return text.startsWith('omega\n')
        && text.includes('pasted')
        && text !== 'omega\nbeta!';
    });
  }, { timeout: 10000 }).toBe(true);
  await expect.poll(async () => {
    return await page.evaluate(async () => {
      const tree = await window.__fui_debug?.getDebugTree();
      return tree?.nodes
        .map((entry) => entry.semanticLabel)
        .find((label) => label.startsWith('TextArea value:')) ?? '';
    });
  }, { timeout: 10000 }).not.toBe('TextArea value: 115 chars • 5 lines');
  await page.keyboard.press('Shift+ArrowLeft');
  await expect.poll(async () => {
    return await page.evaluate(async () => {
      const tree = await window.__fui_debug?.getDebugTree();
      return tree?.nodes
        .map((entry) => entry.semanticLabel)
        .find((label) => label.startsWith('TextArea selection:')) ?? '';
    });
  }, { timeout: 10000 }).not.toBe('TextArea selection: 0..0');
  await expect.poll(async () => {
    return await page.evaluate(() => document.body.innerText.includes('TextArea config: read-only off • wrapping on • tabs insert'));
  }, { timeout: 10000 }).toBe(true);
  await expect.poll(async () => {
    return await page.evaluate(() => document.body.innerText.includes('TextArea scroll offset:'));
  }, { timeout: 10000 }).toBe(true);
  await page.mouse.move(8, 8);
  for (let attempt = 0; attempt < 12; attempt += 1) {
    const visibleHeight = await debugNodeVisibleHeightByNodeId(page, 'stage5-text-area-wrapping-toggle');
    if (visibleHeight > 0) {
      break;
    }
    await page.mouse.wheel(0, 300);
    await page.waitForTimeout(100);
  }
  await clickLargestDebugNodeByNodeId(page, sceneSurface, 'stage5-text-area-wrapping-toggle');
  await expect.poll(async () => {
    return await page.evaluate(() => document.body.innerText.includes('TextArea config: read-only off • wrapping off • tabs insert'));
  }, { timeout: 10000 }).toBe(true);
  for (let attempt = 0; attempt < 12; attempt += 1) {
    const visibleHeight = await debugNodeVisibleHeightByNodeId(page, 'stage5-text-area-readonly-toggle');
    if (visibleHeight > 0) {
      break;
    }
    await page.mouse.wheel(0, 300);
    await page.waitForTimeout(100);
  }
  await clickLargestDebugNodeByNodeId(page, sceneSurface, 'stage5-text-area-readonly-toggle');
  await expect.poll(async () => {
    return await page.evaluate(() => document.body.innerText.includes('TextArea config: read-only on • wrapping off'));
  }, { timeout: 10000 }).toBe(true);
});

test('stage 5 tabs preserve cross-layer editing and horizontal windowing', async ({ page }) => {
  const authoredText = 'ab\tcd efgh ijkl\n0123\t4567 89ab cdef';
  const withoutFirstTab = authoredText.replace('\t', '');

  await page.goto(baseUrl + '/v2/fui-rs/demo/stage5/index.html?debug-logs=1');
  await page.waitForFunction(() => window.__fuiReady === true);
  const sceneSurface = page.locator('[data-effindom-canvas-size-source]');

  for (let attempt = 0; attempt < 20; attempt += 1) {
    if (await debugNodeVisibleHeightByNodeId(page, 'stage5-text-area') > 0) {
      break;
    }
    await page.mouse.wheel(0, 400);
    await page.waitForTimeout(100);
  }
  await clickLargestDebugNodeByNodeId(page, sceneSurface, 'stage5-text-area');
  await page.keyboard.press('ControlOrMeta+A');
  await page.keyboard.insertText('ab');
  await page.keyboard.press('Tab');
  await page.keyboard.insertText('cd efgh ijkl');
  await page.keyboard.press('Enter');
  await page.keyboard.insertText('0123');
  await page.keyboard.press('Tab');
  await page.keyboard.insertText('4567 89ab cdef');
  await expect.poll(async () => {
    return await page.evaluate(() => window.__bridgeActiveEditorWindow?.text ?? '');
  }, { timeout: 10000 }).toBe(authoredText);

  await page.waitForTimeout(1000);
  const canvasBounds = await sceneSurface.boundingBox();
  expect(canvasBounds).not.toBeNull();
  const textAreaBounds = await page.evaluate(async () => {
    const tree = await window.__fui_debug?.getDebugTree();
    return tree?.nodes.find((entry) => entry.nodeId === 'stage5-text-area')?.visibleBounds ?? null;
  });
  expect(textAreaBounds).not.toBeNull();
  if (canvasBounds === null || textAreaBounds === null) {
    throw new Error('Expected visible Stage 5 TextArea bounds.');
  }
  await page.mouse.click(
    canvasBounds.x + textAreaBounds.x + 4,
    canvasBounds.y + textAreaBounds.y + 4,
  );
  await expect.poll(async () => {
    return await page.evaluate((documentLength) => {
      const handle = window.__bridgeActiveEditorWindow?.handle;
      const selection = handle === null || handle === undefined
        ? null
        : window.__bridgeSelectionsByHandle?.[handle] ?? null;
      return selection === null ? null : {
        collapsed: selection.start === selection.end,
        beforeDocumentEnd: selection.end < documentLength,
      };
    }, authoredText.length);
  }, { timeout: 10000 }).toEqual({ collapsed: true, beforeDocumentEnd: true });

  await setHiddenEditorSelection(page, 2, 3);
  await page.keyboard.press('Backspace');
  await expect.poll(async () => {
    return await page.evaluate(() => window.__bridgeActiveEditorWindow?.text ?? '');
  }, { timeout: 10000 }).toBe(withoutFirstTab);
  await page.keyboard.press('ControlOrMeta+Z');
  await expect.poll(async () => {
    return await page.evaluate(() => window.__bridgeActiveEditorWindow?.text ?? '');
  }, { timeout: 10000 }).toBe(authoredText);

  await setHiddenEditorSelection(page, 0);
  await page.keyboard.press('Tab');
  await expect.poll(async () => {
    return await page.evaluate(() => window.__bridgeActiveEditorWindow?.text ?? '');
  }, { timeout: 10000 }).toBe('\t' + authoredText);
  await page.keyboard.press('ControlOrMeta+Z');

  for (let attempt = 0; attempt < 20; attempt += 1) {
    if (await debugNodeVisibleHeightByNodeId(page, 'stage5-text-area-wrapping-toggle') > 0) {
      break;
    }
    await page.mouse.wheel(0, 350);
    await page.waitForTimeout(100);
  }
  await clickLargestDebugNodeByNodeId(page, sceneSurface, 'stage5-text-area-wrapping-toggle');
  for (let attempt = 0; attempt < 20; attempt += 1) {
    if (await debugNodeVisibleHeightByNodeId(page, 'stage5-text-area') > 0) {
      break;
    }
    await page.mouse.wheel(0, -350);
    await page.waitForTimeout(100);
  }
  await clickLargestDebugNodeByNodeId(page, sceneSurface, 'stage5-text-area');
  const longLine = 'ab\t' + 'x'.repeat(5000);
  await page.keyboard.press('ControlOrMeta+A');
  await page.keyboard.insertText('ab');
  await page.keyboard.press('Tab');
  await page.keyboard.insertText('x'.repeat(5000));
  await page.keyboard.press('End');
  await expect.poll(async () => {
    return await page.evaluate((documentLength) => {
      const editor = document.activeElement;
      const activeHandle = window.__bridgeActiveEditorWindow?.handle ?? '';
      return !(editor instanceof HTMLTextAreaElement) ? null : {
        windowed: editor.value.length < documentLength,
        docStartMoved: (window.__bridgeActiveEditorWindow?.docStart ?? 0) > 0,
        caretAtEnd: (window.__bridgeSelectionsByHandle?.[activeHandle]?.end ?? -1) === documentLength,
      };
    }, longLine.length);
  }, { timeout: 10000 }).toEqual({
    windowed: true,
    docStartMoved: true,
    caretAtEnd: true,
  });
});

test('stage 5 textarea selects a windowed pasted document on the first Meta+A', async ({ page, browserName }) => {
  test.skip(browserName === 'webkit', 'WebKit does not support Playwright clipboard permissions.');
  const paragraph = 'Windowed UTF-8 selection regression: Variable-Height — 你好，你好吗？';
  const pastedText = Array.from({ length: 400 }, () => paragraph).join('\n');

  await page.context().grantPermissions(['clipboard-read', 'clipboard-write'], { origin: baseUrl });
  await page.goto(`${baseUrl}/v2/fui-rs/demo/stage5/index.html?debug-logs=1`);
  await page.waitForFunction(() => window.__fuiReady === true);
  const sceneSurface = page.locator('[data-effindom-canvas-size-source]');
  for (let attempt = 0; attempt < 16; attempt += 1) {
    if (await debugNodeVisibleHeightByNodeId(page, 'stage5-text-area') > 0) {
      break;
    }
    await page.mouse.wheel(0, 500);
    await page.waitForTimeout(100);
  }
  await clickLargestDebugNodeByNodeId(page, sceneSurface, 'stage5-text-area');
  await page.evaluate(async (value) => {
    await navigator.clipboard.writeText(value);
  }, pastedText);
  await page.keyboard.press('ControlOrMeta+V');
  await expect.poll(async () => await page.evaluate((minimumLength) => {
    const handle = window.__bridgeActiveEditorWindow?.handle;
    const text = handle === null || handle === undefined ? '' : window.__bridgeTextByHandle?.[handle] ?? '';
    return text.length >= minimumLength;
  }, pastedText.length)).toBe(true);

  await page.keyboard.press('Meta+A');

  await expect.poll(async () => await page.evaluate(() => {
    const handle = window.__bridgeActiveEditorWindow?.handle;
    if (handle === null || handle === undefined) {
      return false;
    }
    const text = window.__bridgeTextByHandle?.[handle] ?? '';
    const selection = window.__bridgeSelectionsByHandle?.[handle];
    return selection?.start === 0 && selection.end === new TextEncoder().encode(text).length;
  })).toBe(true);

  await page.keyboard.press('Backspace');
  await expect.poll(async () => await page.evaluate(() => {
    const handle = window.__bridgeActiveEditorWindow?.handle;
    return handle === null || handle === undefined
      ? null
      : window.__bridgeTextByHandle?.[handle] ?? null;
  })).toBe('');
});

test('stage 5 textarea touch selection handle drag keeps a range selection', async ({ page, browserName }) => {
  test.skip(browserName !== 'chromium', 'Native CDP touch injection is Chromium-only.');
  await page.addInitScript(() => {
    Object.defineProperty(navigator, 'maxTouchPoints', { configurable: true, get: () => 5 });
    const originalMatchMedia = window.matchMedia.bind(window);
    window.matchMedia = (query: string): MediaQueryList => {
      if (query === '(pointer: coarse)') {
        return {
          matches: true,
          media: query,
          onchange: null,
          addEventListener: () => undefined,
          removeEventListener: () => undefined,
          addListener: () => undefined,
          removeListener: () => undefined,
          dispatchEvent(): boolean { return false; },
        } as MediaQueryList;
      }
      return originalMatchMedia(query);
    };
  });
  await page.setViewportSize({ width: 430, height: 932 });
  await page.goto(`${baseUrl}/v2/fui-rs/demo/stage5/index.html?debug-logs=1`);
  await page.waitForFunction(() => window.__fuiReady === true);
  const sceneSurface = page.locator('[data-effindom-canvas-size-source]');
  for (let attempt = 0; attempt < 16; attempt += 1) {
    const visibleHeight = await debugNodeVisibleHeightByNodeId(page, 'stage5-text-area');
    if (visibleHeight > 0) {
      break;
    }
    await page.mouse.wheel(0, 500);
    await page.waitForTimeout(100);
  }
  await page.waitForTimeout(600);

  const textArea = await page.evaluate(async () => {
    const tree = await window.__fui_debug?.getDebugTree();
    const node = tree?.nodes.find((entry) => entry.nodeId === 'stage5-text-area');
    if (node === undefined) {
      throw new Error('Expected stage5 TextArea debug node.');
    }
    return { handle: node.handle, visible: node.visibleBounds };
  });
  expect(textArea.visible.height).toBeGreaterThan(0);

  const client = await page.context().newCDPSession(page);
  const pressPoint = { x: textArea.visible.x + 35, y: textArea.visible.y + 12 };
  await client.send('Input.dispatchTouchEvent', {
    type: 'touchStart',
    touchPoints: [{ x: pressPoint.x, y: pressPoint.y, id: 901, radiusX: 6, radiusY: 6, force: 1 }],
  });
  await page.waitForTimeout(700);
  await client.send('Input.dispatchTouchEvent', { type: 'touchEnd', touchPoints: [] });
  await page.waitForTimeout(250);

  await expect.poll(async () => {
    return await page.evaluate((handle) => {
      const selection = window.__bridgeSelectionsByHandle?.[handle] ?? null;
      return selection === null ? 0 : selection.end - selection.start;
    }, textArea.handle);
  }, { timeout: 10000 }).toBeGreaterThan(0);
  const initialSelection = await page.evaluate((handle) => window.__bridgeSelectionsByHandle?.[handle] ?? null, textArea.handle);

  const selectionHandles = await page.evaluate(async () => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const tree = await window.__fui_debug?.getDebugTree();
    if (runtime === undefined || runtime === null || tree === undefined || typeof runtime.ui._ui_preserves_selection_on_pointer_down !== 'function') {
      throw new Error('Expected runtime selection handle query.');
    }
    const preservesSelectionOnPointerDown = runtime.ui._ui_preserves_selection_on_pointer_down;
    const handles = tree.nodes
      .filter((node) => node.visibleBounds.width > 0 &&
        node.visibleBounds.height > 0 &&
        preservesSelectionOnPointerDown(BigInt(node.handle)) === 1 &&
        Math.round(node.visibleBounds.width) === 90 &&
        Math.round(node.visibleBounds.height) === 90)
      .map((node) => ({ handle: node.handle, bounds: node.visibleBounds }))
      .sort((left, right) => left.bounds.x - right.bounds.x);
    if (handles.length === 0) {
      throw new Error('Expected visible selection handle.');
    }
    return handles;
  });
  expect(selectionHandles.length).toBeGreaterThanOrEqual(2);
  const startHandle = selectionHandles[0];
  const stationaryEndHandle = selectionHandles[1];
  await expect.poll(async () => {
    return await page.evaluate((handle) => {
      const runtime = window.EffinDomBrowserBridge?.getRuntime();
      if (runtime === undefined || runtime === null) {
        return null;
      }
      const probeX = handle.bounds.x + 10;
      const probeY = handle.bounds.y + (handle.bounds.height * 0.5);
      return runtime.getHandleFromPoint(probeX, probeY).toString();
    }, startHandle);
  }, { timeout: 10000 }).toBe(startHandle.handle);
  const start = {
    x: startHandle.bounds.x + 63,
    y: startHandle.bounds.y + 34,
  };
  const dragEnd = {
    x: stationaryEndHandle.bounds.x + stationaryEndHandle.bounds.width + 80,
    y: start.y + 36,
  };
  await expect.poll(async () => {
    return await page.evaluate((args) => {
      const runtime = window.EffinDomBrowserBridge?.getRuntime();
      if (runtime === undefined || runtime === null) {
        return 0;
      }
      const hit = runtime.getHandleFromPoint(args.x, args.y);
      return runtime.ui._ui_preserves_selection_on_pointer_down?.(hit) ?? 0;
    }, { x: start.x, y: start.y });
  }, { timeout: 10000 }).toBe(1);
  await client.send('Input.dispatchTouchEvent', {
    type: 'touchStart',
    touchPoints: [{ x: start.x, y: start.y, id: 902, radiusX: 8, radiusY: 8, force: 1 }],
  });
  await page.waitForTimeout(80);
  await client.send('Input.dispatchTouchEvent', {
    type: 'touchMove',
    touchPoints: [{ x: dragEnd.x, y: dragEnd.y, id: 902, radiusX: 8, radiusY: 8, force: 1 }],
  });
  await page.waitForTimeout(120);
  await expect.poll(async () => {
    return await page.evaluate(async (handle) => {
      const tree = await window.__fui_debug?.getDebugTree();
      const node = tree?.nodes.find((entry) => entry.handle === handle.handle);
      return node === undefined ? null : Math.round(node.visibleBounds.x);
    }, stationaryEndHandle);
  }, { timeout: 10000 }).toBe(Math.round(stationaryEndHandle.bounds.x));
  await client.send('Input.dispatchTouchEvent', { type: 'touchEnd', touchPoints: [] });
  await client.detach();

  const finalSelection = await page.evaluate((handle) => window.__bridgeSelectionsByHandle?.[handle] ?? null, textArea.handle);
  expect(finalSelection).not.toBeNull();
  expect(finalSelection?.end).toBeGreaterThan(finalSelection?.start ?? 0);
  expect(finalSelection).not.toEqual(initialSelection);
  await expect(sceneSurface).toBeVisible();
});

test('routes to the stage 5 text input page with readable dark themed disabled input', async ({ page }) => {
  await page.emulateMedia({ colorScheme: 'dark' });
  await page.goto(`${baseUrl}/v2/fui-rs/demo/stage5/index.html`);
  await page.waitForFunction(() => window.__fuiReady === true);

  for (let attempt = 0; attempt < 16; attempt += 1) {
    const visibleHeight = await debugNodeVisibleHeight(page, 'Disabled field');
    if (visibleHeight > 0) {
      break;
    }
    await page.mouse.wheel(0, 500);
    await page.waitForTimeout(100);
  }

  const darkStyles = await page.evaluate(async () => {
    const debug = window.__fui_debug;
    if (debug === undefined || typeof debug.getDebugTree !== 'function') {
      return null;
    }
    const tree = await debug.getDebugTree();
    const statusCard = tree.nodes.find((entry) => entry.semanticLabel === 'Stage 5 text input status card');
    const disabledCard = tree.nodes.find((entry) => entry.semanticLabel === 'Stage 5 disabled themed card');
    const disabledText = tree.nodes.find((entry) => entry.semanticLabel === 'Disabled field');
    return {
      statusCardBg: statusCard?.style.bgColor ?? 0,
      disabledCardBg: disabledCard?.style.bgColor ?? 0,
      disabledTextColor: disabledText?.style.textColor ?? 0,
      disabledVisibleHeight: disabledText?.visibleBounds.height ?? 0,
    };
  });
  expect(darkStyles).not.toBeNull();
  expect(darkStyles?.statusCardBg).toBe(0x241437FF);
  expect(darkStyles?.disabledCardBg).toBe(0x341528FF);
  expect(darkStyles?.disabledTextColor).toBe(0x94A3B8FF);
  expect(darkStyles?.disabledVisibleHeight).toBeGreaterThan(0);
});

test('live system theme changes preserve retained control geometry', async ({ page }) => {
  await page.emulateMedia({ colorScheme: 'light' });
  await page.goto(`${baseUrl}/v2/fui-rs/demo/stage5/index.html`);
  await page.waitForFunction(() => window.__fuiReady === true);

  const readCard = async () => await page.evaluate(async () => {
    const debug = window.__fui_debug;
    if (debug === undefined || typeof debug.getDebugTree !== 'function') {
      return null;
    }
    const tree = await debug.getDebugTree();
    const card = tree.nodes.find((entry) => entry.semanticLabel === 'Stage 5 text input status card');
    if (card === undefined) {
      return null;
    }
    return {
      background: card.style.bgColor,
      width: card.bounds.width,
      height: card.bounds.height,
    };
  });

  const light = await readCard();
  expect(light).not.toBeNull();

  await page.emulateMedia({ colorScheme: 'dark' });
  await expect.poll(async () => (await readCard())?.background ?? 0).not.toBe(light?.background ?? 0);
  const dark = await readCard();
  expect(dark?.width).toBe(light?.width);
  expect(dark?.height).toBe(light?.height);

  await page.emulateMedia({ colorScheme: 'light' });
  await expect.poll(async () => (await readCard())?.background ?? 0).toBe(light?.background ?? 0);
  const restored = await readCard();
  expect(restored?.width).toBe(light?.width);
  expect(restored?.height).toBe(light?.height);
});

test('stage 5 modal dialog opens and cancels from keyboard', async ({ page }) => {
  await page.goto(`${baseUrl}/v2/fui-rs/demo/stage5/index.html`);
  await page.waitForFunction(() => window.__fuiReady === true);

  const sceneSurface = page.locator('[data-effindom-canvas-size-source]');
  for (let attempt = 0; attempt < 20; attempt += 1) {
    const visibleHeight = await debugNodeVisibleHeight(page, 'Open modal dialog');
    if (visibleHeight > 0) {
      break;
    }
    await page.mouse.wheel(0, 500);
    await page.waitForTimeout(100);
  }

  await clickLargestDebugNode(page, sceneSurface, 'Open modal dialog');
  await expect.poll(async () => {
    return await debugNodeVisibleHeightByNodeId(page, 'stage5-modal-title');
  }, { timeout: 10000 }).toBeGreaterThan(0);
  await expect.poll(async () => {
    return await page.evaluate(() => document.body.innerText.includes('Phase 5.7 status: modal shown'));
  }, { timeout: 10000 }).toBe(true);

  await page.keyboard.press('Escape');
  await expect.poll(async () => {
    return await page.evaluate(() => document.body.innerText.includes('Phase 5.7 status: modal cancelled'));
  }, { timeout: 10000 }).toBe(true);
});
