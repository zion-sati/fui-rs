import * as fs from 'node:fs';
import * as path from 'node:path';

import { expect,test,type Page } from '@playwright/test';

import {
buildEditableTextScene,
buildReadonlyTextScene,
gotoBridgePage,
parseGlyphRuns,
setupServer,
teardownServer,
} from './test-utils';

const WOFF_CJK_FIXTURE = fs.readFileSync(path.join(__dirname, 'fixtures', 'noto-sans-sc-cjk-subset.woff'));

test.beforeAll(async () => {
  await setupServer();
});

test.afterAll(async () => {
  await teardownServer();
});

async function writeEditableText(page: Page, handle: string, text: string): Promise<void> {
  await page.evaluate(({ handle: textHandle, text: nextText }) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const bridge = window.EffinDomBrowserBridge;
    if (runtime === null || runtime === undefined || bridge === undefined) {
      throw new Error('Bridge runtime is not ready.');
    }

    const ui = runtime.ui;
    const toPointer = (pointer: unknown): { ptr: number | bigint; offset: number } =>
      bridge.toHeapPointer(ui, pointer as string | number | bigint | { valueOf(): unknown; toString(): string });
    const bytes = new TextEncoder().encode(nextText);
    const heapText = bytes.length === 0 ? { ptr: ui.usesMemory64 === true ? 0n : 0, offset: 0, len: 0 } : (() => {
      const pointer = toPointer(ui._malloc(bytes.length));
      if (pointer.offset === 0) {
        throw new Error('ui malloc failed.');
      }
      ui.HEAPU8.set(bytes, pointer.offset);
      return { ptr: pointer.ptr, offset: pointer.offset, len: bytes.length };
    })();

    runtime.resetLogs();
    ui._ui_set_text_color(textHandle, 0xf8fafcff);
    try {
      ui._ui_set_text(textHandle, heapText.ptr, heapText.len);
    } finally {
      if (heapText.offset !== 0) {
        ui._free(heapText.ptr);
      }
    }
    runtime.commitFrame();
    runtime.flushPendingCommit();
  }, { handle, text });
}

async function readIncrementalState(page: Page) {
  return await page.evaluate(() => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    if (runtime === null || runtime === undefined) {
      throw new Error('Bridge runtime is not ready.');
    }
    return {
      logs: window.__bridgeLogs,
      bridgeError: window.__bridgeError ?? null,
      cacheState: runtime.getIncrementalFontCacheState(),
      policy: runtime.getIncrementalFontPolicy(),
      fontState: runtime.getIncrementalFontState(1),
      commandBuffer: Array.from(runtime.extractCommandBuffer()),
    };
  });
}

function expectGlyphRunsUseFallbackFont(glyphRuns: ReturnType<typeof parseGlyphRuns>, primaryFontId = 1): void {
  const glyphFontIds = glyphRuns.flatMap((run) => run.glyphFontIds);
  expect(glyphFontIds.length).toBeGreaterThan(0);
  expect(glyphFontIds.some((fontId) => fontId !== primaryFontId)).toBe(true);
}

