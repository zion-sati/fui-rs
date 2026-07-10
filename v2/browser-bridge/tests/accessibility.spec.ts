import { expect,test } from '@playwright/test';

import {
buildClippedSemanticScene,
buildEditableTextScene,
buildMultiStaticTextScene,
buildScrollableSemanticOrderScene,
buildScrollableStaticTextScene,
buildSemanticScene,
buildStaticTextScene,
gotoBridgePage,
parseColoredHighlightRects,
parseHighlightRects,
setupServer,
teardownServer
} from './test-utils';

test.beforeAll(async () => {
  await setupServer();
});

test.afterAll(async () => {
  await teardownServer();
});

test('hidden semantic DOM projects accessible nodes with logical bounds', async ({ page }) => {
  await gotoBridgePage(page);
  await buildSemanticScene(page);

  const projected = await page.evaluate(() => {
    const layer = document.getElementById('semantic-layer');
    const shadow = layer?.shadowRoot;
    const content = shadow?.getElementById('semantic-content');
    if (!(layer instanceof HTMLElement) || !(shadow instanceof ShadowRoot) || !(content instanceof HTMLElement)) {
      throw new Error('Expected semantic layer.');
    }
    const button = shadow.querySelector('[role="button"]');
    const textbox = shadow.querySelector('[role="textbox"][aria-label="Email"]');
    const image = shadow.querySelector('[role="img"][aria-label="Preview"]');
    if (!(button instanceof HTMLElement) || !(textbox instanceof HTMLElement) || !(image instanceof HTMLElement)) {
      throw new Error('Expected projected semantic elements.');
    }
    return {
      buttonLabel: button.getAttribute('aria-label'),
      buttonText: button.textContent,
      buttonLeft: button.style.left,
      buttonTop: button.style.top,
      textboxRole: textbox.getAttribute('role'),
      textboxTag: textbox.tagName,
      textboxValue:
        textbox instanceof HTMLInputElement || textbox instanceof HTMLTextAreaElement
          ? textbox.value
          : textbox.textContent,
      textboxResize: getComputedStyle(textbox).resize,
      imageTag: image.tagName,
      imageText: image.textContent,
      layerWidth: layer.style.width,
      layerHeight: layer.style.height,
      hasShadowRoot: layer.shadowRoot instanceof ShadowRoot,
      contentWidth: content.style.width,
      contentHeight: content.style.height,
      contentWhiteSpace: content.style.whiteSpace,
      hiddenInputAriaHidden:
        document.querySelector('input[data-effindom-hidden-editor="true"]')?.getAttribute('aria-hidden') ?? null,
      hiddenTextareaAriaHidden:
        document.querySelector('textarea[data-effindom-hidden-editor="true"]')?.getAttribute('aria-hidden') ?? null,
    };
  });

  expect(projected.buttonLabel).toBe('Submit');
  expect(projected.buttonText).toBe('');
  expect(projected.buttonLeft).toBe('0px');
  expect(projected.buttonTop).toBe('0px');
  expect(projected.textboxRole).toBe('textbox');
  expect(projected.textboxTag).toBe('TEXTAREA');
  expect(projected.textboxValue).toBe('Email');
  expect(projected.textboxResize).toBe('none');
  expect(projected.imageTag).toBe('DIV');
  expect(projected.imageText).toBe('');
  expect(projected.layerWidth).toBe('320px');
  expect(projected.layerHeight).toBe('220px');
  expect(projected.hasShadowRoot).toBe(true);
  expect(projected.contentWidth).toBe('100%');
  expect(projected.contentHeight).toBe('100%');
  expect(projected.contentWhiteSpace).toBe('nowrap');
  expect(projected.hiddenInputAriaHidden).toBe('true');
  expect(projected.hiddenTextareaAriaHidden).toBe('true');
});


test('hidden semantic DOM clips projected bounds and omits fully hidden nodes', async ({ page }) => {
  await gotoBridgePage(page);
  await buildClippedSemanticScene(page);

  const projected = await page.evaluate(() => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const layer = document.getElementById('semantic-layer');
    const shadow = layer?.shadowRoot;
    if (runtime === null || runtime === undefined || !(layer instanceof HTMLElement) || !(shadow instanceof ShadowRoot)) {
      throw new Error('Expected semantic layer and runtime.');
    }
    const buttons = Array.from(shadow.querySelectorAll('[role="button"]'));
    const clipped = buttons.find((element) => element.getAttribute('aria-label') === 'Clipped');
    const labels = runtime.getSemanticTree().map((node) => node.label);
    if (!(clipped instanceof HTMLElement)) {
      throw new Error('Expected clipped semantic button.');
    }
    return {
      buttonCount: buttons.length,
      labels,
      clippedTop: clipped.style.top,
      clippedHeight: clipped.style.height,
    };
  });

  expect(projected.buttonCount).toBe(1);
  expect(projected.labels).toEqual(['Clipped']);
  expect(projected.clippedTop).toBe('28px');
  expect(projected.clippedHeight).toBe('12px');
});

test('hidden semantic DOM keeps semantic order stable when scrollview items re-enter above', async ({ page }) => {
  await gotoBridgePage(page);
  const { scrollHandle } = await buildScrollableSemanticOrderScene(page);

  const projected = await page.evaluate((handle) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    if (runtime === null || runtime === undefined) {
      throw new Error('Expected bridge runtime.');
    }

    const ui = runtime.ui;
    const bridge = window.EffinDomBrowserBridge;
    if (bridge === undefined) {
      throw new Error('Bridge state is not ready.');
    }
    const scrollHandleValue = bridge.handleToBigInt(handle);
    const readOrder = (): string[] =>
      Array.from(document.getElementById('semantic-layer')?.shadowRoot?.querySelectorAll('#semantic-content > [data-handle]') ?? [])
        .map((element) => {
          const htmlElement = element as HTMLElement;
          const text = htmlElement.textContent;
          return text.length > 0 ? text : (htmlElement.getAttribute('aria-label') ?? '');
        });

    ui._ui_set_scroll_offset(scrollHandleValue, 0, 20);
    runtime.commitFrame();
    runtime.flushPendingCommit();
    const scrolledSemanticOrder = runtime.getSemanticTree().map((node) => node.label);
    const scrolledProjectedOrder = readOrder();

    ui._ui_set_scroll_offset(scrollHandleValue, 0, 0);
    runtime.commitFrame();
    runtime.flushPendingCommit();
    const resetSemanticOrder = runtime.getSemanticTree().map((node) => node.label);
    const resetProjectedOrder = readOrder();

    return {
      scrolledSemanticOrder,
      scrolledProjectedOrder,
      resetSemanticOrder,
      resetProjectedOrder,
    };
  }, scrollHandle);

  expect(projected.scrolledSemanticOrder).toEqual(['Second', 'Third']);
  expect(projected.scrolledProjectedOrder).toEqual(['Second', 'Third']);
  expect(projected.resetSemanticOrder).toEqual(['First', 'Second']);
  expect(projected.resetProjectedOrder).toEqual(['First', 'Second']);
});

