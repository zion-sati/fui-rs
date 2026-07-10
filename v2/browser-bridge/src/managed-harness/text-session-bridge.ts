import { withHeapBytes, type BridgeRuntime, type WasmHandleLike } from '@effindomv2/runtime';

import { currentInteractionTimeMs, toBigIntHandle, zeroPointer, type AppHandleLike } from './interop';

const decoder = new TextDecoder();
const encoder = new TextEncoder();
const TEXTBOX_HARD_CLAMP_MAX_CODEPOINTS = 10000;

interface TextClampRange {
  readonly start: number;
  readonly end: number;
}

function advanceCodeUnitIndex(text: string, index: number): number {
  const codePoint = text.codePointAt(index) ?? 0;
  return index + (codePoint > 0xffff ? 2 : 1);
}

function isLineBreakCodeUnit(text: string, index: number): boolean {
  if (index < 0 || index >= text.length) {
    return false;
  }
  const codeUnit = text.charCodeAt(index);
  return codeUnit === 0x0a || codeUnit === 0x0d;
}

function collectTextboxHardLineClampRanges(text: string): readonly TextClampRange[] {
  const ranges: TextClampRange[] = [];
  let index = 0;
  while (index < text.length) {
    let lineCapEnd = index;
    let lineEnd = index;
    let codePointCount = 0;
    while (lineEnd < text.length && !isLineBreakCodeUnit(text, lineEnd)) {
      const next = advanceCodeUnitIndex(text, lineEnd);
      if (codePointCount < TEXTBOX_HARD_CLAMP_MAX_CODEPOINTS) {
        lineCapEnd = next;
      }
      codePointCount += 1;
      lineEnd = next;
    }
    if (codePointCount > TEXTBOX_HARD_CLAMP_MAX_CODEPOINTS) {
      ranges.push({ start: lineCapEnd, end: lineEnd });
    }
    if (lineEnd >= text.length) {
      break;
    }
    if (text.charCodeAt(lineEnd) === 0x0d && lineEnd + 1 < text.length && text.charCodeAt(lineEnd + 1) === 0x0a) {
      index = lineEnd + 2;
    } else {
      index = lineEnd + 1;
    }
  }
  return ranges;
}

function mapClampedTextIndex(index: number, ranges: readonly TextClampRange[]): number {
  const clampedIndex = Math.max(0, index);
  let removedBefore = 0;
  for (const range of ranges) {
    if (clampedIndex <= range.start) {
      break;
    }
    if (clampedIndex < range.end) {
      return range.start - removedBefore;
    }
    removedBefore += range.end - range.start;
  }
  return clampedIndex - removedBefore;
}

function clampTextboxHardLines(text: string): {
  readonly text: string;
  mapIndex(index: number): number;
} {
  const ranges = collectTextboxHardLineClampRanges(text);
  if (ranges.length === 0) {
    return {
      text,
      mapIndex: (index: number) => Math.max(0, Math.min(index, text.length)),
    };
  }
  let result = '';
  let cursor = 0;
  for (const range of ranges) {
    result += text.slice(cursor, range.start);
    cursor = range.end;
  }
  result += text.slice(cursor);
  return {
    text: result,
    mapIndex: (index: number) => mapClampedTextIndex(index, ranges),
  };
}

function computeReplacementEdit(previousText: string, nextText: string): {
  readonly start: number;
  readonly end: number;
  readonly insertedText: string;
} | null {
  if (previousText === nextText) {
    return null;
  }

  const sharedPrefixLimit = Math.min(previousText.length, nextText.length);
  let prefix = 0;
  while (prefix < sharedPrefixLimit && previousText.charCodeAt(prefix) === nextText.charCodeAt(prefix)) {
    prefix += 1;
  }

  let suffix = 0;
  while (
    suffix < (previousText.length - prefix) &&
    suffix < (nextText.length - prefix) &&
    previousText.charCodeAt(previousText.length - suffix - 1) === nextText.charCodeAt(nextText.length - suffix - 1)
  ) {
    suffix += 1;
  }

  return {
    start: prefix,
    end: previousText.length - suffix,
    insertedText: nextText.slice(prefix, nextText.length - suffix),
  };
}

