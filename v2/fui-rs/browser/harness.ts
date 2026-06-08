import type { BridgeRuntime, BridgeState, WasmHandleLike } from '../../browser-bridge/src/index.js';
import { normalizePointerForWasm } from '../../browser-bridge/src/bridge/utils/encoding.js';

declare global {
  interface Window {
    __fuiRsReady?: boolean;
    __fuiRsError?: string;
    __fuiRsState?: {
      readonly commandWordCount: number;
      readonly commandWords: readonly number[];
      readonly rootHandle: string | null;
    };
    EffinDomBrowserBridge?: BridgeState;
  }
}

interface FuiRsExports {
  readonly memory: WebAssembly.Memory;
  __runSmokeApp(): void;
  __flushRenders(): void;
}

type AppHandleLike = number | bigint;

const decoder = new TextDecoder();
const encoder = new TextEncoder();

let appMemory: WebAssembly.Memory | null = null;
let appExports: FuiRsExports | null = null;
let flushQueued = false;
let latestCommandWords: number[] = [];
let latestRootHandle: string | null = null;

function toBigIntHandle(handle: WasmHandleLike | AppHandleLike): bigint {
  if (typeof handle === 'bigint') {
    return handle;
  }
  if (typeof handle === 'number') {
    return BigInt(handle);
  }
  if (typeof handle === 'string') {
    return BigInt(handle);
  }
  const primitive = handle.valueOf();
  if (typeof primitive === 'bigint') {
    return primitive;
  }
  if (typeof primitive === 'number') {
    return BigInt(primitive);
  }
  if (typeof primitive === 'string') {
    return BigInt(primitive);
  }
  return BigInt(handle.toString());
}

function toNumberHandle(handle: WasmHandleLike): number {
  return Number(toBigIntHandle(handle));
}

function readAppUtf8(ptr: number, len: number): string {
  if (len === 0) {
    return '';
  }
  if (appMemory === null) {
    throw new Error('Rust app memory is not available.');
  }
  return decoder.decode(new Uint8Array(appMemory.buffer, ptr, len));
}

function withUiUtf8(runtime: BridgeRuntime, text: string, callback: (ptr: WasmHandleLike | number, len: number) => void): void {
  if (text.length === 0) {
    callback(0, 0);
    return;
  }
  const bytes = encoder.encode(text);
  const rawPtr = runtime.ui._malloc(bytes.length);
  const ptr = normalizePointerForWasm(runtime.ui, rawPtr);
  const offset = Number(ptr);
  runtime.ui.refreshHeapViews?.();
  if (bytes.length > 0) {
    runtime.ui.HEAPU8.set(bytes, offset);
  }
  callback(ptr, bytes.length);
  runtime.ui._free(rawPtr);
}

function updateWindowState(): void {
  window.__fuiRsState = {
    commandWordCount: latestCommandWords.length,
    commandWords: latestCommandWords,
    rootHandle: latestRootHandle,
  };
}

function waitForFrame(): Promise<void> {
  return new Promise<void>((resolve) => {
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        resolve();
      });
    });
  });
}