test('missing Thai coverage dedupes one Google shard request and swaps both nodes', async ({ page }) => {
  await gotoBridgePage(page);

  await page.evaluate(() => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const bridge = window.EffinDomBrowserBridge;
    if (runtime === null || runtime === undefined || bridge === undefined) {
      throw new Error('Bridge runtime is not ready.');
    }

    const ui = runtime.ui;
    const toHandle = (handle: unknown): string =>
      bridge.handleToString(handle as string | number | bigint | { valueOf(): unknown; toString(): string });
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

    runtime.resetLogs();
    runtime.resetAppSession();
    ui._ui_reset();

    const root = toHandle(ui._ui_create_node(0));
    const first = toHandle(ui._ui_create_node(1));
    const second = toHandle(ui._ui_create_node(1));
    ui._ui_set_root(root);
    ui._ui_set_width(root, 320, 0);
    ui._ui_set_height(root, 220, 0);
    ui._ui_set_bg_color(root, 0x111827ff);
    ui._ui_node_add_child(root, first);
    ui._ui_node_add_child(root, second);

    for (const handle of [first, second]) {
      ui._ui_set_width(handle, 260, 0);
      ui._ui_set_height(handle, 60, 0);
      ui._ui_set_font(handle, 1, 24);
      ui._ui_set_text_color(handle, 0xf8fafcff);
      const heapText = writeText('ภาษาไทยภาษาไทย');
      try {
        ui._ui_set_text(handle, heapText.ptr, heapText.len);
      } finally {
        if (heapText.offset !== 0) {
          ui._free(heapText.ptr);
        }
      }
    }

    runtime.commitFrame();
    runtime.flushPendingCommit();
  });
  await expect.poll(async () => (await readIncrementalState(page)).fontState).toMatchObject({
    requestedSegmentIds: expect.arrayContaining([expect.stringMatching(/^thai-core:/)]),
    appliedSegmentIds: expect.arrayContaining([expect.stringMatching(/^thai-core:/)]),
    revision: 1,
  });

  const result = await readIncrementalState(page);
  const glyphRuns = parseGlyphRuns(result.commandBuffer);
  expect(result.bridgeError).toBeNull();

  expect(result.logs?.missingFontCoverageRequests).toEqual(expect.arrayContaining([
    { fontId: 1, coverageKind: 2, sampleText: 'ภาษาไทยภาษาไทย' },
  ]));
  expect(result.logs?.incrementalFontPackageRequests).toEqual(expect.arrayContaining([
    {
      primaryFontId: 1,
      coverageKind: 2,
      packageId: 'thai-sans',
      segmentIds: expect.arrayContaining([expect.stringMatching(/^thai-core:/)]),
      sampleText: expect.stringContaining('ภ'),
    },
  ]));
  expect(glyphRuns).toHaveLength(2);
  for (const run of glyphRuns) {
    expect(run.glyphCount).toBeGreaterThan(0);
  }
});

test('Chinese text swaps through a Google-hosted CJK shard with CJK punctuation', async ({ page }) => {
  await gotoBridgePage(page);

  const scene = await buildEditableTextScene(page, '');
  await writeEditableText(page, scene.textHandle, '你好，你好吗？');
  await expect.poll(async () => (await readIncrementalState(page)).fontState).toMatchObject({
    requestedSegmentIds: expect.arrayContaining([expect.stringMatching(/^cjk-sc:/)]),
    appliedSegmentIds: expect.arrayContaining([expect.stringMatching(/^cjk-sc:/)]),
    revision: 1,
  });

  const result = await readIncrementalState(page);
  const glyphRuns = parseGlyphRuns(result.commandBuffer);
  expect(result.bridgeError).toBeNull();

  expect(result.logs?.incrementalFontPackageRequests).toEqual(expect.arrayContaining([
    {
      primaryFontId: 1,
      coverageKind: 3,
      packageId: 'cjk-sans',
      segmentIds: expect.arrayContaining([expect.stringMatching(/^cjk-sc:/)]),
      sampleText: expect.stringContaining('你'),
    },
  ]));
  expect(result.logs?.incrementalFontPackageRequests).toEqual(expect.arrayContaining([
    expect.objectContaining({
      sampleText: expect.stringContaining('，'),
    }),
    expect.objectContaining({
      sampleText: expect.stringContaining('？'),
    }),
  ]));
  expect(glyphRuns).toHaveLength(1);
  expect(glyphRuns[0]?.glyphCount).toBeGreaterThan(0);
  expectGlyphRunsUseFallbackFont(glyphRuns);
});

