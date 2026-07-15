import * as fs from 'node:fs';
import * as path from 'node:path';

import { expect, type Page } from '@playwright/test';

import type { BridgeLoaderInfo } from '../src/core-types';
import type { StaticServerHandle } from '../../ui/tests/integration/helpers/static_server';

declare global {
  interface Window {
    __bridgeReady?: boolean;
    __bridgeError?: string;
    __bridgeState?: {
      readonly commandWordCount: number;
      readonly commandWords: readonly number[];
      readonly rootHandle: string;
    };
    __bridgeLoaderInfo?: BridgeLoaderInfo;
    __bridgeTextByHandle?: Record<string, string>;
    __bridgeSelectionsByHandle?: Record<string, { start: number; end: number }>;
    __bridgeActiveEditorWindow?: {
      readonly handle: string | null;
      readonly text: string;
      readonly docStart: number;
      readonly docEnd: number;
    };
    __bridgeDebug?: { forceDeviceLost(): void };
  }
}

interface RenderedPixel {
  readonly red: number;
  readonly green: number;
  readonly blue: number;
  readonly alpha: number;
}

interface EditableSceneState {
  readonly textHandle: string;
}

interface MultiStaticTextSceneState {
  readonly textHandles: readonly string[];
}

interface EditableSceneOptions {
  readonly multiline?: boolean;
  readonly wrapping?: boolean;
  readonly nodeWidth?: number;
  readonly nodeHeight?: number;
  readonly topSpacerHeight?: number;
}

interface SelectionAreaSceneState {
  readonly areaHandle: string;
  readonly firstHandle: string;
  readonly secondHandle: string;
}

interface BoxSceneState {
  readonly boxHandle: string;
}

interface ScrollSceneState {
  readonly scrollHandle: string;
}

interface NestedProxyScrollSceneState {
  readonly outerScrollHandle: string;
  readonly innerScrollHandle: string;
  readonly proxyHandle: string;
}

interface SemanticSceneState {
  readonly buttonHandle: string;
  readonly textboxHandle: string;
  readonly imageHandle: string;
}

interface ClippedSemanticSceneState {
  readonly partialHandle: string;
  readonly hiddenHandle: string;
}

interface GlyphRunSnapshot {
  readonly fontId: number;
  readonly glyphCount: number;
  readonly glyphIds: number[];
  readonly glyphFontIds: number[];
  readonly yPositions: number[];
}

interface HighlightRectSnapshot {
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
}

interface ColoredHighlightRectSnapshot extends HighlightRectSnapshot {
  readonly color: number;
}

interface CanvasInkStats {
  readonly nonBackgroundPixelCount: number;
  readonly brightPixelCount: number;
}

interface CanvasColorStats {
  readonly nonBackgroundPixelCount: number;
  readonly chromaticPixelCount: number;
  readonly yellowPixelCount: number;
}

const PUBLIC_DIR = path.join(__dirname, '..', '..', '..', 'public');
const SCREENSHOT_DIR = path.join(__dirname, 'screenshots');
const WRAPPED_TEXT_FIXTURE_PATH = path.join(__dirname, '..', '..', 'ui', 'tests', 'fixtures', 'wrapped_large_document.txt');
const CMD_CREATE_NODE = 1;
const CMD_DELETE_NODE = 2;
const CMD_SET_BOUNDS = 10;
const CMD_SET_BOX_STYLE = 20;
const CMD_SET_GLYPH_RUN = 40;
const CMD_SET_HIGHLIGHTS = 43;
const CMD_SET_GLYPH_RUN_COLORED = 44;
const CMD_SET_HIGHLIGHTS_COLORED = 45;
const CMD_COMMIT_PAINT_ORDER = 98;
const CMD_COMMIT_SCENE = 99;

let baseUrl: string;

function getBaseUrl(): string {
  return baseUrl;
}

function screenshotPath(name: string): string {
  fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });
  return path.join(SCREENSHOT_DIR, name);
}

function readWrappedTextFixture(): string {
  return fs.readFileSync(WRAPPED_TEXT_FIXTURE_PATH, 'utf8');
}

function getWrappedTextFixtureTargets(text: string): {
  readonly reverseSelectionStart: number;
  readonly blockStart: number;
  readonly blockEnd: number;
} {
  const reverseSelectionStart = text.lastIndexOf('\n    for (') >= 0
    ? text.lastIndexOf('\n    for (') + 1
    : text.lastIndexOf('for (');
  expect(reverseSelectionStart).toBeGreaterThan(5000);

  const blockStart =
    text.lastIndexOf("        if (ch == '\\r' && index + 1U < utf8.size() && utf8[index + 1U] == '\\n') {");
  expect(blockStart).toBeGreaterThan(5000);

  const blockEnd = text.indexOf('        segment_start = ', blockStart);
  expect(blockEnd).toBeGreaterThan(blockStart);

  return {
    reverseSelectionStart,
    blockStart,
    blockEnd,
  };
}

async function readScenePixel(page: Page, x: number, y: number): Promise<RenderedPixel> {
  return await page.evaluate(async ({ sampleX, sampleY }) => {
    const overlay = document.querySelector('[data-effindom-software-overlay="true"]');
    if (overlay instanceof HTMLCanvasElement) {
      // For CPU software rendering: read directly from the overlay's 2D context.
      const ctx = overlay.getContext('2d');
      if (ctx !== null) {
        const clampedX = Math.max(0, Math.min(overlay.width - 1, Math.round(sampleX)));
        const clampedY = Math.max(0, Math.min(overlay.height - 1, Math.round(sampleY)));
        const pixel = ctx.getImageData(clampedX, clampedY, 1, 1).data;
        return {
          red: pixel[0] ?? 0,
          green: pixel[1] ?? 0,
          blue: pixel[2] ?? 0,
          alpha: pixel[3] ?? 0,
        };
      }
    }

    // Fallback: GPU canvas readback via toDataURL().
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
      red: pixel[0] ?? 0,
      green: pixel[1] ?? 0,
      blue: pixel[2] ?? 0,
      alpha: pixel[3] ?? 0,
    };
  }, { sampleX: x, sampleY: y });
}

function decodeFloat32(word: number): number {
  const buffer = new ArrayBuffer(4);
  const view = new DataView(buffer);
  view.setUint32(0, word >>> 0, true);
  return view.getFloat32(0, true);
}