test('static text projects as paragraph with a StaticText child', async ({ page }) => {
  await gotoBridgePage(page);
  const sample = 'Semantic paragraph sample';
  await buildStaticTextScene(page, sample);

  const projection = await page.evaluate(() => {
    const layer = document.getElementById('semantic-layer');
    const shadow = layer?.shadowRoot;
    const paragraph = shadow?.querySelector('[data-role="text"]');
    if (!(paragraph instanceof HTMLElement)) {
      throw new Error('Expected projected static text paragraph.');
    }
    const textRun = paragraph.querySelector('[data-semantic-text-run="true"]');
    const textContent = textRun?.querySelector('[data-semantic-text-content="true"]');
    if (!(textRun instanceof HTMLSpanElement) || !(textContent instanceof HTMLSpanElement)) {
      throw new Error('Expected projected text run.');
    }
    return {
      childElementCount: paragraph.childElementCount,
      textRunChildElementCount: textRun.childElementCount,
      textRunFirstChildType: textRun.firstChild?.nodeType ?? -1,
      textContentChildNodeCount: textContent.childNodes.length,
      textContentFirstChildType: textContent.firstChild?.nodeType ?? -1,
      textContent: paragraph.textContent,
    };
  });

  const client = await page.context().newCDPSession(page);
  const axTree = await client.send('Accessibility.getFullAXTree');
  const nodes = axTree.nodes as {
    nodeId: string;
    role?: { value?: string };
    name?: { value?: string };
    childIds?: string[];
  }[];
  const staticTextNodes = nodes.filter((node) => node.role?.value === 'StaticText' && node.name?.value === sample);
  const paragraphNode = nodes.find((node) =>
    node.role?.value === 'paragraph' &&
    staticTextNodes.some((candidate) => (node.childIds ?? []).includes(candidate.nodeId)),
  );

  expect(projection.childElementCount).toBe(1);
  expect(projection.textRunChildElementCount).toBe(1);
  expect(projection.textRunFirstChildType).toBe(1);
  expect(projection.textContentChildNodeCount).toBe(1);
  expect(projection.textContentFirstChildType).toBe(3);
  expect(projection.textContent).toBe(sample);
  expect(staticTextNodes.length).toBeGreaterThan(0);
  expect(paragraphNode?.name?.value ?? '').toBe('');
});

test('static text text-run geometry matches Ui visible text bounds', async ({ page }) => {
  await gotoBridgePage(page);
  const sample = 'Hot-swap notes';
  const scene = await buildStaticTextScene(page, sample);

  const geometry = await page.evaluate((textHandle) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const bridge = window.EffinDomBrowserBridge;
    const layer = document.getElementById('semantic-layer');
    const shadow = layer?.shadowRoot;
    const paragraph = shadow?.querySelector('[data-role="text"]');
    const textRun = paragraph?.querySelector('[data-semantic-text-run="true"]');
    const textContent = textRun?.querySelector('[data-semantic-text-content="true"]');
    if (runtime === null || runtime === undefined || bridge === undefined) {
      throw new Error('Expected bridge runtime.');
    }
    if (!(layer instanceof HTMLElement) || !(paragraph instanceof HTMLElement) || !(textContent instanceof HTMLSpanElement) || !(textContent.firstChild instanceof Text)) {
      throw new Error('Expected projected text run.');
    }
    const semanticNode = runtime.getSemanticTree().find((node) => node.handle === textHandle);
    if (semanticNode === undefined) {
      throw new Error('Expected semantic node.');
    }
    const ui = runtime.ui;
    const allocation = bridge.toHeapPointer(ui, ui._malloc(16));
    if (allocation.offset === 0) {
      throw new Error('Expected visible bounds allocation.');
    }
    const addPointerOffset = (pointer: number | bigint, offset: number): number | bigint =>
      typeof pointer === 'bigint' ? pointer + BigInt(offset) : pointer + offset;
    let target;
    try {
      const copied = ui._ui_get_text_visible_bounds(
        bridge.handleToBigInt(textHandle),
        allocation.ptr,
        addPointerOffset(allocation.ptr, 4),
        addPointerOffset(allocation.ptr, 8),
        addPointerOffset(allocation.ptr, 12),
      );
      ui.refreshHeapViews?.();
      if (copied === 0) {
        throw new Error('Expected visible text bounds.');
      }
      const base = allocation.offset >>> 2;
      target = {
        x: ui.HEAPF32[base] ?? 0,
        y: ui.HEAPF32[base + 1] ?? 0,
        width: ui.HEAPF32[base + 2] ?? 0,
        height: ui.HEAPF32[base + 3] ?? 0,
      };
    } finally {
      ui._free(allocation.ptr);
    }
    const range = document.createRange();
    range.selectNodeContents(textContent.firstChild);
    const measured = range.getBoundingClientRect();
    const layerRect = layer.getBoundingClientRect();
    return {
      semanticBounds: semanticNode.bounds,
      measured: {
        x: measured.x - layerRect.x,
        y: measured.y - layerRect.y,
        width: measured.width,
        height: measured.height,
      },
      target,
    };
  }, scene.textHandle);

  expect(Math.abs(geometry.measured.x - geometry.target.x)).toBeLessThanOrEqual(1);
  expect(Math.abs(geometry.measured.y - geometry.target.y)).toBeLessThanOrEqual(1);
  expect(Math.abs(geometry.measured.width - geometry.target.width)).toBeLessThanOrEqual(1);
  expect(Math.abs(geometry.measured.height - geometry.target.height)).toBeLessThanOrEqual(3);
  expect(geometry.measured.width).toBeLessThanOrEqual(geometry.semanticBounds.width);
  expect(geometry.measured.height).toBeLessThanOrEqual(geometry.semanticBounds.height);
});

test('static text geometry still matches visible bounds for custom fonts', async ({ page }) => {
  await gotoBridgePage(page);
  await page.evaluate(async () => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    if (runtime === null || runtime === undefined) {
      throw new Error('Expected bridge runtime.');
    }
    await runtime.registerFont({
      id: 101,
      url: '/v2/fonts/DejaVuSans-Bold.ttf',
      fallbackIds: [4, 3],
    });
  });
  const sample = 'Custom n🙂 width';
  const scene = await buildStaticTextScene(page, sample, 101);

  const geometry = await page.evaluate((textHandle) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const layer = document.getElementById('semantic-layer');
    const shadow = layer?.shadowRoot;
    const paragraph = shadow?.querySelector('[data-role="text"]');
    const textRun = paragraph?.querySelector('[data-semantic-text-run="true"]');
    const textContent = textRun?.querySelector('[data-semantic-text-content="true"]');
    const bridge = window.EffinDomBrowserBridge;
    if (runtime === null || runtime === undefined) {
      throw new Error('Expected bridge runtime.');
    }
    if (bridge === undefined) {
      throw new Error('Expected bridge state.');
    }
    if (!(layer instanceof HTMLElement) || !(paragraph instanceof HTMLElement) || !(textContent instanceof HTMLSpanElement) || !(textContent.firstChild instanceof Text)) {
      throw new Error('Expected projected text run.');
    }
    const ui = runtime.ui;
    const allocation = bridge.toHeapPointer(ui, ui._malloc(16));
    if (allocation.offset === 0) {
      throw new Error('Expected visible bounds allocation.');
    }
    const addPointerOffset = (pointer: number | bigint, offset: number): number | bigint =>
      typeof pointer === 'bigint' ? pointer + BigInt(offset) : pointer + offset;
    let target;
    try {
      const copied = ui._ui_get_text_visible_bounds(
        bridge.handleToBigInt(textHandle),
        allocation.ptr,
        addPointerOffset(allocation.ptr, 4),
        addPointerOffset(allocation.ptr, 8),
        addPointerOffset(allocation.ptr, 12),
      );
      ui.refreshHeapViews?.();
      if (copied === 0) {
        throw new Error('Expected visible text bounds.');
      }
      const base = allocation.offset >>> 2;
      target = {
        x: ui.HEAPF32[base] ?? 0,
        y: ui.HEAPF32[base + 1] ?? 0,
        width: ui.HEAPF32[base + 2] ?? 0,
        height: ui.HEAPF32[base + 3] ?? 0,
      };
    } finally {
      ui._free(allocation.ptr);
    }
    const range = document.createRange();
    range.selectNodeContents(textContent.firstChild);
    const measured = range.getBoundingClientRect();
    const layerRect = layer.getBoundingClientRect();
    return {
      measured: {
        x: measured.x - layerRect.x,
        y: measured.y - layerRect.y,
        width: measured.width,
        height: measured.height,
      },
      target,
    };
  }, scene.textHandle);

  expect(Math.abs(geometry.measured.x - geometry.target.x)).toBeLessThanOrEqual(1);
  expect(Math.abs(geometry.measured.y - geometry.target.y)).toBeLessThanOrEqual(1);
  expect(Math.abs(geometry.measured.width - geometry.target.width)).toBeLessThanOrEqual(2);
  expect(Math.abs(geometry.measured.height - geometry.target.height)).toBeLessThanOrEqual(3);
});