function createUiImports(runtime: BridgeRuntime): Record<string, unknown> {
  return {
    ui_reset(): void {
      runtime.ui._ui_reset();
      latestCommandWords = [];
      latestRootHandle = null;
      updateWindowState();
    },
    ui_create_node(type: number): bigint {
      return toBigIntHandle(runtime.ui._ui_create_node(type));
    },
    ui_delete_node(handle: AppHandleLike): void {
      runtime.ui._ui_delete_node(toBigIntHandle(handle));
    },
    ui_node_add_child(parent: AppHandleLike, child: AppHandleLike): void {
      runtime.ui._ui_node_add_child(toBigIntHandle(parent), toBigIntHandle(child));
    },
    ui_node_remove_child(parent: AppHandleLike, child: AppHandleLike): void {
      runtime.ui._ui_node_remove_child(toBigIntHandle(parent), toBigIntHandle(child));
    },
    ui_set_root(handle: AppHandleLike): void {
      const rootHandle = toBigIntHandle(handle);
      latestRootHandle = rootHandle.toString();
      runtime.ui._ui_set_root(rootHandle);
      updateWindowState();
    },
    ui_set_width(handle: AppHandleLike, value: number, unit: number): void {
      runtime.ui._ui_set_width(toBigIntHandle(handle), value, unit);
    },
    ui_set_height(handle: AppHandleLike, value: number, unit: number): void {
      runtime.ui._ui_set_height(toBigIntHandle(handle), value, unit);
    },
    ui_set_fill_width(handle: AppHandleLike, fill: number): void {
      runtime.ui._ui_set_fill_width(toBigIntHandle(handle), fill);
    },
    ui_set_fill_height(handle: AppHandleLike, fill: number): void {
      runtime.ui._ui_set_fill_height(toBigIntHandle(handle), fill);
    },
    ui_set_fill_width_percent(handle: AppHandleLike, percent: number): void {
      runtime.ui._ui_set_fill_width_percent(toBigIntHandle(handle), percent);
    },
    ui_set_fill_height_percent(handle: AppHandleLike, percent: number): void {
      runtime.ui._ui_set_fill_height_percent(toBigIntHandle(handle), percent);
    },
    ui_set_min_width(handle: AppHandleLike, value: number, unit: number): void {
      runtime.ui._ui_set_min_width(toBigIntHandle(handle), value, unit);
    },
    ui_set_max_width(handle: AppHandleLike, value: number, unit: number): void {
      runtime.ui._ui_set_max_width(toBigIntHandle(handle), value, unit);
    },
    ui_set_min_height(handle: AppHandleLike, value: number, unit: number): void {
      runtime.ui._ui_set_min_height(toBigIntHandle(handle), value, unit);
    },
    ui_set_max_height(handle: AppHandleLike, value: number, unit: number): void {
      runtime.ui._ui_set_max_height(toBigIntHandle(handle), value, unit);
    },
    ui_set_bg_color(handle: AppHandleLike, color: number): void {
      runtime.ui._ui_set_bg_color(toBigIntHandle(handle), color);
    },
    ui_set_font(handle: AppHandleLike, fontId: number, size: number): void {
      runtime.ui._ui_set_font(toBigIntHandle(handle), fontId, size);
    },
    ui_set_text_color(handle: AppHandleLike, color: number): void {
      runtime.ui._ui_set_text_color(toBigIntHandle(handle), color);
    },
    ui_set_text(handle: AppHandleLike, ptr: number, len: number): void {
      const text = readAppUtf8(ptr, len);
      withUiUtf8(runtime, text, (uiPtr, uiLen) => {
        runtime.ui._ui_set_text(toBigIntHandle(handle), uiPtr, uiLen);
      });
    },
    ui_commit_frame(): void {
      runtime.commitFrame();
      latestCommandWords = Array.from(runtime.syncCommandBufferToCore());
      updateWindowState();
    },
    ui_set_padding(handle: AppHandleLike, top: number, right: number, bottom: number, left: number): void {
      runtime.ui._ui_set_padding(toBigIntHandle(handle), top, right, bottom, left);
    },
    ui_set_flex_direction(handle: AppHandleLike, direction: number): void {
      runtime.ui._ui_set_flex_direction(toBigIntHandle(handle), direction);
    },
    ui_resize_window(width: number, height: number): void {
      runtime.ui._ui_resize_window(width, height);
    },
  };
}

function createHostImports(runtime: BridgeRuntime): Record<string, unknown> {
  return {
    request_render(): void {
      runtime.requestFrame();
    },
    get_viewport_width(): number {
      const rect = runtime.canvas.getBoundingClientRect();
      return rect.width > 0 ? rect.width : runtime.canvas.width;
    },
    get_viewport_height(): number {
      const rect = runtime.canvas.getBoundingClientRect();
      return rect.height > 0 ? rect.height : runtime.canvas.height;
    },
  };
}

async function bootHarness(runtime: BridgeRuntime): Promise<void> {
  const wasmResponse = await fetch('./app.wasm');
  const wasmBytes = await wasmResponse.arrayBuffer();
  const imports: WebAssembly.Imports = {
    effindom_v2_ui: createUiImports(runtime),
    fui_host: createHostImports(runtime),
  } as unknown as WebAssembly.Imports;

  const { instance } = await WebAssembly.instantiate(wasmBytes, imports);
  const exports = instance.exports as unknown as FuiRsExports;
  appExports = exports;
  appMemory = exports.memory;

  runtime.resetLogs();
  exports.__runSmokeApp();
  await waitForFrame();

  updateWindowState();
  window.__fuiRsReady = true;
  delete window.__fuiRsError;
}

void window.EffinDomBrowserBridge?.ready.then(async (runtime: BridgeRuntime) => {
  await bootHarness(runtime);
}).catch((error: unknown) => {
  const message = error instanceof Error ? error.message : String(error);
  window.__fuiRsError = message;
  throw error;
});

export {};