function applyReplacementEdit(text: string, start: number, end: number, insertedText: string): string {
  const clampedStart = Math.max(0, Math.min(start, text.length));
  const clampedEnd = Math.max(clampedStart, Math.min(end, text.length));
  return `${text.slice(0, clampedStart)}${insertedText}${text.slice(clampedEnd)}`;
}

function getHiddenTextEditor(): HTMLInputElement | HTMLTextAreaElement | null {
  const activeElement = document.activeElement;
  if (
    (activeElement instanceof HTMLInputElement || activeElement instanceof HTMLTextAreaElement) &&
    activeElement.dataset.effindomHiddenEditor === 'true'
  ) {
    return activeElement;
  }
  const editor = document.querySelector('input[data-effindom-hidden-editor="true"], textarea[data-effindom-hidden-editor="true"]');
  return editor instanceof HTMLInputElement || editor instanceof HTMLTextAreaElement ? editor : null;
}

export interface TextSessionLike {
  readonly memory: WebAssembly.Memory;
  readonly textBufferPtr: number;
  readonly textBufferSize: number;
}

interface FrozenTextSelectionSnapshot {
  readonly handleKey: string;
  readonly text: string;
  readonly start: number;
  readonly end: number;
}

export class TextSessionBridge {
  private readonly latestTextByHandle = new Map<string, string>();
  private readonly latestSelectionByHandle = new Map<string, { start: number; end: number }>();
  private frozenTextSelectionSnapshot: FrozenTextSelectionSnapshot | null = null;

  constructor(
    private readonly getRuntime: () => BridgeRuntime,
    private readonly getCurrentMemory: () => WebAssembly.Memory,
    private readonly queueHarnessFrame: () => void,
  ) {}

  clearState(): void {
    this.latestTextByHandle.clear();
    this.latestSelectionByHandle.clear();
    this.frozenTextSelectionSnapshot = null;
  }

  readAppUtf8(ptr: number, len: number): string {
    if (len === 0) {
      return '';
    }
    return decoder.decode(new Uint8Array(this.getCurrentMemory().buffer, ptr, len));
  }

  readAppFloats(ptr: number, count: number): Float32Array {
    if (count === 0) {
      return new Float32Array(0);
    }
    return new Float32Array(this.getCurrentMemory().buffer.slice(ptr, ptr + (count * 4)));
  }

  readAppBytes(ptr: number, len: number): Uint8Array {
    if (len === 0) {
      return new Uint8Array(0);
    }
    return new Uint8Array(this.getCurrentMemory().buffer.slice(ptr, ptr + len));
  }

  readAppTextParts(ptr: number, len: number): string[] {
    if (len === 0) {
      return [];
    }
    const source = new Uint8Array(this.getCurrentMemory().buffer, ptr, len);
    if (source.byteLength < 4) {
      throw new Error('Fetch request header payload was truncated.');
    }
    const dataView = new DataView(source.buffer, source.byteOffset, source.byteLength);
    let byteOffset = 0;
    const count = dataView.getUint32(byteOffset, true);
    byteOffset += 4;
    const values: string[] = [];
    for (let index = 0; index < count; index += 1) {
      if (byteOffset + 4 > source.byteLength) {
        throw new Error('Fetch request header length was truncated.');
      }
      const partLen = dataView.getUint32(byteOffset, true);
      byteOffset += 4;
      if (byteOffset + partLen > source.byteLength) {
        throw new Error('Fetch request header value was truncated.');
      }
      values.push(partLen > 0 ? decoder.decode(source.subarray(byteOffset, byteOffset + partLen)) : '');
      byteOffset += partLen;
    }
    return values;
  }

