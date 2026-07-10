import { expect, test } from '@playwright/test';

import {
  buildSemanticScene,
  buildInteractiveBoxScene,
  buildClippedSemanticScene,
  buildScrollableStaticTextScene,
  gotoBridgePage,
  setupServer,
  teardownServer,
} from './test-utils';

test.beforeAll(async () => {
  await setupServer();
});

test.afterAll(async () => {
  await teardownServer();
});

test('DevTools DOM mirror projects retained nodes when enabled at startup', async ({ page }) => {
  await gotoBridgePage(page, '', { devToolsDomMirror: 'enabled' });

  const rootHandle = await page.evaluate(() => window.__bridgeState?.rootHandle ?? null);
  expect(rootHandle).not.toBeNull();
  if (rootHandle === null) {
    throw new Error('Expected root handle.');
  }

  const mirror = page.locator('#effindom-devtools-dom-mirror');
  await expect(mirror).toHaveCount(1);
  await expect(mirror).toHaveAttribute('aria-hidden', 'true');
  await expect(mirror).toHaveAttribute('inert', '');
  await expect(mirror).toHaveAttribute('data-fui-devtools-dom-mirror', 'true');

  const node = mirror.locator('fui-flex-box').first();
  await expect(node).toHaveCount(1);
  await expect(node).toHaveAttribute('data-fui-handle', rootHandle);
  await expect(node).toHaveAttribute('data-fui-type', 'flex-box');
  await expect(node).toHaveAttribute('data-fui-node-type', '0');
  await expect(node).toHaveAttribute('data-fui-bounds', '0,0,322,222');
  await expect(node).toHaveAttribute('data-fui-visible-bounds', '0,0,322,222');
  await expect(node).toHaveCSS('position', 'absolute');
  await expect(node).toHaveCSS('width', '322px');
  await expect(node).toHaveCSS('height', '222px');
});

test('DevTools DOM mirror defaults to on-requested in debug mode and activates through console API', async ({ page }) => {
  await gotoBridgePage(page, '', { buildMode: 'debug' });

  const mirror = page.locator('#effindom-devtools-dom-mirror');
  await expect(mirror).toHaveCount(0);

  const enableResult = await page.evaluate(() => window.EffinDomBrowserBridge?.devTools.enableDomMirror() ?? false);
  expect(enableResult).toBe(true);

  const rootHandle = await page.evaluate(() => window.__bridgeState?.rootHandle ?? null);
  await expect(mirror).toHaveCount(1);
  await expect(mirror.locator('fui-flex-box').first()).toHaveAttribute('data-fui-handle', rootHandle ?? '');

  const disableResult = await page.evaluate(() => window.EffinDomBrowserBridge?.devTools.disableDomMirror() ?? true);
  expect(disableResult).toBe(false);
  await expect(mirror).toHaveCount(0);
});

test('DevTools debug dialog shortcut toggles the dialog without enabling the mirror', async ({ page }) => {
  await gotoBridgePage(page, '', { devToolsDomMirror: 'on-requested' });

  await expect(page.locator('#effindom-devtools-dom-mirror')).toHaveCount(0);
  await expect(page.locator('#effindom-devtools-debug-dialog')).toHaveCount(0);

  await page.keyboard.press('Meta+Shift+F12');

  const rootHandle = await page.evaluate(() => window.__bridgeState?.rootHandle ?? null);
  const mirror = page.locator('#effindom-devtools-dom-mirror');
  const dialog = page.locator('#effindom-devtools-debug-dialog');
  await expect(mirror).toHaveCount(0);
  await expect(dialog).toHaveCount(1);
  await expect(dialog).toHaveAttribute('role', 'dialog');
  await expect(dialog.locator('[data-fui-devtools-dialog-mirror-status="true"]')).toHaveText('Mirror off');

  await dialog.locator('[data-fui-devtools-dialog-mirror-row="true"]').click();
  await expect(mirror).toHaveCount(1);
  await expect(dialog.locator('[data-fui-devtools-dialog-mirror-status="true"]')).toHaveText('Mirror on');
  await expect(mirror.locator('fui-flex-box').first()).toHaveAttribute('data-fui-handle', rootHandle ?? '');

  await page.keyboard.press('Meta+Shift+F12');
  await expect(dialog).toHaveCount(0);
  await expect(mirror).toHaveCount(1);
});

