import { expect, test } from '@playwright/test';

import { parseDebugTreeBuffer, parseDebugTreeBufferSafe } from '../src/debug-tree';

const MAGIC = 0x44544231;
const VERSION = 1;
const RECORD_WORDS = 52;

const FLAG_ACTIVE = 1 << 0;
const FLAG_VISIBLE_NORMAL = 1 << 1;
const FLAG_CLIPPED_OR_EMPTY = 1 << 3;
const FLAG_HAS_NODE_ID = 1 << 4;
const FLAG_HAS_SEMANTIC_LABEL = 1 << 5;
const FLAG_HAS_BOX_STYLE = 1 << 6;

const BEHAVIOR_INTERACTIVE = 1 << 0;
const BEHAVIOR_FOCUSABLE = 1 << 1;
const BEHAVIOR_SCROLL_VIEW = 1 << 5;
const BEHAVIOR_SCROLL_ENABLED_Y = 1 << 11;
const BEHAVIOR_TEXT_NODE = 1 << 13;

function floatToWord(value: number): number {
  const view = new DataView(new ArrayBuffer(4));
  view.setFloat32(0, value, true);
  return view.getUint32(0, true);
}

function writeHandle(words: number[], index: number, handle: bigint): void {
  words[index] = Number(handle & 0xFFFFFFFFn);
  words[index + 1] = Number(handle >> 32n);
}

function packString(value: string): number[] {
  const bytes = new TextEncoder().encode(value);
  const words = [bytes.length];
  const padded = new Uint8Array(Math.ceil(bytes.length / 4) * 4);
  padded.set(bytes);
  const view = new DataView(padded.buffer);
  for (let index = 0; index < padded.byteLength; index += 4) {
    words.push(view.getUint32(index, true));
  }
  return words;
}

interface RecordOptions {
  readonly handle: bigint;
  readonly parent?: bigint;
  readonly type?: number;
  readonly flags?: number;
  readonly behavior?: number;
  readonly semanticRole?: number;
  readonly bounds?: readonly [number, number, number, number];
  readonly visibleBounds?: readonly [number, number, number, number];
  readonly padding?: readonly [number, number, number, number];
  readonly margin?: readonly [number, number, number, number];
  readonly border?: readonly [number, number, number, number];
  readonly bgColor?: number;
  readonly borderColor?: number;
  readonly radius?: readonly [number, number, number, number];
  readonly fontId?: number;
  readonly fontSize?: number;
  readonly textColor?: number;
  readonly nearestScrollAncestor?: bigint;
  readonly scroll?: readonly [number, number, number, number, number, number];
  readonly scrollProxyTarget?: bigint;
  readonly nodeId?: string;
  readonly semanticLabel?: string;
}

function record(options: RecordOptions): number[] {
  const words = new Array<number>(RECORD_WORDS).fill(0);
  writeHandle(words, 0, options.handle);
  writeHandle(words, 2, options.parent ?? 0n);
  words[4] = options.type ?? 0;
  words[5] = options.flags ?? (FLAG_ACTIVE | FLAG_VISIBLE_NORMAL);
  words[6] = options.behavior ?? 0;
  words[7] = options.semanticRole ?? 0;
  const bounds = options.bounds ?? [0, 0, 0, 0];
  const visibleBounds = options.visibleBounds ?? bounds;
  const padding = options.padding ?? [0, 0, 0, 0];
  const margin = options.margin ?? [0, 0, 0, 0];
  const border = options.border ?? [0, 0, 0, 0];
  for (let index = 0; index < 4; index += 1) {
    words[8 + index] = floatToWord(bounds[index] ?? 0);
    words[12 + index] = floatToWord(visibleBounds[index] ?? 0);
    words[16 + index] = floatToWord(padding[index] ?? 0);
    words[20 + index] = floatToWord(margin[index] ?? 0);
    words[24 + index] = floatToWord(border[index] ?? 0);
  }
  words[28] = options.bgColor ?? 0;
  words[29] = options.borderColor ?? 0;
  words[30] = 0;
  const radius = options.radius ?? [0, 0, 0, 0];
  for (let index = 0; index < 4; index += 1) {
    words[31 + index] = floatToWord(radius[index] ?? 0);
  }
  words[35] = floatToWord(1);
  words[36] = options.fontId ?? 0;
  words[37] = floatToWord(options.fontSize ?? 16);
  words[38] = options.textColor ?? 0;
  writeHandle(words, 39, options.nearestScrollAncestor ?? 0n);
  const scroll = options.scroll ?? [0, 0, 0, 0, 0, 0];
  for (let index = 0; index < 6; index += 1) {
    words[41 + index] = floatToWord(scroll[index] ?? 0);
  }
  writeHandle(words, 47, options.scrollProxyTarget ?? 0n);
  words[49] = 0;
  words[50] = 0;
  words[51] = 0;
  return [
    ...words,
    ...packString(options.nodeId ?? ''),
    ...packString(options.semanticLabel ?? ''),
  ];
}

function buffer(records: number[][]): Uint32Array {
  return Uint32Array.from([MAGIC, VERSION, RECORD_WORDS, records.length, ...records.flat()]);
}

