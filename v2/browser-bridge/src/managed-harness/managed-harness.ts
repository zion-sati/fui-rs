import { instantiate } from '@assemblyscript/loader';

import type { BridgeRuntime,EffinDomCallbacks,WasmHandleLike } from '@effindomv2/runtime';
import { listHostEventMethods,type HostEventsDefinition,type NormalizedHostEventMethod } from './host-events';
import { createHostServiceImportModule,getHostServiceImportNames,type HostServicesDefinition } from './host-services';
import { createWorkerManager } from './worker-manager';
import { assertCompatibleAbi } from './abi-version';
import { createHostImportModule } from './host-imports';
import {
type AppHandleLike,
normalizePointer,
toBigIntHandle,
zeroPointer
} from './interop';
import { createManagedHarnessBitmapHost } from './managed-harness-bitmap-host';
import { createManagedHarnessCanvasHost } from './managed-harness-canvas-host';
import { createManagedHarnessFetchHost } from './managed-harness-fetch-host';
import { createManagedHarnessFileHost } from './managed-harness-file-host';
import {
EXTERNAL_DRAG_EVENT_DROP,
EXTERNAL_DRAG_EVENT_ENTER,
EXTERNAL_DRAG_EVENT_LEAVE,
EXTERNAL_DRAG_EVENT_OVER,
} from './managed-harness-file-types';
import type { HarnessAppSession } from './managed-harness-session';
import {
canBrowserNavigateBack,
canBrowserNavigateForward,
ensureManagedHistoryInitialized,
navigateBrowserBack,
navigateBrowserForward,
pushManagedHistoryEntry,
replaceManagedHistoryEntry,
syncManagedHistoryPop
} from './managed-history';
import { PersistedUiStateController } from './persisted-ui-state-controller';
import { TextSessionBridge } from './text-session-bridge';
import type {
HarnessAppOptions,
HarnessContext,
HarnessController,
HarnessDebugApi,
HarnessExports,
HarnessNavigationMode,
HarnessOptions,
HarnessState,
ManagedHarnessOptions,
} from './types';
import { HarnessUiChrome,waitForFrame } from './ui-chrome';
import { createUiImportModule } from './ui-imports';

type AutoHarnessExports = HarnessExports & {
  __runApp?: () => void;
  __disposeApp?: () => void;
};

function tryResolveNavigationTarget(target: string): URL | null {
  try {
    return new URL(target, window.location.href);
  } catch {
    return null;
  }
}

function toAppRoute(url: URL): string {
  return `${url.pathname}${url.search}${url.hash}`;
}

const encoder = new TextEncoder();
const harnessUiChrome = new HarnessUiChrome();

function isHandledResult(value: unknown): boolean {
  return value === true || value === 1;
}

function applyHarnessRuntimeOptions(options: {
  readonly buildMode?: string;
  readonly devToolsDomMirror?: string;
  readonly pageZoom?: string;
}): void {
  const update: Record<string, string> = {};
  if (options.buildMode !== undefined) {
    update.buildMode = options.buildMode;
  }
  if (options.devToolsDomMirror !== undefined) {
    update.devToolsDomMirror = options.devToolsDomMirror;
  }
  if (options.pageZoom !== undefined) {
    update.pageZoom = options.pageZoom;
  }
  if (Object.keys(update).length === 0) {
    return;
  }
  const runtimeWindow = window as unknown as Window & { __effindomRuntime?: Partial<Record<'manifestUrl' | 'buildMode' | 'devToolsDomMirror' | 'pageZoom', string>> };
  runtimeWindow.__effindomRuntime = Object.assign({}, runtimeWindow.__effindomRuntime, update);
}