function parseGlyphRuns(words: readonly number[]): GlyphRunSnapshot[] {
  const runs: GlyphRunSnapshot[] = [];
  for (let index = 0; index < words.length; index += 1) {
    if (words[index] !== CMD_SET_GLYPH_RUN) {
      continue;
    }
    const glyphCount = words[index + 6] ?? 0;
    const glyphIds: number[] = [];
    const glyphFontIds: number[] = [];
    const yPositions: number[] = [];
    for (let glyphIndex = 0; glyphIndex < glyphCount; glyphIndex += 1) {
      const glyphIdWord = words[index + 7 + (glyphIndex * 4)];
      const fontWord = words[index + 10 + (glyphIndex * 4)];
      const yWord = words[index + 9 + (glyphIndex * 4)];
      if (glyphIdWord !== undefined) {
        glyphIds.push(glyphIdWord);
      }
      if (fontWord !== undefined) {
        glyphFontIds.push(fontWord);
      }
      if (yWord !== undefined) {
        yPositions.push(decodeFloat32(yWord));
      }
    }
    runs.push({
      fontId: words[index + 3] ?? 0,
      glyphCount,
      glyphIds,
      glyphFontIds,
      yPositions,
    });
    index += 6 + (glyphCount * 4);
  }
  return runs;
}

function parseColoredHighlightRects(words: readonly number[], targetHandle: string): ColoredHighlightRectSnapshot[] {
  const target = BigInt(targetHandle);
  const rects: ColoredHighlightRectSnapshot[] = [];
  for (let index = 0; index < words.length; index += 1) {
    const opcode = words[index];
    if (opcode === undefined) {
      break;
    }
    if (opcode === CMD_SET_HIGHLIGHTS) {
      const handle = (BigInt(words[index + 2] ?? 0) << 32n) | BigInt(words[index + 1] ?? 0);
      const rectCount = words[index + 4] ?? 0;
      const color = words[index + 3] ?? 0;
      if (handle === target) {
        for (let rectIndex = 0; rectIndex < rectCount; rectIndex += 1) {
          const base = index + 5 + (rectIndex * 4);
          rects.push({
            x: decodeFloat32(words[base] ?? 0),
            y: decodeFloat32(words[base + 1] ?? 0),
            width: decodeFloat32(words[base + 2] ?? 0),
            height: decodeFloat32(words[base + 3] ?? 0),
            color,
          });
        }
        return rects;
      }
      index += 4 + (rectCount * 4);
      continue;
    }
    if (opcode === CMD_SET_HIGHLIGHTS_COLORED) {
      const handle = (BigInt(words[index + 2] ?? 0) << 32n) | BigInt(words[index + 1] ?? 0);
      const rectCount = words[index + 3] ?? 0;
      if (handle === target) {
        for (let rectIndex = 0; rectIndex < rectCount; rectIndex += 1) {
          const base = index + 4 + (rectIndex * 5);
          rects.push({
            x: decodeFloat32(words[base] ?? 0),
            y: decodeFloat32(words[base + 1] ?? 0),
            width: decodeFloat32(words[base + 2] ?? 0),
            height: decodeFloat32(words[base + 3] ?? 0),
            color: words[base + 4] ?? 0,
          });
        }
        return rects;
      }
      index += 3 + (rectCount * 5);
      continue;
    }
    if (opcode === CMD_CREATE_NODE || opcode === CMD_DELETE_NODE) {
      index += 2;
      continue;
    }
    if (opcode === CMD_SET_BOUNDS) {
      index += 15;
      continue;
    }
    if (opcode === CMD_SET_BOX_STYLE) {
      index += 12;
      continue;
    }
    if (opcode === 21 || opcode === CMD_SET_GLYPH_RUN_COLORED) {
      index += 5;
      continue;
    }
    if (opcode === 46) {
      index += 7;
      continue;
    }
    if (opcode === 30 || opcode === 31) {
      index += 4;
      continue;
    }
    if (opcode === 32) {
      index += 7;
      continue;
    }
    if (opcode === 41) {
      index += 3;
      continue;
    }
    if (opcode === 42) {
      index += 7;
      continue;
    }
    if (opcode === CMD_SET_GLYPH_RUN) {
      index += 6 + ((words[index + 6] ?? 0) * 4);
      continue;
    }
    if (opcode === CMD_COMMIT_PAINT_ORDER) {
      index += 1 + ((words[index + 1] ?? 0) * 2);
      continue;
    }
    if (opcode === CMD_COMMIT_SCENE) {
      index += 1 + ((words[index + 1] ?? 0) * 5);
      continue;
    }
    break;
  }
  return rects;
}

function parseHighlightRects(words: readonly number[], targetHandle: string): HighlightRectSnapshot[] {
  return parseColoredHighlightRects(words, targetHandle).map((rect) => ({
    x: rect.x,
    y: rect.y,
    width: rect.width,
    height: rect.height,
  }));
}