test('DevTools debug dialog can be controlled through console API', async ({ page }) => {
  await gotoBridgePage(page, '', { devToolsDomMirror: 'on-requested' });

  const openResult = await page.evaluate(() => window.EffinDomBrowserBridge?.devTools.openDebugDialog() ?? false);
  expect(openResult).toBe(true);
  await expect(page.locator('#effindom-devtools-debug-dialog')).toHaveCount(1);
  await expect(page.locator('#effindom-devtools-dom-mirror')).toHaveCount(0);

  const isOpen = await page.evaluate(() => window.EffinDomBrowserBridge?.devTools.isDebugDialogOpen() ?? false);
  expect(isOpen).toBe(true);

  const closeResult = await page.evaluate(() => window.EffinDomBrowserBridge?.devTools.closeDebugDialog() ?? true);
  expect(closeResult).toBe(false);
  await expect(page.locator('#effindom-devtools-debug-dialog')).toHaveCount(0);

  const toggleOpenResult = await page.evaluate(() => window.EffinDomBrowserBridge?.devTools.toggleDebugDialog() ?? false);
  expect(toggleOpenResult).toBe(true);
  await expect(page.locator('#effindom-devtools-debug-dialog')).toHaveCount(1);
  await expect(page.locator('#effindom-devtools-dom-mirror')).toHaveCount(0);
});

test('DevTools Inspect Mode selects hit-tested nodes and suppresses app clicks until Escape', async ({ page }) => {
  await gotoBridgePage(page, '', { devToolsDomMirror: 'on-requested' });
  const scene = await buildInteractiveBoxScene(page);

  await page.keyboard.press('Meta+Shift+F12');
  const dialog = page.locator('#effindom-devtools-debug-dialog');
  await expect(dialog).toHaveCount(1);
  await dialog.locator('[data-fui-devtools-dialog-inspect-row="true"]').click();
  await expect(dialog.locator('[data-fui-devtools-dialog-inspect-toggle="true"]')).toHaveAttribute('aria-checked', 'true');

  const canvas = page.locator('#fui-canvas');
  const box = await canvas.boundingBox();
  expect(box).not.toBeNull();
  if (box === null) {
    throw new Error('Expected canvas bounds.');
  }
  const targetX = box.x + 40;
  const targetY = box.y + 40;
  const mirror = page.locator('#effindom-devtools-dom-mirror');
  const targetNode = mirror.locator(`[data-fui-handle="${scene.boxHandle}"]`).first();

  await page.mouse.move(targetX, targetY);
  await expect(targetNode).toHaveAttribute('data-fui-inspect-hovered', 'true');

  await page.evaluate(() => {
    window.EffinDomBrowserBridge?.resetLogs();
  });
  await page.mouse.click(targetX, targetY);
  await expect(targetNode).toHaveAttribute('data-fui-selected', 'true');
  await expect(dialog.locator('[data-fui-devtools-dialog-selected-handle="true"]')).toHaveText(scene.boxHandle);
  const inspectClickPointerEventCount = await page.evaluate(() => window.EffinDomBrowserBridge?.getRuntime()?.logs.pointerEvents.length ?? -1);
  expect(inspectClickPointerEventCount).toBe(0);

  await page.keyboard.press('Escape');
  await expect(dialog.locator('[data-fui-devtools-dialog-inspect-toggle="true"]')).toHaveAttribute('aria-checked', 'false');
  await expect(targetNode).not.toHaveAttribute('data-fui-inspect-hovered', 'true');
  await expect(targetNode).not.toHaveAttribute('data-fui-selected', 'true');
  await expect(dialog.locator('[data-fui-devtools-dialog-selected-handle="true"]')).toHaveText('None');

  await page.evaluate(() => {
    window.EffinDomBrowserBridge?.resetLogs();
  });
  await page.mouse.click(targetX, targetY);
  const normalPointerEvents = await page.evaluate(() => window.EffinDomBrowserBridge?.getRuntime()?.logs.pointerEvents ?? []);
  expect(normalPointerEvents.some((entry) => entry.handle === scene.boxHandle)).toBe(true);
});