test('static text skips text-run refitting when only scroll position changes', async ({ page }) => {
  await gotoBridgePage(page);
  const sample = 'Semantic scroll performance cache';
  const scene = await buildScrollableStaticTextScene(page, sample);

  await page.evaluate((scrollHandle) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const bridge = window.EffinDomBrowserBridge;
    if (runtime === null || runtime === undefined || bridge === undefined) {
      throw new Error('Expected bridge runtime.');
    }
    runtime.ui._ui_set_scroll_offset(bridge.handleToBigInt(scrollHandle), 0, 16);
    runtime.commitFrame();
    runtime.flushPendingCommit();
  }, scene.scrollHandle);

  const createRangeCalls = await page.evaluate((scrollHandle) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const bridge = window.EffinDomBrowserBridge;
    if (runtime === null || runtime === undefined || bridge === undefined) {
      throw new Error('Expected bridge runtime.');
    }
    let createRangeCalls = 0;
    const originalCreateRange = document.createRange.bind(document);
    document.createRange = () => {
      createRangeCalls += 1;
      return originalCreateRange();
    };
    try {
      runtime.ui._ui_set_scroll_offset(bridge.handleToBigInt(scrollHandle), 0, 32);
      runtime.commitFrame();
      runtime.flushPendingCommit();
      runtime.ui._ui_set_scroll_offset(bridge.handleToBigInt(scrollHandle), 0, 48);
      runtime.commitFrame();
      runtime.flushPendingCommit();
      return createRangeCalls;
    } finally {
      document.createRange = originalCreateRange;
    }
  }, scene.scrollHandle);

  expect(createRangeCalls).toBe(0);
});

test('semantic projection keeps child nodes stable across focus commits', async ({ page }) => {
  await gotoBridgePage(page);
  const scene = await buildSemanticScene(page);

  const mutationCount = await page.evaluate(async (textboxHandle) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const layer = document.getElementById('semantic-layer');
    const shadow = layer?.shadowRoot;
    const content = shadow?.getElementById('semantic-content');
    if (runtime === null || runtime === undefined || !(content instanceof HTMLElement)) {
      throw new Error('Expected bridge runtime and semantic content.');
    }
    const bridge = window.EffinDomBrowserBridge;
    if (bridge === undefined) {
      throw new Error('Bridge state is not ready.');
    }
    let mutationCount = 0;
    const observer = new MutationObserver((records) => {
      for (const record of records) {
        mutationCount += record.addedNodes.length;
        mutationCount += record.removedNodes.length;
      }
    });
    observer.observe(content, { childList: true });
    runtime.ui._ui_request_focus(bridge.handleToBigInt(textboxHandle));
    runtime.commitFrame();
    runtime.flushPendingCommit();
    await new Promise((resolve) => setTimeout(resolve, 0));
    observer.disconnect();
    return mutationCount;
  }, scene.textboxHandle);

  expect(mutationCount).toBe(0);
});

test('semantic projection keeps projected nodes inside the shadow root host', async ({ page }) => {
  await gotoBridgePage(page);
  await buildSemanticScene(page);

  const projectionState = await page.evaluate(() => {
    const layer = document.getElementById('semantic-layer');
    const shadow = layer?.shadowRoot;
    return {
      hostProjectedCount: layer?.querySelectorAll('[data-handle]').length ?? 0,
      shadowProjectedCount: shadow?.querySelectorAll('[data-handle]').length ?? 0,
    };
  });

  expect(projectionState.hostProjectedCount).toBe(0);
  expect(projectionState.shadowProjectedCount).toBeGreaterThan(0);
});


test('semantic layer tracks canvas logical size after resize', async ({ page }) => {
  await gotoBridgePage(page);
  await buildSemanticScene(page);

  const sizes = await page.evaluate(() => {
    const canvas = document.getElementById('fui-canvas');
    const layer = document.getElementById('semantic-layer');
    if (!(canvas instanceof HTMLCanvasElement) || !(layer instanceof HTMLElement)) {
      throw new Error('Expected canvas and semantic layer.');
    }
    canvas.style.width = '400px';
    canvas.style.height = '260px';
    window.dispatchEvent(new Event('resize'));
    return {
      canvasWidth: canvas.style.width,
      canvasHeight: canvas.style.height,
      layerWidth: layer.style.width,
      layerHeight: layer.style.height,
    };
  });

  expect(sizes.canvasWidth).toBe('400px');
  expect(sizes.canvasHeight).toBe('260px');
  expect(sizes.layerWidth).toBe('400px');
  expect(sizes.layerHeight).toBe('260px');
});


test('__OPEN_CANVAS_API__ exposes the cached semantic tree and bounding boxes', async ({ page }) => {
  await gotoBridgePage(page);
  const scene = await buildSemanticScene(page);

  const apiState = await page.evaluate((buttonHandle) => {
    const api = window.__OPEN_CANVAS_API__;
    if (api === undefined) {
      throw new Error('Expected __OPEN_CANVAS_API__.');
    }
    return {
      tree: api.getSemanticTree(),
      bounds: api.getBoundingBox(buttonHandle),
    };
  }, scene.buttonHandle);

  expect(apiState.tree.some((node) => node.roleName === 'button' && node.label === 'Submit')).toBe(true);
  expect(apiState.tree.some((node) => node.roleName === 'textbox' && node.label === 'Email')).toBe(true);
  expect(apiState.bounds).not.toBeNull();
  expect(apiState.bounds?.x).toBe(0);
  expect(apiState.bounds?.y).toBe(0);
  expect(apiState.bounds?.width).toBe(100);
});

test('__OPEN_CANVAS_API__ exposes realized Text documents, visible bounds, range rects, and find match writes', async ({ page }) => {
  await gotoBridgePage(page);
  const sample = 'Find bridges reveal ranges cleanly.';
  const scene = await buildStaticTextScene(page, sample);

  const apiState = await page.evaluate((textHandle) => {
    const api = window.__OPEN_CANVAS_API__;
    if (api === undefined) {
      throw new Error('Expected __OPEN_CANVAS_API__.');
    }
    const document = api.getTextDocument(textHandle);
    if (document === null) {
      throw new Error('Expected a realized Text document.');
    }
    const start = document.text.indexOf('bridges');
    const end = start + 'bridges'.length;
    return {
      document,
      visibleBounds: api.getTextVisibleBounds(textHandle),
      rects: api.getRangeRects(textHandle, start, end),
      setMatch: api.setFindMatch({ handle: textHandle, start, end }),
      clearMatch: api.setFindMatch(null),
    };
  }, scene.textHandle);

  expect(apiState.document).toEqual({
    handle: scene.textHandle,
    text: sample,
  });
  expect(apiState.visibleBounds).not.toBeNull();
  expect(apiState.visibleBounds?.width ?? 0).toBeGreaterThan(0);
  expect(apiState.visibleBounds?.height ?? 0).toBeGreaterThan(0);
  expect(apiState.rects.length).toBeGreaterThan(0);
  expect(apiState.rects[0]?.width ?? 0).toBeGreaterThan(0);
  expect(apiState.rects[0]?.height ?? 0).toBeGreaterThan(0);
  expect(apiState.setMatch).toBe(true);
  expect(apiState.clearMatch).toBe(true);
});