async function getWrappedTextIndexPoint(
  page: Page,
  textHandle: string,
  targetIndex: number,
  logicalWidth: number,
  logicalHeight: number,
): Promise<{ readonly x: number; readonly y: number }> {
  const logicalPoint = await page.evaluate(({ textHandle, targetIndex }) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    if (runtime === null || runtime === undefined) {
      throw new Error('Expected bridge runtime.');
    }
    const bridge = window.EffinDomBrowserBridge;
    if (bridge === undefined) {
      throw new Error('Bridge state is not ready.');
    }
    const ui = runtime.ui;
    const handleArg = bridge.handleToBigInt(textHandle);
    ui._ui_set_interaction_time(BigInt(Math.floor(performance.now())));
    ui._ui_request_focus(handleArg);
    runtime.commitFrame();
    runtime.flushPendingCommit();

    const readRangePoint = (start: number, end: number): { x: number; y: number } | null => {
      if (start === end) {
        return null;
      }
      const rectCount = ui._ui_get_text_range_rect_count(handleArg, start, end);
      if (rectCount <= 0) {
        return null;
      }
      const rectWordsPtr = runtime.ui._malloc(rectCount * 4 * 4);
      try {
        const copiedCount = ui._ui_copy_text_range_rects(handleArg, start, end, rectWordsPtr, rectCount);
        if (copiedCount > 0) {
          const base = Number(rectWordsPtr) >>> 2;
          const rectX = runtime.ui.HEAPF32[base] ?? 0;
          const rectY = runtime.ui.HEAPF32[base + 1] ?? 0;
          const rectWidth = runtime.ui.HEAPF32[base + 2] ?? 0;
          const rectHeight = runtime.ui.HEAPF32[base + 3] ?? 0;
          return {
            x: rectX + (rectWidth * 0.5),
            y: rectY + (rectHeight * 0.5),
          };
        }
      } finally {
        runtime.ui._free(rectWordsPtr);
      }
      return null;
    };

    const rangePoint = readRangePoint(targetIndex, targetIndex + 1) ?? readRangePoint(Math.max(0, targetIndex - 1), targetIndex);
    if (rangePoint !== null) {
      return rangePoint;
    }

    const baseX = 0;
    const baseY = 0;
    for (let radius = 0; radius <= 8; radius += 1) {
      const minY = Math.round(baseY) - radius;
      const maxY = Math.round(baseY) + radius;
      const minX = Math.round(baseX) - radius;
      const maxX = Math.round(baseX) + radius;
      for (let y = minY; y <= maxY; y += 1) {
        for (let x = minX; x <= maxX; x += 1) {
          if (runtime.getHandleFromPoint(x, y).toString() === textHandle) {
            return { x, y };
          }
        }
      }
    }
    return { x: baseX, y: baseY };
  }, { textHandle, targetIndex });

  const canvas = page.locator('#fui-canvas');
  const canvasBox = await canvas.boundingBox();
  expect(canvasBox).not.toBeNull();
  if (canvasBox === null) {
    throw new Error('Expected scene canvas bounds.');
  }

  const scaleX = canvasBox.width / logicalWidth;
  const scaleY = canvasBox.height / logicalHeight;
  return {
    x: canvasBox.x + (logicalPoint.x * scaleX),
    y: canvasBox.y + (logicalPoint.y * scaleY),
  };
}

async function getWrappedSelectionDragPoints(
  page: Page,
  textHandle: string,
  start: number,
  end: number,
  logicalWidth: number,
  logicalHeight: number,
): Promise<{
  readonly forwardStart: { readonly x: number; readonly y: number };
  readonly forwardEnd: { readonly x: number; readonly y: number };
}> {
  const words = await page.evaluate(({ textHandle, start, end }) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    if (runtime === null || runtime === undefined) {
      throw new Error('Expected bridge runtime.');
    }
    const bridge = window.EffinDomBrowserBridge;
    if (bridge === undefined) {
      throw new Error('Bridge state is not ready.');
    }
    const ui = runtime.ui;
    const handleArg = bridge.handleToBigInt(textHandle);
    ui._ui_set_interaction_time(BigInt(Math.floor(performance.now())));
    ui._ui_request_focus(handleArg);
    runtime.commitFrame();
    runtime.flushPendingCommit();
    ui._ui_set_text_selection_range(handleArg, start, end);
    runtime.commitFrame();
    runtime.flushPendingCommit();
    return Array.from(runtime.extractCommandBuffer());
  }, { textHandle, start, end });

  const highlightRects = parseHighlightRects(words, textHandle);
  expect(highlightRects.length).toBeGreaterThan(0);
  const firstRect = highlightRects[0];
  const lastRect = highlightRects[highlightRects.length - 1];
  if (firstRect === undefined || lastRect === undefined) {
    throw new Error('Expected selection highlight rects.');
  }

  const canvas = page.locator('#fui-canvas');
  const canvasBox = await canvas.boundingBox();
  expect(canvasBox).not.toBeNull();
  if (canvasBox === null) {
    throw new Error('Expected scene canvas bounds.');
  }

  const scaleX = canvasBox.width / logicalWidth;
  const scaleY = canvasBox.height / logicalHeight;
  return {
    forwardStart: {
      x: canvasBox.x + ((firstRect.x + 1) * scaleX),
      y: canvasBox.y + ((firstRect.y + (firstRect.height * 0.5)) * scaleY),
    },
    forwardEnd: {
      x: canvasBox.x + ((lastRect.x + Math.max(1, lastRect.width - 1)) * scaleX),
      y: canvasBox.y + ((lastRect.y + (lastRect.height * 0.5)) * scaleY),
    },
  };
}

interface RuntimeConfigOverrides {
  readonly manifestUrls?: readonly string[];
  readonly expectedRuntimeSetHash?: string;
  readonly buildMode?: 'debug' | 'release';
  readonly devToolsDomMirror?: 'disabled' | 'enabled' | 'on-requested';
  readonly pageZoom?: 'disabled' | 'enabled';
}

async function gotoBridgePage(page: Page, query = '', runtimeConfig: RuntimeConfigOverrides = {}): Promise<void> {
  await page.addInitScript((config: RuntimeConfigOverrides) => {
    window.__effindomRuntime = {
      manifestUrls: ['/v2/browser-bridge/effindom.v2.manifest.json'],
      ...config,
    };
  }, runtimeConfig);
  await page.goto(`${baseUrl}/v2/browser-bridge/index.html${query}`);
  await page.waitForFunction(() => window.__bridgeReady === true || typeof window.__bridgeError === 'string');
  const error = await page.evaluate(() => window.__bridgeError ?? null);
  expect(error).toBeNull();
}

async function readActiveRenderer(page: Page): Promise<BridgeLoaderInfo['activeRenderer'] | null> {
  return await page.evaluate(() => window.__bridgeLoaderInfo?.activeRenderer ?? null);
}

async function buildEditableTextScene(
  page: Page,
  text: string,
  fontId = 1,
  options: EditableSceneOptions = {},
): Promise<EditableSceneState> {
  return await page.evaluate((scene) => {
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
    const textNode = toHandle(ui._ui_create_node(1));
    ui._ui_set_root(root);
    ui._ui_node_add_child(root, textNode);
    ui._ui_set_width(root, 320, 0);
    ui._ui_set_height(root, 220, 0);
    ui._ui_set_bg_color(root, 0x111827ff);
    ui._ui_set_width(textNode, scene.nodeWidth, 0);
    ui._ui_set_height(textNode, scene.nodeHeight, 0);
    ui._ui_set_font(textNode, scene.fontId, 24);
    ui._ui_set_semantic_role(textNode, 2);
    ui._ui_set_interactive(textNode, 1);
    ui._ui_set_focusable(textNode, 1, 0);
    ui._ui_set_selectable(textNode, 1, 0x93c5fdff);
    ui._ui_set_editable(textNode, 1);
    ui._ui_set_caret_color(textNode, 0xffffffff);
    ui._ui_set_text_limits(textNode, 2147483647, scene.multiline ? 0 : 1);
    ui._ui_set_text_wrapping(textNode, scene.wrapping ? 1 : 0);

    const heapText = writeText(scene.text);
    try {
      ui._ui_set_text(textNode, heapText.ptr, heapText.len);
    } finally {
      if (heapText.offset !== 0) {
        ui._free(heapText.ptr);
      }
    }

    runtime.commitFrame();
    runtime.flushPendingCommit();
    runtime.resetLogs();

    return { textHandle: textNode };
  }, {
    text,
    fontId,
    multiline: options.multiline ?? false,
    wrapping: options.wrapping ?? true,
    nodeWidth: options.nodeWidth ?? 260,
    nodeHeight: options.nodeHeight ?? 42,
  });
}

