import {
createHostServiceImportModule,
getHostServiceImportNames,
type HostServicesDefinition,
} from './host-services';
import type {
WorkerBootstrapFileProcessStartMessage,
WorkerBootstrapInboundMessage,
WorkerBootstrapOutboundMessage,
WorkerBootstrapStartMessage,
WorkerHostServicesBundleConfig,
} from './worker-types';

const workerScope = globalThis as typeof globalThis & {
  importScripts(...urls: string[]): void;
  postMessage(message: WorkerBootstrapOutboundMessage): void;
  onmessage: ((event: MessageEvent<WorkerBootstrapInboundMessage>) => void) | null;
  __fuiWorkerHostServicesModule?: Record<string, unknown>;
};

const decoder = new TextDecoder();
const encoder = new TextEncoder();
let activeWorkerId: number | null = null;
let activeCancellationRequested = false;
let pendingCancellationRequested = false;
let activeFile: File | null = null;

const allowedWorkerHostImports = new Set([
  'fui_fetch_start',
  'fui_fetch_cancel',
  'fui_file_read_chunk',
  'fui_file_worker_write_chunk',
  'fui_worker_report_progress',
  'fui_worker_complete_string',
  'fui_worker_fail',
  'fui_worker_is_cancelled',
  'fui_worker_request_yield',
  'fui_worker_request_yield_delay',
]);