test('DevTools Inspect Mode can select non-interactive images from the debug tree', async ({ page }) => {
  await gotoBridgePage(page, '', { devToolsDomMirror: 'on-requested' });
  const scene = await buildSemanticScene(page);

  await page.keyboard.press('Meta+Shift+F12');
  const dialog = page.locator('#effindom-devtools-debug-dialog');
  await dialog.locator('[data-fui-devtools-dialog-inspect-row="true"]').click();

  const target = await page.evaluate((handle) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const node = runtime?.getDebugTree().nodesByHandle[handle];
    if (node === undefined) {
      throw new Error('Expected image debug node.');
    }
    return {
      x: node.visibleBounds.x + (node.visibleBounds.width / 2),
      y: node.visibleBounds.y + (node.visibleBounds.height / 2),
    };
  }, scene.imageHandle);
  const canvasBox = await page.locator('#fui-canvas').boundingBox();
  expect(canvasBox).not.toBeNull();
  if (canvasBox === null) {
    throw new Error('Expected canvas bounds.');
  }

  await page.mouse.click(canvasBox.x + target.x, canvasBox.y + target.y);
  const imageNode = page.locator(`#effindom-devtools-dom-mirror [data-fui-handle="${scene.imageHandle}"]`).first();
  await expect(imageNode).toHaveAttribute('data-fui-selected', 'true');
  await expect(dialog.locator('[data-fui-devtools-dialog-selected-handle="true"]')).toHaveText(scene.imageHandle);
});

test('DevTools Inspect Mode prefers nested flex nodes over ancestor scroll views', async ({ page }) => {
  await gotoBridgePage(page, '', { devToolsDomMirror: 'on-requested' });
  const scene = await page.evaluate(() => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const bridge = window.EffinDomBrowserBridge;
    if (runtime === null || runtime === undefined || bridge === undefined) {
      throw new Error('Bridge runtime is not ready.');
    }
    const ui = runtime.ui;
    const toHandle = (handle: unknown): string =>
      bridge.handleToString(handle as string | number | bigint | { valueOf(): unknown; toString(): string });
    runtime.resetLogs();
    runtime.resetAppSession();
    ui._ui_reset();
    const root = toHandle(ui._ui_create_node(0));
    const scroll = toHandle(ui._ui_create_node(4));
    const child = toHandle(ui._ui_create_node(0));
    ui._ui_set_root(root);
    ui._ui_node_add_child(root, scroll);
    ui._ui_node_add_child(scroll, child);
    ui._ui_set_width(root, 320, 0);
    ui._ui_set_height(root, 220, 0);
    ui._ui_set_width(scroll, 220, 0);
    ui._ui_set_height(scroll, 140, 0);
    ui._ui_set_bg_color(scroll, 0x1f2937ff);
    ui._ui_set_scroll_enabled(scroll, 1, 1);
    ui._ui_set_width(child, 80, 0);
    ui._ui_set_height(child, 40, 0);
    ui._ui_set_bg_color(child, 0x22c55eff);
    runtime.commitFrame();
    runtime.flushPendingCommit();
    runtime.resetLogs();
    return { scrollHandle: scroll, childHandle: child };
  });

  await page.keyboard.press('Meta+Shift+F12');
  const dialog = page.locator('#effindom-devtools-debug-dialog');
  await dialog.locator('[data-fui-devtools-dialog-inspect-row="true"]').click();

  const target = await page.evaluate((handle) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const node = runtime?.getDebugTree().nodesByHandle[handle];
    if (node === undefined) {
      throw new Error('Expected nested flex debug node.');
    }
    return {
      x: node.visibleBounds.x + (node.visibleBounds.width / 2),
      y: node.visibleBounds.y + (node.visibleBounds.height / 2),
    };
  }, scene.childHandle);
  const canvasBox = await page.locator('#fui-canvas').boundingBox();
  expect(canvasBox).not.toBeNull();
  if (canvasBox === null) {
    throw new Error('Expected canvas bounds.');
  }

  await page.mouse.click(canvasBox.x + target.x, canvasBox.y + target.y);
  const childNode = page.locator(`#effindom-devtools-dom-mirror [data-fui-handle="${scene.childHandle}"]`).first();
  const scrollNode = page.locator(`#effindom-devtools-dom-mirror [data-fui-handle="${scene.scrollHandle}"]`).first();
  await expect(childNode).toHaveAttribute('data-fui-selected', 'true');
  await expect(scrollNode).not.toHaveAttribute('data-fui-selected', 'true');
  await expect(dialog.locator('[data-fui-devtools-dialog-selected-handle="true"]')).toHaveText(scene.childHandle);
});