test('__OPEN_CANVAS_API__ expands ligature-backed find highlights to the full matched cluster coverage', async ({ page }) => {
  await gotoBridgePage(page);
  const sample = 'EffinDom';
  const scene = await buildStaticTextScene(page, sample);

  const apiState = await page.evaluate((textHandle) => {
    const api = window.__OPEN_CANVAS_API__;
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    if (api === undefined || runtime === undefined || runtime === null) {
      throw new Error('Expected bridge runtime and __OPEN_CANVAS_API__.');
    }
    const eff = api.findText('Eff');
    const e = api.findText('E');
    const effMatch = eff.matches[0];
    const eMatch = e.matches[0];
    if (effMatch === undefined || eMatch === undefined) {
      throw new Error('Expected Eff and E matches.');
    }
    const effRects = api.getRangeRects(textHandle, effMatch.start, effMatch.end);
    const eRects = api.getRangeRects(textHandle, eMatch.start, eMatch.end);
    if (!api.setFindState({
      query: eff.query,
      options: eff.options,
      matches: eff.matches,
      activeMatchIndex: 0,
    }, false)) {
      throw new Error('Expected setFindState to succeed for Eff match.');
    }
    return {
      effRects,
      eRects,
      words: Array.from(runtime.extractCommandBuffer()),
    };
  }, scene.textHandle);

  const highlightRects = parseHighlightRects(apiState.words, scene.textHandle);
  expect(apiState.effRects.length).toBeGreaterThan(0);
  expect(apiState.eRects.length).toBeGreaterThan(0);
  expect(highlightRects.length).toBeGreaterThan(0);
  expect(apiState.effRects[0]?.width ?? 0).toBeGreaterThan(apiState.eRects[0]?.width ?? 0);
  expect(highlightRects[0]?.width ?? 0).toBeGreaterThan(apiState.eRects[0]?.width ?? 0);
});

test('__OPEN_CANVAS_API__.findText applies intentful options and highlight-all retained state', async ({ page }) => {
  await gotoBridgePage(page);
  const sample = 'Cafe café CAFE bridge bridged bridge';
  const scene = await buildStaticTextScene(page, sample);

  const apiState = await page.evaluate(() => {
    const api = window.__OPEN_CANVAS_API__;
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    if (api === undefined || runtime === undefined || runtime === null) {
      throw new Error('Expected bridge runtime and __OPEN_CANVAS_API__.');
    }
    const defaultResults = api.findText('cafe');
    const caseSensitiveResults = api.findText('CAFE', { matchCase: true, matchDiacritics: true });
    const diacriticSensitiveResults = api.findText('cafe', { matchDiacritics: true });
    const wholeWordResults = api.findText('bridge', { wholeWords: true });
    const highlightResults = api.findText('bridge', { highlightAll: true });
    const applied = api.setFindState({
      query: highlightResults.query,
      options: highlightResults.options,
      matches: highlightResults.matches,
      activeMatchIndex: 0,
    }, false);
    return {
      defaultCount: defaultResults.matches.length,
      caseSensitiveCount: caseSensitiveResults.matches.length,
      diacriticSensitiveCount: diacriticSensitiveResults.matches.length,
      wholeWordCount: wholeWordResults.matches.length,
      applied,
      activeState: api.getFindState(),
      words: Array.from(runtime.extractCommandBuffer()),
      bridgeState: window.__bridgeFindState,
    };
  });

  expect(apiState.defaultCount).toBe(3);
  expect(apiState.caseSensitiveCount).toBe(1);
  expect(apiState.diacriticSensitiveCount).toBe(2);
  expect(apiState.wholeWordCount).toBe(2);
  expect(apiState.applied).toBe(true);
  expect(apiState.activeState).not.toBeNull();
  expect(apiState.activeState?.options.highlightAll).toBe(true);
  expect(apiState.activeState?.matches).toHaveLength(3);
  expect(apiState.bridgeState?.matches).toHaveLength(3);

  const highlightRects = parseColoredHighlightRects(apiState.words, scene.textHandle);
  expect(highlightRects.length).toBeGreaterThanOrEqual(3);
  expect(highlightRects.some((rect) => rect.color === 0xffeb3b80)).toBe(true);
  expect(highlightRects.some((rect) => rect.color === 0xffeb3b38)).toBe(true);
});

test('__OPEN_CANVAS_API__.setFindMatch emits retained highlight commands', async ({ page }) => {
  await gotoBridgePage(page);
  const sample = 'Find bridges reveal ranges cleanly.';
  const scene = await buildStaticTextScene(page, sample);

  const words = await page.evaluate((textHandle) => {
    const api = window.__OPEN_CANVAS_API__;
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    if (api === undefined || runtime === null || runtime === undefined) {
      throw new Error('Expected bridge runtime and __OPEN_CANVAS_API__.');
    }
    const document = api.getTextDocument(textHandle);
    if (document === null) {
      throw new Error('Expected a realized Text document.');
    }
    const start = document.text.indexOf('bridges');
    const end = start + 'bridges'.length;
    if (!api.setFindMatch({ handle: textHandle, start, end })) {
      throw new Error('Expected setFindMatch to succeed.');
    }
    return Array.from(runtime.extractCommandBuffer());
  }, scene.textHandle);

  const highlightRects = parseHighlightRects(words, scene.textHandle);
  expect(highlightRects.length).toBeGreaterThan(0);
  expect(highlightRects[0]?.width ?? 0).toBeGreaterThan(0);
  expect(highlightRects[0]?.height ?? 0).toBeGreaterThan(0);
});

test('__OPEN_CANVAS_API__ keeps editable text out of the Find text-document contract', async ({ page }) => {
  await gotoBridgePage(page);
  const scene = await buildEditableTextScene(page, 'Editable text should stay out of Find.');

  const apiState = await page.evaluate((textHandle) => {
    const api = window.__OPEN_CANVAS_API__;
    if (api === undefined) {
      throw new Error('Expected __OPEN_CANVAS_API__.');
    }
    return {
      document: api.getTextDocument(textHandle),
      rects: api.getRangeRects(textHandle, 0, 8),
      setMatch: api.setFindMatch({ handle: textHandle, start: 0, end: 8 }),
      reveal: api.revealRange(textHandle, 0, 8),
    };
  }, scene.textHandle);

  expect(apiState.document).toBeNull();
  expect(apiState.rects).toEqual([]);
  expect(apiState.setMatch).toBe(false);
  expect(apiState.reveal).toBe(false);
});

test('find-on-page mirror projects realized Text with canvas-scoped metadata', async ({ page }) => {
  await gotoBridgePage(page);
  const sample = 'Find mirror metadata stays canvas-scoped.';
  const scene = await buildStaticTextScene(page, sample);

  const mirrorState = await page.evaluate(() => {
    const root = document.querySelector('[data-ed-find-root="1"]');
    const fragment = document.querySelector('[data-ed-find-fragment="1"]');
    if (!(root instanceof HTMLElement) || !(fragment instanceof HTMLElement)) {
      throw new Error('Expected hidden Find-on-Page mirror elements.');
    }
    return {
      rootCanvasId: root.dataset.edCanvasId ?? null,
      rootAriaHidden: root.getAttribute('aria-hidden'),
      fragmentCanvasId: fragment.dataset.edCanvasId ?? null,
      fragmentHandle: fragment.dataset.edHandle ?? null,
      fragmentStart: fragment.dataset.edStart ?? null,
      fragmentEnd: fragment.dataset.edEnd ?? null,
      fragmentText: fragment.textContent,
    };
  }, scene.textHandle);

  expect(mirrorState.rootCanvasId).toBeTruthy();
  expect(mirrorState.rootAriaHidden).toBe('true');
  expect(mirrorState.fragmentCanvasId).toBe(mirrorState.rootCanvasId);
  expect(mirrorState.fragmentHandle).toBe(scene.textHandle);
  expect(mirrorState.fragmentStart).toBe('0');
  expect(mirrorState.fragmentEnd).toBe(String(new TextEncoder().encode(sample).byteLength));
  expect(mirrorState.fragmentText).toBe(sample);
});

