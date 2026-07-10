import type { CoreModule, UiModule, WasmHandleLike } from '../../core-types';
import { normalizePointerForWasm, pointerToHeapOffset } from './encoding';

const textEncoder = new TextEncoder();

type HeapByteModule = Pick<UiModule | CoreModule, 'HEAPU8' | 'refreshHeapViews' | 'usesMemory64'>;
type HeapAllocModule = HeapByteModule & Pick<UiModule | CoreModule, '_malloc' | '_free'>;

export interface HeapAllocation {
  readonly ptr: number | bigint;
  readonly offset: number;
  readonly len: number;
}

export function withHeapAllocation<T>(
  module: HeapAllocModule,
  byteLength: number,
  callback: (allocation: HeapAllocation) => T,
): T {
  const ptr = normalizePointerForWasm(module, byteLength === 0 ? 0 : module._malloc(byteLength));
  const offset = pointerToHeapOffset(ptr);
  module.refreshHeapViews?.();
  if (byteLength > 0 && offset === 0) {
    throw new Error('WASM heap malloc failed.');
  }
  try {
    return callback({ ptr, offset, len: byteLength });
  } finally {
    if (offset !== 0) {
      module._free(ptr);
    }
  }
}

export function copyBytesToHeap(
  module: HeapByteModule,
  ptr: WasmHandleLike,
  bytes: Uint8Array,
): number {
  const offset = pointerToHeapOffset(normalizePointerForWasm(module, ptr));
  module.refreshHeapViews?.();
  if (bytes.byteLength > 0 && offset === 0) {
    throw new Error('WASM heap copy destination is null.');
  }
  if (bytes.byteLength > 0) {
    new Uint8Array(module.HEAPU8.buffer, offset, bytes.byteLength).set(bytes);
  }
  return offset;
}

export function copyBytesFromHeap(
  module: HeapByteModule,
  ptr: WasmHandleLike,
  byteLength: number,
): Uint8Array {
  if (byteLength <= 0) {
    return new Uint8Array(0);
  }
  const offset = pointerToHeapOffset(normalizePointerForWasm(module, ptr));
  module.refreshHeapViews?.();
  if (offset === 0) {
    throw new Error('WASM heap copy source is null.');
  }
  return new Uint8Array(new Uint8Array(module.HEAPU8.buffer, offset, byteLength));
}

export function withHeapBytes<T>(
  module: HeapAllocModule,
  bytes: Uint8Array,
  callback: (allocation: HeapAllocation) => T,
): T {
  return withHeapAllocation(module, bytes.byteLength, (allocation) => {
    copyBytesToHeap(module, allocation.ptr, bytes);
    return callback(allocation);
  });
}

export function writeUtf8ToHeap(
  module: HeapAllocModule,
  text: string,
): { readonly ptr: WasmHandleLike; readonly offset: number; readonly len: number; dispose(): void } {
  const bytes = textEncoder.encode(text);
  const allocation = allocateHeapBytes(module, bytes);
  return {
    ptr: allocation.ptr,
    offset: allocation.offset,
    len: bytes.byteLength,
    dispose: () => {
      if (allocation.offset !== 0) {
        module._free(allocation.ptr);
      }
    },
  };
}

export function writeBytesToHeap(
  module: HeapAllocModule,
  bytes: Uint8Array,
): { readonly ptr: WasmHandleLike; readonly offset: number; readonly len: number; dispose(): void } {
  const allocation = allocateHeapBytes(module, bytes);
  return {
    ptr: allocation.ptr,
    offset: allocation.offset,
    len: bytes.byteLength,
    dispose: () => {
      if (allocation.offset !== 0) {
        module._free(allocation.ptr);
      }
    },
  };
}

function allocateHeapBytes(module: HeapAllocModule, bytes: Uint8Array): HeapAllocation {
  const ptr = normalizePointerForWasm(module, bytes.byteLength === 0 ? 0 : module._malloc(bytes.byteLength));
  const offset = bytes.byteLength === 0 ? 0 : copyBytesToHeap(module, ptr, bytes);
  return { ptr, offset, len: bytes.byteLength };
}

export function extractCommandBuffer(ui: UiModule): Uint32Array {
  const lengthPtr = normalizePointerForWasm(ui, ui._malloc(4));
  const lengthOffset = pointerToHeapOffset(lengthPtr);
  if (lengthOffset === 0) {
    throw new Error('ui length malloc failed.');
  }

  try {
    const bufferPtr = ui._ui_get_command_buffer(lengthPtr);
    ui.refreshHeapViews?.();
    const wordCount = ui.HEAPU32[lengthOffset >>> 2] ?? 0;
    const bufferOffset = pointerToHeapOffset(normalizePointerForWasm(ui, bufferPtr));
    if (bufferOffset === 0 || wordCount === 0) {
      return new Uint32Array();
    }
    const wordOffset = bufferOffset >>> 2;
    return ui.HEAPU32.slice(wordOffset, wordOffset + wordCount);
  } finally {
    ui._free(lengthPtr);
  }
}

export function extractSemanticBuffer(ui: UiModule): Uint32Array {
  const lengthPtr = normalizePointerForWasm(ui, ui._malloc(4));
  const lengthOffset = pointerToHeapOffset(lengthPtr);
  if (lengthOffset === 0) {
    throw new Error('ui semantic length malloc failed.');
  }

  try {
    const bufferPtr = ui._ui_get_semantic_buffer(lengthPtr);
    ui.refreshHeapViews?.();
    const wordCount = ui.HEAPU32[lengthOffset >>> 2] ?? 0;
    const bufferOffset = pointerToHeapOffset(normalizePointerForWasm(ui, bufferPtr));
    if (bufferOffset === 0 || wordCount === 0) {
      return new Uint32Array();
    }
    const wordOffset = bufferOffset >>> 2;
    return ui.HEAPU32.slice(wordOffset, wordOffset + wordCount);
  } finally {
    ui._free(lengthPtr);
  }
}

export function extractDebugTreeBuffer(ui: UiModule): Uint32Array {
  const lengthPtr = normalizePointerForWasm(ui, ui._malloc(4));
  const lengthOffset = pointerToHeapOffset(lengthPtr);
  if (lengthOffset === 0) {
    throw new Error('ui debug tree length malloc failed.');
  }

  try {
    const bufferPtr = ui._ui_get_debug_tree_buffer(lengthPtr);
    ui.refreshHeapViews?.();
    const wordCount = ui.HEAPU32[lengthOffset >>> 2] ?? 0;
    const bufferOffset = pointerToHeapOffset(normalizePointerForWasm(ui, bufferPtr));
    if (bufferOffset === 0 || wordCount === 0) {
      return new Uint32Array();
    }
    const wordOffset = bufferOffset >>> 2;
    return ui.HEAPU32.slice(wordOffset, wordOffset + wordCount);
  } finally {
    ui._free(lengthPtr);
  }
}

export function executeCommandBuffer(core: CoreModule, words: Uint32Array): void {
  if (words.length === 0) {
    return;
  }
  const ptr = normalizePointerForWasm(core, core._malloc(words.byteLength));
  const offset = pointerToHeapOffset(ptr);
  core.refreshHeapViews?.();
  if (offset === 0) {
    throw new Error('core command malloc failed.');
  }

  try {
    copyBytesToHeap(core, ptr, new Uint8Array(words.buffer, words.byteOffset, words.byteLength));
    core._ed_execute_command_buffer(ptr, words.length);
  } finally {
    core._free(ptr);
  }
}