test('DevTools DOM mirror draws overlays for partially visible selected nodes', async ({ page }) => {
  await gotoBridgePage(page, '', { devToolsDomMirror: 'on-requested' });
  const scene = await buildClippedSemanticScene(page);

  const selected = await page.evaluate((handle) => (
    window.EffinDomBrowserBridge?.devTools.selectHandle(handle) ?? false
  ), scene.partialHandle);
  expect(selected).toBe(true);

  const mirror = page.locator('#effindom-devtools-dom-mirror');
  const selectedNode = mirror.locator(`[data-fui-handle="${scene.partialHandle}"]`).first();
  const overlayBox = page.locator('#effindom-devtools-overlay [data-fui-devtools-overlay-box="true"]');
  await expect(selectedNode).toHaveAttribute('data-fui-selected', 'true');
  await expect(selectedNode).toHaveAttribute('data-fui-clipped', 'true');
  await expect(overlayBox).toHaveCSS('display', 'block');
  await expect(overlayBox).toHaveCSS('height', '12px');
});

test('DevTools debug dialog truncates long selected labels without overlapping the row header', async ({ page }) => {
  await gotoBridgePage(page, '', { devToolsDomMirror: 'on-requested' });
  const scene = await buildSemanticScene(page);
  await page.evaluate((handle) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const bridge = window.EffinDomBrowserBridge;
    if (runtime === null || runtime === undefined || bridge === undefined) {
      throw new Error('Bridge runtime is not ready.');
    }
    const text = 'This selected label is intentionally long enough to overflow the debug dialog label row and must be truncated.';
    const bytes = new TextEncoder().encode(text);
    const pointer = bridge.toHeapPointer(runtime.ui, runtime.ui._malloc(bytes.length));
    if (pointer.offset === 0) {
      throw new Error('ui malloc failed.');
    }
    try {
      runtime.ui.HEAPU8.set(bytes, pointer.offset);
      runtime.ui._ui_set_semantic_label(handle, pointer.ptr, bytes.length);
    } finally {
      runtime.ui._free(pointer.ptr);
    }
    runtime.commitFrame();
    runtime.flushPendingCommit();
    window.EffinDomBrowserBridge?.devTools.selectHandle(handle);
    window.EffinDomBrowserBridge?.devTools.openDebugDialog();
  }, scene.buttonHandle);

  const labelHeader = page.locator('#effindom-devtools-debug-dialog [data-fui-devtools-dialog-label="true"]').filter({ hasText: 'Label' });
  const labelValue = page.locator('#effindom-devtools-debug-dialog [data-fui-devtools-dialog-selected-label="true"]');
  await expect(labelValue).toContainText('This selected label');
  const geometry = await page.evaluate(() => {
    const header = Array.from(document.querySelectorAll<HTMLElement>('#effindom-devtools-debug-dialog [data-fui-devtools-dialog-label="true"]'))
      .find((element) => element.textContent === 'Label');
    const value = document.querySelector<HTMLElement>('#effindom-devtools-debug-dialog [data-fui-devtools-dialog-selected-label="true"]');
    const dialog = document.querySelector<HTMLElement>('#effindom-devtools-debug-dialog');
    if (header === undefined || value === null || dialog === null) {
      throw new Error('Expected label row.');
    }
    const headerRect = header.getBoundingClientRect();
    const valueRect = value.getBoundingClientRect();
    const dialogRect = dialog.getBoundingClientRect();
    return {
      headerRight: headerRect.right,
      valueLeft: valueRect.left,
      valueRight: valueRect.right,
      dialogRight: dialogRect.right,
      valueScrollWidth: value.scrollWidth,
      valueClientWidth: value.clientWidth,
    };
  });
  await expect(labelHeader).toHaveCount(1);
  expect(geometry.valueLeft).toBeGreaterThanOrEqual(geometry.headerRight);
  expect(geometry.valueScrollWidth).toBeGreaterThan(geometry.valueClientWidth);
  expect(geometry.valueRight).toBeLessThanOrEqual(geometry.dialogRight);
});