function describeError(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function loadWorkerHostServices(config: WorkerHostServicesBundleConfig | undefined): HostServicesDefinition | undefined {
  if (config === undefined) {
    return undefined;
  }
  workerScope.importScripts(config.scriptUrl);
  const bundle = workerScope.__fuiWorkerHostServicesModule;
  if (bundle === undefined) {
    throw new Error(`Worker host-services bundle ${config.scriptUrl} did not initialize __fuiWorkerHostServicesModule.`);
  }
  const exported = bundle[config.exportName];
  if (typeof exported !== 'object' || exported === null) {
    throw new Error(`Worker host-services bundle ${config.scriptUrl} does not export "${config.exportName}".`);
  }
  return exported as HostServicesDefinition;
}

function validateWorkerImports(module: WebAssembly.Module, hostServices: HostServicesDefinition | undefined): void {
  const allowedHostServiceImports = getHostServiceImportNames(hostServices);
  const imports = WebAssembly.Module.imports(module);
  for (const imported of imports) {
    if (imported.kind !== 'function') {
      throw new Error(`Worker import ${imported.module}.${imported.name} is not allowed.`);
    }
    if (imported.module === 'env' && imported.name === 'abort') {
      continue;
    }
    if (imported.module === 'fui_worker_host' && allowedWorkerHostImports.has(imported.name)) {
      continue;
    }
    if (imported.module === 'fui_fetch_host' && allowedWorkerHostImports.has(imported.name)) {
      continue;
    }
    if (imported.module === 'fui_host_service' && allowedHostServiceImports.has(imported.name)) {
      continue;
    }
    throw new Error(`Worker import ${imported.module}.${imported.name} is not allowed.`);
  }
}

function readUtf8(memory: WebAssembly.Memory | null, ptr: number, len: number): string {
  if (memory === null || len <= 0) {
    return '';
  }
  return decoder.decode(new Uint8Array(memory.buffer, ptr, len));
}

function writeUtf8(memory: WebAssembly.Memory | null, ptr: number, capacity: number, text: string, context: string): number {
  if (memory === null) {
    throw new Error(`${context} requires worker memory.`);
  }
  if (capacity <= 0) {
    if (text.length === 0) {
      return 0;
    }
    throw new Error(`${context} cannot write into a zero-length worker host-service buffer.`);
  }
  const encoded = encoder.encode(text);
  if (encoded.length > capacity) {
    throw new Error(`${context} exceeds the worker host-service result buffer.`);
  }
  if (encoded.length > 0) {
    new Uint8Array(memory.buffer, ptr, encoded.length).set(encoded);
  }
  return encoded.length;
}

function readBytes(memory: WebAssembly.Memory | null, ptr: number, len: number): Uint8Array {
  if (memory === null || len <= 0) {
    return new Uint8Array(0);
  }
  const bytes = new Uint8Array(len);
  bytes.set(new Uint8Array(memory.buffer, ptr, len));
  return bytes;
}

function writeBytes(memory: WebAssembly.Memory | null, ptr: number, capacity: number, bytes: Uint8Array, context: string): number {
  if (memory === null) {
    throw new Error(`${context} requires worker memory.`);
  }
  if (capacity < 0) {
    throw new Error(`${context} has invalid worker host-service buffer capacity.`);
  }
  if (bytes.length > capacity) {
    throw new Error(`${context} exceeds the worker host-service result buffer.`);
  }
  if (bytes.length > 0) {
    new Uint8Array(memory.buffer, ptr, bytes.length).set(bytes);
  }
  return bytes.length;
}

function encodeTextPartsPayload(values: readonly string[]): Uint8Array {
  const encodedValues = values.map((value) => encoder.encode(value));
  let totalBytes = 4;
  for (const encoded of encodedValues) {
    totalBytes += 4 + encoded.length;
  }
  const bytes = new Uint8Array(totalBytes);
  const dataView = new DataView(bytes.buffer);
  let byteOffset = 0;
  dataView.setUint32(byteOffset, values.length >>> 0, true);
  byteOffset += 4;
  for (const encoded of encodedValues) {
    dataView.setUint32(byteOffset, encoded.length >>> 0, true);
    byteOffset += 4;
    if (encoded.length > 0) {
      bytes.set(encoded, byteOffset);
      byteOffset += encoded.length;
    }
  }
  return bytes;
}

async function startWorker(message: WorkerBootstrapStartMessage): Promise<void> {
  let memory: WebAssembly.Memory | null = null;
  let terminalSent = false;
  let yieldRequested = false;
  let requestedYieldDelayMs = 0;
  let resumeScheduled = false;
  let callbackBufferPtr = 0;
  let callbackBufferSize = 0;
  let wasmExports: (Record<string, unknown> & {
    memory?: WebAssembly.Memory;
    __fui_worker_text_buffer?: () => number;
    __fui_worker_text_buffer_size?: () => number;
  }) | null = null;
  const activeFetchRequests = new Map<number, AbortController>();
  const inputBytes = new TextEncoder().encode(message.input);
  const hostServices = loadWorkerHostServices(message.workerHostServices);
  let entry: (() => void) | null = null;
  let entryStarted = false;
  activeWorkerId = message.workerId;
  activeCancellationRequested = pendingCancellationRequested;
  pendingCancellationRequested = false;

  function readCancelFlag(): boolean {
    return activeWorkerId === message.workerId && activeCancellationRequested;
  }

  function cancelAllFetchRequests(): void {
    for (const controller of activeFetchRequests.values()) {
      controller.abort();
    }
    activeFetchRequests.clear();
  }

  function writeCallbackBytes(bytes: Uint8Array, context: string): { ptr: number; len: number } {
    if (callbackBufferSize <= 0) {
      throw new Error(`${context} requires the worker callback buffer.`);
    }
    if (bytes.length > callbackBufferSize) {
      throw new Error(`${context} exceeds the worker callback buffer.`);
    }
    if (memory === null) {
      throw new Error(`${context} requires worker memory.`);
    }
    if (bytes.length > 0) {
      new Uint8Array(memory.buffer, callbackBufferPtr, bytes.length).set(bytes);
    }
    return {
      ptr: bytes.length > 0 ? callbackBufferPtr : 0,
      len: bytes.length,
    };
  }

  function emitFetchComplete(
    requestId: number,
    ok: boolean,
    status: number,
    statusText: string,
    url: string,
    exports: Record<string, unknown>,
  ): void {
    const callback = exports.__fui_on_fetch_complete;
    if (typeof callback !== 'function') {
      throw new Error('Worker module is missing __fui_on_fetch_complete.');
    }
    const payload = writeCallbackBytes(encodeTextPartsPayload([statusText, url]), 'Worker fetch completion payload');
    (callback as (requestId: number, ok: boolean, status: number, payloadPtr: number, payloadLen: number) => void)(
      requestId,
      ok,
      status,
      payload.ptr,
      payload.len,
    );
  }

  function emitFetchError(
    requestId: number,
    message: string,
    exports: Record<string, unknown>,
  ): void {
    const callback = exports.__fui_on_fetch_error;
    if (typeof callback !== 'function') {
      throw new Error('Worker module is missing __fui_on_fetch_error.');
    }
    const payload = writeCallbackBytes(encoder.encode(message), 'Worker fetch failure payload');
    (callback as (requestId: number, payloadPtr: number, payloadLen: number) => void)(
      requestId,
      payload.ptr,
      payload.len,
    );
  }

  function scheduleResume(): void {
    if (resumeScheduled || terminalSent || entry === null) {
      return;
    }
    resumeScheduled = true;
    const delayMs = requestedYieldDelayMs > 0 ? requestedYieldDelayMs : 0;
    requestedYieldDelayMs = 0;
    setTimeout(() => {
      resumeScheduled = false;
      if (terminalSent || entry === null) {
        return;
      }
      runEntry();
    }, delayMs);
  }

  function hasTerminalSent(): boolean {
    return terminalSent;
  }

  function hasYieldRequested(): boolean {
    return yieldRequested;
  }

  function failWorker(error: unknown): void {
    if (terminalSent) {
      return;
    }
    terminalSent = true;
    cancelAllFetchRequests();
    activeWorkerId = null;
    activeCancellationRequested = false;
    workerScope.postMessage({
      type: 'error',
      workerId: message.workerId,
      text: describeError(error),
    });
  }

  function runEntry(): void {
    if (entry === null || hasTerminalSent()) {
      return;
    }
    try {
      yieldRequested = false;
      if (!entryStarted) {
        entryStarted = true;
        if (inputBytes.length > callbackBufferSize) {
          throw new Error('Worker input exceeds the worker callback buffer.');
        }
        if (inputBytes.length > 0) {
          if (memory === null) {
            throw new Error('Worker input requires worker memory.');
          }
          new Uint8Array(memory.buffer, callbackBufferPtr, inputBytes.length).set(inputBytes);
        }
        (entry as (inputPtr: number, inputLen: number) => void)(
          inputBytes.length > 0 ? callbackBufferPtr : 0,
          inputBytes.length,
        );
      } else {
        (entry as (inputPtr: number, inputLen: number) => void)(0, 0);
      }
    } catch (error: unknown) {
      failWorker(error);
      return;
    }
    if (hasTerminalSent()) {
      return;
    }
    if (hasYieldRequested()) {
      scheduleResume();
      return;
    }
    failWorker('Worker exited without calling Worker.complete(...), Worker.fail(...), or Worker.yield(...).');
  }
  try {
    const response = await fetch(message.wasmUrl, {
      cache: 'no-store',
      credentials: 'same-origin',
    });
    if (!response.ok) {
      throw new Error(`Failed to load worker wasm from ${message.wasmUrl}.`);
    }
    const bytes = await response.arrayBuffer();
    const module = await WebAssembly.compile(bytes);
    validateWorkerImports(module, hostServices);
    const instance = await WebAssembly.instantiate(module, {
      env: {
        abort(_message?: number, _fileName?: number, line?: number, column?: number): never {
          throw new Error(`Worker aborted at ${String(line ?? 0)}:${String(column ?? 0)}.`);
        },
      },
      fui_host_service: createHostServiceImportModule(hostServices, {
        readString: (ptr, len) => readUtf8(memory, ptr, len),
        writeString: (ptr, capacity, text, context) => writeUtf8(memory, ptr, capacity, text, context),
        readBytes: (ptr, len) => readBytes(memory, ptr, len),
        writeBytes: (ptr, capacity, bytes, context) => writeBytes(memory, ptr, capacity, bytes, context),
      }),
      fui_worker_host: {
        fui_worker_report_progress(ptr: number, len: number): void {
          if (terminalSent) {
            return;
          }
          workerScope.postMessage({
            type: 'progress',
            workerId: message.workerId,
            text: readUtf8(memory, ptr, len),
          });
        },
        fui_worker_complete_string(ptr: number, len: number): void {
          if (terminalSent) {
            return;
          }
          terminalSent = true;
          cancelAllFetchRequests();
          activeWorkerId = null;
          activeCancellationRequested = false;
          workerScope.postMessage({
            type: 'complete',
            workerId: message.workerId,
            text: readUtf8(memory, ptr, len),
          });
        },
        fui_worker_fail(ptr: number, len: number): void {
          if (terminalSent) {
            return;
          }
          terminalSent = true;
          cancelAllFetchRequests();
          activeWorkerId = null;
          activeCancellationRequested = false;
          workerScope.postMessage({
            type: 'error',
            workerId: message.workerId,
            text: readUtf8(memory, ptr, len),
          });
        },
        fui_worker_is_cancelled(): number {
          return readCancelFlag() ? 1 : 0;
        },
        fui_worker_request_yield(): void {
          yieldRequested = true;
          requestedYieldDelayMs = 0;
        },
        fui_worker_request_yield_delay(delayMs: number): void {
          yieldRequested = true;
          requestedYieldDelayMs = Number.isFinite(delayMs) && delayMs > 0 ? Math.floor(delayMs) : 0;
        },
        fui_file_read_chunk(offsetLow: number, offsetHigh: number, length: number): number {
          if (activeFile === null) {
            return 0;
          }
          const offset = Number(BigInt(offsetLow >>> 0) | (BigInt(offsetHigh >>> 0) << 32n));
          const safeLength = Math.max(0, length | 0);
          if (offset >= activeFile.size || safeLength <= 0) {
            return 0;
          }
          const blob = activeFile.slice(offset, Math.min(offset + safeLength, activeFile.size));
          const reader = new FileReaderSync();
          const buffer = reader.readAsArrayBuffer(blob);
          const bytes = new Uint8Array(buffer);
          const written = bytes.length;
          if (written <= 0) {
            return 0;
          }
          if (memory === null || callbackBufferSize <= 0) {
            return 0;
          }
          if (written > callbackBufferSize) {
            throw new Error('File chunk exceeds the worker callback buffer.');
          }
          new Uint8Array(memory.buffer, callbackBufferPtr, written).set(bytes);
          return written;
        },
        fui_file_worker_write_chunk(ptr: number, len: number): void {
          if (memory === null || len <= 0) {
            return;
          }
          const bytes = new Uint8Array(len);
          bytes.set(new Uint8Array(memory.buffer, ptr, len));
          const buffer = bytes.buffer.slice(0, bytes.byteLength);
          workerScope.postMessage({
            type: 'file-process-chunk',
            workerId: message.workerId,
            bytes: buffer,
          }, [buffer]);
        },
      },
      fui_fetch_host: {
        fui_fetch_start(
          requestId: number,
          methodPtr: number,
          methodLen: number,
          urlPtr: number,
          urlLen: number,
          headersPtr: number,
          headersLen: number,
          bodyPtr: number,
          bodyLen: number,
        ): void {
          const controller = new AbortController();
          const method = readUtf8(memory, methodPtr, methodLen);
          const url = readUtf8(memory, urlPtr, urlLen);
          const headerBytes = memory === null || headersLen <= 0
            ? new Uint8Array(0)
            : new Uint8Array(memory.buffer.slice(headersPtr, headersPtr + headersLen));
          if (headerBytes.byteLength < 4 && headersLen > 0) {
            throw new Error('Worker fetch header payload was truncated.');
          }
          const headers = new Headers();
          if (headerBytes.byteLength >= 4) {
            const dataView = new DataView(headerBytes.buffer, headerBytes.byteOffset, headerBytes.byteLength);
            let byteOffset = 0;
            const count = dataView.getUint32(byteOffset, true);
            byteOffset += 4;
            const values: string[] = [];
            for (let index = 0; index < count; index += 1) {
              if (byteOffset + 4 > headerBytes.byteLength) {
                throw new Error('Worker fetch header length was truncated.');
              }
              const partLen = dataView.getUint32(byteOffset, true);
              byteOffset += 4;
              if (byteOffset + partLen > headerBytes.byteLength) {
                throw new Error('Worker fetch header value was truncated.');
              }
              values.push(partLen > 0 ? decoder.decode(headerBytes.subarray(byteOffset, byteOffset + partLen)) : '');
              byteOffset += partLen;
            }
            if ((values.length & 1) != 0) {
              throw new Error('Worker fetch headers were malformed.');
            }
            for (let index = 0; index < values.length; index += 2) {
              headers.append(values[index] ?? '', values[index + 1] ?? '');
            }
          }
          const body = memory === null || bodyLen <= 0
            ? null
            : memory.buffer.slice(bodyPtr, bodyPtr + bodyLen);
          activeFetchRequests.set(requestId, controller);
          const init: RequestInit = {
            method,
            headers,
            signal: controller.signal,
          };
          if (body !== null) {
            init.body = body;
          }
          void fetch(url, init).then((response) => {
            const active = activeFetchRequests.get(requestId);
            if (active === undefined || active !== controller || terminalSent) {
              return;
            }
            activeFetchRequests.delete(requestId);
            if (wasmExports === null) {
              throw new Error('Worker fetch completed before wasm exports were ready.');
            }
            emitFetchComplete(requestId, response.ok, response.status, response.statusText, response.url, wasmExports);
          }).catch((error: unknown) => {
            const active = activeFetchRequests.get(requestId);
            if (active === undefined || active !== controller) {
              return;
            }
            activeFetchRequests.delete(requestId);
            if (controller.signal.aborted || terminalSent) {
              return;
            }
            if (wasmExports === null) {
              throw new Error('Worker fetch failed before wasm exports were ready.');
            }
            emitFetchError(requestId, describeError(error), wasmExports);
          });
        },
        fui_fetch_cancel(requestId: number): void {
          const controller = activeFetchRequests.get(requestId);
          if (controller === undefined) {
            return;
          }
          activeFetchRequests.delete(requestId);
          controller.abort();
        },
      },
    });
    const exports = instance.exports as Record<string, unknown> & {
      memory?: WebAssembly.Memory;
      __fui_worker_text_buffer?: () => number;
      __fui_worker_text_buffer_size?: () => number;
    };
    wasmExports = exports;
    if (!(exports.memory instanceof WebAssembly.Memory)) {
      throw new Error('Worker module did not export memory.');
    }
    memory = exports.memory;
    if (typeof exports.__fui_worker_text_buffer !== 'function' || typeof exports.__fui_worker_text_buffer_size !== 'function') {
      throw new Error('Worker module did not export the fetch callback buffer.');
    }
    callbackBufferPtr = exports.__fui_worker_text_buffer();
    callbackBufferSize = exports.__fui_worker_text_buffer_size();
    const exportedEntry = exports[message.entryName];
    if (typeof exportedEntry !== 'function') {
      throw new Error(`Worker export "${message.entryName}" is missing.`);
    }
    entry = exportedEntry as () => void;
    runEntry();
  } catch (error: unknown) {
    cancelAllFetchRequests();
    activeWorkerId = null;
    activeCancellationRequested = false;
    workerScope.postMessage({
      type: 'error',
      workerId: message.workerId,
      text: describeError(error),
    });
  }
}

async function startFileProcessWorker(message: WorkerBootstrapFileProcessStartMessage): Promise<void> {
  let memory: WebAssembly.Memory | null = null;
  let terminalSent = false;
  let callbackBufferPtr = 0;
  let callbackBufferSize = 0;
  let wasmExports: (Record<string, unknown> & {
    memory?: WebAssembly.Memory;
    __fui_worker_text_buffer?: () => number;
    __fui_worker_text_buffer_size?: () => number;
  }) | null = null;
  const activeFetchRequests = new Map<number, AbortController>();
  let entry: (() => void) | null = null;
  activeWorkerId = message.workerId;
  activeCancellationRequested = pendingCancellationRequested;
  activeFile = message.file;
  pendingCancellationRequested = false;

  function readCancelFlag(): boolean {
    return activeWorkerId === message.workerId && activeCancellationRequested;
  }

  function cancelAllFetchRequests(): void {
    for (const controller of activeFetchRequests.values()) {
      controller.abort();
    }
    activeFetchRequests.clear();
  }

  function writeCallbackBytes(bytes: Uint8Array, context: string): { ptr: number; len: number } {
    if (callbackBufferSize <= 0) {
      throw new Error(`${context} requires the worker callback buffer.`);
    }
    if (bytes.length > callbackBufferSize) {
      throw new Error(`${context} exceeds the worker callback buffer.`);
    }
    if (memory === null) {
      throw new Error(`${context} requires worker memory.`);
    }
    if (bytes.length > 0) {
      new Uint8Array(memory.buffer, callbackBufferPtr, bytes.length).set(bytes);
    }
    return {
      ptr: bytes.length > 0 ? callbackBufferPtr : 0,
      len: bytes.length,
    };
  }

  function emitFetchComplete(
    requestId: number,
    ok: boolean,
    status: number,
    statusText: string,
    url: string,
    exports: Record<string, unknown>,
  ): void {
    const callback = exports.__fui_on_fetch_complete;
    if (typeof callback !== 'function') {
      throw new Error('Worker module is missing __fui_on_fetch_complete.');
    }
    const payload = writeCallbackBytes(encodeTextPartsPayload([statusText, url]), 'Worker fetch completion payload');
    (callback as (requestId: number, ok: boolean, status: number, payloadPtr: number, payloadLen: number) => void)(
      requestId,
      ok,
      status,
      payload.ptr,
      payload.len,
    );
  }

  function emitFetchError(
    requestId: number,
    message: string,
    exports: Record<string, unknown>,
  ): void {
    const callback = exports.__fui_on_fetch_error;
    if (typeof callback !== 'function') {
      throw new Error('Worker module is missing __fui_on_fetch_error.');
    }
    const payload = writeCallbackBytes(encoder.encode(message), 'Worker fetch failure payload');
    (callback as (requestId: number, payloadPtr: number, payloadLen: number) => void)(
      requestId,
      payload.ptr,
      payload.len,
    );
  }

  function runEntry(): void {
    if (entry === null || wasmExports === null) {
      return;
    }
    try {
    (entry as (inputPtr: number, inputLen: number) => void)(0, 0);
    } catch (error: unknown) {
      if (terminalSent) {
        return;
      }
      terminalSent = true;
      cancelAllFetchRequests();
      activeWorkerId = null;
      activeCancellationRequested = false;
      activeFile = null;
      workerScope.postMessage({
        type: 'error',
        workerId: message.workerId,
        text: describeError(error),
      });
      return;
    }
    if (terminalSent) {
      return;
    }
  }

  function cleanupTerminal(): void {
    terminalSent = true;
    cancelAllFetchRequests();
    activeWorkerId = null;
    activeCancellationRequested = false;
  }
  try {
    const response = await fetch(message.wasmUrl, {
      cache: 'no-store',
      credentials: 'same-origin',
    });
    if (!response.ok) {
      throw new Error(`Failed to load worker wasm from ${message.wasmUrl}.`);
    }
    const bytes = await response.arrayBuffer();
    const module = await WebAssembly.compile(bytes);
    const hostServices = loadWorkerHostServices(message.workerHostServices);
    validateWorkerImports(module, hostServices);
    const instance = await WebAssembly.instantiate(module, {
      env: {
        abort(_message?: number, _fileName?: number, line?: number, column?: number): never {
          throw new Error(`Worker aborted at ${String(line ?? 0)}:${String(column ?? 0)}.`);
        },
      },
      fui_host_service: createHostServiceImportModule(hostServices, {
        readString: (ptr, len) => readUtf8(memory, ptr, len),
        writeString: (ptr, capacity, text, context) => writeUtf8(memory, ptr, capacity, text, context),
        readBytes: (ptr, len) => readBytes(memory, ptr, len),
        writeBytes: (ptr, capacity, bytes, context) => writeBytes(memory, ptr, capacity, bytes, context),
      }),
      fui_worker_host: {
        fui_worker_report_progress(ptr: number, len: number): void {
          if (terminalSent) {
            return;
          }
          workerScope.postMessage({
            type: 'progress',
            workerId: message.workerId,
            text: readUtf8(memory, ptr, len),
          });
        },
        fui_worker_complete_string(ptr: number, len: number): void {
          if (terminalSent) {
            return;
          }
          cleanupTerminal();
          activeFile = null;
          workerScope.postMessage({
            type: 'complete',
            workerId: message.workerId,
            text: readUtf8(memory, ptr, len),
          });
        },
        fui_worker_fail(ptr: number, len: number): void {
          if (terminalSent) {
            return;
          }
          cleanupTerminal();
          activeFile = null;
          workerScope.postMessage({
            type: 'error',
            workerId: message.workerId,
            text: readUtf8(memory, ptr, len),
          });
        },
        fui_worker_is_cancelled(): number {
          return readCancelFlag() ? 1 : 0;
        },
        fui_worker_request_yield(): void {
          // File processing is synchronous — yield is a no-op.
        },
        fui_worker_request_yield_delay(): void {
          // File processing is synchronous — yield is a no-op.
        },
        fui_file_read_chunk(offsetLow: number, offsetHigh: number, length: number): number {
          if (activeFile === null) {
            return 0;
          }
          const offset = Number(BigInt(offsetLow >>> 0) | (BigInt(offsetHigh >>> 0) << 32n));
          const safeLength = Math.max(0, length | 0);
          if (offset >= activeFile.size || safeLength <= 0) {
            return 0;
          }
          const blob = activeFile.slice(offset, Math.min(offset + safeLength, activeFile.size));
          const reader = new FileReaderSync();
          const buffer = reader.readAsArrayBuffer(blob);
          const readBytes = new Uint8Array(buffer);
          const written = readBytes.length;
          if (written <= 0) {
            return 0;
          }
          if (memory === null || callbackBufferSize <= 0) {
            return 0;
          }
          if (written > callbackBufferSize) {
            throw new Error('File chunk exceeds the worker callback buffer.');
          }
          new Uint8Array(memory.buffer, callbackBufferPtr, written).set(readBytes);
          return written;
        },
        fui_file_worker_write_chunk(ptr: number, len: number): void {
          if (memory === null || len <= 0) {
            return;
          }
          const chunkBytes = new Uint8Array(len);
          chunkBytes.set(new Uint8Array(memory.buffer, ptr, len));
          const buffer = chunkBytes.buffer.slice(0, chunkBytes.byteLength);
          workerScope.postMessage({
            type: 'file-process-chunk',
            workerId: message.workerId,
            bytes: buffer,
          }, [buffer]);
        },
      },
      fui_fetch_host: {
        fui_fetch_start(
          requestId: number,
          methodPtr: number,
          methodLen: number,
          urlPtr: number,
          urlLen: number,
          headersPtr: number,
          headersLen: number,
          bodyPtr: number,
          bodyLen: number,
        ): void {
          const controller = new AbortController();
          const method = readUtf8(memory, methodPtr, methodLen);
          const url = readUtf8(memory, urlPtr, urlLen);
          const headerBytes = memory === null || headersLen <= 0
            ? new Uint8Array(0)
            : new Uint8Array(memory.buffer.slice(headersPtr, headersPtr + headersLen));
          if (headerBytes.byteLength < 4 && headersLen > 0) {
            throw new Error('Worker fetch header payload was truncated.');
          }
          const headers = new Headers();
          if (headerBytes.byteLength >= 4) {
            const dataView = new DataView(headerBytes.buffer, headerBytes.byteOffset, headerBytes.byteLength);
            let byteOffset = 0;
            const count = dataView.getUint32(byteOffset, true);
            byteOffset += 4;
            const values: string[] = [];
            for (let index = 0; index < count; index += 1) {
              if (byteOffset + 4 > headerBytes.byteLength) {
                throw new Error('Worker fetch header length was truncated.');
              }
              const partLen = dataView.getUint32(byteOffset, true);
              byteOffset += 4;
              if (byteOffset + partLen > headerBytes.byteLength) {
                throw new Error('Worker fetch header value was truncated.');
              }
              values.push(partLen > 0 ? decoder.decode(headerBytes.subarray(byteOffset, byteOffset + partLen)) : '');
              byteOffset += partLen;
            }
            if ((values.length & 1) != 0) {
              throw new Error('Worker fetch headers were malformed.');
            }
            for (let index = 0; index < values.length; index += 2) {
              headers.append(values[index] ?? '', values[index + 1] ?? '');
            }
          }
          const bodyBytes = memory === null || bodyLen <= 0
            ? new Uint8Array(0)
            : new Uint8Array(memory.buffer, bodyPtr, bodyLen);
          const body = bodyBytes.length > 0 ? bodyBytes : null;
          activeFetchRequests.set(requestId, controller);
          const init: RequestInit = {
            method,
            headers,
            signal: controller.signal,
          };
          if (body !== null) {
            init.body = body;
          }
          void fetch(url, init).then((response) => {
            const active = activeFetchRequests.get(requestId);
            if (active === undefined || active !== controller || terminalSent) {
              return;
            }
            activeFetchRequests.delete(requestId);
            if (wasmExports === null) {
              throw new Error('Worker fetch completed before wasm exports were ready.');
            }
            emitFetchComplete(requestId, response.ok, response.status, response.statusText, response.url, wasmExports);
          }).catch((error: unknown) => {
            const active = activeFetchRequests.get(requestId);
            if (active === undefined || active !== controller) {
              return;
            }
            activeFetchRequests.delete(requestId);
            if (controller.signal.aborted || terminalSent) {
              return;
            }
            if (wasmExports === null) {
              throw new Error('Worker fetch failed before wasm exports were ready.');
            }
            emitFetchError(requestId, describeError(error), wasmExports);
          });
        },
        fui_fetch_cancel(requestId: number): void {
          const controller = activeFetchRequests.get(requestId);
          if (controller === undefined) {
            return;
          }
          activeFetchRequests.delete(requestId);
          controller.abort();
        },
      },
    });
    const exports = instance.exports as Record<string, unknown> & {
      memory?: WebAssembly.Memory;
      __fui_worker_text_buffer?: () => number;
      __fui_worker_text_buffer_size?: () => number;
    };
    wasmExports = exports;
    if (!(exports.memory instanceof WebAssembly.Memory)) {
      throw new Error('Worker module did not export memory.');
    }
    memory = exports.memory;
    if (typeof exports.__fui_worker_text_buffer !== 'function' || typeof exports.__fui_worker_text_buffer_size !== 'function') {
      throw new Error('Worker module did not export the fetch callback buffer.');
    }
    callbackBufferPtr = exports.__fui_worker_text_buffer();
    callbackBufferSize = exports.__fui_worker_text_buffer_size();
    const exportedEntry = exports[message.entryName];
    if (typeof exportedEntry !== 'function') {
      throw new Error(`Worker export "${message.entryName}" is missing.`);
    }
    entry = exportedEntry as () => void;
    runEntry();
  } catch (error: unknown) {
    cancelAllFetchRequests();
    activeWorkerId = null;
    activeCancellationRequested = false;
    activeFile = null;
    workerScope.postMessage({
      type: 'error',
      workerId: message.workerId,
      text: describeError(error),
    });
  }
}

workerScope.onmessage = (event: MessageEvent<WorkerBootstrapInboundMessage>) => {
  const message = event.data;
  if (message.type === 'start') {
    void startWorker(message);
    return;
  }
  if (message.type === 'start-file-process') {
    void startFileProcessWorker(message);
    return;
  }
  if (activeWorkerId === message.workerId) {
    activeCancellationRequested = true;
    return;
  }
  pendingCancellationRequested = true;
};
