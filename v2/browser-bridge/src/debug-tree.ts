const DEBUG_TREE_MAGIC = 0x44544231;
const DEBUG_TREE_VERSION = 1;
const DEBUG_TREE_FIXED_RECORD_WORDS = 52;

const FLAG_ACTIVE = 1 << 0;
const FLAG_VISIBLE_NORMAL = 1 << 1;
const FLAG_CLIP_TO_BOUNDS = 1 << 2;
const FLAG_CLIPPED_OR_EMPTY = 1 << 3;
const FLAG_HAS_NODE_ID = 1 << 4;
const FLAG_HAS_SEMANTIC_LABEL = 1 << 5;
const FLAG_HAS_BOX_STYLE = 1 << 6;
const FLAG_HAS_LAYER_EFFECT = 1 << 7;
const FLAG_HAS_DROP_SHADOW = 1 << 8;
const FLAG_HAS_BACKGROUND_BLUR = 1 << 9;
const FLAG_HAS_LINEAR_GRADIENT = 1 << 10;
const FLAG_HAS_IMAGE = 1 << 11;
const FLAG_HAS_IMAGE_NINE = 1 << 12;
const FLAG_HAS_SVG = 1 << 13;
const FLAG_HAS_TEXT_STYLE_RUNS = 1 << 14;

const BEHAVIOR_INTERACTIVE = 1 << 0;
const BEHAVIOR_FOCUSABLE = 1 << 1;
const BEHAVIOR_SELECTABLE = 1 << 2;
const BEHAVIOR_EDITABLE = 1 << 3;
const BEHAVIOR_PORTAL = 1 << 4;
const BEHAVIOR_SCROLL_VIEW = 1 << 5;
const BEHAVIOR_GRID = 1 << 6;
const BEHAVIOR_SELECTION_AREA = 1 << 7;
const BEHAVIOR_SELECTION_AREA_BARRIER = 1 << 8;
const BEHAVIOR_CUSTOM_DRAWABLE = 1 << 9;
const BEHAVIOR_SCROLL_ENABLED_X = 1 << 10;
const BEHAVIOR_SCROLL_ENABLED_Y = 1 << 11;
const BEHAVIOR_SHOW_SCROLLBARS = 1 << 12;
const BEHAVIOR_TEXT_NODE = 1 << 13;
const BEHAVIOR_SVG_NODE = 1 << 14;
const BEHAVIOR_EDITOR_COMMAND_KEYS = 1 << 15;
const BEHAVIOR_EDITOR_ACCEPTS_TAB = 1 << 16;
const BEHAVIOR_TEXT_EDITOR = 1 << 17;

const NODE_TYPE_NAMES: Record<number, string> = {
  0: 'flex-box',
  1: 'text',
  2: 'image',
  3: 'svg',
  4: 'scroll-view',
  5: 'grid',
  6: 'path',
};

const textDecoder = new TextDecoder();
const floatWordView = new DataView(new ArrayBuffer(4));

export interface DebugTreeBounds {
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
}

export interface DebugTreeInsets {
  readonly left: number;
  readonly top: number;
  readonly right: number;
  readonly bottom: number;
}

export interface DebugTreeFlags {
  readonly active: boolean;
  readonly visibleNormal: boolean;
  readonly clipToBounds: boolean;
  readonly clippedOrEmpty: boolean;
  readonly hasNodeId: boolean;
  readonly hasSemanticLabel: boolean;
  readonly hasBoxStyle: boolean;
  readonly hasLayerEffect: boolean;
  readonly hasDropShadow: boolean;
  readonly hasBackgroundBlur: boolean;
  readonly hasLinearGradient: boolean;
  readonly hasImage: boolean;
  readonly hasImageNine: boolean;
  readonly hasSvg: boolean;
  readonly hasTextStyleRuns: boolean;
}

export interface DebugTreeBehaviorFlags {
  readonly interactive: boolean;
  readonly focusable: boolean;
  readonly selectable: boolean;
  readonly editable: boolean;
  readonly portal: boolean;
  readonly scrollView: boolean;
  readonly grid: boolean;
  readonly selectionArea: boolean;
  readonly selectionAreaBarrier: boolean;
  readonly customDrawable: boolean;
  readonly scrollEnabledX: boolean;
  readonly scrollEnabledY: boolean;
  readonly showScrollbars: boolean;
  readonly textNode: boolean;
  readonly svgNode: boolean;
  readonly editorCommandKeys: boolean;
  readonly editorAcceptsTab: boolean;
  readonly textEditor: boolean;
}