test('DevTools DOM mirror can select a handle and draw a canvas-pinned overlay', async ({ page }) => {
  await gotoBridgePage(page, '', { devToolsDomMirror: 'on-requested' });

  const rootHandle = await page.evaluate(() => window.__bridgeState?.rootHandle ?? null);
  expect(rootHandle).not.toBeNull();
  if (rootHandle === null) {
    throw new Error('Expected root handle.');
  }

  const selected = await page.evaluate((handle) => (
    window.EffinDomBrowserBridge?.devTools.selectHandle(handle) ?? false
  ), rootHandle);
  expect(selected).toBe(true);

  const mirror = page.locator('#effindom-devtools-dom-mirror');
  const selectedNode = mirror.locator(`[data-fui-handle="${rootHandle}"]`).first();
  const overlay = page.locator('#effindom-devtools-overlay');
  const overlayBox = overlay.locator('[data-fui-devtools-overlay-box="true"]');

  await expect(mirror).toHaveCount(1);
  await expect(selectedNode).toHaveAttribute('data-fui-selected', 'true');
  await expect(overlay).toHaveCount(1);
  await expect(overlay).toHaveAttribute('aria-hidden', 'true');
  await expect(overlay).toHaveAttribute('inert', '');
  await expect(overlayBox).toHaveCSS('display', 'block');
  await expect(overlayBox).toHaveCSS('left', '0px');
  await expect(overlayBox).toHaveCSS('top', '0px');
  await expect(overlayBox).toHaveCSS('width', '322px');
  await expect(overlayBox).toHaveCSS('height', '222px');

  await page.evaluate(() => {
    window.EffinDomBrowserBridge?.devTools.openDebugDialog();
  });
  const dialog = page.locator('#effindom-devtools-debug-dialog');
  await expect(dialog.locator('[data-fui-devtools-dialog-selected-handle="true"]')).toHaveText(rootHandle);
  await expect(dialog.locator('[data-fui-devtools-dialog-selected-type="true"]')).toHaveText('flex-box');

  const selectedHandle = await page.evaluate(() => window.EffinDomBrowserBridge?.devTools.getSelectedHandle() ?? null);
  expect(selectedHandle).toBe(rootHandle);

  await page.evaluate(() => {
    window.EffinDomBrowserBridge?.devTools.clearSelection();
  });
  const selectedByNumber = await page.evaluate((handle) => (
    window.EffinDomBrowserBridge?.devTools.selectHandle(Number(handle)) ?? false
  ), rootHandle);
  expect(selectedByNumber).toBe(true);
  const selectedNumberHandle = await page.evaluate(() => window.EffinDomBrowserBridge?.devTools.getSelectedHandle() ?? null);
  expect(selectedNumberHandle).toBe(rootHandle);

  await page.evaluate(() => {
    window.EffinDomBrowserBridge?.devTools.clearSelection();
  });
  await expect(overlay).toHaveCount(0);
  await expect(selectedNode).not.toHaveAttribute('data-fui-selected', 'true');
  await expect(dialog.locator('[data-fui-devtools-dialog-selected-handle="true"]')).toHaveText('None');
  const clearedHandle = await page.evaluate(() => (
    window.EffinDomBrowserBridge === undefined
      ? 'unexpected'
      : window.EffinDomBrowserBridge.devTools.getSelectedHandle()
  ));
  expect(clearedHandle).toBeNull();
});

test('DevTools DOM mirror specializes semantic control roots', async ({ page }) => {
  await gotoBridgePage(page, '', { devToolsDomMirror: 'enabled' });

  const scene = await buildSemanticScene(page);
  const mirror = page.locator('#effindom-devtools-dom-mirror');
  const buttonNode = mirror.locator(`[data-fui-handle="${scene.buttonHandle}"]`);
  const textboxNode = mirror.locator(`[data-fui-handle="${scene.textboxHandle}"]`);
  const imageNode = mirror.locator(`[data-fui-handle="${scene.imageHandle}"]`);

  await expect(buttonNode).toHaveJSProperty('localName', 'fui-button');
  await expect(buttonNode).toHaveAttribute('data-fui-type', 'button');
  await expect(buttonNode).toHaveAttribute('data-fui-render-node-type', 'flex-box');
  await expect(buttonNode).toHaveAttribute('data-fui-semantic-role-name', 'button');

  await expect(textboxNode).toHaveJSProperty('localName', 'fui-textbox');
  await expect(textboxNode).toHaveAttribute('data-fui-type', 'textbox');
  await expect(textboxNode).toHaveAttribute('data-fui-render-node-type', 'text');
  await expect(textboxNode).toHaveAttribute('data-fui-semantic-role-name', 'textbox');

  await expect(imageNode).toHaveJSProperty('localName', 'fui-image');
  await expect(imageNode).toHaveAttribute('data-fui-type', 'image');
  await expect(imageNode).toHaveAttribute('data-fui-render-node-type', 'flex-box');
  await expect(imageNode).toHaveAttribute('data-fui-semantic-role-name', 'image');
});

