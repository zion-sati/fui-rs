import { EdBackendType, EdDeviceState } from '../../core-types';
import type {
  CoreModule,
  EdBackendType as EdBackendTypeValue,
  EdDeviceState as EdDeviceStateValue,
  UiModule,
  WasmHandleLike,
} from '../../core-types';

export type WasmModuleMemoryView = Pick<UiModule | CoreModule, 'usesMemory64'>;

function extractHandlePrimitive(handle: WasmHandleLike): bigint | number | string {
  if (typeof handle === 'bigint' || typeof handle === 'number' || typeof handle === 'string') {
    return handle;
  }
  const symbolPrimitive = (handle as { [Symbol.toPrimitive]?: (hint: string) => unknown })[Symbol.toPrimitive]?.('default');
  if (
    typeof symbolPrimitive === 'bigint' ||
    typeof symbolPrimitive === 'number' ||
    typeof symbolPrimitive === 'string'
  ) {
    return symbolPrimitive;
  }
  const primitive = handle.valueOf();
  if (typeof primitive === 'bigint' || typeof primitive === 'number' || typeof primitive === 'string') {
    return primitive;
  }
  const stringified = handle.toString();
  if (typeof stringified === 'string') {
    return stringified;
  }
  throw new TypeError(`Cannot convert ${String(handle)} to BigInt.`);
}

export function handleToBigInt(handle: WasmHandleLike): bigint {
  const primitive = extractHandlePrimitive(handle);
  if (typeof primitive === 'bigint') {
    return primitive;
  }
  if (typeof primitive === 'number') {
    if (!Number.isInteger(primitive)) {
      throw new TypeError(`Cannot convert non-integer handle ${String(primitive)} to BigInt.`);
    }
    return BigInt(primitive);
  }
  if (typeof primitive === 'string') {
    return BigInt(primitive);
  }
  throw new TypeError(`Cannot convert ${String(handle)} to BigInt.`);
}

export function handleToString(handle: WasmHandleLike): string {
  return handleToBigInt(handle).toString();
}

export function pointerToHeapOffset(pointer: WasmHandleLike): number {
  if (typeof pointer === 'number') {
    if (!Number.isInteger(pointer)) {
      throw new TypeError(`Cannot convert non-integer pointer ${String(pointer)} to a heap offset.`);
    }
    return pointer;
  }
  const value = handleToBigInt(pointer);
  if (value > BigInt(Number.MAX_SAFE_INTEGER)) {
    throw new RangeError(`Pointer ${value.toString()} exceeds JavaScript heap offset precision.`);
  }
  return Number(value);
}

export function normalizePointerForWasm(
  module: WasmModuleMemoryView,
  pointer: WasmHandleLike,
): number | bigint {
  return module.usesMemory64 === true ? handleToBigInt(pointer) : pointerToHeapOffset(pointer);
}

export function toHeapPointer(
  module: WasmModuleMemoryView,
  pointer: WasmHandleLike,
): { readonly ptr: number | bigint; readonly offset: number } {
  const ptr = normalizePointerForWasm(module, pointer);
  return {
    ptr,
    offset: pointerToHeapOffset(ptr),
  };
}

export function normalizeBackendType(value: number): EdBackendTypeValue {
  switch (value) {
    case EdBackendType.WEBGPU:
    case EdBackendType.WEBGL2:
    case EdBackendType.CPU:
      return value;
    default:
      return EdBackendType.NONE;
  }
}

export function normalizeDeviceState(value: number): EdDeviceStateValue {
  switch (value) {
    case EdDeviceState.LOST:
    case EdDeviceState.RECOVERING:
      return value;
    default:
      return EdDeviceState.OK;
  }
}

type ModifierSource =
  | Pick<KeyboardEvent, 'shiftKey' | 'ctrlKey' | 'altKey' | 'metaKey'>
  | Pick<PointerEvent, 'shiftKey' | 'ctrlKey' | 'altKey' | 'metaKey'>;

export function computeModifiers(event: ModifierSource): number {
  let modifiers = 0;
  if (event.shiftKey) {
    modifiers |= 1 << 0;
  }
  if (event.ctrlKey) {
    modifiers |= 1 << 1;
  }
  if (event.altKey) {
    modifiers |= 1 << 2;
  }
  if (event.metaKey) {
    modifiers |= 1 << 3;
  }
  return modifiers;
}