async function buildReadonlyTextScene(
  page: Page,
  text: string,
  fontId = 1,
): Promise<EditableSceneState> {
  return await page.evaluate((scene) => {
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
    const textNode = toHandle(ui._ui_create_node(1));
    ui._ui_set_root(root);
    ui._ui_node_add_child(root, textNode);
    ui._ui_set_width(root, 320, 0);
    ui._ui_set_height(root, 220, 0);
    ui._ui_set_bg_color(root, 0x111827ff);
    ui._ui_set_width(textNode, 260, 0);
    ui._ui_set_height(textNode, 42, 0);
    ui._ui_set_font(textNode, scene.fontId, 24);
    ui._ui_set_semantic_role(textNode, 2);
    ui._ui_set_interactive(textNode, 1);
    ui._ui_set_focusable(textNode, 1, 0);
    ui._ui_set_selectable(textNode, 1, 0x93c5fdff);
    ui._ui_set_editable(textNode, 0);
    ui._ui_set_caret_color(textNode, 0xffffffff);
    ui._ui_set_text_limits(textNode, 2147483647, 1);
    ui._ui_set_text_wrapping(textNode, 1);

    const heapText = writeText(scene.text);
    try {
      ui._ui_set_text(textNode, heapText.ptr, heapText.len);
    } finally {
      if (heapText.offset !== 0) {
        ui._free(heapText.ptr);
      }
    }

    runtime.commitFrame();
    runtime.flushPendingCommit();
    runtime.resetLogs();

    return { textHandle: textNode };
  }, { text, fontId });
}

async function buildStaticTextScene(
  page: Page,
  text: string,
  fontId = 1,
): Promise<EditableSceneState> {
  return await page.evaluate((scene) => {
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
    const textNode = toHandle(ui._ui_create_node(1));
    ui._ui_set_root(root);
    ui._ui_node_add_child(root, textNode);
    ui._ui_set_width(root, 320, 0);
    ui._ui_set_height(root, 220, 0);
    ui._ui_set_bg_color(root, 0x111827ff);
    ui._ui_set_width(textNode, 260, 0);
    ui._ui_set_height(textNode, 80, 0);
    ui._ui_set_font(textNode, scene.fontId, 24);
    ui._ui_set_selectable(textNode, 1, 0x93c5fdff);
    ui._ui_set_text_wrapping(textNode, 1);

    const heapText = writeText(scene.text);
    try {
      ui._ui_set_text(textNode, heapText.ptr, heapText.len);
    } finally {
      if (heapText.offset !== 0) {
        ui._free(heapText.ptr);
      }
    }

    runtime.commitFrame();
    runtime.flushPendingCommit();
    runtime.resetLogs();

    return { textHandle: textNode };
  }, { text, fontId });
}

async function buildMultiStaticTextScene(
  page: Page,
  texts: readonly string[],
  fontId = 1,
): Promise<MultiStaticTextSceneState> {
  return await page.evaluate((scene) => {
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
    ui._ui_set_root(root);
    ui._ui_set_width(root, 320, 0);
    ui._ui_set_height(root, 220, 0);
    ui._ui_set_bg_color(root, 0x111827ff);

    const textHandles: string[] = [];
    for (const value of scene.texts) {
      const textNode = toHandle(ui._ui_create_node(1));
      textHandles.push(textNode);
      ui._ui_node_add_child(root, textNode);
      ui._ui_set_width(textNode, 260, 0);
      ui._ui_set_height(textNode, 56, 0);
      ui._ui_set_font(textNode, scene.fontId, 24);
      ui._ui_set_selectable(textNode, 1, 0x93c5fdff);
      ui._ui_set_text_wrapping(textNode, 1);

      const heapText = writeText(value);
      try {
        ui._ui_set_text(textNode, heapText.ptr, heapText.len);
      } finally {
        if (heapText.offset !== 0) {
          ui._free(heapText.ptr);
        }
      }
    }

    runtime.commitFrame();
    runtime.flushPendingCommit();
    runtime.resetLogs();

    return { textHandles };
  }, { texts, fontId });
}

async function buildScrollableEditableTextScene(
  page: Page,
  text: string,
  fontId = 1,
  options: EditableSceneOptions = {},
): Promise<{ readonly textHandle: string; readonly scrollHandle: string }> {
  return await page.evaluate((scene) => {
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
    const scroll = toHandle(ui._ui_create_node(4));
    const textNode = toHandle(ui._ui_create_node(1));
    ui._ui_set_root(root);
    ui._ui_resize_window(140, 80);
    ui._ui_node_add_child(root, scroll);
    if (scene.topSpacerHeight > 0) {
      const content = toHandle(ui._ui_create_node(0));
      const spacer = toHandle(ui._ui_create_node(0));
      ui._ui_node_add_child(scroll, content);
      ui._ui_node_add_child(content, spacer);
      ui._ui_node_add_child(content, textNode);
      ui._ui_set_width(content, 120, 0);
      ui._ui_set_height(content, scene.topSpacerHeight + scene.nodeHeight, 0);
      ui._ui_set_width(spacer, 120, 0);
      ui._ui_set_height(spacer, scene.topSpacerHeight, 0);
    } else {
      ui._ui_node_add_child(scroll, textNode);
    }
    ui._ui_set_width(root, 140, 0);
    ui._ui_set_height(root, 80, 0);
    ui._ui_set_width(scroll, 120, 0);
    ui._ui_set_height(scroll, 60, 0);
    ui._ui_set_width(textNode, scene.nodeWidth, 0);
    ui._ui_set_height(textNode, scene.nodeHeight, 0);
    ui._ui_set_font(textNode, scene.fontId, 20);
    ui._ui_set_semantic_role(textNode, 2);
    ui._ui_set_interactive(textNode, 1);
    ui._ui_set_focusable(textNode, 1, 0);
    ui._ui_set_selectable(textNode, 1, 0x93c5fdff);
    ui._ui_set_editable(textNode, 1);
    ui._ui_set_caret_color(textNode, 0xffffffff);
    ui._ui_set_text_limits(textNode, 2147483647, scene.multiline ? 0 : 1);
    ui._ui_set_text_wrapping(textNode, scene.wrapping ? 1 : 0);

    const heapText = writeText(scene.text);
    try {
      ui._ui_set_text(textNode, heapText.ptr, heapText.len);
    } finally {
      if (heapText.offset !== 0) {
        ui._free(heapText.ptr);
      }
    }

    runtime.commitFrame();
    runtime.flushPendingCommit();
    runtime.resetLogs();

    return { textHandle: textNode, scrollHandle: scroll };
  }, {
    text,
    fontId,
    multiline: options.multiline ?? true,
    wrapping: options.wrapping ?? true,
    nodeWidth: options.nodeWidth ?? 120,
    nodeHeight: options.nodeHeight ?? 320,
    topSpacerHeight: options.topSpacerHeight ?? 0,
  });
}