test('DevTools DOM mirror scrolls retained children through one content transform', async ({ page }) => {
  await gotoBridgePage(page, '', { devToolsDomMirror: 'enabled' });

  const scene = await buildScrollableStaticTextScene(page, 'Scroll mirror performance '.repeat(20));
  const scrollNode = page.locator(`#effindom-devtools-dom-mirror [data-fui-handle="${scene.scrollHandle}"]`);
  const scrollContent = scrollNode.locator(':scope > fui-scroll-content');
  const textNode = scrollContent.locator(`[data-fui-handle="${scene.textHandle}"]`);

  await expect(scrollNode).toHaveJSProperty('localName', 'fui-scroll-view');
  await expect(scrollContent).toHaveCount(1);
  await expect(textNode).toHaveJSProperty('localName', 'fui-text');
  await expect(textNode).toHaveCount(1);
  const before = await textNode.evaluate((element) => ({
    bounds: element.getAttribute('data-fui-bounds') ?? '',
    scroll: element.parentElement?.parentElement?.getAttribute('data-fui-scroll') ?? '',
    top: (element as HTMLElement).style.top,
    transform: element.parentElement?.style.transform ?? '',
  }));

  const after = await page.evaluate(({ scrollHandle, textHandle }) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    const bridge = window.EffinDomBrowserBridge;
    if (runtime === null || runtime === undefined || bridge === undefined) {
      throw new Error('Expected bridge runtime.');
    }
    const handle = bridge.handleToBigInt(scrollHandle);
    runtime.ui._ui_set_scroll_offset(handle, 0, 24);
    runtime.commitFrame();
    runtime.flushPendingCommit();
    const scroll = document.querySelector<HTMLElement>(`#effindom-devtools-dom-mirror [data-fui-handle="${scrollHandle}"]`);
    const text = document.querySelector<HTMLElement>(`#effindom-devtools-dom-mirror [data-fui-handle="${textHandle}"]`);
    return {
      bounds: text?.getAttribute('data-fui-bounds') ?? '',
      scroll: scroll?.getAttribute('data-fui-scroll') ?? '',
      top: text?.style.top ?? '',
      transform: text?.parentElement?.style.transform ?? '',
    };
  }, scene);

  expect(after.bounds).toBe(before.bounds);
  expect(after.scroll).toBe(before.scroll);
  expect(after.top).toBe(before.top);
  expect(before.transform).toBe('translate(0px, 0px)');
  expect(after.transform).toBe('translate(0px, -24px)');

  await expect(scrollNode).toHaveAttribute('data-fui-scroll', /,24,/);
});

test('DevTools DOM mirror disabled mode creates no mirror and no request shortcut', async ({ page }) => {
  await gotoBridgePage(page, '', { devToolsDomMirror: 'disabled' });

  await expect(page.locator('#effindom-devtools-dom-mirror')).toHaveCount(0);
  await page.keyboard.press('Meta+Shift+F12');
  await expect(page.locator('#effindom-devtools-dom-mirror')).toHaveCount(0);
  await expect(page.locator('#effindom-devtools-debug-dialog')).toHaveCount(0);
  const enableResult = await page.evaluate(() => window.EffinDomBrowserBridge?.devTools.enableDomMirror() ?? true);
  expect(enableResult).toBe(false);
  const openResult = await page.evaluate(() => window.EffinDomBrowserBridge?.devTools.openDebugDialog() ?? true);
  expect(openResult).toBe(false);
  const selectResult = await page.evaluate(() => window.EffinDomBrowserBridge?.devTools.selectHandle('1') ?? true);
  expect(selectResult).toBe(false);
  await expect(page.locator('#effindom-devtools-dom-mirror')).toHaveCount(0);
  await expect(page.locator('#effindom-devtools-overlay')).toHaveCount(0);
});