test('find-on-page mirror excludes editable text', async ({ page }) => {
  await gotoBridgePage(page);
  await buildEditableTextScene(page, 'Editable text should stay out of the Find mirror.');

  const mirrorState = await page.evaluate(() => {
    const root = document.querySelector('[data-ed-find-root="1"]');
    return {
      hasRoot: root instanceof HTMLElement,
      fragmentCount: document.querySelectorAll('[data-ed-find-fragment="1"]').length,
    };
  });

  expect(mirrorState.hasRoot).toBe(true);
  expect(mirrorState.fragmentCount).toBe(0);
});

test('find-on-page mirror preserves realized Text tree order', async ({ page }) => {
  await gotoBridgePage(page);
  const scene = await buildMultiStaticTextScene(page, ['First projected block', 'Second projected block']);

  const mirrorTexts = await page.evaluate(() => {
    return Array.from(document.querySelectorAll('[data-ed-find-fragment="1"]'))
      .map((node) => node.textContent);
  });

  expect(scene.textHandles).toHaveLength(2);
  expect(mirrorTexts).toEqual(['First projected block', 'Second projected block']);
});

test('find-on-page selectionchange maps mirror selections back to UTF-8 byte ranges', async ({ page }) => {
  await gotoBridgePage(page);
  const sample = 'AéB';
  const scene = await buildStaticTextScene(page, sample);

  await page.evaluate((textHandle) => {
    const fragment = document.querySelector(`[data-ed-find-fragment="1"][data-ed-handle="${textHandle}"]`);
    if (!(fragment instanceof HTMLElement)) {
      throw new Error('Expected Find fragment for test handle.');
    }
    const textNode = fragment.firstChild;
    if (textNode?.nodeType !== Node.TEXT_NODE) {
      throw new Error('Expected plain text node inside Find fragment.');
    }
    const selection = window.getSelection();
    if (selection === null) {
      throw new Error('Expected window selection.');
    }
    const range = document.createRange();
    range.setStart(textNode, 1);
    range.setEnd(textNode, 2);
    selection.removeAllRanges();
    selection.addRange(range);
  }, scene.textHandle);

  await page.waitForFunction((textHandle) => {
    const match = window.__bridgeFindMatch;
    return (
      match?.handle === textHandle &&
      match.start === 1 &&
      match.end === 3
    );
  }, scene.textHandle);

  const match = await page.evaluate(() => window.__bridgeFindMatch);
  expect(match).toEqual({
    handle: scene.textHandle,
    start: 1,
    end: 3,
  });

  await page.evaluate(() => {
    window.getSelection()?.removeAllRanges();
    window.dispatchEvent(new Event('focus'));
  });

  await expect.poll(async () => {
    return await page.evaluate(() => window.__bridgeFindMatch);
  }).toBeNull();
});

test('desktop find dialog uses Control+F on non-Apple platforms', async ({ page }) => {
  await page.addInitScript(() => {
    Object.defineProperty(window.navigator, 'platform', {
      configurable: true,
      get: () => 'Win32',
    });
    Object.defineProperty(window.navigator, 'userAgentData', {
      configurable: true,
      get: () => ({ platform: 'Windows', mobile: false }),
    });
  });
  await gotoBridgePage(page);
  const sample = 'Edge bridge find dialog';
  const scene = await buildStaticTextScene(page, sample);

  await page.keyboard.press('Control+F');

  await expect.poll(async () => {
    return await page.evaluate(() => {
      const dialog = document.querySelector('[data-ed-find-dialog="1"]');
      const input = dialog?.querySelector('input[aria-label="Find query"]');
      const disclosure = dialog?.querySelector('button[aria-label="Show advanced find options"]');
      const previous = dialog?.querySelector('button[aria-label="Previous result"]');
      const next = dialog?.querySelector('button[aria-label="Next result"]');
      const close = dialog?.querySelector('button[aria-label="Close find dialog"]');
      const rect = dialog instanceof HTMLElement ? dialog.getBoundingClientRect() : null;
      return {
        open: dialog instanceof HTMLElement && dialog.style.display === 'flex',
        focused: input instanceof HTMLInputElement && document.activeElement === input,
        position: dialog instanceof HTMLElement ? dialog.style.position : null,
        width: rect === null ? null : Math.round(rect.width),
        height: rect === null ? null : Math.round(rect.height),
        disclosureBackground: disclosure instanceof HTMLButtonElement ? disclosure.style.background : null,
        previousWidth: previous instanceof HTMLButtonElement ? Math.round(previous.getBoundingClientRect().width) : null,
        nextWidth: next instanceof HTMLButtonElement ? Math.round(next.getBoundingClientRect().width) : null,
        disclosureWidth: disclosure instanceof HTMLButtonElement ? Math.round(disclosure.getBoundingClientRect().width) : null,
        closeWidth: close instanceof HTMLButtonElement ? Math.round(close.getBoundingClientRect().width) : null,
      };
    });
  }).toEqual({
    open: true,
    focused: true,
    position: 'fixed',
    width: 356,
    height: 44,
    disclosureBackground: 'transparent',
    previousWidth: 32,
    nextWidth: 32,
    disclosureWidth: 32,
    closeWidth: 32,
  });

  await page.hover('[data-ed-find-dialog="1"] button[aria-label="Show advanced find options"]');

  await expect.poll(async () => {
    return await page.evaluate(() => {
      const disclosure = document.querySelector('[data-ed-find-dialog="1"] button[aria-label="Show advanced find options"]');
      const close = document.querySelector('[data-ed-find-dialog="1"] button[aria-label="Close find dialog"]');
      if (!(disclosure instanceof HTMLButtonElement) || !(close instanceof HTMLButtonElement)) {
        return null;
      }
      const disclosureBackground = getComputedStyle(disclosure).backgroundColor;
      const closeBackground = getComputedStyle(close).backgroundColor;
      return {
        disclosureFilled: disclosureBackground !== 'rgba(0, 0, 0, 0)',
        closeTransparent: closeBackground === 'rgba(0, 0, 0, 0)',
      };
    });
  }).toEqual({
    disclosureFilled: true,
    closeTransparent: true,
  });

  await page.keyboard.type('bridge');

  await expect.poll(async () => {
    return await page.evaluate(() => ({
      match: window.__bridgeFindMatch,
      status: document.querySelector('[data-ed-find-dialog="1"] span')?.textContent ?? '',
      focused: document.activeElement?.getAttribute('aria-label') === 'Find query',
    }));
  }).toEqual({
    match: { handle: scene.textHandle, start: 5, end: 11 },
    status: '1/1',
    focused: true,
  });

  await page.keyboard.press('Control+F');

  await expect.poll(async () => {
    return await page.evaluate(() => {
      const input = document.querySelector('[data-ed-find-dialog="1"] input[aria-label="Find query"]');
      if (!(input instanceof HTMLInputElement)) {
        return null;
      }
      return {
        focused: document.activeElement === input,
        query: input.value,
        selectionStart: input.selectionStart,
        selectionEnd: input.selectionEnd,
      };
    });
  }).toEqual({
    focused: true,
    query: 'bridge',
    selectionStart: 0,
    selectionEnd: 6,
  });

  await page.keyboard.press('Escape');
  await expect.poll(async () => {
    return await page.evaluate(() => ({
      match: window.__bridgeFindMatch,
      open: document.querySelector<HTMLElement>('[data-ed-find-dialog="1"]')?.style.display === 'flex',
    }));
  }).toEqual({
    match: null,
    open: false,
  });
});