test('Google-hosted WOFF CJK shards are normalized before UI registration', async ({ page }) => {
  await page.route('https://fonts.googleapis.com/css2?**', async (route) => {
    const requestUrl = new URL(route.request().url());
    if (
      requestUrl.searchParams.get('family') !== 'Noto Sans SC:wght@400'
      || requestUrl.searchParams.get('text') !== '我想睡觉'
    ) {
      await route.continue();
      return;
    }
    await route.fulfill({
      status: 200,
      contentType: 'text/css; charset=utf-8',
      body: `@font-face {
  font-family: 'Noto Sans SC';
  font-style: normal;
  font-weight: 400;
  font-display: swap;
  src: url(https://fonts.gstatic.com/effindom-test/noto-sans-sc-cjk-subset.woff) format('woff');
  unicode-range: U+60f3, U+6211, U+7761, U+89c9;
}`,
    });
  });
  await page.route('https://fonts.gstatic.com/effindom-test/noto-sans-sc-cjk-subset.woff', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'font/woff',
      body: WOFF_CJK_FIXTURE,
    });
  });
  await gotoBridgePage(page);

  const scene = await buildEditableTextScene(page, '');
  await writeEditableText(page, scene.textHandle, '我想睡觉');
  await expect.poll(async () => (await readIncrementalState(page)).fontState).toMatchObject({
    requestedSegmentIds: expect.arrayContaining([expect.stringMatching(/^cjk-sc:/)]),
    appliedSegmentIds: expect.arrayContaining([expect.stringMatching(/^cjk-sc:/)]),
    revision: 1,
  });

  const result = await readIncrementalState(page);
  const glyphRuns = parseGlyphRuns(result.commandBuffer);
  expect(result.bridgeError).toBeNull();
  expect(result.logs?.incrementalFontPackageRequests).toEqual(expect.arrayContaining([
    expect.objectContaining({
      primaryFontId: 1,
      coverageKind: 3,
      packageId: 'cjk-sans',
      sampleText: expect.stringContaining('我'),
    }),
  ]));
  expect(glyphRuns).toHaveLength(1);
  expect(glyphRuns[0]?.glyphCount).toBeGreaterThan(0);
  expectGlyphRunsUseFallbackFont(glyphRuns);
});

for (const testCase of [
  { label: 'Hebrew', text: 'שלום', segmentPrefix: 'hebrew:', sampleText: 'שלום' },
  { label: 'Devanagari', text: 'काला', segmentPrefix: 'devanagari:', sampleText: 'काला' },
] as const) {
  test(`${testCase.label} text swaps through the supplemental Google shard path`, async ({ page }) => {
    await gotoBridgePage(page);

    const scene = await buildEditableTextScene(page, '');
    await writeEditableText(page, scene.textHandle, testCase.text);
    await expect.poll(async () => (await readIncrementalState(page)).fontState).toMatchObject({
      requestedSegmentIds: expect.arrayContaining([expect.stringMatching(new RegExp(`^${testCase.segmentPrefix}`))]),
      appliedSegmentIds: expect.arrayContaining([expect.stringMatching(new RegExp(`^${testCase.segmentPrefix}`))]),
      revision: 1,
    });

    const result = await readIncrementalState(page);
    const glyphRuns = parseGlyphRuns(result.commandBuffer);
    expect(result.bridgeError).toBeNull();

    expect(result.logs?.incrementalFontPackageRequests).toEqual(expect.arrayContaining([
      {
        primaryFontId: 1,
        coverageKind: 4,
        packageId: 'supplemental-sans',
        segmentIds: expect.arrayContaining([expect.stringMatching(new RegExp(`^${testCase.segmentPrefix}`))]),
        sampleText: expect.stringContaining(testCase.sampleText[0] ?? ''),
      },
    ]));
    expect(glyphRuns).toHaveLength(1);
    expect(glyphRuns[0]?.glyphCount).toBeGreaterThan(0);
  });
}

test('incremental font policy can block package auto-grow and later allow it again', async ({ page }) => {
  await gotoBridgePage(page);

  await page.evaluate(() => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    if (runtime === null || runtime === undefined) {
      throw new Error('Bridge runtime is not ready.');
    }
    runtime.setIncrementalFontPolicy({
      blockedPackageIds: ['cjk-sans'],
    });
  });

  const blockedScene = await buildEditableTextScene(page, '');
  await writeEditableText(page, blockedScene.textHandle, '你好');
  await expect.poll(async () => (await readIncrementalState(page)).fontState).toMatchObject({
    autoGrowAllowed: true,
    blockedPackageIds: expect.arrayContaining(['cjk-sans']),
    lastBlockedReason: 'package-blocked',
    appliedSegmentIds: [],
  });

  await page.evaluate(() => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    if (runtime === null || runtime === undefined) {
      throw new Error('Bridge runtime is not ready.');
    }
    runtime.setIncrementalFontPolicy({
      blockedPackageIds: null,
    });
  });

  const scene = await buildEditableTextScene(page, '');
  await writeEditableText(page, scene.textHandle, '你好');
  await expect.poll(async () => (await readIncrementalState(page)).fontState).toMatchObject({
    appliedSegmentIds: expect.arrayContaining([expect.stringMatching(/^cjk-sc:/)]),
    lastBlockedReason: null,
  });
});