async function buildScrollableStaticTextScene(
  page: Page,
  text: string,
  fontId = 1,
): Promise<{ readonly textHandle: string; readonly scrollHandle: string }> {
  return await page.evaluate((scene) => {
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
    const scroll = toHandle(ui._ui_create_node(4));
    const textNode = toHandle(ui._ui_create_node(1));
    ui._ui_set_root(root);
    ui._ui_resize_window(140, 80);
    ui._ui_node_add_child(root, scroll);
    ui._ui_node_add_child(scroll, textNode);
    ui._ui_set_width(root, 140, 0);
    ui._ui_set_height(root, 80, 0);
    ui._ui_set_width(scroll, 120, 0);
    ui._ui_set_height(scroll, 60, 0);
    ui._ui_set_width(textNode, 120, 0);
    ui._ui_set_height(textNode, 960, 0);
    ui._ui_set_font(textNode, scene.fontId, 20);
    ui._ui_set_selectable(textNode, 1, 0x93c5fdff);
    ui._ui_set_text_wrapping(textNode, 1);

    const heapText = writeText(scene.text);
    try {
      ui._ui_set_text(textNode, heapText.ptr, heapText.len);
    } finally {
      if (heapText.offset !== 0) {
        ui._free(heapText.ptr);
      }
    }

    runtime.commitFrame();
    runtime.flushPendingCommit();
    runtime.resetLogs();

    return { textHandle: textNode, scrollHandle: scroll };
  }, { text, fontId });
}

async function buildSelectionAreaScene(page: Page, fontId = 1): Promise<SelectionAreaSceneState> {
  return await page.evaluate(async (scene) => {
    const runtime = window.EffinDomBrowserBridge?.getRuntime();
    if (runtime === null || runtime === undefined) {
      throw new Error('Bridge runtime is not ready.');
    }

    await runtime.loadFont(scene.fontId, './DejaVuSans.ttf');

    const ui = runtime.ui;
    const bridge = window.EffinDomBrowserBridge;
    if (bridge === undefined) {
      throw new Error('Bridge state is not ready.');
    }
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
    const area = toHandle(ui._ui_create_node(0));
    const first = toHandle(ui._ui_create_node(1));
    const second = toHandle(ui._ui_create_node(1));
    ui._ui_set_root(root);
    ui._ui_resize_window(220, 140);
    ui._ui_node_add_child(root, area);
    ui._ui_node_add_child(area, first);
    ui._ui_node_add_child(area, second);
    ui._ui_set_width(root, 220, 0);
    ui._ui_set_height(root, 140, 0);
    ui._ui_set_width(area, 180, 0);
    ui._ui_set_selection_area(area, 1);
    for (const handle of [first, second]) {
      ui._ui_set_width(handle, 180, 0);
      ui._ui_set_font(handle, scene.fontId, 20);
      ui._ui_set_semantic_role(handle, 2);
      ui._ui_set_interactive(handle, 1);
      ui._ui_set_focusable(handle, 1, 0);
      ui._ui_set_selectable(handle, 1, 0x93c5fdff);
    }

    const firstText = writeText('First paragraph');
    const secondText = writeText('Second paragraph');
    try {
      ui._ui_set_text(first, firstText.ptr, firstText.len);
      ui._ui_set_text(second, secondText.ptr, secondText.len);
    } finally {
      if (firstText.offset !== 0) {
        ui._free(firstText.ptr);
      }
      if (secondText.offset !== 0) {
        ui._free(secondText.ptr);
      }
    }

    runtime.commitFrame();
    runtime.flushPendingCommit();
    runtime.resetLogs();

    return {
      areaHandle: area,
      firstHandle: first,
      secondHandle: second,
    };
  }, { fontId });
}

async function buildSemanticScene(page: Page): Promise<SemanticSceneState> {
  return await page.evaluate(() => {
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
    const button = toHandle(ui._ui_create_node(0));
    const textbox = toHandle(ui._ui_create_node(1));
    const image = toHandle(ui._ui_create_node(0));

    ui._ui_set_root(root);
    ui._ui_node_add_child(root, button);
    ui._ui_node_add_child(root, textbox);
    ui._ui_node_add_child(root, image);
    ui._ui_set_width(root, 320, 0);
    ui._ui_set_height(root, 220, 0);
    ui._ui_set_bg_color(root, 0x111827ff);

    ui._ui_set_width(button, 100, 0);
    ui._ui_set_height(button, 32, 0);
    ui._ui_set_bg_color(button, 0x2563ebff);
    ui._ui_set_semantic_role(button, 1);
    const buttonLabel = writeText('Submit');
    try {
      ui._ui_set_semantic_label(button, buttonLabel.ptr, buttonLabel.len);
    } finally {
      if (buttonLabel.offset !== 0) {
        ui._free(buttonLabel.ptr);
      }
    }

    ui._ui_set_width(textbox, 180, 0);
    ui._ui_set_height(textbox, 34, 0);
    ui._ui_set_font(textbox, 1, 18);
    ui._ui_set_semantic_role(textbox, 2);
    ui._ui_set_interactive(textbox, 1);
    ui._ui_set_focusable(textbox, 1, 0);
    ui._ui_set_editable(textbox, 1);
    const textValue = writeText('Email');
    try {
      ui._ui_set_text(textbox, textValue.ptr, textValue.len);
    } finally {
      if (textValue.offset !== 0) {
        ui._free(textValue.ptr);
      }
    }

    ui._ui_set_width(image, 96, 0);
    ui._ui_set_height(image, 64, 0);
    ui._ui_set_semantic_role(image, 8);
    const imageLabel = writeText('Preview');
    try {
      ui._ui_set_semantic_label(image, imageLabel.ptr, imageLabel.len);
    } finally {
      if (imageLabel.offset !== 0) {
        ui._free(imageLabel.ptr);
      }
    }

    runtime.commitFrame();
    runtime.flushPendingCommit();
    runtime.resetLogs();

    return { buttonHandle: button, textboxHandle: textbox, imageHandle: image };
  });
}