test('desktop find dialog uses Meta+F on Apple platforms', async ({ page }) => {
  await page.addInitScript(() => {
    Object.defineProperty(window.navigator, 'platform', {
      configurable: true,
      get: () => 'MacIntel',
    });
    Object.defineProperty(window.navigator, 'userAgentData', {
      configurable: true,
      get: () => ({ platform: 'macOS', mobile: false }),
    });
  });
  await gotoBridgePage(page);
  const sample = 'Apple bridge find dialog';
  const scene = await buildStaticTextScene(page, sample);

  await page.keyboard.press('Control+F');
  await expect.poll(async () => {
    return await page.evaluate(() => {
      const dialog = document.querySelector('[data-ed-find-dialog="1"]');
      return dialog instanceof HTMLElement && dialog.style.display === 'flex';
    });
  }).toBe(false);

  await page.keyboard.press('Meta+F');

  await expect.poll(async () => {
    return await page.evaluate(() => {
      const dialog = document.querySelector('[data-ed-find-dialog="1"]');
      const input = dialog?.querySelector('input[aria-label="Find query"]');
      return {
        open: dialog instanceof HTMLElement && dialog.style.display === 'flex',
        focused: input instanceof HTMLInputElement && document.activeElement === input,
      };
    });
  }).toEqual({
    open: true,
    focused: true,
  });

  await page.keyboard.type('bridge');

  await expect.poll(async () => {
    return await page.evaluate(() => ({
      match: window.__bridgeFindMatch,
      status: document.querySelector('[data-ed-find-dialog="1"] span')?.textContent ?? '',
    }));
  }).toEqual({
    match: { handle: scene.textHandle, start: 6, end: 12 },
    status: '1/1',
  });
});

test('desktop find dialog keeps focus through option toggles and no-match states', async ({ page }) => {
  await page.addInitScript(() => {
    Object.defineProperty(window.navigator, 'platform', {
      configurable: true,
      get: () => 'Win32',
    });
    Object.defineProperty(window.navigator, 'userAgentData', {
      configurable: true,
      get: () => ({ platform: 'Windows', mobile: false }),
    });
  });
  await gotoBridgePage(page);
  const sample = 'Cafe café CAFE bridge bridged bridge';
  await buildStaticTextScene(page, sample);

  await page.keyboard.press('Control+F');
  await page.keyboard.type('bridge');

  await expect.poll(async () => {
    return await page.evaluate(() => ({
      status: document.querySelector('[data-ed-find-dialog="1"] span')?.textContent ?? '',
      focused: document.activeElement?.getAttribute('aria-label') === 'Find query',
    }));
  }).toEqual({
    status: '1/3',
    focused: true,
  });

  await page.click('[data-ed-find-dialog="1"] button[aria-label="Show advanced find options"]');
  await page.click('[data-ed-find-dialog="1"] button[data-ed-find-option="wholeWords"]');

  await expect.poll(async () => {
    return await page.evaluate(() => ({
      status: document.querySelector('[data-ed-find-dialog="1"] span')?.textContent ?? '',
      focused: document.activeElement?.getAttribute('aria-label') === 'Find query',
      wholeWords: window.__bridgeFindState?.options.wholeWords ?? false,
      optionRowHeight: (() => {
        const button = document.querySelector('[data-ed-find-dialog="1"] button[data-ed-find-option="wholeWords"]');
        return button instanceof HTMLButtonElement ? Math.round(button.getBoundingClientRect().height) : null;
      })(),
      optionFontSize: (() => {
        const button = document.querySelector('[data-ed-find-dialog="1"] button[data-ed-find-option="wholeWords"]');
        return button instanceof HTMLButtonElement ? getComputedStyle(button).fontSize : null;
      })(),
      toggleWidth: (() => {
        const button = document.querySelector('[data-ed-find-dialog="1"] button[data-ed-find-option="wholeWords"]');
        const track = button?.querySelector('span:last-child');
        return track instanceof HTMLElement ? Math.round(track.getBoundingClientRect().width) : null;
      })(),
      toggleHeight: (() => {
        const button = document.querySelector('[data-ed-find-dialog="1"] button[data-ed-find-option="wholeWords"]');
        const track = button?.querySelector('span:last-child');
        return track instanceof HTMLElement ? Math.round(track.getBoundingClientRect().height) : null;
      })(),
    }));
  }).toEqual({
    status: '1/2',
    focused: true,
    wholeWords: true,
    optionRowHeight: 38,
    optionFontSize: '12px',
    toggleWidth: 30,
    toggleHeight: 18,
  });

  await page.evaluate(() => {
    const input = document.querySelector('[data-ed-find-dialog="1"] input[aria-label="Find query"]');
    if (!(input instanceof HTMLInputElement)) {
      throw new Error('Expected desktop find input.');
    }
    input.value = 'missing';
    input.dispatchEvent(new Event('input', { bubbles: true }));
  });

  await expect.poll(async () => {
    return await page.evaluate(() => ({
      status: document.querySelector('[data-ed-find-dialog="1"] span')?.textContent ?? '',
      focused: document.activeElement?.getAttribute('aria-label') === 'Find query',
      bridgeMatch: window.__bridgeFindMatch,
    }));
  }).toEqual({
    status: '0/0',
    focused: true,
    bridgeMatch: null,
  });
});

test('desktop find dialog adapts to light color scheme', async ({ page }) => {
  await page.emulateMedia({ colorScheme: 'light' });
  await page.addInitScript(() => {
    Object.defineProperty(window.navigator, 'platform', {
      configurable: true,
      get: () => 'Win32',
    });
    Object.defineProperty(window.navigator, 'userAgentData', {
      configurable: true,
      get: () => ({ platform: 'Windows', mobile: false }),
    });
  });
  await gotoBridgePage(page);
  await buildStaticTextScene(page, 'Light mode bridge find dialog');

  await page.keyboard.press('Control+F');

  await expect.poll(async () => {
    return await page.evaluate(() => {
      const dialog = document.querySelector('[data-ed-find-dialog="1"]');
      const input = dialog?.querySelector('input[aria-label="Find query"]');
      if (!(dialog instanceof HTMLElement) || !(input instanceof HTMLInputElement)) {
        return null;
      }
      return {
        open: dialog.style.display === 'flex',
        colorScheme: dialog.style.colorScheme,
        background: dialog.style.background,
        inputColor: input.style.color,
      };
    });
  }).toEqual({
    open: true,
    colorScheme: 'light',
    background: 'rgba(251, 251, 251, 0.98)',
    inputColor: 'rgb(17, 24, 39)',
  });
});

test('find-on-page selectionchange ignores other canvas roots', async ({ page }) => {
  await gotoBridgePage(page);
  const sample = 'Canvas-owned match';
  const scene = await buildStaticTextScene(page, sample);

  await page.evaluate((textHandle) => {
    const fragment = document.querySelector(`[data-ed-find-fragment="1"][data-ed-handle="${textHandle}"]`);
    if (!(fragment instanceof HTMLElement)) {
      throw new Error('Expected Find fragment for test handle.');
    }
    const textNode = fragment.firstChild;
    if (textNode?.nodeType !== Node.TEXT_NODE) {
      throw new Error('Expected plain text node inside Find fragment.');
    }
    const selection = window.getSelection();
    if (selection === null) {
      throw new Error('Expected window selection.');
    }
    const range = document.createRange();
    range.setStart(textNode, 0);
    range.setEnd(textNode, 6);
    selection.removeAllRanges();
    selection.addRange(range);
  }, scene.textHandle);

  await page.waitForFunction((textHandle) => window.__bridgeFindMatch?.handle === textHandle, scene.textHandle);

  const retainedMatch = await page.evaluate(async () => {
    const otherRoot = document.createElement('div');
    otherRoot.setAttribute('data-ed-find-root', '1');
    otherRoot.setAttribute('data-ed-canvas-id', 'other-canvas');
    const otherFragment = document.createElement('div');
    otherFragment.setAttribute('data-ed-find-fragment', '1');
    otherFragment.setAttribute('data-ed-canvas-id', 'other-canvas');
    otherFragment.setAttribute('data-ed-handle', '999');
    otherFragment.setAttribute('data-ed-start', '0');
    otherFragment.setAttribute('data-ed-end', '4');
    otherFragment.textContent = 'fake';
    otherRoot.appendChild(otherFragment);
    document.body.appendChild(otherRoot);

    const textNode = otherFragment.firstChild;
    if (textNode?.nodeType !== Node.TEXT_NODE) {
      throw new Error('Expected plain text node inside fake fragment.');
    }
    const selection = window.getSelection();
    if (selection === null) {
      throw new Error('Expected window selection.');
    }
    const range = document.createRange();
    range.setStart(textNode, 0);
    range.setEnd(textNode, 4);
    selection.removeAllRanges();
    selection.addRange(range);

    await new Promise((resolve) => setTimeout(resolve, 0));
    const match = window.__bridgeFindMatch;
    otherRoot.remove();
    return match;
  });

  expect(retainedMatch).toEqual({
    handle: scene.textHandle,
    start: 0,
    end: 6,
  });
});