test('incremental font cache evicts least-recently-used shards when capped', async ({ page }) => {
  await gotoBridgePage(page);

  await page.evaluate(() => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    if (runtime === null || runtime === undefined) {
      throw new Error('Bridge runtime is not ready.');
    }
    runtime.setIncrementalFontPolicy({
      maxCachedShardFonts: 1,
      blockedPackageIds: null,
    });
  });

  const thaiScene = await buildEditableTextScene(page, '');
  await writeEditableText(page, thaiScene.textHandle, 'ภาษาไทยภาษาไทย');
  await expect.poll(async () => (await readIncrementalState(page)).fontState).toMatchObject({
    appliedSegmentIds: expect.arrayContaining([expect.stringMatching(/^thai-core:/)]),
    revision: 1,
  });

  const cjkScene = await buildEditableTextScene(page, '');
  await writeEditableText(page, cjkScene.textHandle, '你好');
  await expect.poll(async () => (await readIncrementalState(page)).fontState).toMatchObject({
    appliedSegmentIds: expect.arrayContaining([expect.stringMatching(/^cjk-sc:/)]),
    evictedSegmentIds: expect.arrayContaining([expect.stringMatching(/^thai-core:/)]),
    revision: 3,
  });

  const result = await readIncrementalState(page);
  expect(result.cacheState).toMatchObject({
    maxCachedShardFonts: 1,
    cachedShardCount: 1,
    evictedShardKeys: expect.arrayContaining([expect.stringMatching(/^thai-core:/)]),
  });
  expect(result.cacheState.cachedShardKeys).toEqual(expect.arrayContaining([expect.stringMatching(/^cjk-sc:/)]));
});

test('tofu swap preserves editable text and readonly select/find flows after shard resolution', async ({ page }) => {
  await gotoBridgePage(page);

  const editableScene = await buildEditableTextScene(page, '');
  await writeEditableText(page, editableScene.textHandle, '你好，你好吗？');
  await expect.poll(async () => (await readIncrementalState(page)).fontState).toMatchObject({
    appliedSegmentIds: expect.arrayContaining([expect.stringMatching(/^cjk-sc:/)]),
  });

  const editableResult = await page.evaluate(({ handle }) => ({
    text: window.__bridgeTextByHandle?.[handle] ?? null,
  }), { handle: editableScene.textHandle });
  expect(editableResult.text).toBe('你好，你好吗？');
  expectGlyphRunsUseFallbackFont(parseGlyphRuns((await readIncrementalState(page)).commandBuffer));

  const readonlyScene = await buildReadonlyTextScene(page, '你好，你好吗？');
  const readonlyResult = await page.evaluate(({ handle }) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    if (runtime === null || runtime === undefined) {
      throw new Error('Bridge runtime is not ready.');
    }
    runtime.resetLogs();
    runtime.ui._ui_set_text_selection_range(handle, 0, 3);
    runtime.commitFrame();
    runtime.flushPendingCommit();
    return {
      docs: runtime.getFindDocuments(),
      logs: window.__bridgeLogs,
    };
  }, { handle: readonlyScene.textHandle });

  expect(readonlyResult.docs).toEqual(expect.arrayContaining([
    expect.objectContaining({
      text: expect.stringContaining('你好，你好吗？'),
    }),
  ]));
  expect(readonlyResult.logs?.selectionChanges).toEqual(expect.arrayContaining([
    expect.objectContaining({
      handle: readonlyScene.textHandle,
      start: 0,
      end: 3,
    }),
  ]));
});