  writeAppFloat32(ptr: number, value: number): void {
    const appView = new DataView(this.getCurrentMemory().buffer);
    appView.setFloat32(ptr, value, true);
  }

  writeAppUint32(ptr: number, value: number): void {
    const appView = new DataView(this.getCurrentMemory().buffer);
    appView.setUint32(ptr, value >>> 0, true);
  }

  writeAppUtf8(ptr: number, capacity: number, text: string, context: string): number {
    if (capacity <= 0) {
      if (text.length === 0) {
        return 0;
      }
      throw new Error(`${context} cannot write into a zero-length host-service buffer.`);
    }
    const encoded = encoder.encode(text);
    if (encoded.length > capacity) {
      throw new Error(`${context} exceeds the provided host-service buffer.`);
    }
    if (encoded.length > 0) {
      const memory = new Uint8Array(this.getCurrentMemory().buffer, ptr, encoded.length);
      memory.set(encoded);
    }
    return encoded.length;
  }

  writeTextCallbackPayload(session: TextSessionLike, text: string, context: string): number {
    const encoded = encoder.encode(text);
    if (encoded.length > session.textBufferSize) {
      throw new Error(`${context} exceeds the shared AssemblyScript text buffer.`);
    }
    if (encoded.length > 0) {
      const memory = new Uint8Array(this.getCurrentMemory().buffer, session.textBufferPtr, encoded.length);
      memory.set(encoded);
    }
    return encoded.length;
  }

  writeWorkerTextCallbackPayload(session: TextSessionLike, text: string, context: string): number {
    return this.writeTextCallbackPayload(session, text, context);
  }

  writeTextToSessionBuffer(session: TextSessionLike, text: string): number {
    if (session.textBufferPtr === 0 || session.textBufferSize === 0) {
      return 0;
    }
    const encoded = encoder.encode(text);
    const length = Math.min(encoded.length, session.textBufferSize);
    if (length > 0) {
      const memory = new Uint8Array(this.getCurrentMemory().buffer, session.textBufferPtr, length);
      memory.set(encoded.subarray(0, length));
    }
    return length;
  }

  withUiUtf8(
    text: string,
    callback: (ptr: WasmHandleLike | number, len: number) => void,
  ): void {
    const runtime = this.getRuntime();
    if (text.length === 0) {
      callback(zeroPointer(runtime), 0);
      return;
    }
    const bytes = encoder.encode(text);
    withHeapBytes(runtime.ui, bytes, (heap) => {
      callback(heap.ptr, heap.len);
    });
  }

  withUiGridData(
    values: Float32Array,
    types: Uint8Array,
    callback: (valuesPtr: WasmHandleLike | number, typesPtr: WasmHandleLike | number) => void,
  ): void {
    const runtime = this.getRuntime();
    const valueBytes = new Uint8Array(values.buffer);
    withHeapBytes(runtime.ui, valueBytes, (valueHeap) => {
      withHeapBytes(runtime.ui, types, (typeHeap) => {
        callback(valueHeap.ptr, typeHeap.ptr);
      });
    });
  }

  withUiGradientData(
    offsets: Float32Array,
    colors: Uint32Array,
    callback: (offsetsPtr: WasmHandleLike | number, colorsPtr: WasmHandleLike | number) => void,
  ): void {
    const runtime = this.getRuntime();
    const offsetBytes = new Uint8Array(offsets.buffer);
    const colorBytes = new Uint8Array(colors.buffer);
    withHeapBytes(runtime.ui, offsetBytes, (offsetHeap) => {
      withHeapBytes(runtime.ui, colorBytes, (colorHeap) => {
        callback(offsetHeap.ptr, colorHeap.ptr);
      });
    });
  }

  recordTextChanged(handle: AppHandleLike, text: string): void {
    this.latestTextByHandle.set(toBigIntHandle(handle).toString(), text);
  }