test('find-on-page selectionchange ignores hidden editor selections', async ({ page }) => {
  await gotoBridgePage(page);
  const sample = 'Mirror match stays stable';
  const scene = await buildStaticTextScene(page, sample);

  await page.evaluate((textHandle) => {
    const fragment = document.querySelector(`[data-ed-find-fragment="1"][data-ed-handle="${textHandle}"]`);
    if (!(fragment instanceof HTMLElement)) {
      throw new Error('Expected Find fragment for test handle.');
    }
    const textNode = fragment.firstChild;
    if (textNode?.nodeType !== Node.TEXT_NODE) {
      throw new Error('Expected plain text node inside Find fragment.');
    }
    const selection = window.getSelection();
    if (selection === null) {
      throw new Error('Expected window selection.');
    }
    const range = document.createRange();
    range.setStart(textNode, 0);
    range.setEnd(textNode, 6);
    selection.removeAllRanges();
    selection.addRange(range);
  }, scene.textHandle);

  await page.waitForFunction((textHandle) => window.__bridgeFindMatch?.handle === textHandle, scene.textHandle);

  await page.evaluate(() => {
    const editor = document.querySelector('[data-effindom-hidden-editor="true"]');
    if (!(editor instanceof HTMLInputElement || editor instanceof HTMLTextAreaElement)) {
      throw new Error('Expected hidden editor.');
    }
    editor.value = 'editor selection';
    editor.focus();
    editor.setSelectionRange(0, 6);
  });

  // Wait for the hidden editor to actually become the activeElement. Programmatic focus
  // with aria-hidden toggles is macrotask-dependent across platforms, so poll until it
  // becomes active rather than relying on a single macrotask sleep.
  await page.waitForFunction(() => document.activeElement?.getAttribute('data-effindom-hidden-editor') === 'true');

  const retainedMatch = await page.evaluate(() => ({
    activeElementHiddenEditor: document.activeElement?.getAttribute('data-effindom-hidden-editor') ?? null,
    match: window.__bridgeFindMatch,
  }));

  expect(retainedMatch.activeElementHiddenEditor).toBe('true');
  expect(retainedMatch.match).toEqual({
    handle: scene.textHandle,
    start: 0,
    end: 6,
  });
});

test('focus changes publish a live announcement', async ({ page }) => {
  await gotoBridgePage(page);
  const scene = await buildSemanticScene(page);

  const announcement = await page.evaluate(async (textboxHandle) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const layer = document.getElementById('semantic-layer');
    const shadow = layer?.shadowRoot;
    if (runtime === null || runtime === undefined || !(shadow instanceof ShadowRoot)) {
      throw new Error('Expected bridge runtime and semantic shadow root.');
    }
    const announcer = shadow.querySelector('[data-effindom-live-announcer="true"]');
    if (!(announcer instanceof HTMLOutputElement)) {
      throw new Error('Expected live announcer.');
    }
    const bridge = window.EffinDomBrowserBridge;
    if (bridge === undefined) {
      throw new Error('Bridge state is not ready.');
    }
    runtime.ui._ui_request_focus(bridge.handleToBigInt(textboxHandle));
    runtime.commitFrame();
    runtime.flushPendingCommit();
    await new Promise((resolve) => setTimeout(resolve, 220));
    return announcer.textContent;
  }, scene.textboxHandle);

  expect(announcement).toContain('Email');
  expect(announcement).toContain('text area');
});

test('requested semantic announcements replay the focused control summary', async ({ page }) => {
  await gotoBridgePage(page);
  const scene = await buildSemanticScene(page);

  const announcement = await page.evaluate(async (textboxHandle) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const bridge = window.EffinDomBrowserBridge;
    const layer = document.getElementById('semantic-layer');
    const shadow = layer?.shadowRoot;
    if (runtime === null || runtime === undefined || bridge === undefined || !(shadow instanceof ShadowRoot)) {
      throw new Error('Expected bridge runtime and semantic shadow root.');
    }
    const announcer = shadow.querySelector('[data-effindom-live-announcer="true"]');
    if (!(announcer instanceof HTMLOutputElement)) {
      throw new Error('Expected live announcer.');
    }

    const handle = bridge.handleToBigInt(textboxHandle);
    runtime.ui._ui_request_focus(handle);
    runtime.commitFrame();
    runtime.flushPendingCommit();
    await new Promise((resolve) => setTimeout(resolve, 220));

    window.__effindomCallbacks?.onRequestSemanticAnnouncement?.(textboxHandle);
    runtime.commitFrame();
    runtime.flushPendingCommit();
    await new Promise((resolve) => setTimeout(resolve, 220));
    return announcer.textContent;
  }, scene.textboxHandle);

  expect(announcement).toContain('Email');
  expect(announcement).toContain('text area');
});

test('same-frame focus and checkbox announcements use the refreshed checked state', async ({ page }) => {
  await gotoBridgePage(page);

  const announcement = await page.evaluate(async () => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const bridge = window.EffinDomBrowserBridge;
    const layer = document.getElementById('semantic-layer');
    const shadow = layer?.shadowRoot;
    if (runtime === null || runtime === undefined || bridge === undefined || !(shadow instanceof ShadowRoot)) {
      throw new Error('Expected bridge runtime and semantic shadow root.');
    }
    const announcer = shadow.querySelector('[data-effindom-live-announcer="true"]');
    if (!(announcer instanceof HTMLOutputElement)) {
      throw new Error('Expected live announcer.');
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
    const checkbox = toHandle(ui._ui_create_node(0));
    ui._ui_set_root(root);
    ui._ui_node_add_child(root, checkbox);
    ui._ui_set_width(root, 320, 0);
    ui._ui_set_height(root, 120, 0);
    ui._ui_set_bg_color(root, 0x111827ff);
    ui._ui_set_width(checkbox, 160, 0);
    ui._ui_set_height(checkbox, 32, 0);
    ui._ui_set_semantic_role(checkbox, 11);
    ui._ui_set_interactive(checkbox, 1);
    ui._ui_set_focusable(checkbox, 1, 0);
    ui._ui_set_semantic_checked(checkbox, 1);
    const label = writeText('Accept terms');
    try {
      ui._ui_set_semantic_label(checkbox, label.ptr, label.len);
    } finally {
      if (label.offset !== 0) {
        ui._free(label.ptr);
      }
    }

    runtime.commitFrame();
    runtime.flushPendingCommit();

    ui._ui_request_focus(bridge.handleToBigInt(checkbox));
    ui._ui_set_semantic_checked(bridge.handleToBigInt(checkbox), 2);
    window.__effindomCallbacks?.onRequestSemanticAnnouncement?.(checkbox);
    runtime.commitFrame();
    runtime.flushPendingCommit();
    await new Promise((resolve) => setTimeout(resolve, 220));
    return announcer.textContent;
  });

  expect(announcement).toContain('Accept terms');
  expect(announcement).toContain('checkbox');
  expect(announcement).toContain('checked');
  expect(announcement).not.toContain('unchecked');
});