export interface DebugTreeStyle {
  readonly bgColor: number;
  readonly borderColor: number;
  readonly borderStyle: number;
  readonly radiusTopLeft: number;
  readonly radiusTopRight: number;
  readonly radiusBottomRight: number;
  readonly radiusBottomLeft: number;
  readonly opacity: number;
  readonly fontId: number;
  readonly fontSize: number;
  readonly textColor: number;
  readonly textAlign: number;
  readonly textVerticalAlign: number;
}

export interface DebugTreeScrollMetrics {
  readonly nearestScrollAncestorHandle: string | null;
  readonly scrollProxyTargetHandle: string | null;
  readonly offsetX: number;
  readonly offsetY: number;
  readonly contentWidth: number;
  readonly contentHeight: number;
  readonly viewportWidth: number;
  readonly viewportHeight: number;
}

export interface DebugTreeNode {
  readonly handle: string;
  readonly parentHandle: string | null;
  readonly childHandles: readonly string[];
  readonly nodeType: number;
  readonly nodeTypeName: string;
  readonly nodeId: string;
  readonly semanticRole: number;
  readonly semanticLabel: string;
  readonly bounds: DebugTreeBounds;
  readonly visibleBounds: DebugTreeBounds;
  readonly padding: DebugTreeInsets;
  readonly margin: DebugTreeInsets;
  readonly border: DebugTreeInsets;
  readonly flags: DebugTreeFlags;
  readonly behavior: DebugTreeBehaviorFlags;
  readonly style: DebugTreeStyle;
  readonly scroll: DebugTreeScrollMetrics;
  readonly visibility: number;
}

export interface DebugTreeSnapshot {
  readonly version: number;
  readonly roots: readonly DebugTreeNode[];
  readonly nodes: readonly DebugTreeNode[];
  readonly nodesByHandle: Readonly<Record<string, DebugTreeNode>>;
}

function wordToFloat(word: number): number {
  floatWordView.setUint32(0, word >>> 0, true);
  return floatWordView.getFloat32(0, true);
}

function readHandle(words: Uint32Array, index: number): string | null {
  const low = words[index] ?? 0;
  const high = words[index + 1] ?? 0;
  if (low === 0 && high === 0) {
    return null;
  }
  return ((BigInt(high) << 32n) | BigInt(low)).toString();
}

function decodeString(words: Uint32Array, index: number): { value: string; nextIndex: number } {
  if (index >= words.length) {
    throw new Error('Debug tree buffer ended before string length.');
  }
  const byteLength = words[index] ?? 0;
  const wordLength = Math.ceil(byteLength / 4);
  const stringStart = index + 1;
  const nextIndex = stringStart + wordLength;
  if (nextIndex > words.length) {
    throw new Error('Debug tree buffer ended mid-string.');
  }
  if (byteLength === 0) {
    return { value: '', nextIndex };
  }
  const byteOffset = words.byteOffset + (stringStart * 4);
  const paddedByteLength = wordLength * 4;
  const bytes = new Uint8Array(words.buffer, byteOffset, paddedByteLength);
  return {
    value: textDecoder.decode(bytes.subarray(0, byteLength)),
    nextIndex,
  };
}

function decodeFlags(flags: number): DebugTreeFlags {
  return {
    active: (flags & FLAG_ACTIVE) !== 0,
    visibleNormal: (flags & FLAG_VISIBLE_NORMAL) !== 0,
    clipToBounds: (flags & FLAG_CLIP_TO_BOUNDS) !== 0,
    clippedOrEmpty: (flags & FLAG_CLIPPED_OR_EMPTY) !== 0,
    hasNodeId: (flags & FLAG_HAS_NODE_ID) !== 0,
    hasSemanticLabel: (flags & FLAG_HAS_SEMANTIC_LABEL) !== 0,
    hasBoxStyle: (flags & FLAG_HAS_BOX_STYLE) !== 0,
    hasLayerEffect: (flags & FLAG_HAS_LAYER_EFFECT) !== 0,
    hasDropShadow: (flags & FLAG_HAS_DROP_SHADOW) !== 0,
    hasBackgroundBlur: (flags & FLAG_HAS_BACKGROUND_BLUR) !== 0,
    hasLinearGradient: (flags & FLAG_HAS_LINEAR_GRADIENT) !== 0,
    hasImage: (flags & FLAG_HAS_IMAGE) !== 0,
    hasImageNine: (flags & FLAG_HAS_IMAGE_NINE) !== 0,
    hasSvg: (flags & FLAG_HAS_SVG) !== 0,
    hasTextStyleRuns: (flags & FLAG_HAS_TEXT_STYLE_RUNS) !== 0,
  };
}

