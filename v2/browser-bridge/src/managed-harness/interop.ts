import type { BridgeRuntime, WasmHandleLike } from '@effindomv2/runtime';
import {
  handleToBigInt,
  normalizePointerForWasm,
  pointerToHeapOffset,
} from '@effindomv2/runtime';

export type AppHandleLike = number | bigint;

export function toBigIntHandle(handle: WasmHandleLike | AppHandleLike): bigint {
  return handleToBigInt(handle);
}

export function toNumberHandle(handle: WasmHandleLike | AppHandleLike): number {
  return pointerToHeapOffset(handle);
}

export function zeroPointer(runtime: BridgeRuntime): WasmHandleLike | number {
  return normalizePointerForWasm(runtime.ui, 0);
}

export function normalizePointer(runtime: BridgeRuntime, ptr: WasmHandleLike | number): WasmHandleLike | number {
  return normalizePointerForWasm(runtime.ui, ptr);
}

export function addUiPointer(
  runtime: BridgeRuntime,
  ptr: WasmHandleLike | number,
  byteOffset: number,
): WasmHandleLike | number {
  if (runtime.ui.usesMemory64 === true) {
    return toBigIntHandle(ptr) + BigInt(byteOffset);
  }
  return toNumberHandle(ptr) + byteOffset;
}

export function currentInteractionTimeMs(): bigint {
  return BigInt(Math.floor(performance.now()));
}