async function buildClippedSemanticScene(page: Page): Promise<ClippedSemanticSceneState> {
  return await page.evaluate(() => {
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
    const clip = toHandle(ui._ui_create_node(0));
    const spacer = toHandle(ui._ui_create_node(0));
    const partial = toHandle(ui._ui_create_node(0));
    const hidden = toHandle(ui._ui_create_node(0));

    ui._ui_set_root(root);
    ui._ui_node_add_child(root, clip);
    ui._ui_node_add_child(clip, spacer);
    ui._ui_node_add_child(clip, partial);
    ui._ui_node_add_child(clip, hidden);
    ui._ui_set_width(root, 160, 0);
    ui._ui_set_height(root, 100, 0);
    ui._ui_set_width(clip, 80, 0);
    ui._ui_set_height(clip, 40, 0);
    ui._ui_set_clip_to_bounds(clip, 1);
    ui._ui_set_width(spacer, 80, 0);
    ui._ui_set_height(spacer, 28, 0);

    ui._ui_set_width(partial, 80, 0);
    ui._ui_set_height(partial, 20, 0);
    ui._ui_set_semantic_role(partial, 1);
    const partialLabel = writeText('Clipped');
    try {
      ui._ui_set_semantic_label(partial, partialLabel.ptr, partialLabel.len);
    } finally {
      if (partialLabel.offset !== 0) {
        ui._free(partialLabel.ptr);
      }
    }

    ui._ui_set_width(hidden, 80, 0);
    ui._ui_set_height(hidden, 20, 0);
    ui._ui_set_semantic_role(hidden, 1);
    const hiddenLabel = writeText('Hidden');
    try {
      ui._ui_set_semantic_label(hidden, hiddenLabel.ptr, hiddenLabel.len);
    } finally {
      if (hiddenLabel.offset !== 0) {
        ui._free(hiddenLabel.ptr);
      }
    }

    runtime.commitFrame();
    runtime.flushPendingCommit();
    runtime.resetLogs();

    return {
      partialHandle: partial,
      hiddenHandle: hidden,
    };
  });
}

async function buildScrollableSemanticOrderScene(page: Page): Promise<{ scrollHandle: string }> {
  return await page.evaluate(() => {
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
    const assignSemanticLabel = (handle: string, label: string): void => {
      const text = writeText(label);
      try {
        ui._ui_set_semantic_label(handle, text.ptr, text.len);
      } finally {
        if (text.offset !== 0) {
          ui._free(text.ptr);
        }
      }
    };

    runtime.resetLogs();
    runtime.resetAppSession();
    ui._ui_reset();

    const root = toHandle(ui._ui_create_node(0));
    const scroll = toHandle(ui._ui_create_node(4));
    const content = toHandle(ui._ui_create_node(0));
    const first = toHandle(ui._ui_create_node(0));
    const second = toHandle(ui._ui_create_node(0));
    const third = toHandle(ui._ui_create_node(0));

    ui._ui_set_root(root);
    ui._ui_node_add_child(root, scroll);
    ui._ui_node_add_child(scroll, content);
    ui._ui_node_add_child(content, first);
    ui._ui_node_add_child(content, second);
    ui._ui_node_add_child(content, third);

    ui._ui_set_width(root, 160, 0);
    ui._ui_set_height(root, 48, 0);
    ui._ui_set_width(scroll, 160, 0);
    ui._ui_set_height(scroll, 40, 0);
    ui._ui_set_scroll_enabled(scroll, 0, 1);
    ui._ui_set_width(content, 160, 0);
    ui._ui_set_height(content, 60, 0);

    for (const handle of [first, second, third]) {
      ui._ui_set_width(handle, 160, 0);
      ui._ui_set_height(handle, 20, 0);
      ui._ui_set_semantic_role(handle, 1);
    }

    assignSemanticLabel(first, 'First');
    assignSemanticLabel(second, 'Second');
    assignSemanticLabel(third, 'Third');

    runtime.commitFrame();
    runtime.flushPendingCommit();
    runtime.resetLogs();

    return { scrollHandle: scroll };
  });
}

async function buildInteractiveBoxScene(page: Page): Promise<BoxSceneState> {
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
        if (!Number.isInteger(handle)) {
          throw new TypeError(`Cannot convert non-integer handle ${String(handle)} to string.`);
        }
        return BigInt(handle).toString();
      }
      if (typeof handle === 'string') {
        return BigInt(handle).toString();
      }
      if (handle !== null && typeof handle === 'object') {
        const symbolPrimitive = (handle as { [Symbol.toPrimitive]?: (hint: string) => unknown })[Symbol.toPrimitive]?.('default');
        if (typeof symbolPrimitive === 'bigint' || typeof symbolPrimitive === 'number' || typeof symbolPrimitive === 'string') {
          return toHandle(symbolPrimitive);
        }
        if ('valueOf' in handle && typeof handle.valueOf === 'function') {
          return toHandle(handle.valueOf());
        }
      }
      throw new TypeError(`Cannot convert ${String(handle)} to a handle string.`);
    };
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
    ui._ui_set_focusable(box, 1, 0);
    runtime.commitFrame();
    runtime.flushPendingCommit();
    runtime.resetLogs();

    return { boxHandle: box };
  });
}