function decodeBehaviorFlags(flags: number): DebugTreeBehaviorFlags {
  return {
    interactive: (flags & BEHAVIOR_INTERACTIVE) !== 0,
    focusable: (flags & BEHAVIOR_FOCUSABLE) !== 0,
    selectable: (flags & BEHAVIOR_SELECTABLE) !== 0,
    editable: (flags & BEHAVIOR_EDITABLE) !== 0,
    portal: (flags & BEHAVIOR_PORTAL) !== 0,
    scrollView: (flags & BEHAVIOR_SCROLL_VIEW) !== 0,
    grid: (flags & BEHAVIOR_GRID) !== 0,
    selectionArea: (flags & BEHAVIOR_SELECTION_AREA) !== 0,
    selectionAreaBarrier: (flags & BEHAVIOR_SELECTION_AREA_BARRIER) !== 0,
    customDrawable: (flags & BEHAVIOR_CUSTOM_DRAWABLE) !== 0,
    scrollEnabledX: (flags & BEHAVIOR_SCROLL_ENABLED_X) !== 0,
    scrollEnabledY: (flags & BEHAVIOR_SCROLL_ENABLED_Y) !== 0,
    showScrollbars: (flags & BEHAVIOR_SHOW_SCROLLBARS) !== 0,
    textNode: (flags & BEHAVIOR_TEXT_NODE) !== 0,
    svgNode: (flags & BEHAVIOR_SVG_NODE) !== 0,
    editorCommandKeys: (flags & BEHAVIOR_EDITOR_COMMAND_KEYS) !== 0,
    editorAcceptsTab: (flags & BEHAVIOR_EDITOR_ACCEPTS_TAB) !== 0,
    textEditor: (flags & BEHAVIOR_TEXT_EDITOR) !== 0,
  };
}

function emptyDebugTreeSnapshot(version = DEBUG_TREE_VERSION): DebugTreeSnapshot {
  return {
    version,
    roots: [],
    nodes: [],
    nodesByHandle: {},
  };
}