function describeHarnessError(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function defaultRunHarnessApp(exports: HarnessExports): void {
  const autoExports = exports as AutoHarnessExports;
  if (typeof autoExports.__runApp !== 'function') {
    throw new Error(
      'startHarness default run requires an exported __runApp(). Provide run(...) if your entrypoint uses a different symbol.',
    );
  }
  autoExports.__runApp();
}

function defaultOnDispose(exports: HarnessExports): void {
  (exports as AutoHarnessExports).__disposeApp?.();
}

function defaultOnStateUpdated(state: HarnessState): void {
  (window as Window & { __fuiAsState?: HarnessState }).__fuiAsState = state;
}

function defaultOnReady(): void {
  const windowWithHarnessState = window as Window & { __fuiAsReady?: boolean; __fuiAsError?: string };
  windowWithHarnessState.__fuiAsReady = true;
  delete windowWithHarnessState.__fuiAsError;
}

function defaultOnError(error: unknown): void {
  const windowWithHarnessState = window as Window & { __fuiAsReady?: boolean; __fuiAsError?: string };
  windowWithHarnessState.__fuiAsReady = false;
  windowWithHarnessState.__fuiAsError = error instanceof Error ? error.message : String(error);
}

export function startHarness<Exports extends HarnessExports>(options: HarnessOptions<Exports>): void {
  applyHarnessRuntimeOptions(options);
  const loadOptions: HarnessAppOptions<Exports> = {
    ...options,
    run: options.run ?? ((exports) => { defaultRunHarnessApp(exports); }),
    onStateUpdated: options.onStateUpdated ?? defaultOnStateUpdated,
    onReady: options.onReady ?? (() => { defaultOnReady(); }),
    onDispose: options.onDispose ?? ((exports) => { defaultOnDispose(exports); }),
  };
  const onError = options.onError ?? defaultOnError;
  startManagedHarness({
    async onReady(controller): Promise<void> {
      await controller.loadApp(loadOptions);
    },
    onError,
  });
}

export function startManagedHarness(options: ManagedHarnessOptions): void {
  applyHarnessRuntimeOptions(options);
  let cleanup = () => {
    delete window.__fui_debug;
  };
  const loadingOverlayText = harnessUiChrome.getLoadingOverlayText();
  const loadingOverlayTitle = loadingOverlayText.title;
  let loadingOverlayDetail = loadingOverlayText.detail;
  let loadingOverlayVisible = false;
  let loadingOverlayTimer: number | null = null;

  function clearLoadingOverlayTimer(): void {
    if (loadingOverlayTimer === null) {
      return;
    }
    window.clearTimeout(loadingOverlayTimer);
    loadingOverlayTimer = null;
  }

  function scheduleLoadingOverlay(): void {
    if (loadingOverlayVisible || loadingOverlayTimer !== null) {
      return;
    }
    loadingOverlayTimer = window.setTimeout(() => {
      loadingOverlayTimer = null;
      loadingOverlayVisible = true;
      harnessUiChrome.setLoadingOverlay('loading', loadingOverlayTitle, loadingOverlayDetail);
    }, 1000);
  }

  function updateLoadingOverlay(detail: string): void {
    loadingOverlayDetail = detail;
    if (loadingOverlayVisible) {
      harnessUiChrome.setLoadingOverlay('loading', loadingOverlayTitle, loadingOverlayDetail);
      return;
    }
    scheduleLoadingOverlay();
  }

  function finishLoadingOverlay(): void {
    clearLoadingOverlayTimer();
    loadingOverlayVisible = false;
    harnessUiChrome.hideLoadingOverlay();
  }

  function failLoadingOverlay(detail: string): void {
    clearLoadingOverlayTimer();
    loadingOverlayVisible = true;
    harnessUiChrome.setLoadingOverlay('error', loadingOverlayTitle, detail);
  }

  const bridge = window.EffinDomBrowserBridge;
  if (bridge === undefined) {
    failLoadingOverlay('EffinDomBrowserBridge is unavailable.');
    throw new Error('EffinDomBrowserBridge is unavailable.');
  }
  if (typeof Worker !== 'function') {
    failLoadingOverlay('Managed harness requires browser Worker support.');
    throw new Error('Managed harness requires browser Worker support.');
  }
  const bridgeState = bridge;
  void bridgeState.ready.then(async (initialRuntime: BridgeRuntime) => {
    assertCompatibleAbi(initialRuntime);
    ensureManagedHistoryInitialized();
    const debugLogsEnabled = new URLSearchParams(window.location.search).get('debug-logs') === '1';
    const darkModeQuery = window.matchMedia('(prefers-color-scheme: dark)');
    let runtime = initialRuntime;
    let currentSession: HarnessAppSession | null = null;
    let navigationHandler: ((target: URL, mode: HarnessNavigationMode) => void | Promise<void>) | null = null;
    let harnessFrameQueued = false;
    let appFlushRequested = false;
    let missingWheelExportLogged = false;
    const hostTimers = new Map<number, number>();
    let lastHandledUrlHref = window.location.href;
    let latestCommandWords: number[] = [];
    let latestRootHandle: string | null = null;
    const wasmByteCache = new Map<string, Promise<ArrayBuffer>>();
    const wasmModuleCache = new Map<string, Promise<WebAssembly.Module>>();
    let lastSystemAccentColor = -1;
    harnessUiChrome.setUrlPreviewText('');

    function getCurrentSession(): HarnessAppSession {
      if (currentSession === null) {
        throw new Error('No managed app is currently mounted.');
      }
      return currentSession;
    }

    function getCurrentMemory(): WebAssembly.Memory {
      return getCurrentSession().memory;
    }

    const persistedUiStateController = new PersistedUiStateController();
    const textBridge: TextSessionBridge = new TextSessionBridge(() => runtime, getCurrentMemory, queueHarnessFrame);
    let recordRuntimeTextChangedFromAppSet: ((handle: AppHandleLike, text: string) => void) | null = null;

    function readAppUtf8(ptr: number, len: number): string {
      return textBridge.readAppUtf8(ptr, len);
    }

    function readAppFloats(ptr: number, count: number): Float32Array {
      return textBridge.readAppFloats(ptr, count);
    }

    function readAppBytes(ptr: number, len: number): Uint8Array {
      return textBridge.readAppBytes(ptr, len);
    }

    function readAppTextParts(ptr: number, len: number): string[] {
      return textBridge.readAppTextParts(ptr, len);
    }

    function writeTextCallbackPayload(session: HarnessAppSession, text: string, context: string): number {
      return textBridge.writeTextCallbackPayload(session, text, context);
    }

    function writeWorkerTextCallbackPayload(session: {
      readonly memory: WebAssembly.Memory;
      readonly textBufferPtr: number;
      readonly textBufferSize: number;
    }, text: string, context: string): number {
      return textBridge.writeWorkerTextCallbackPayload(session, text, context);
    }

    function writeTextToSessionBuffer(session: HarnessAppSession, text: string): number {
      return textBridge.writeTextToSessionBuffer(session, text);
    }

    function writeAppFloat32(ptr: number, value: number): void {
      textBridge.writeAppFloat32(ptr, value);
    }

    function writeAppUint32(ptr: number, value: number): void {
      textBridge.writeAppUint32(ptr, value);
    }

    function writeAppUtf8(ptr: number, capacity: number, text: string, context: string): number {
      return textBridge.writeAppUtf8(ptr, capacity, text, context);
    }

    function writeAppBytes(ptr: number, capacity: number, bytes: Uint8Array, context: string): number {
      if (capacity < 0) {
        throw new Error(`${context} has invalid buffer capacity ${String(capacity)}.`);
      }
      if (bytes.length > capacity) {
        throw new Error(`${context} returned ${String(bytes.length)} bytes but the shared result buffer only holds ${String(capacity)}.`);
      }
      if (bytes.length > 0) {
        new Uint8Array(getCurrentMemory().buffer, ptr, bytes.length).set(bytes);
      }
      return bytes.length;
    }

    function withUiUtf8(
      text: string,
      callback: (ptr: WasmHandleLike | number, len: number) => void,
    ): void {
      textBridge.withUiUtf8(text, callback);
    }

    function withUiGridData(
      values: Float32Array,
      types: Uint8Array,
      callback: (valuesPtr: WasmHandleLike | number, typesPtr: WasmHandleLike | number) => void,
    ): void {
      textBridge.withUiGridData(values, types, callback);
    }

    function withUiGradientData(
      offsets: Float32Array,
      colors: Uint32Array,
      callback: (offsetsPtr: WasmHandleLike | number, colorsPtr: WasmHandleLike | number) => void,
    ): void {
      textBridge.withUiGradientData(offsets, colors, callback);
    }

    const bitmapHost = createManagedHarnessBitmapHost({
      getRuntime: () => runtime,
      readAppBytes,
      writeAppBytes,
      notifyBitmapChanged(): void {
        appFlushRequested = true;
        runtime.requestFrame();
        queueHarnessFrame();
      },
    });
    bitmapHost.installReplay(runtime);

    const canvasHost = createManagedHarnessCanvasHost({
      getRuntime: () => runtime,
      readAppBytes,
      writeAppBytes,
    });

    async function settleCurrentSessionAfterRestore(context: string): Promise<void> {
      const session = currentSession;
      if (session === null) {
        return;
      }
      for (let iteration = 0; iteration < 2; iteration += 1) {
        session.exports.__flushRenders();
        runtime.flushPendingCommit();
        await waitForFrame();
      }
      persistedUiStateController.restoreCurrentPersistedUiState(
        `${context} after initial paint`,
        currentSession?.exports.__fui_restore_persisted_ui_state,
      );
      for (let iteration = 0; iteration < 2; iteration += 1) {
        session.exports.__flushRenders();
        runtime.flushPendingCommit();
        await waitForFrame();
      }
    }

    function updateState(): void {
      currentSession?.onStateUpdated?.({
        commandWordCount: latestCommandWords.length,
        commandWords: latestCommandWords,
        rootHandle: latestRootHandle,
      });
    }

    function queueHarnessFrame(): void {
      if (harnessFrameQueued) {
        return;
      }
      harnessFrameQueued = true;
      requestAnimationFrame(() => {
        requestAnimationFrame(() => {
          latestCommandWords = Array.from(runtime.extractCommandBuffer());
          updateState();
          harnessFrameQueued = false;
        });
      });
    }

    function queuePersistedUiStateWork<T>(work: () => Promise<T>): Promise<T> {
      return persistedUiStateController.queuePersistedUiStateWork(work);
    }

    async function loadPopPersistedSnapshot(context: string, routeHref: string = window.location.href) {
      return persistedUiStateController.loadPopPersistedSnapshot(context, routeHref);
    }

    async function loadInitialPersistedSnapshot(context: string) {
      return persistedUiStateController.loadInitialPersistedSnapshot(context);
    }

    async function saveCurrentHistoryEntrySnapshot(context: string): Promise<string | null> {
      return persistedUiStateController.saveCurrentHistoryEntrySnapshot(
        context,
        currentSession?.exports.__fui_capture_persisted_ui_state,
      );
    }

    async function saveRouteHeadSnapshotForHref(routeHref: string, context: string): Promise<string | null> {
      return persistedUiStateController.saveRouteHeadSnapshotForHref(
        routeHref,
        context,
        currentSession?.exports.__fui_capture_persisted_ui_state,
      );
    }

    async function ensureCurrentHistoryEntrySnapshot(context: string): Promise<string | null> {
      return persistedUiStateController.ensureCurrentHistoryEntrySnapshot(
        context,
        currentSession?.exports.__fui_capture_persisted_ui_state,
      );
    }

    function hydrateCurrentPersistedEntries(snapshot: unknown): void {
      persistedUiStateController.hydrateCurrentPersistedEntries(snapshot as never);
    }

    function restoreCurrentPersistedUiState(context: string): void {
      persistedUiStateController.restoreCurrentPersistedUiState(
        context,
        currentSession?.exports.__fui_restore_persisted_ui_state,
      );
    }

    function resetUiState(): void {
      latestCommandWords = [];
      latestRootHandle = null;
      textBridge.clearState();
      updateState();
    }

    function cancelHostTimer(timerId: number): void {
      const timeoutId = hostTimers.get(timerId);
      if (timeoutId === undefined) {
        return;
      }
      window.clearTimeout(timeoutId);
      hostTimers.delete(timerId);
    }

    function cancelAllHostTimers(): void {
      for (const timeoutId of hostTimers.values()) {
        window.clearTimeout(timeoutId);
      }
      hostTimers.clear();
    }

    function notifyRouteChanged(session: HarnessAppSession | null, route: string): void {
      if (
        session === null ||
        session.textBufferPtr === 0 ||
        session.textBufferSize === 0
      ) {
        return;
      }
      const encoded = encoder.encode(route);
      if (encoded.length > session.textBufferSize) {
        throw new Error('Route text exceeds the shared AssemblyScript text buffer.');
      }
      if (encoded.length > 0) {
        const memory = new Uint8Array(session.memory.buffer, session.textBufferPtr, encoded.length);
        memory.set(encoded);
      }
      session.exports.__fui_on_route_changed(session.textBufferPtr, encoded.length);
    }

    function notifyRouteForCurrentLocation(session: HarnessAppSession | null = currentSession): void {
      notifyRouteChanged(session, `${window.location.pathname}${window.location.search}${window.location.hash}`);
    }

    function resolveHarnessBaseUrl(): string {
      const scripts = Array.from(document.scripts);
      for (let index = scripts.length - 1; index >= 0; index -= 1) {
        const source = scripts[index]?.src ?? '';
        if (source.endsWith('/harness.js') || source.endsWith('harness.js')) {
          return source;
        }
      }
      return window.location.href;
    }

    const harnessBaseUrl = resolveHarnessBaseUrl();
    const workerBootstrapUrl = new URL('./worker-bootstrap.js', harnessBaseUrl).toString();

    const fileHost = createManagedHarnessFileHost({
      getCurrentSession: () => currentSession,
      getRuntime: () => runtime,
      readAppUtf8,
      readAppBytes,
      writeTextCallbackPayload,
      describeHarnessError,
      workerBootstrapUrl,
      getCurrentWorkerHostServices: () => currentSession?.workerHostServices,
    });

    const fetchHost = createManagedHarnessFetchHost({
      getCurrentSession: () => currentSession,
      readAppUtf8,
      readAppBytes,
      readAppTextParts,
      writeTextCallbackPayload,
      describeHarnessError,
    });

    const workerManager = createWorkerManager({
      scriptBaseUrl: harnessBaseUrl,
      getCurrentSession: () => currentSession,
      getCurrentWorkerHostServices: () => currentSession?.workerHostServices,
      writeTextCallbackPayload: writeWorkerTextCallbackPayload,
    });
    function notifyViewport(session: HarnessAppSession | null = currentSession): void {
      if (session === null) {
        return;
      }
      const rect = runtime.canvas.getBoundingClientRect();
      const width = rect.width > 0 ? rect.width : runtime.canvas.width;
      const height = rect.height > 0 ? rect.height : runtime.canvas.height;
      session.exports.__fui_on_viewport_changed(width, height);
    }

    function notifySystemTheme(session: HarnessAppSession | null = currentSession, isDark = darkModeQuery.matches): void {
      if (session === null) {
        return;
      }
      session.exports.__fui_on_system_dark_mode_changed(isDark);
      notifySystemAccentColor(session, true);
    }

    function notifySystemAccentColor(session: HarnessAppSession | null = currentSession, force = false): void {
      if (session === null) {
        return;
      }
      const accentColor = harnessUiChrome.readHostAccentColor() >>> 0;
      if (!force && accentColor === lastSystemAccentColor) {
        return;
      }
      lastSystemAccentColor = accentColor;
      const callback = session.exports.__fui_on_system_accent_color_changed;
      if (typeof callback === 'function') {
        callback(accentColor);
      }
    }

    function encodeHostEventCallArgs(
      session: HarnessAppSession,
      method: NormalizedHostEventMethod,
      args: readonly unknown[],
    ): unknown[] {
      function alignOffset(value: number, alignment: number): number {
        return alignment <= 1 ? value : (value + alignment - 1) & ~(alignment - 1);
      }

      function encodeTypedArrayArg(
        type: "bytes" | "i32_array" | "u32_array" | "i64_array" | "u64_array" | "f64_array",
        arg: unknown,
        context: string,
      ): { bytes: Uint8Array; elementCount: number; alignment: number } {
        if (type === "bytes") {
          if (!(arg instanceof Uint8Array)) {
            throw new Error(`${context} must be a Uint8Array.`);
          }
          return { bytes: arg, elementCount: arg.length, alignment: 1 };
        }
        if (type === "i32_array") {
          if (!(arg instanceof Int32Array)) {
            throw new Error(`${context} must be an Int32Array.`);
          }
          return {
            bytes: new Uint8Array(arg.buffer, arg.byteOffset, arg.byteLength),
            elementCount: arg.length,
            alignment: 4,
          };
        }
        if (type === "u32_array") {
          if (!(arg instanceof Uint32Array)) {
            throw new Error(`${context} must be a Uint32Array.`);
          }
          return {
            bytes: new Uint8Array(arg.buffer, arg.byteOffset, arg.byteLength),
            elementCount: arg.length,
            alignment: 4,
          };
        }
        if (type === "i64_array") {
          if (!(arg instanceof BigInt64Array)) {
            throw new Error(`${context} must be a BigInt64Array.`);
          }
          return {
            bytes: new Uint8Array(arg.buffer, arg.byteOffset, arg.byteLength),
            elementCount: arg.length,
            alignment: 8,
          };
        }
        if (type === "u64_array") {
          if (!(arg instanceof BigUint64Array)) {
            throw new Error(`${context} must be a BigUint64Array.`);
          }
          return {
            bytes: new Uint8Array(arg.buffer, arg.byteOffset, arg.byteLength),
            elementCount: arg.length,
            alignment: 8,
          };
        }
        if (!(arg instanceof Float64Array)) {
          throw new Error(`${context} must be a Float64Array.`);
        }
        return {
          bytes: new Uint8Array(arg.buffer, arg.byteOffset, arg.byteLength),
          elementCount: arg.length,
          alignment: 8,
        };
      }

      if (args.length != method.args.length) {
        throw new Error(`Host event ${method.serviceName}.${method.methodName} expected ${String(method.args.length)} args but received ${String(args.length)}.`);
      }
      const callArgs: unknown[] = [];
      let byteOffset = 0;
      for (let index = 0; index < method.args.length; index += 1) {
        const type = method.args[index];
        const arg = args[index];
        const context = `Host event ${method.serviceName}.${method.methodName} arg ${String(index)}`;
        if (type === 'string') {
          if (typeof arg !== 'string') {
            throw new Error(`${context} must be a string.`);
          }
          const encoded = encoder.encode(arg);
          if (encoded.length > 0) {
            if (session.textBufferPtr === 0 || byteOffset + encoded.length > session.textBufferSize) {
              throw new Error(`${context} exceeds the shared AssemblyScript text buffer.`);
            }
            const memory = new Uint8Array(session.memory.buffer, session.textBufferPtr + byteOffset, encoded.length);
            memory.set(encoded);
            callArgs.push(session.textBufferPtr + byteOffset, encoded.length);
            byteOffset += encoded.length;
          } else {
            callArgs.push(0, 0);
          }
          continue;
        }
        if (type === "bytes" || type === "i32_array" || type === "u32_array" || type === "i64_array" || type === "u64_array" || type === "f64_array") {
          const payload = encodeTypedArrayArg(type, arg, context);
          if (payload.bytes.length > 0) {
            const alignedOffset = alignOffset(byteOffset, payload.alignment);
            if (session.textBufferPtr === 0 || alignedOffset + payload.bytes.length > session.textBufferSize) {
              throw new Error(`${context} exceeds the shared AssemblyScript text buffer.`);
            }
            const memory = new Uint8Array(session.memory.buffer, session.textBufferPtr + alignedOffset, payload.bytes.length);
            memory.set(payload.bytes);
            callArgs.push(session.textBufferPtr + alignedOffset, payload.elementCount);
            byteOffset = alignedOffset + payload.bytes.length;
          } else {
            callArgs.push(0, 0);
          }
          continue;
        }
        if (type === 'bool') {
          if (typeof arg !== 'boolean') {
            throw new Error(`${context} must be a boolean.`);
          }
          callArgs.push(arg ? 1 : 0);
          continue;
        }
        if (type === "i64" || type === "u64") {
          if (typeof arg !== "bigint") {
            throw new Error(`${context} must be a bigint.`);
          }
          if (type === "i64" && (arg < -9223372036854775808n || arg > 9223372036854775807n)) {
            throw new Error(`${context} must be a signed 64-bit integer.`);
          }
          if (type === "u64" && (arg < 0n || arg > 18446744073709551615n)) {
            throw new Error(`${context} must be an unsigned 64-bit integer.`);
          }
          callArgs.push(arg);
          continue;
        }
        if (typeof arg !== 'number' || Number.isNaN(arg)) {
          throw new Error(`${context} must be a number.`);
        }
        if (type === 'i32') {
          if (!Number.isInteger(arg) || arg < -2147483648 || arg > 2147483647) {
            throw new Error(`${context} must be a signed 32-bit integer.`);
          }
        } else if (type === "u32") {
          if (!Number.isInteger(arg) || arg < 0 || arg > 4294967295) {
            throw new Error(`${context} must be an unsigned 32-bit integer.`);
          }
        }
        callArgs.push(arg);
      }
      return callArgs;
    }

    function disposeHostEventDisposers(session: HarnessAppSession): void {
      const disposers = session.hostEventDisposers;
      while (disposers.length > 0) {
        const dispose = disposers.pop();
        dispose?.();
      }
    }

    function connectHostEvents(
      session: HarnessAppSession,
      exports: HarnessExports,
      hostEvents: HostEventsDefinition | undefined,
    ): void {
      const exportRecord = exports as unknown as Record<string, unknown>;
      for (const method of listHostEventMethods(hostEvents)) {
        const exportedHandler = exportRecord[method.exportName];
        if (typeof exportedHandler !== 'function') {
          console.error(
            `[fui_host_event] Missing wasm export "${method.exportName}" for ${method.serviceName}.${method.methodName}.`,
          );
          continue;
        }
        const dispose = method.subscribe((...args: readonly unknown[]) => {
          if (currentSession !== session) {
            return;
          }
          const activeHandler = exportRecord[method.exportName];
          if (typeof activeHandler !== 'function') {
            console.error(
              `[fui_host_event] Lost wasm export "${method.exportName}" while dispatching ${method.serviceName}.${method.methodName}.`,
            );
            return;
          }
          try {
            const callArgs = encodeHostEventCallArgs(session, method, args);
            (activeHandler as (...rawArgs: unknown[]) => void)(...callArgs);
          } catch (error: unknown) {
            const message = error instanceof Error ? error.stack ?? error.message : String(error);
            console.error(
              `[fui_host_event] Dispatch failed for ${method.serviceName}.${method.methodName}: ${message}`,
            );
            throw error;
          }
        });
        if (typeof dispose === 'function') {
          session.hostEventDisposers.push(dispose);
        }
      }
    }

    function notifySvgLoaded(session: HarnessAppSession | null, svgId: number, width: number, height: number): void {
      if (session === null) {
        return;
      }
      session.exports.__fui_on_svg_loaded(svgId, width, height);
    }

    function notifySvgFailed(session: HarnessAppSession | null, svgId: number, error: string): void {
      if (
        session === null ||
        session.textBufferPtr === 0 ||
        session.textBufferSize === 0
      ) {
        return;
      }
      const length = writeTextToSessionBuffer(session, error);
      session.exports.__fui_on_svg_failed(svgId, session.textBufferPtr, length);
    }

    function notifyTextureLoaded(session: HarnessAppSession | null, textureId: number, width: number, height: number): void {
      if (session === null) {
        return;
      }
      session.exports.__fui_on_texture_loaded(textureId, width, height);
    }

    function notifyTextureFailed(session: HarnessAppSession | null, textureId: number, error: string): void {
      if (
        session === null ||
        session.textBufferPtr === 0 ||
        session.textBufferSize === 0
      ) {
        return;
      }
      const length = writeTextToSessionBuffer(session, error);
      session.exports.__fui_on_texture_failed(textureId, session.textBufferPtr, length);
    }

    async function handleSameOriginNavigation(target: URL, mode: HarnessNavigationMode): Promise<void> {
      const previousUrlHref = lastHandledUrlHref;
      if (mode !== 'pop') {
        await queuePersistedUiStateWork(() => saveCurrentHistoryEntrySnapshot(`navigating ${mode} to ${target.href}`));
      } else if (previousUrlHref !== target.href) {
        await queuePersistedUiStateWork(() => saveRouteHeadSnapshotForHref(
          previousUrlHref,
          `leaving ${previousUrlHref} via ${mode} to ${target.href}`,
        ));
      }
      if (navigationHandler !== null) {
        await navigationHandler(target, mode);
        lastHandledUrlHref = target.href;
        return;
      }
      if (mode === 'push') {
        pushManagedHistoryEntry(target);
      } else if (mode === 'replace') {
        replaceManagedHistoryEntry(target);
      }
      const targetSnapshot = mode === 'pop'
        ? await queuePersistedUiStateWork(() => loadPopPersistedSnapshot(`navigating ${mode} to ${target.href}`, target.href))
        : null;
      hydrateCurrentPersistedEntries(targetSnapshot);
      notifyRouteChanged(currentSession, toAppRoute(target));
      if (targetSnapshot !== null) {
        restoreCurrentPersistedUiState(`navigating ${mode} to ${target.href}`);
        await settleCurrentSessionAfterRestore(`navigating ${mode} to ${target.href}`);
      }
      await queuePersistedUiStateWork(() => ensureCurrentHistoryEntrySnapshot(`navigating ${mode} to ${target.href}`));
      lastHandledUrlHref = target.href;
    }

    function handleSameOriginNavigationFailure(target: URL, mode: HarnessNavigationMode, error: unknown): void {
      const route = toAppRoute(target);
      const detail = `Failed to load ${mode === 'pop' ? 'history route' : 'route'} ${route}: ${error instanceof Error ? error.message : String(error)}`;
      console.error(error instanceof Error ? error.stack ?? error : error);
      failLoadingOverlay(detail);
      const windowWithHarnessError = window as Window & { __fuiAsError?: string; __fuiAsReady?: boolean };
      windowWithHarnessError.__fuiAsReady = false;
      windowWithHarnessError.__fuiAsError = detail;
      options.onError?.(error);
    }

    function navigateWithinDocument(rawTarget: string, openInNewTab: boolean): void {
      const target = tryResolveNavigationTarget(rawTarget);
      if (target === null) {
        throw new Error(`Invalid navigation target: ${rawTarget}`);
      }
      if (openInNewTab) {
        const anchor = document.createElement('a');
        anchor.href = target.href;
        anchor.target = '_blank';
        anchor.rel = 'noopener';
        anchor.hidden = true;
        document.body.appendChild(anchor);
        anchor.click();
        anchor.remove();
        return;
      }
      const isWebUrl = target.protocol === 'http:' || target.protocol === 'https:';
      if (isWebUrl && target.origin === window.location.origin) {
        void handleSameOriginNavigation(target, 'push').catch((error: unknown) => {
          handleSameOriginNavigationFailure(target, 'push', error);
        });
        return;
      }
      window.location.assign(target.href);
    }

    async function flushDebugInteraction(session: HarnessAppSession): Promise<void> {
      session.exports.__flushRenders();
      while (appFlushRequested) {
        appFlushRequested = false;
        session.exports.__flushRenders();
      }
      const words = runtime.flushPendingCommit();
      latestCommandWords = words === null ? [] : Array.from(words);
      updateState();
      await waitForFrame();
      updateState();
    }

    function syncUiHostCapabilities(): void {
      runtime.ui._ui_set_coarse_pointer_mode(harnessUiChrome.detectCoarsePointer() ? 1 : 0);
      runtime.ui._ui_set_platform_family(harnessUiChrome.detectPlatformFamily());
    }

    function createAppImports(hostServices: HostServicesDefinition | undefined) {
      const hostServiceImports = createHostServiceImportModule(hostServices, {
        readString: readAppUtf8,
        writeString: writeAppUtf8,
        readBytes: readAppBytes,
        writeBytes: writeAppBytes,
      });
      return {
        effindom_v2_ui: createUiImportModule({
          getRuntime: () => runtime,
          readAppUtf8,
          readAppFloats,
          readAppBytes,
          withUiUtf8,
          withUiGridData,
          withUiGradientData,
          zeroPointer: () => zeroPointer(runtime),
          normalizePointer: (ptr) => normalizePointer(runtime, ptr),
          getCurrentMemory,
          setLatestRootHandle(rootHandle: string | null): void {
            latestRootHandle = rootHandle;
          },
          updateState,
          queueHarnessFrame,
          syncUiHostCapabilities,
          resetUiState,
          recordTextChangedFromAppSet(handle: AppHandleLike, text: string): void {
            textBridge.recordTextChanged(handle, text);
            recordRuntimeTextChangedFromAppSet?.(handle, text);
          },
        }),
        fui_host: {
          ...createHostImportModule({
            getRuntime: () => runtime,
            getCurrentSession,
            getCurrentSessionOrNull: () => currentSession,
            setAppFlushRequested(value: boolean): void {
              appFlushRequested = value;
            },
            queueHarnessFrame,
            uiChrome: harnessUiChrome,
            readAppUtf8,
            writeAppFloat32,
            writeAppUint32,
            writeAppUtf8,
            textBridge,
            persistedUiStateController,
            navigateWithinDocument,
            canBrowserNavigateBack,
            canBrowserNavigateForward,
            navigateBrowserBack,
            navigateBrowserForward,
            cancelHostTimer,
            getHostTimer(timerId: number): number | undefined {
              return hostTimers.get(timerId);
            },
            setHostTimer(timerId: number, timeoutId: number): void {
              hostTimers.set(timerId, timeoutId);
            },
            deleteHostTimer(timerId: number): void {
              hostTimers.delete(timerId);
            },
            workerManager,
            debugLogsEnabled,
            notifySvgLoaded,
            notifySvgFailed,
            notifyTextureLoaded,
            notifyTextureFailed,
          }),
          ...bitmapHost.imports,
          ...canvasHost.imports,
          ...fileHost.imports,
        },
        fui_fetch_host: fetchHost.imports,
        fui_host_service: hostServiceImports,
      };
    }

    const callbacks: EffinDomCallbacks = window.__effindomCallbacks ?? {};
    const previousPointerCallback = callbacks.onPointerEventWithCoords;
    callbacks.onPointerEventWithCoords = (type, handle, x, y, modifiers) => {
      previousPointerCallback?.(type, handle, x, y, modifiers);
      const session = currentSession;
      if (session !== null) {
        return isHandledResult(session.exports.__fui_on_pointer_event_with_metadata(
          type,
          toBigIntHandle(handle),
          x,
          y,
          modifiers ?? 0,
          -1,
          1,
          0,
          0,
          0,
          0,
          0,
          0,
        ));
      }
      return false;
    };
    const previousPointerMetadataCallback = callbacks.onPointerEventWithMetadata;
    callbacks.onPointerEventWithMetadata = (
      type,
      handle,
      x,
      y,
      modifiers,
      pointerId,
      pointerType,
      button,
      buttons,
      pressure,
      width,
      height,
      clickCount,
    ) => {
      previousPointerMetadataCallback?.(
        type,
        handle,
        x,
        y,
        modifiers,
        pointerId,
        pointerType,
        button,
        buttons,
        pressure,
        width,
        height,
        clickCount,
      );
      previousPointerCallback?.(type, handle, x, y, modifiers);
      const session = currentSession;
      if (session === null) {
        return false;
      }
      return isHandledResult(session.exports.__fui_on_pointer_event_with_metadata(
        type,
        toBigIntHandle(handle),
        x,
        y,
        modifiers,
        pointerId,
        pointerType,
        button,
        buttons,
        pressure,
        width,
        height,
        clickCount,
      ));
    };
    const previousWheelCallback = callbacks.onWheelEventWithCoords;
    callbacks.onWheelEventWithCoords = (handle, x, y, deltaX, deltaY, deltaMode, modifiers) => {
      const previousHandled = previousWheelCallback?.(handle, x, y, deltaX, deltaY, deltaMode, modifiers) === true;
      const session = currentSession;
      if (session === null) {
        return previousHandled;
      }
      const onWheelEvent = session.exports.__fui_on_wheel_event;
      if (typeof onWheelEvent !== 'function') {
        if (!missingWheelExportLogged) {
          missingWheelExportLogged = true;
          console.error(
            '[fui_host] AssemblyScript app does not export __fui_on_wheel_event; rebuild the app with the current @effindomv2/fui-as exports.',
          );
        }
        return previousHandled;
      }
      return previousHandled || onWheelEvent(
        toBigIntHandle(handle),
        x,
        y,
        deltaX,
        deltaY,
        deltaMode,
        modifiers,
      ) !== 0;
    };
    callbacks.resolveGestureOwner = (handle) => {
      const session = currentSession;
      const resolveGestureOwner = session?.exports.__fui_resolve_gesture_owner;
      if (session === null || typeof resolveGestureOwner !== 'function') {
        return null;
      }
      const owner = resolveGestureOwner(toBigIntHandle(handle));
      return owner === 0n ? null : owner;
    };
    callbacks.getGestureIntent = (handle) => {
      const session = currentSession;
      const getGestureIntent = session?.exports.__fui_get_gesture_intent;
      if (session === null || typeof getGestureIntent !== 'function') {
        return 0;
      }
      return getGestureIntent(toBigIntHandle(handle));
    };
    callbacks.onGestureEventWithCoords = (handle, phase, kind, x, y, deltaX, deltaY, scale, pointerCount) => {
      const session = currentSession;
      const onGestureEvent = session?.exports.__fui_on_gesture_event;
      if (session === null || typeof onGestureEvent !== 'function') {
        return false;
      }
      return isHandledResult(onGestureEvent(
        toBigIntHandle(handle),
        phase,
        kind,
        x,
        y,
        deltaX,
        deltaY,
        scale,
        pointerCount,
      ));
    };
    callbacks.resolveLongPressOwner = (handle) => {
      const session = currentSession;
      const resolveLongPressOwner = session?.exports.__fui_resolve_long_press_owner;
      if (session === null || typeof resolveLongPressOwner !== 'function') {
        return null;
      }
      const owner = resolveLongPressOwner(toBigIntHandle(handle));
      return owner === 0n ? null : owner;
    };
    callbacks.getLongPressMinimumDurationMs = (handle) => {
      const session = currentSession;
      const getLongPressMinimumDurationMs = session?.exports.__fui_get_long_press_minimum_duration_ms;
      if (session === null || typeof getLongPressMinimumDurationMs !== 'function') {
        return 500;
      }
      return getLongPressMinimumDurationMs(toBigIntHandle(handle));
    };
    callbacks.getLongPressMovementTolerance = (handle) => {
      const session = currentSession;
      const getLongPressMovementTolerance = session?.exports.__fui_get_long_press_movement_tolerance;
      if (session === null || typeof getLongPressMovementTolerance !== 'function') {
        return 10;
      }
      return getLongPressMovementTolerance(toBigIntHandle(handle));
    };
    callbacks.onLongPressEventWithCoords = (handle, x, y, pointerId, pointerType, modifiers, durationMs) => {
      const session = currentSession;
      const onLongPressEvent = session?.exports.__fui_on_long_press_event;
      if (session === null || typeof onLongPressEvent !== 'function') {
        return false;
      }
      return isHandledResult(onLongPressEvent(
        toBigIntHandle(handle),
        x,
        y,
        pointerId,
        pointerType,
        modifiers,
        durationMs,
      ));
    };
    const previousBeforeContextMenuHitTest = callbacks.onBeforeContextMenuHitTest;
    callbacks.onBeforeContextMenuHitTest = () => {
      previousBeforeContextMenuHitTest?.();
      const session = currentSession;
      if (session !== null) {
        session.exports.__fui_hide_active_context_menu();
      }
      runtime.commitFrame();
      runtime.flushPendingCommit();
      queueHarnessFrame();
    };
    const previousContextMenu = callbacks.onContextMenu;
    callbacks.onContextMenu = (handle, x, y) => {
      previousContextMenu?.(handle, x, y);
      const session = currentSession;
      if (session !== null) {
        session.exports.__fui_on_context_menu(toBigIntHandle(handle), x, y);
      }
    };
    callbacks.canShowContextMenu = (handle) => {
      const session = currentSession;
      if (session === null) {
        return true;
      }
      const canShowContextMenu = session.exports.__fui_can_show_context_menu;
      return typeof canShowContextMenu === 'function'
        ? canShowContextMenu(toBigIntHandle(handle))
        : true;
    };
    const previousFocusChanged = callbacks.onFocusChanged;
    callbacks.onFocusChanged = (handle, isFocused) => {
      previousFocusChanged?.(handle, isFocused);
      const session = currentSession;
      if (session !== null) {
        session.exports.__fui_on_focus_changed(toBigIntHandle(handle), isFocused);
        session.exports.__flushRenders();
      }
    };
    const previousFontLoaded = callbacks.onFontLoaded;
    callbacks.onFontLoaded = (fontId) => {
      previousFontLoaded?.(fontId);
      const session = currentSession;
      if (session !== null) {
        session.exports.__fui_on_font_loaded(fontId);
        session.exports.__flushRenders();
        runtime.flushPendingCommit();
      }
    };
    const previousMissingFontCoverage = callbacks.onMissingFontCoverage;
    callbacks.onMissingFontCoverage = (fontId, coverageKind, sampleText) => {
      if (previousMissingFontCoverage !== undefined) {
        previousMissingFontCoverage(fontId, coverageKind, sampleText);
        return;
      }
      runtime.logs.missingFontCoverageRequests.push({
        fontId,
        coverageKind,
        sampleText,
      });
      runtime.handleMissingFontCoverage(fontId, coverageKind, sampleText);
    };
    const previousTextChanged = callbacks.onTextChanged;
    recordRuntimeTextChangedFromAppSet = previousTextChanged === undefined
      ? null
      : (handle: AppHandleLike, text: string): void => {
        previousTextChanged(toBigIntHandle(handle), text);
      };
    callbacks.onTextChanged = (handle, text) => {
      previousTextChanged?.(handle, text);
      textBridge.recordTextChanged(toBigIntHandle(handle), text);
      const session = currentSession;
      if (
        session === null ||
        session.textBufferPtr === 0 ||
        session.textBufferSize === 0
      ) {
        return;
      }
      const length = writeTextCallbackPayload(session, text, 'Text changed payload');
      session.exports.__fui_on_text_changed(
        toBigIntHandle(handle),
        length > 0 ? session.textBufferPtr : 0,
        length,
      );
      session.exports.__flushRenders();
      runtime.flushPendingCommit();
    };
    const previousTextReplaced = callbacks.onTextReplaced;
    callbacks.onTextReplaced = (handle, start, end, text) => {
      previousTextReplaced?.(handle, start, end, text);
      textBridge.recordTextReplaced(toBigIntHandle(handle), start, end, text);
      const session = currentSession;
      if (
        session === null ||
        session.textBufferPtr === 0 ||
        session.textBufferSize === 0
      ) {
        return;
      }
      const length = writeTextCallbackPayload(session, text, 'Text replacement payload');
      session.exports.__fui_on_text_replaced(
        toBigIntHandle(handle),
        start,
        end,
        length > 0 ? session.textBufferPtr : 0,
        length,
      );
      session.exports.__flushRenders();
      runtime.flushPendingCommit();
    };
    const previousSelectionChanged = callbacks.onSelectionChanged;
    callbacks.onSelectionChanged = (handle, start, end) => {
      previousSelectionChanged?.(handle, start, end);
      textBridge.recordSelectionChanged(toBigIntHandle(handle), start, end);
      const session = currentSession;
      if (session !== null) {
        session.exports.__fui_on_selection_changed(toBigIntHandle(handle), start, end);
        session.exports.__flushRenders();
        runtime.flushPendingCommit();
      }
    };
    const previousCrossSelectionChanged = callbacks.onCrossSelectionChanged;
    callbacks.onCrossSelectionChanged = (handle, text) => {
      previousCrossSelectionChanged?.(handle, text);
      const session = currentSession;
      if (
        session === null ||
        session.textBufferPtr === 0 ||
        session.textBufferSize === 0
      ) {
        return;
      }
      const length = writeTextCallbackPayload(session, text, 'Cross-selection payload');
      session.exports.__fui_on_cross_selection_changed(toBigIntHandle(handle), session.textBufferPtr, length);
    };
    const previousKeyEvent = callbacks.onKeyEventWithKey;
    callbacks.onKeyEventWithKey = (type, key, modifiers) => {
      const previousHandled = previousKeyEvent?.(type, key, modifiers) === true;
      const session = currentSession;
      if (session === null || session.keyBufferPtr === 0) {
        return previousHandled;
      }
      const encoded = encoder.encode(key);
      if (encoded.length > 256) {
        return previousHandled;
      }
      const memory = new Uint8Array(session.memory.buffer, session.keyBufferPtr, encoded.length);
      memory.set(encoded);
      const handled =
        previousHandled ||
        session.exports.__fui_on_key_event(type, session.keyBufferPtr, encoded.length, modifiers) !== 0;
      session.exports.__flushRenders();
      return handled;
    };
    const previousScroll = callbacks.onScroll;
    callbacks.onScroll = (handle, offsetX, offsetY, contentWidth, contentHeight, viewportWidth, viewportHeight) => {
      previousScroll?.(handle, offsetX, offsetY, contentWidth, contentHeight, viewportWidth, viewportHeight);
      const session = currentSession;
      if (session !== null) {
        session.exports.__fui_on_scroll(
          toBigIntHandle(handle),
          offsetX,
          offsetY,
          contentWidth,
          contentHeight,
          viewportWidth,
          viewportHeight,
        );
      }
    };
    window.__effindomCallbacks = callbacks;

    const handleViewportChange = () => {
      notifyViewport();
    };
    window.addEventListener('resize', handleViewportChange);

    const handleDarkModeChange = (event: MediaQueryListEvent) => {
      notifySystemTheme(currentSession, event.matches);
    };
    darkModeQuery.addEventListener('change', handleDarkModeChange);

    const handleWindowFocus = () => {
      notifySystemAccentColor();
    };
    window.addEventListener('focus', handleWindowFocus);

    const handlePopState = () => {
      const target = new URL(window.location.href);
      syncManagedHistoryPop(target);
      void handleSameOriginNavigation(target, 'pop').catch((error: unknown) => {
        handleSameOriginNavigationFailure(target, 'pop', error);
      });
    };
    window.addEventListener('popstate', handlePopState);

    const dismissTransientUi = () => {
      const session = currentSession;
      if (session !== null) {
        session.exports.__fui_hide_active_context_menu();
      }
      runtime.clearPointerHover();
      harnessUiChrome.setUrlPreviewText('');
    };
    const handleWindowBlur = () => {
      dismissTransientUi();
    };
    const handleCanvasBlur = () => {
      dismissTransientUi();
    };
    const handleCanvasDragEnter = (event: DragEvent) => {
      const effect = fileHost.dispatchExternalDragEvent(EXTERNAL_DRAG_EVENT_ENTER, event, { reuseActiveItems: false });
      if (effect === 0) {
        return;
      }
      event.preventDefault();
      if (event.dataTransfer !== null) {
        event.dataTransfer.dropEffect = fileHost.mapExternalDropEffect(effect);
      }
    };
    const handleCanvasDragOver = (event: DragEvent) => {
      const effect = fileHost.dispatchExternalDragEvent(EXTERNAL_DRAG_EVENT_OVER, event);
      if (effect === 0) {
        return;
      }
      event.preventDefault();
      if (event.dataTransfer !== null) {
        event.dataTransfer.dropEffect = fileHost.mapExternalDropEffect(effect);
      }
    };
    const handleCanvasDragLeave = (event: DragEvent) => {
      const effect = fileHost.dispatchExternalDragEvent(EXTERNAL_DRAG_EVENT_LEAVE, event, { handle: 0n });
      if (effect !== 0) {
        event.preventDefault();
      }
    };
    const handleCanvasDrop = (event: DragEvent) => {
      const effect = fileHost.dispatchExternalDragEvent(EXTERNAL_DRAG_EVENT_DROP, event);
      if (effect === 0) {
        return;
      }
      event.preventDefault();
      if (event.dataTransfer !== null) {
        event.dataTransfer.dropEffect = fileHost.mapExternalDropEffect(effect);
      }
    };
    const handleVisibilityChange = () => {
      if (document.visibilityState === 'visible') {
        notifySystemAccentColor();
        return;
      }
      dismissTransientUi();
      void queuePersistedUiStateWork(() => saveCurrentHistoryEntrySnapshot('visibility change'));
    };
    const handlePageHide = () => {
      void queuePersistedUiStateWork(() => saveCurrentHistoryEntrySnapshot('page hide'));
    };
    const canvasDragEnterListener: EventListener = (event) => {
      handleCanvasDragEnter(event as DragEvent);
    };
    const canvasDragOverListener: EventListener = (event) => {
      handleCanvasDragOver(event as DragEvent);
    };
    const canvasDragLeaveListener: EventListener = (event) => {
      handleCanvasDragLeave(event as DragEvent);
    };
    const canvasDropListener: EventListener = (event) => {
      handleCanvasDrop(event as DragEvent);
    };
    const externalDragTargets: (HTMLElement | HTMLCanvasElement)[] = [];
    const registerExternalDragTarget = (target: HTMLElement | HTMLCanvasElement | null) => {
      if (target === null) {
        return;
      }
      for (const externalDragTarget of externalDragTargets) {
        if (externalDragTarget === target) {
          return;
        }
      }
      externalDragTargets.push(target);
    };
    registerExternalDragTarget(runtime.canvas);
    registerExternalDragTarget(runtime.canvas.parentElement);
    registerExternalDragTarget(harnessUiChrome.getCanvasSizeSource(runtime.canvas));
    window.addEventListener('blur', handleWindowBlur);
    runtime.canvas.addEventListener('blur', handleCanvasBlur);
    for (const target of externalDragTargets) {
      target.addEventListener('dragenter', canvasDragEnterListener);
      target.addEventListener('dragover', canvasDragOverListener);
      target.addEventListener('dragleave', canvasDragLeaveListener);
      target.addEventListener('drop', canvasDropListener);
    }
    document.addEventListener('visibilitychange', handleVisibilityChange);
    window.addEventListener('pagehide', handlePageHide);

    cleanup = () => {
      workerManager.terminateAll();
      harnessUiChrome.setUrlPreviewText('');
      window.removeEventListener('resize', handleViewportChange);
      darkModeQuery.removeEventListener('change', handleDarkModeChange);
      window.removeEventListener('popstate', handlePopState);
      window.removeEventListener('focus', handleWindowFocus);
      window.removeEventListener('blur', handleWindowBlur);
      runtime.canvas.removeEventListener('blur', handleCanvasBlur);
      for (const target of externalDragTargets) {
        target.removeEventListener('dragenter', canvasDragEnterListener);
        target.removeEventListener('dragover', canvasDragOverListener);
        target.removeEventListener('dragleave', canvasDragLeaveListener);
        target.removeEventListener('drop', canvasDropListener);
      }
      document.removeEventListener('visibilitychange', handleVisibilityChange);
      window.removeEventListener('pagehide', handlePageHide);
      delete window.__fui_debug;
    };

    const debugApi: HarnessDebugApi = {
      async flush(): Promise<void> {
        await flushDebugInteraction(getCurrentSession());
      },
      getDebugTree() {
        return Promise.resolve(runtime.getDebugTree());
      },
      async externalDragEvent(type, handle, x, y, files) {
        const session = getCurrentSession();
        const dataTransfer = new DataTransfer();
        for (const file of files) {
          dataTransfer.items.add(new File([file.text], file.name, {
            type: file.type ?? 'application/octet-stream',
          }));
        }
        const eventName =
          type === EXTERNAL_DRAG_EVENT_ENTER ? 'dragenter'
            : type === EXTERNAL_DRAG_EVENT_OVER ? 'dragover'
              : type === EXTERNAL_DRAG_EVENT_LEAVE ? 'dragleave'
                : 'drop';
        const event = new DragEvent(eventName, {
          bubbles: true,
          cancelable: true,
          clientX: x,
          clientY: y,
          dataTransfer,
        });
        const effect = fileHost.dispatchExternalDragEvent(type, event, {
          handle: toBigIntHandle(handle),
          reuseActiveItems: type !== EXTERNAL_DRAG_EVENT_ENTER,
        });
        await flushDebugInteraction(session);
        return effect;
      },
      async pointerEvent(type: number, handle: WasmHandleLike, x: number, y: number, modifiers = 0): Promise<void> {
        const session = getCurrentSession();
        const debugPointerEvent = session.exports.__fui_debug_pointer_event;
        if (debugPointerEvent === undefined) {
          throw new Error('Debug pointer events are not available for this app.');
        }
        debugPointerEvent(type, toBigIntHandle(handle), x, y, modifiers);
        await flushDebugInteraction(session);
      },
      async focusChanged(handle: WasmHandleLike, focused: boolean): Promise<void> {
        const session = getCurrentSession();
        const debugFocusChanged = session.exports.__fui_debug_focus_changed;
        if (debugFocusChanged === undefined) {
          throw new Error('Debug focus changes are not available for this app.');
        }
        debugFocusChanged(toBigIntHandle(handle), focused);
        await flushDebugInteraction(session);
      },
      async keyEvent(type: number, key: string, modifiers = 0): Promise<void> {
        const session = getCurrentSession();
        const debugKeyEvent = session.exports.__fui_debug_key_event;
        if (session.keyBufferPtr === 0 || debugKeyEvent === undefined) {
          throw new Error('Debug key events are not available for this app.');
        }
        const encoded = encoder.encode(key);
        if (encoded.length > 256) {
          throw new Error('Debug key event exceeds the shared AssemblyScript key buffer.');
        }
        const memory = new Uint8Array(session.memory.buffer, session.keyBufferPtr, encoded.length);
        memory.set(encoded);
        debugKeyEvent(type, session.keyBufferPtr, encoded.length, modifiers);
        await flushDebugInteraction(session);
      },
      navigateTo(target: string): Promise<void> {
        navigateWithinDocument(target, false);
        return Promise.resolve();
      },
      async scroll(
        handle: WasmHandleLike,
        offsetX: number,
        offsetY: number,
        contentWidth: number,
        contentHeight: number,
        viewportWidth: number,
        viewportHeight: number,
      ): Promise<void> {
        const session = getCurrentSession();
        const debugScroll = session.exports.__fui_debug_scroll;
        if (debugScroll === undefined) {
          throw new Error('Debug scroll events are not available for this app.');
        }
        debugScroll(
          toBigIntHandle(handle),
          offsetX,
          offsetY,
          contentWidth,
          contentHeight,
          viewportWidth,
          viewportHeight,
        );
        await flushDebugInteraction(session);
      },
    };
    window.__fui_debug = debugApi;

    async function fetchWasmBytes(wasmPath: string): Promise<ArrayBuffer> {
      const cached = wasmByteCache.get(wasmPath);
      if (cached !== undefined) {
        return cached;
      }
      const fetchPromise = fetch(wasmPath, { cache: 'no-store' }).then(async (response) => {
        if (!response.ok) {
          throw new Error(`Failed to load wasm app: ${wasmPath}`);
        }
        return response.arrayBuffer();
      });
      wasmByteCache.set(wasmPath, fetchPromise);
      return fetchPromise;
    }

    async function loadWasmModule(wasmPath: string): Promise<WebAssembly.Module> {
      const cached = wasmModuleCache.get(wasmPath);
      if (cached !== undefined) {
        return cached;
      }
      const compilePromise = fetchWasmBytes(wasmPath).then((bytes) => WebAssembly.compile(bytes));
      wasmModuleCache.set(wasmPath, compilePromise);
      return compilePromise;
    }

    function validateAppImports(wasmModule: WebAssembly.Module, hostServices: HostServicesDefinition | undefined): void {
      const allowedHostServiceImports = getHostServiceImportNames(hostServices);
      for (const imported of WebAssembly.Module.imports(wasmModule)) {
        if (imported.kind !== 'function') {
          throw new Error(`App import ${imported.module}.${imported.name} is not allowed.`);
        }
        if (
          imported.module === 'effindom_v2_ui' ||
          imported.module === 'fui_host' ||
          imported.module === 'fui_fetch_host' ||
          imported.module === 'env'
        ) {
          continue;
        }
        if (imported.module === 'fui_host_service' && allowedHostServiceImports.has(imported.name)) {
          continue;
        }
        throw new Error(`App import ${imported.module}.${imported.name} is not allowed.`);
      }
    }

    async function unloadApp(): Promise<void> {
      const session = currentSession;
      if (session === null) {
        return;
      }
      disposeHostEventDisposers(session);
      session.onDispose?.(session.exports);
      fetchHost.cancelAllForSession(session);
      fileHost.cancelAllForSession(session);
      currentSession = null;
      workerManager.terminateAll();
      appFlushRequested = false;
      cancelAllHostTimers();
      bitmapHost.clearTextures(runtime);
      runtime.setAppFrameHandler(null);
      runtime.setCapturedPointerHandle(null);
      runtime.clearPointerHover();
      runtime.canvas.style.cursor = 'default';
      harnessUiChrome.setUrlPreviewText('');
      runtime.core._ed_clear_focus_state?.();
      runtime.core._ed_clear_text_input_state?.();
      runtime.core._ed_reset_scene();
      runtime.ui._ui_reset();
      syncUiHostCapabilities();
      resetUiState();
      runtime.resetLogs();
      runtime.commitFrame();
      queueHarnessFrame();
      runtime.flushPendingCommit();
      await waitForFrame();
    }

    async function recreateRuntime(): Promise<BridgeRuntime> {
      const session = currentSession;
      if (session !== null) {
        disposeHostEventDisposers(session);
        session.onDispose?.(session.exports);
        fetchHost.cancelAllForSession(session);
        fileHost.cancelAllForSession(session);
        currentSession = null;
      }
      workerManager.terminateAll();
      appFlushRequested = false;
      cancelAllHostTimers();
      bitmapHost.clearTextures(runtime);
      runtime.setAppFrameHandler(null);
      runtime.setCapturedPointerHandle(null);
      runtime.clearPointerHover();
      harnessUiChrome.setUrlPreviewText('');
      latestCommandWords = [];
      latestRootHandle = null;
      updateState();
      runtime = await bridgeState.recreateRuntime();
      assertCompatibleAbi(runtime);
      bitmapHost.installReplay(runtime);
      syncUiHostCapabilities();
      resetUiState();
      return runtime;
    }

    async function loadApp<Exports extends HarnessExports>(
      loadOptions: HarnessAppOptions<Exports>,
    ): Promise<HarnessContext<Exports>> {
      if (loadOptions.showLoadingOverlay !== false) {
        updateLoadingOverlay(`Loading ${loadOptions.wasmPath}`);
      }
      await unloadApp();
      const restoredSnapshot = await queuePersistedUiStateWork(() => {
        switch (loadOptions.persistedRestoreMode ?? 'initial') {
          case 'none':
            return Promise.resolve(null);
          case 'pop':
            return loadPopPersistedSnapshot(`loading ${loadOptions.wasmPath}`);
          case 'initial':
          default:
            return loadInitialPersistedSnapshot(`loading ${loadOptions.wasmPath}`);
        }
      });
      hydrateCurrentPersistedEntries(restoredSnapshot);
      const wasmModule = await loadWasmModule(loadOptions.wasmPath);
      validateAppImports(wasmModule, loadOptions.hostServices);
      const instance = await instantiate(wasmModule, createAppImports(loadOptions.hostServices));
      const exports = instance.exports as unknown as Exports;
      const keyBufferPtr = exports.__fui_key_buffer();
      const textBufferPtr = exports.__fui_text_buffer();
      const textBufferSize = textBufferPtr !== 0 ? exports.__fui_text_buffer_size() : 0;
      const sessionBase = {
        exports,
        memory: exports.memory,
        keyBufferPtr,
        textBufferPtr,
        textBufferSize,
        hostEventDisposers: [],
      };
      const session: HarnessAppSession = {
        ...sessionBase,
        ...(loadOptions.workerHostServices === undefined ? {} : { workerHostServices: loadOptions.workerHostServices }),
        ...(loadOptions.onStateUpdated === undefined ? {} : { onStateUpdated: loadOptions.onStateUpdated }),
        ...(loadOptions.onDispose === undefined
          ? {}
          : { onDispose: (activeExports: HarnessExports) => {
            loadOptions.onDispose?.(activeExports as Exports);
          } }),
      };
      currentSession = session;

      // Wire the immediate-mode custom draw callback (Tier 1 → JS → Tier 3)
      (window as unknown as Record<string, unknown>).__effindomV2CustomDraw = (
        handleLo: number,
        handleHi: number,
        canvasPtrLo: number,
        canvasPtrHi = 0,
      ): void => {
        const handle = (BigInt(handleHi >>> 0) << 32n) | BigInt(handleLo >>> 0);
        const canvasPtr = (BigInt(canvasPtrHi >>> 0) << 32n) | BigInt(canvasPtrLo >>> 0);
        const canvasToken = canvasHost.tokenForCanvasPointer(canvasPtr);
        const rawExports = exports as unknown as Record<string, unknown>;
        if (typeof rawExports.fui_dispatch_custom_draw === 'function') {
          (rawExports.fui_dispatch_custom_draw as (h: bigint, p: number) => void)(handle, canvasToken);
        }
      };

      notifyRouteForCurrentLocation(session);
      runtime.setAppFrameHandler((timestampMs: number) => {
        if (currentSession !== session) {
          return;
        }
        exports.__fui_on_frame(timestampMs);
        appFlushRequested = false;
        exports.__flushRenders();
      });
      runtime.resetLogs();
      loadOptions.run(exports);
      connectHostEvents(session, exports, loadOptions.hostEvents);
      runtime.runAppFrameHandler(performance.now());
      notifyViewport(session);
      notifySystemTheme(session);
      if (restoredSnapshot !== null) {
        restoreCurrentPersistedUiState(`loading ${loadOptions.wasmPath}`);
        await settleCurrentSessionAfterRestore(`loading ${loadOptions.wasmPath}`);
      }
      const context: HarnessContext<Exports> = {
        runtime,
        exports,
        waitForFrame,
      };
      await loadOptions.onReady?.(context);
      runtime.clearPointerHover();
      runtime.refreshPointerHover();
      runtime.flushPendingCommit();
      await waitForFrame();
      await queuePersistedUiStateWork(() => ensureCurrentHistoryEntrySnapshot(`loading ${loadOptions.wasmPath}`));
      lastHandledUrlHref = window.location.href;
      updateState();
      finishLoadingOverlay();
      return context;
    }

    const controller: HarnessController = {
      get runtime() {
        return runtime;
      },
      waitForFrame,
      loadApp,
      unloadApp,
      recreateRuntime,
      setSameOriginNavigationHandler(handler) {
        navigationHandler = handler;
      },
    };

    await options.onReady?.(controller);
  }).catch((error: unknown) => {
    cleanup();
    const message = error instanceof Error ? error.message : String(error);
    console.error(error instanceof Error ? error.stack ?? error : error);
    failLoadingOverlay(message);
    const windowWithHarnessError = window as Window & { __fuiAsError?: string; __fuiAsReady?: boolean };
    windowWithHarnessError.__fuiAsReady = false;
    windowWithHarnessError.__fuiAsError = message;
    options.onError?.(error);
    throw error;
  });
}