test('debug tree parser returns stable nodes keyed by handle and validates children', () => {
  const rootHandle = 0x100000001n;
  const childHandle = 0x100000002n;
  const words = buffer([
    record({
      handle: rootHandle,
      type: 4,
      behavior: BEHAVIOR_SCROLL_VIEW | BEHAVIOR_SCROLL_ENABLED_Y,
      bounds: [0, 0, 320, 240],
      scroll: [0, 12, 320, 600, 320, 240],
    }),
    record({
      handle: childHandle,
      parent: rootHandle,
      type: 1,
      flags: FLAG_ACTIVE | FLAG_VISIBLE_NORMAL | FLAG_CLIPPED_OR_EMPTY | FLAG_HAS_NODE_ID | FLAG_HAS_SEMANTIC_LABEL | FLAG_HAS_BOX_STYLE,
      behavior: BEHAVIOR_TEXT_NODE | BEHAVIOR_INTERACTIVE | BEHAVIOR_FOCUSABLE,
      semanticRole: 10,
      bounds: [8, 300, 120, 32],
      visibleBounds: [0, 0, 0, 0],
      padding: [1, 2, 3, 4],
      margin: [5, 6, 7, 8],
      border: [1.5, 1.5, 1.5, 1.5],
      bgColor: 0x11223344,
      borderColor: 0xAABBCCDD,
      radius: [2, 3, 4, 5],
      fontId: 7,
      fontSize: 18,
      textColor: 0x010203FF,
      nearestScrollAncestor: rootHandle,
      nodeId: 'value-label',
      semanticLabel: 'Value label',
    }),
  ]);

  const tree = parseDebugTreeBuffer(words);

  expect(tree.version).toBe(1);
  expect(tree.nodes.map((node) => node.handle)).toEqual([rootHandle.toString(), childHandle.toString()]);
  expect(tree.roots.map((node) => node.handle)).toEqual([rootHandle.toString()]);
  expect(Object.keys(tree.nodesByHandle)).toEqual([rootHandle.toString(), childHandle.toString()]);

  const root = tree.nodesByHandle[rootHandle.toString()];
  const child = tree.nodesByHandle[childHandle.toString()];
  expect(root).toBeDefined();
  expect(child).toBeDefined();
  expect(root?.nodeTypeName).toBe('scroll-view');
  expect(root?.childHandles).toEqual([childHandle.toString()]);
  expect(root?.behavior.scrollView).toBe(true);
  expect(root?.behavior.scrollEnabledY).toBe(true);
  expect(root?.scroll.offsetY).toBeCloseTo(12);
  expect(root?.scroll.contentHeight).toBeCloseTo(600);

  expect(child?.parentHandle).toBe(rootHandle.toString());
  expect(child?.nodeTypeName).toBe('text');
  expect(child?.nodeId).toBe('value-label');
  expect(child?.semanticRole).toBe(10);
  expect(child?.semanticLabel).toBe('Value label');
  expect(child?.flags.clippedOrEmpty).toBe(true);
  expect(child?.flags.hasBoxStyle).toBe(true);
  expect(child?.behavior.textNode).toBe(true);
  expect(child?.behavior.interactive).toBe(true);
  expect(child?.bounds.y).toBeCloseTo(300);
  expect(child?.visibleBounds.height).toBe(0);
  expect(child?.padding).toEqual({ left: 1, top: 2, right: 3, bottom: 4 });
  expect(child?.margin).toEqual({ left: 5, top: 6, right: 7, bottom: 8 });
  expect(child?.border.left).toBeCloseTo(1.5);
  expect(child?.style.bgColor).toBe(0x11223344);
  expect(child?.style.borderColor).toBe(0xAABBCCDD);
  expect(child?.style.radiusBottomLeft).toBeCloseTo(5);
  expect(child?.style.fontId).toBe(7);
  expect(child?.style.fontSize).toBeCloseTo(18);
  expect(child?.style.textColor).toBe(0x010203FF);
  expect(child?.scroll.nearestScrollAncestorHandle).toBe(rootHandle.toString());
});

test('debug tree parser rejects malformed parent and duplicate handle buffers', () => {
  const rootHandle = 1n;
  const childHandle = 2n;

  expect(() => parseDebugTreeBuffer(buffer([
    record({ handle: childHandle, parent: rootHandle }),
  ]))).toThrow(/missing parent/);

  expect(() => parseDebugTreeBuffer(buffer([
    record({ handle: rootHandle }),
    record({ handle: rootHandle }),
  ]))).toThrow(/duplicate handle/);
});

test('debug tree safe parser logs malformed buffers and returns an empty snapshot', () => {
  const messages: string[] = [];
  const tree = parseDebugTreeBufferSafe(
    Uint32Array.from([MAGIC, VERSION, RECORD_WORDS, 1]),
    (message) => { messages.push(message); },
  );

  expect(tree.nodes).toEqual([]);
  expect(tree.nodesByHandle).toEqual({});
  expect(messages).toHaveLength(1);
  expect(messages[0]).toContain('Failed to parse debug tree buffer');
});