export function parseDebugTreeBuffer(words: Uint32Array): DebugTreeSnapshot {
  if (words.length === 0) {
    return emptyDebugTreeSnapshot();
  }
  if (words.length < 4) {
    throw new Error('Debug tree buffer is shorter than the header.');
  }
  if (words[0] !== DEBUG_TREE_MAGIC) {
    throw new Error('Debug tree buffer has an invalid magic.');
  }
  const version = words[1] ?? 0;
  if (version !== DEBUG_TREE_VERSION) {
    throw new Error(`Unsupported debug tree buffer version ${String(version)}.`);
  }
  const recordWords = words[2] ?? 0;
  if (recordWords !== DEBUG_TREE_FIXED_RECORD_WORDS) {
    throw new Error(`Unsupported debug tree record width ${String(recordWords)}.`);
  }

  const recordCount = words[3] ?? 0;
  if (recordCount === 0) {
    if (words.length !== 4) {
      throw new Error('Debug tree buffer has trailing words after an empty header.');
    }
    return emptyDebugTreeSnapshot(version);
  }

  let index = 4;
  const mutableNodes: (DebugTreeNode & { childHandles: string[] })[] = [];
  const nodesByHandle: Record<string, DebugTreeNode & { childHandles: string[] }> = {};

  for (let recordIndex = 0; recordIndex < recordCount; recordIndex += 1) {
    if ((index + recordWords) > words.length) {
      throw new Error('Debug tree buffer ended mid-record.');
    }
    const base = index;
    const handle = readHandle(words, base);
    if (handle === null) {
      throw new Error('Debug tree record has an invalid zero handle.');
    }
    if (nodesByHandle[handle] !== undefined) {
      throw new Error(`Debug tree buffer contains duplicate handle ${handle}.`);
    }
    const parentHandle = readHandle(words, base + 2);
    const nodeType = words[base + 4] ?? 0;
    const rawFlags = words[base + 5] ?? 0;
    const rawBehaviorFlags = words[base + 6] ?? 0;
    const nearestScrollAncestorHandle = readHandle(words, base + 39);
    const scrollProxyTargetHandle = readHandle(words, base + 47);

    index += recordWords;
    const nodeId = decodeString(words, index);
    index = nodeId.nextIndex;
    const semanticLabel = decodeString(words, index);
    index = semanticLabel.nextIndex;

    const node: DebugTreeNode & { childHandles: string[] } = {
      handle,
      parentHandle,
      childHandles: [],
      nodeType,
      nodeTypeName: NODE_TYPE_NAMES[nodeType] ?? `unknown-${String(nodeType)}`,
      nodeId: nodeId.value,
      semanticRole: words[base + 7] ?? 0,
      semanticLabel: semanticLabel.value,
      bounds: {
        x: wordToFloat(words[base + 8] ?? 0),
        y: wordToFloat(words[base + 9] ?? 0),
        width: wordToFloat(words[base + 10] ?? 0),
        height: wordToFloat(words[base + 11] ?? 0),
      },
      visibleBounds: {
        x: wordToFloat(words[base + 12] ?? 0),
        y: wordToFloat(words[base + 13] ?? 0),
        width: wordToFloat(words[base + 14] ?? 0),
        height: wordToFloat(words[base + 15] ?? 0),
      },
      padding: {
        left: wordToFloat(words[base + 16] ?? 0),
        top: wordToFloat(words[base + 17] ?? 0),
        right: wordToFloat(words[base + 18] ?? 0),
        bottom: wordToFloat(words[base + 19] ?? 0),
      },
      margin: {
        left: wordToFloat(words[base + 20] ?? 0),
        top: wordToFloat(words[base + 21] ?? 0),
        right: wordToFloat(words[base + 22] ?? 0),
        bottom: wordToFloat(words[base + 23] ?? 0),
      },
      border: {
        left: wordToFloat(words[base + 24] ?? 0),
        top: wordToFloat(words[base + 25] ?? 0),
        right: wordToFloat(words[base + 26] ?? 0),
        bottom: wordToFloat(words[base + 27] ?? 0),
      },
      flags: decodeFlags(rawFlags),
      behavior: decodeBehaviorFlags(rawBehaviorFlags),
      style: {
        bgColor: words[base + 28] ?? 0,
        borderColor: words[base + 29] ?? 0,
        borderStyle: words[base + 30] ?? 0,
        radiusTopLeft: wordToFloat(words[base + 31] ?? 0),
        radiusTopRight: wordToFloat(words[base + 32] ?? 0),
        radiusBottomRight: wordToFloat(words[base + 33] ?? 0),
        radiusBottomLeft: wordToFloat(words[base + 34] ?? 0),
        opacity: wordToFloat(words[base + 35] ?? 0),
        fontId: words[base + 36] ?? 0,
        fontSize: wordToFloat(words[base + 37] ?? 0),
        textColor: words[base + 38] ?? 0,
        textAlign: words[base + 49] ?? 0,
        textVerticalAlign: words[base + 50] ?? 0,
      },
      scroll: {
        nearestScrollAncestorHandle,
        scrollProxyTargetHandle,
        offsetX: wordToFloat(words[base + 41] ?? 0),
        offsetY: wordToFloat(words[base + 42] ?? 0),
        contentWidth: wordToFloat(words[base + 43] ?? 0),
        contentHeight: wordToFloat(words[base + 44] ?? 0),
        viewportWidth: wordToFloat(words[base + 45] ?? 0),
        viewportHeight: wordToFloat(words[base + 46] ?? 0),
      },
      visibility: words[base + 51] ?? 0,
    };

    mutableNodes.push(node);
    nodesByHandle[handle] = node;
  }

  if (index !== words.length) {
    throw new Error('Debug tree buffer has trailing words.');
  }

  const roots: (DebugTreeNode & { childHandles: string[] })[] = [];
  for (const node of mutableNodes) {
    if (node.parentHandle === null) {
      roots.push(node);
      continue;
    }
    const parent = nodesByHandle[node.parentHandle];
    if (parent === undefined) {
      throw new Error(`Debug tree node ${node.handle} references missing parent ${node.parentHandle}.`);
    }
    parent.childHandles.push(node.handle);
  }

  return {
    version,
    roots,
    nodes: mutableNodes,
    nodesByHandle,
  };
}

export function parseDebugTreeBufferSafe(
  words: Uint32Array,
  logError: (message: string) => void = (message) => { console.error(message); },
): DebugTreeSnapshot {
  try {
    return parseDebugTreeBuffer(words);
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    logError(`[effindom-devtools] Failed to parse debug tree buffer: ${message}`);
    return emptyDebugTreeSnapshot();
  }
}