async function buildScrollScene(page: Page): Promise<ScrollSceneState> {
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
        if (!Number.isInteger(handle)) {
          throw new TypeError(`Cannot convert non-integer handle ${String(handle)} to string.`);
        }
        return BigInt(handle).toString();
      }
      if (typeof handle === 'string') {
        return BigInt(handle).toString();
      }
      if (handle !== null && typeof handle === 'object') {
        const symbolPrimitive = (handle as { [Symbol.toPrimitive]?: (hint: string) => unknown })[Symbol.toPrimitive]?.('default');
        if (typeof symbolPrimitive === 'bigint' || typeof symbolPrimitive === 'number' || typeof symbolPrimitive === 'string') {
          return toHandle(symbolPrimitive);
        }
        if ('valueOf' in handle && typeof handle.valueOf === 'function') {
          return toHandle(handle.valueOf());
        }
      }
      throw new TypeError(`Cannot convert ${String(handle)} to a handle string.`);
    };

    runtime.resetLogs();
    runtime.resetAppSession();
    ui._ui_reset();

    const root = toHandle(ui._ui_create_node(0));
    const scroll = toHandle(ui._ui_create_node(4));
    const content = toHandle(ui._ui_create_node(0));
    ui._ui_set_root(root);
    ui._ui_node_add_child(root, scroll);
    ui._ui_node_add_child(scroll, content);
    ui._ui_set_width(root, 320, 0);
    ui._ui_set_height(root, 220, 0);
    ui._ui_set_bg_color(root, 0x111827ff);
    ui._ui_set_width(scroll, 200, 0);
    ui._ui_set_height(scroll, 120, 0);
    ui._ui_set_bg_color(scroll, 0x1f2937ff);
    ui._ui_set_width(content, 200, 0);
    ui._ui_set_height(content, 420, 0);
    ui._ui_set_bg_color(content, 0x2563ebff);
    runtime.commitFrame();
    runtime.flushPendingCommit();
    runtime.resetLogs();

    return { scrollHandle: scroll };
  });
}

async function buildNestedProxyScrollScene(page: Page): Promise<NestedProxyScrollSceneState> {
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
        if (!Number.isInteger(handle)) {
          throw new TypeError(`Cannot convert non-integer handle ${String(handle)} to string.`);
        }
        return BigInt(handle).toString();
      }
      if (typeof handle === 'string') {
        return BigInt(handle).toString();
      }
      if (handle !== null && typeof handle === 'object') {
        const symbolPrimitive = (handle as { [Symbol.toPrimitive]?: (hint: string) => unknown })[Symbol.toPrimitive]?.('default');
        if (typeof symbolPrimitive === 'bigint' || typeof symbolPrimitive === 'number' || typeof symbolPrimitive === 'string') {
          return toHandle(symbolPrimitive);
        }
        if ('valueOf' in handle && typeof handle.valueOf === 'function') {
          return toHandle(handle.valueOf());
        }
      }
      throw new TypeError(`Cannot convert ${String(handle)} to a handle string.`);
    };

    runtime.resetLogs();
    runtime.resetAppSession();
    ui._ui_reset();

    const root = toHandle(ui._ui_create_node(0));
    const outerScroll = toHandle(ui._ui_create_node(4));
    const outerContent = toHandle(ui._ui_create_node(0));
    const proxyOwner = toHandle(ui._ui_create_node(0));
    const innerScroll = toHandle(ui._ui_create_node(4));
    const innerContent = toHandle(ui._ui_create_node(0));
    const spacer = toHandle(ui._ui_create_node(0));

    ui._ui_set_root(root);
    ui._ui_resize_window(220, 180);
    ui._ui_node_add_child(root, outerScroll);
    ui._ui_node_add_child(outerScroll, outerContent);
    ui._ui_node_add_child(outerContent, proxyOwner);
    ui._ui_node_add_child(outerContent, spacer);
    ui._ui_node_add_child(proxyOwner, innerScroll);
    ui._ui_node_add_child(innerScroll, innerContent);

    ui._ui_set_width(root, 220, 0);
    ui._ui_set_height(root, 180, 0);
    ui._ui_set_bg_color(root, 0x111827ff);

    ui._ui_set_width(outerScroll, 180, 0);
    ui._ui_set_height(outerScroll, 120, 0);
    ui._ui_set_bg_color(outerScroll, 0x1f2937ff);
    ui._ui_set_width(outerContent, 320, 0);
    ui._ui_set_height(outerContent, 320, 0);

    ui._ui_set_width(proxyOwner, 120, 0);
    ui._ui_set_height(proxyOwner, 80, 0);
    ui._ui_set_bg_color(proxyOwner, 0x334155ff);
    ui._ui_set_scroll_proxy_target(proxyOwner, innerScroll);

    ui._ui_set_width(innerScroll, 80, 0);
    ui._ui_set_height(innerScroll, 80, 0);
    ui._ui_set_bg_color(innerScroll, 0x475569ff);
    ui._ui_set_width(innerContent, 220, 0);
    ui._ui_set_height(innerContent, 220, 0);
    ui._ui_set_bg_color(innerContent, 0x2563ebff);

    ui._ui_set_width(spacer, 320, 0);
    ui._ui_set_height(spacer, 240, 0);
    ui._ui_set_bg_color(spacer, 0x0f172aff);

    runtime.commitFrame();
    runtime.flushPendingCommit();
    runtime.resetLogs();

    return {
      outerScrollHandle: outerScroll,
      innerScrollHandle: innerScroll,
      proxyHandle: proxyOwner,
    };
  });
}

async function buildWrappedThaiScene(page: Page): Promise<readonly number[]> {
  return await page.evaluate(async () => {
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

    ui._ui_reset();
    await runtime.loadFont(2, './NotoSansThai-Regular.ttf');
    const root = toHandle(ui._ui_create_node(0));
    const textNode = toHandle(ui._ui_create_node(1));
    ui._ui_set_root(root);
    ui._ui_node_add_child(root, textNode);
    ui._ui_set_width(root, 320, 0);
    ui._ui_set_height(root, 220, 0);
    ui._ui_set_bg_color(root, 0x111827ff);
    ui._ui_set_width(textNode, 108, 0);
    ui._ui_set_height(textNode, 160, 0);
    ui._ui_set_font(textNode, 2, 26);
    ui._ui_set_text_color(textNode, 0xf8fafcff);
    const heapText = writeText('ภาษาไทยภาษาไทยภาษาไทยภาษาไทยภาษาไทยภาษาไทย');
    try {
      ui._ui_set_text(textNode, heapText.ptr, heapText.len);
    } finally {
      if (heapText.offset !== 0) {
        ui._free(heapText.ptr);
      }
    }
    runtime.commitFrame();
    runtime.flushPendingCommit();
    return Array.from(runtime.extractCommandBuffer());
  });
}