  recordTextReplaced(handle: AppHandleLike, start: number, end: number, text: string): void {
    const handleKey = toBigIntHandle(handle).toString();
    const previousText = this.latestTextByHandle.get(handleKey) ?? '';
    this.latestTextByHandle.set(handleKey, applyReplacementEdit(previousText, start, end, text));
  }

  recordSelectionChanged(handle: AppHandleLike, start: number, end: number): void {
    this.latestSelectionByHandle.set(toBigIntHandle(handle).toString(), { start, end });
  }

  getLatestText(handle: AppHandleLike): string {
    return this.latestTextByHandle.get(toBigIntHandle(handle).toString()) ?? '';
  }

  resolveFrozenOrLiveTextSelection(handle: AppHandleLike): FrozenTextSelectionSnapshot | null {
    const handleKey = toBigIntHandle(handle).toString();
    if (
      this.frozenTextSelectionSnapshot !== null &&
      this.frozenTextSelectionSnapshot.handleKey === handleKey &&
      this.frozenTextSelectionSnapshot.start !== this.frozenTextSelectionSnapshot.end
    ) {
      return this.frozenTextSelectionSnapshot;
    }
    const text = this.latestTextByHandle.get(handleKey) ?? '';
    const selection = this.latestSelectionByHandle.get(handleKey) ?? null;
    if (selection === null || selection.start === selection.end || text.length === 0) {
      return null;
    }
    const start = Math.max(0, Math.min(selection.start, selection.end));
    const end = Math.max(start, Math.min(text.length, Math.max(selection.start, selection.end)));
    return { handleKey, text, start, end };
  }

  freezeTextSelectionSnapshot(handle: AppHandleLike): void {
    this.frozenTextSelectionSnapshot = this.resolveFrozenOrLiveTextSelection(handle);
  }

  clearFrozenTextSelectionSnapshot(): void {
    this.frozenTextSelectionSnapshot = null;
  }

  getHiddenTextEditor(): HTMLInputElement | HTMLTextAreaElement | null {
    return getHiddenTextEditor();
  }

  syncEditableTextToRuntime(handle: AppHandleLike, text: string, caret: number): void {
    const runtime = this.getRuntime();
    const handleKey = toBigIntHandle(handle).toString();
    const previousText = this.latestTextByHandle.get(handleKey) ?? '';
    const replacement = computeReplacementEdit(previousText, text);
    if (replacement === null) {
      runtime.commitFrame();
      this.queueHarnessFrame();
      return;
    }
    const intendedText =
      `${previousText.slice(0, replacement.start)}${replacement.insertedText}${previousText.slice(replacement.end)}`;
    const intendedCaret = Math.max(0, Math.min(caret, intendedText.length));
    const clamped = clampTextboxHardLines(intendedText);
    const committedText = clamped.text;
    const clampedCaret = clamped.mapIndex(intendedCaret);
    const editor = getHiddenTextEditor();
    if (editor !== null && editor.value !== committedText) {
      editor.value = committedText;
      editor.setSelectionRange(clampedCaret, clampedCaret, 'none');
    }
    const committedReplacement = computeReplacementEdit(previousText, committedText);
    if (committedReplacement === null) {
      runtime.commitFrame();
      this.queueHarnessFrame();
      return;
    }
    runtime.ui._ui_set_interaction_time(currentInteractionTimeMs());
    this.withUiUtf8(committedReplacement.insertedText, (uiPtr, uiLen) => {
      runtime.ui._ui_replace_text_range(
        toBigIntHandle(handle),
        committedReplacement.start,
        committedReplacement.end,
        uiPtr,
        uiLen,
        clampedCaret,
      );
    });
    runtime.commitFrame();
    this.queueHarnessFrame();
  }

  updateLiveTextAfterCut(handleKey: string, text: string, caret: number): void {
    this.latestTextByHandle.set(handleKey, text);
    this.latestSelectionByHandle.set(handleKey, { start: caret, end: caret });
  }
}