test('slider announcements do not repeat the current value', async ({ page }) => {
  await gotoBridgePage(page);

  const announcement = await page.evaluate(async () => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const bridge = window.EffinDomBrowserBridge;
    const layer = document.getElementById('semantic-layer');
    const shadow = layer?.shadowRoot;
    if (runtime === null || runtime === undefined || bridge === undefined || !(shadow instanceof ShadowRoot)) {
      throw new Error('Expected bridge runtime and semantic shadow root.');
    }
    const announcer = shadow.querySelector('[data-effindom-live-announcer="true"]');
    if (!(announcer instanceof HTMLOutputElement)) {
      throw new Error('Expected live announcer.');
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
    const slider = toHandle(ui._ui_create_node(0));
    ui._ui_set_root(root);
    ui._ui_node_add_child(root, slider);
    ui._ui_set_width(root, 320, 0);
    ui._ui_set_height(root, 120, 0);
    ui._ui_set_bg_color(root, 0x111827ff);
    ui._ui_set_width(slider, 160, 0);
    ui._ui_set_height(slider, 32, 0);
    ui._ui_set_semantic_role(slider, 15);
    ui._ui_set_interactive(slider, 1);
    ui._ui_set_focusable(slider, 1, 0);
    ui._ui_set_semantic_value_range(slider, 1, 50, 0, 100);
    const label = writeText('Slider');
    try {
      ui._ui_set_semantic_label(slider, label.ptr, label.len);
    } finally {
      if (label.offset !== 0) {
        ui._free(label.ptr);
      }
    }

    runtime.commitFrame();
    runtime.flushPendingCommit();

    ui._ui_request_focus(bridge.handleToBigInt(slider));
    runtime.commitFrame();
    runtime.flushPendingCommit();
    await new Promise((resolve) => setTimeout(resolve, 220));
    return announcer.textContent;
  });

  expect(announcement).toContain('Slider');
  expect(announcement).toContain('value 50');
  expect((announcement.match(/value 50/g) ?? []).length).toBe(1);
  expect(announcement).not.toContain('Slider, slider');
});

test('__OPEN_CANVAS_API__.revealRange scrolls retained text into view', async ({ page }) => {
  await gotoBridgePage(page);
  const sample = [
    'Line 0 filler text keeps the viewport busy.',
    'Line 1 filler text keeps the viewport busy.',
    'Line 2 filler text keeps the viewport busy.',
    'Line 3 filler text keeps the viewport busy.',
    'Line 4 filler text keeps the viewport busy.',
    'Line 5 filler text keeps the viewport busy.',
    'Line 6 carries the FinalTarget marker for reveal.',
  ].join('\n');
  const scene = await buildScrollableStaticTextScene(page, sample);
  const start = sample.indexOf('FinalTarget');
  expect(start).toBeGreaterThanOrEqual(0);
  const end = start + 'FinalTarget'.length;

  const revealState = await page.evaluate(({ textHandle, scrollHandle, start, end }) => {
    const api = window.__OPEN_CANVAS_API__;
    if (api === undefined) {
      throw new Error('Expected __OPEN_CANVAS_API__.');
    }
    const beforeRects = api.getRangeRects(textHandle, start, end);
    const revealed = api.revealRange(textHandle, start, end);
    const afterRects = api.getRangeRects(textHandle, start, end);
    const scrollEvents = (window.__bridgeLogs?.scrollEvents ?? []).filter((entry) => entry.handle === scrollHandle);
    return {
      beforeCount: beforeRects.length,
      beforeMaxY: Math.max(...beforeRects.map((rect) => rect.y + rect.height)),
      afterCount: afterRects.length,
      afterMaxY: Math.max(...afterRects.map((rect) => rect.y + rect.height)),
      revealed,
      scrollEvents,
    };
  }, { ...scene, start, end });

  expect(revealState.beforeCount).toBeGreaterThan(0);
  expect(revealState.beforeMaxY).toBeGreaterThan(60);
  expect(revealState.revealed).toBe(true);
  expect(revealState.scrollEvents.length).toBeGreaterThan(0);
  expect(revealState.scrollEvents[revealState.scrollEvents.length - 1]?.offsetY ?? 0).toBeGreaterThan(0);
  expect(revealState.afterCount).toBeGreaterThan(0);
  expect(revealState.afterMaxY).toBeLessThanOrEqual(60.1);
});

test('find-on-page selectionchange reveals retained scrollview matches and refreshes highlight commands', async ({ page }) => {
  await gotoBridgePage(page);
  const sample = [
    'Line 0 filler text keeps the viewport busy.',
    'Line 1 filler text keeps the viewport busy.',
    'Line 2 filler text keeps the viewport busy.',
    'Line 3 filler text keeps the viewport busy.',
    'Line 4 filler text keeps the viewport busy.',
    'Line 5 filler text keeps the viewport busy.',
    'Line 6 carries the FinalTarget marker for reveal.',
  ].join('\n');
  const scene = await buildScrollableStaticTextScene(page, sample);
  const start = sample.indexOf('FinalTarget');
  expect(start).toBeGreaterThanOrEqual(0);
  const end = start + 'FinalTarget'.length;

  const revealState = await page.evaluate(async ({ textHandle, scrollHandle, start, end }) => {
    const fragment = document.querySelector(`[data-ed-find-fragment="1"][data-ed-handle="${textHandle}"]`);
    const api = window.__OPEN_CANVAS_API__;
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    if (
      !(fragment instanceof HTMLElement) ||
      api === undefined ||
      runtime === null ||
      runtime === undefined
    ) {
      throw new Error('Expected Find fragment, bridge runtime, and __OPEN_CANVAS_API__.');
    }
    const textNode = fragment.firstChild;
    if (textNode?.nodeType !== Node.TEXT_NODE) {
      throw new Error('Expected plain text node inside Find fragment.');
    }
    const selection = window.getSelection();
    if (selection === null) {
      throw new Error('Expected window selection.');
    }
    const range = document.createRange();
    range.setStart(textNode, start);
    range.setEnd(textNode, end);
    selection.removeAllRanges();
    selection.addRange(range);

    await new Promise((resolve) => setTimeout(resolve, 0));
    const rects = api.getRangeRects(textHandle, start, end);
    const words = Array.from(runtime.extractCommandBuffer());
    const scrollEvents = (window.__bridgeLogs?.scrollEvents ?? []).filter((entry) => entry.handle === scrollHandle);
    return {
      match: window.__bridgeFindMatch,
      rects,
      words,
      scrollEvents,
    };
  }, { ...scene, start, end });

  expect(revealState.match).toEqual({
    handle: scene.textHandle,
    start,
    end,
  });
  expect(revealState.scrollEvents.length).toBeGreaterThan(0);
  expect(revealState.scrollEvents[revealState.scrollEvents.length - 1]?.offsetY ?? 0).toBeGreaterThan(0);
  expect(revealState.rects.length).toBeGreaterThan(0);
  const afterMaxY = Math.max(...revealState.rects.map((rect) => rect.y + rect.height));
  expect(afterMaxY).toBeLessThanOrEqual(60.1);
  const highlightRects = parseHighlightRects(revealState.words, scene.textHandle);
  expect(highlightRects.length).toBeGreaterThan(0);
  expect(highlightRects[0]?.width ?? 0).toBeGreaterThan(0);
  expect(highlightRects[0]?.height ?? 0).toBeGreaterThan(0);
});