async function buildColorEmojiScene(
  page: Page,
  options: {
    readonly loadFontId: number | null;
    readonly textFontId: number;
  },
): Promise<readonly number[]> {
  return await page.evaluate(async ({ loadFontId, textFontId }) => {
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

    ui._ui_reset();
    if (loadFontId !== null) {
      await runtime.loadFont(loadFontId, './NotoColorEmoji.ttf');
    }
    const root = toHandle(ui._ui_create_node(0));
    const textNode = toHandle(ui._ui_create_node(1));
    ui._ui_set_root(root);
    ui._ui_node_add_child(root, textNode);
    ui._ui_set_width(root, 320, 0);
    ui._ui_set_height(root, 220, 0);
    ui._ui_set_bg_color(root, 0x111827ff);
    ui._ui_set_width(textNode, 260, 0);
    ui._ui_set_height(textNode, 160, 0);
    ui._ui_set_font(textNode, textFontId, 96);
    ui._ui_set_text_color(textNode, 0xffffffff);
    const heapText = writeText('😀');
    try {
      ui._ui_set_text(textNode, heapText.ptr, heapText.len);
    } finally {
      if (heapText.offset !== 0) {
        ui._free(heapText.ptr);
      }
    }
    runtime.commitFrame();
    runtime.flushPendingCommit();
    return Array.from(runtime.extractCommandBuffer());
  }, options);
}

async function readCanvasInkStats(page: Page): Promise<CanvasInkStats> {
  return await page.evaluate(async () => {
    const overlay = document.querySelector('[data-effindom-software-overlay="true"]');
    const canvas = overlay instanceof HTMLCanvasElement ? overlay : document.getElementById('fui-canvas');
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
    const pixels = context.getImageData(0, 0, probe.width, probe.height).data;
    let nonBackgroundPixelCount = 0;
    let brightPixelCount = 0;
    for (let index = 0; index < pixels.length; index += 4) {
      const red = pixels[index] ?? 0;
      const green = pixels[index + 1] ?? 0;
      const blue = pixels[index + 2] ?? 0;
      const alpha = pixels[index + 3] ?? 0;
      if (alpha === 0) {
        continue;
      }
      const isBackground = red < 30 && green < 40 && blue < 50;
      if (!isBackground) {
        nonBackgroundPixelCount += 1;
      }
      const isBright = red > 200 && green > 200 && blue > 200;
      if (isBright) {
        brightPixelCount += 1;
      }
    }
    return { nonBackgroundPixelCount, brightPixelCount };
  });
}

async function readCanvasColorStats(page: Page): Promise<CanvasColorStats> {
  return await page.evaluate(async () => {
    const overlay = document.querySelector('[data-effindom-software-overlay="true"]');
    const canvas = overlay instanceof HTMLCanvasElement ? overlay : document.getElementById('fui-canvas');
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
    const pixels = context.getImageData(0, 0, probe.width, probe.height).data;
    let nonBackgroundPixelCount = 0;
    let chromaticPixelCount = 0;
    let yellowPixelCount = 0;
    for (let index = 0; index < pixels.length; index += 4) {
      const red = pixels[index] ?? 0;
      const green = pixels[index + 1] ?? 0;
      const blue = pixels[index + 2] ?? 0;
      const alpha = pixels[index + 3] ?? 0;
      if (alpha === 0) {
        continue;
      }
      const isBackground = red < 30 && green < 40 && blue < 50;
      if (isBackground) {
        continue;
      }
      nonBackgroundPixelCount += 1;
      if (Math.abs(red - green) > 20 || Math.abs(green - blue) > 20 || Math.abs(red - blue) > 20) {
        chromaticPixelCount += 1;
      }
      if (red > 180 && green > 120 && blue < 120) {
        yellowPixelCount += 1;
      }
    }
    return { nonBackgroundPixelCount, chromaticPixelCount, yellowPixelCount };
  });
}

async function waitForCanvasInk(page: Page, minimumInkPixels: number): Promise<void> {
  await expect.poll(async () => (await readCanvasInkStats(page)).nonBackgroundPixelCount).toBeGreaterThan(minimumInkPixels);
}

function setupServer(): Promise<void> {
  const port = process.env.BRIDGE_TEST_SERVER_PORT;
  if (!port) {
    throw new Error('BRIDGE_TEST_SERVER_PORT environment variable not set. Global setup may have failed.');
  }
  baseUrl = `http://127.0.0.1:${port}`;
  return Promise.resolve();
}

function teardownServer(): Promise<void> {
  // No-op: global teardown will handle server cleanup
  return Promise.resolve();
}

export {
  type RenderedPixel,
  type EditableSceneState,
  type MultiStaticTextSceneState,
  type EditableSceneOptions,
  type SelectionAreaSceneState,
  type BoxSceneState,
  type ScrollSceneState,
  type NestedProxyScrollSceneState,
  type SemanticSceneState,
  type ClippedSemanticSceneState,
  type GlyphRunSnapshot,
  type HighlightRectSnapshot,
  type CanvasInkStats,
  type CanvasColorStats,
  PUBLIC_DIR,
  SCREENSHOT_DIR,
  WRAPPED_TEXT_FIXTURE_PATH,
  CMD_CREATE_NODE,
  CMD_DELETE_NODE,
  CMD_SET_BOUNDS,
  CMD_SET_BOX_STYLE,
  CMD_SET_GLYPH_RUN,
  CMD_COMMIT_PAINT_ORDER,
  CMD_COMMIT_SCENE,
  screenshotPath,
  readWrappedTextFixture,
  getWrappedTextFixtureTargets,
  readScenePixel,
  decodeFloat32,
  parseGlyphRuns,
  parseColoredHighlightRects,
  parseHighlightRects,
  getWrappedTextIndexPoint,
  getWrappedSelectionDragPoints,
  gotoBridgePage,
  readActiveRenderer,
  buildEditableTextScene,
  buildReadonlyTextScene,
  buildStaticTextScene,
  buildMultiStaticTextScene,
  buildScrollableEditableTextScene,
  buildScrollableStaticTextScene,
  buildSelectionAreaScene,
  buildSemanticScene,
  buildClippedSemanticScene,
  buildScrollableSemanticOrderScene,
  buildInteractiveBoxScene,
  buildScrollScene,
  buildNestedProxyScrollScene,
  buildWrappedThaiScene,
  buildColorEmojiScene,
  readCanvasInkStats,
  readCanvasColorStats,
  waitForCanvasInk,
  setupServer,
  teardownServer,
  getBaseUrl,
  type StaticServerHandle,
};
