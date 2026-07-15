import type {
  BridgeLoaderInfo,
  BridgeLogs,
  CoreModule,
  EdBackendType as EdBackendTypeValue,
  UiModule,
} from '../../core-types';
import { EdBackendType, EdDeviceState } from '../../core-types';
import type {
  ArchitectureSelection,
  PreparedRuntimeAssets,
  PreparedWasmAsset,
  RequestedRendererBackend,
  RuntimeManifest,
} from '../local-types';
import { ASSET_FETCH_ATTEMPTS, fetchBinaryAsset, fetchScriptSource, fetchWithRetry, loadScriptResource, resolveAssetUrl } from './fetch';
import { DEFAULT_BACKEND_LADDER } from './backends';
import { writeBytesToHeap } from './heap';

declare const HEAPU8: Uint8Array | undefined;
declare const HEAPU32: Uint32Array | undefined;

declare global {
  interface EffinDomRuntimeConfig {
    manifestUrl?: string;
    manifestUrls?: readonly string[];
    expectedRuntimeSetHash?: string;
    buildMode?: 'debug' | 'release';
    devToolsDomMirror?: 'disabled' | 'enabled' | 'on-requested';
    pageZoom?: 'disabled' | 'enabled';
  }

  interface Window {
    __effindomRuntime?: EffinDomRuntimeConfig;
    __effindomResolvedRuntimeAssets?: {
      readonly manifestUrl: string;
      readonly fontUrls: Readonly<Record<string, string>>;
    };
  }
}

interface LoadedRuntimeManifest {
  readonly manifest: RuntimeManifest;
  readonly manifestUrl: string;
}

const MEMORY64_VALIDATION_MODULE_BYTES = new Uint8Array([
  0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x0f, 0x03, 0x60,
  0x02, 0x7f, 0x7e, 0x01, 0x7f, 0x60, 0x01, 0x7e, 0x00, 0x60, 0x00, 0x01,
  0x7e, 0x03, 0x04, 0x03, 0x00, 0x01, 0x02, 0x04, 0x05, 0x01, 0x70, 0x05,
  0x01, 0x01, 0x05, 0x06, 0x01, 0x05, 0x82, 0x02, 0x82, 0x02, 0x06, 0x08,
  0x01, 0x7e, 0x01, 0x42, 0x80, 0x88, 0x04, 0x0b, 0x07, 0x68, 0x05, 0x06,
  0x6d, 0x65, 0x6d, 0x6f, 0x72, 0x79, 0x02, 0x00, 0x04, 0x6d, 0x61, 0x69,
  0x6e, 0x00, 0x00, 0x19, 0x5f, 0x5f, 0x69, 0x6e, 0x64, 0x69, 0x72, 0x65,
  0x63, 0x74, 0x5f, 0x66, 0x75, 0x6e, 0x63, 0x74, 0x69, 0x6f, 0x6e, 0x5f,
  0x74, 0x61, 0x62, 0x6c, 0x65, 0x01, 0x00, 0x19, 0x5f, 0x65, 0x6d, 0x73,
  0x63, 0x72, 0x69, 0x70, 0x74, 0x65, 0x6e, 0x5f, 0x73, 0x74, 0x61, 0x63,
  0x6b, 0x5f, 0x72, 0x65, 0x73, 0x74, 0x6f, 0x72, 0x65, 0x00, 0x01, 0x1c,
  0x65, 0x6d, 0x73, 0x63, 0x72, 0x69, 0x70, 0x74, 0x65, 0x6e, 0x5f, 0x73,
  0x74, 0x61, 0x63, 0x6b, 0x5f, 0x67, 0x65, 0x74, 0x5f, 0x63, 0x75, 0x72,
  0x72, 0x65, 0x6e, 0x74, 0x00, 0x02, 0x0a, 0x12, 0x03, 0x04, 0x00, 0x41,
  0x00, 0x0b, 0x06, 0x00, 0x20, 0x00, 0x24, 0x00, 0x0b, 0x04, 0x00, 0x23,
  0x00, 0x0b,
]);

const SIMD_VALIDATION_MODULE_BYTES = new Uint8Array([
  0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
  0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
  0x03, 0x02, 0x01, 0x00,
  0x0a, 0x17, 0x01, 0x15, 0x00, 0xfd, 0x0c,
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
  0x1a, 0x0b,
]);

export function showIcuError(message: string): void {
  const errorBox = document.getElementById('icu-error');
  const messageNode = document.getElementById('icu-error-message');
  if (errorBox instanceof HTMLElement && messageNode instanceof HTMLElement) {
    messageNode.textContent = message;
    errorBox.style.display = 'block';
    return;
  }

  const overlay = document.getElementById('effindom-loading-overlay');
  const overlayTitle = document.getElementById('effindom-loading-title');
  const overlayDetail = document.getElementById('effindom-loading-detail');
  if (
    overlay instanceof HTMLElement &&
    overlayTitle instanceof HTMLElement &&
    overlayDetail instanceof HTMLElement
  ) {
    overlay.dataset.state = 'error';
    overlay.hidden = false;
    overlay.setAttribute('aria-hidden', 'false');
    overlayTitle.textContent = 'The typesetter dragon sneezed on the runtime.';
    overlayDetail.textContent = message;
  }
}

export function createErrorWithCause(message: string, cause: unknown): Error {
  const wrappedError = new Error(message) as Error & { cause?: unknown };
  wrappedError.cause = cause;
  return wrappedError;
}

export async function waitForAnimationFrame(): Promise<void> {
  await new Promise<void>((resolve) => {
    requestAnimationFrame(() => {
      resolve();
    });
  });
}

export function delay(ms: number): Promise<void> {
  return new Promise<void>((resolve) => {
    window.setTimeout(resolve, ms);
  });
}

export function clearRecordMap<T>(recordMap: Record<string, T>): void {
  for (const key of Object.keys(recordMap)) {
    Reflect.deleteProperty(recordMap, key);
  }
}

export function resetBridgeLogs(logs: BridgeLogs): void {
  logs.pointerEvents.length = 0;
  logs.focusEvents.length = 0;
  logs.textChanges.length = 0;
  logs.selectionChanges.length = 0;
  logs.crossSelectionChanges.length = 0;
  logs.clipboardWrites.length = 0;
  logs.clipboardReadRequests.length = 0;
  logs.scrollEvents.length = 0;
  logs.missingFontCoverageRequests.length = 0;
  logs.incrementalFontPackageRequests.length = 0;
}

function supportsMemory64(): boolean {
  try {
    new WebAssembly.Memory({ initial: 1, maximum: 1, index: 'i64' } as WebAssembly.MemoryDescriptor & { index: 'i64' });
    return WebAssembly.validate(MEMORY64_VALIDATION_MODULE_BYTES);
  } catch {
    return false;
  }
}

function supportsSimd(): boolean {
  try {
    return WebAssembly.validate(SIMD_VALIDATION_MODULE_BYTES);
  } catch {
    return false;
  }
}

function normalizeWasmArchitecture(requestedArchitecture: string | null | undefined): ArchitectureSelection['requestedArchitecture'] {
  const value = (requestedArchitecture ?? 'auto').toLowerCase();
  if (value === 'wasm32' || value === 'wasm32-simd' || value === 'wasm64' || value === 'wasm64-simd') {
    return value;
  }
  return 'auto';
}

function getManifestArchitectureEntry(
  manifest: RuntimeManifest,
  architecture: Exclude<ArchitectureSelection['requestedArchitecture'], 'auto'>,
): ArchitectureSelection['manifestEntry'] | null {
  return manifest.architectures[architecture] ?? null;
}

function selectManifestArchitecture(manifest: RuntimeManifest, requestedArchitecture: string | null | undefined): ArchitectureSelection {
  const normalizedRequest = normalizeWasmArchitecture(requestedArchitecture);
  const memory64Supported = supportsMemory64();
  const simdSupported = supportsSimd();
  const availableArchitectures = Object.keys(manifest.architectures).filter((architecture) => manifest.architectures[architecture as Exclude<ArchitectureSelection['requestedArchitecture'], 'auto'>] !== undefined);
  let selectedArchitecture: Exclude<ArchitectureSelection['requestedArchitecture'], 'auto'> | null = null;
  let selectionReason: string;

  const architectureUsesMemory64 = (architecture: string): boolean => architecture.startsWith('wasm64');
  const architectureUsesSimd = (architecture: string): boolean => architecture.endsWith('-simd');

  if (normalizedRequest !== 'auto') {
    const explicitEntry = getManifestArchitectureEntry(manifest, normalizedRequest);
    if (explicitEntry === null) {
      throw new Error(`Manifest does not include ${normalizedRequest} bundles.`);
    }
    if (architectureUsesMemory64(normalizedRequest) && !memory64Supported) {
      throw new Error(`${normalizedRequest} was explicitly requested but this browser does not support Memory64.`);
    }
    if (architectureUsesSimd(normalizedRequest) && !simdSupported) {
      throw new Error(`${normalizedRequest} was explicitly requested but this browser does not support WebAssembly SIMD.`);
    }
    selectedArchitecture = normalizedRequest;
    selectionReason = `Explicit ${normalizedRequest} request.`;
  } else {
    const preferredArchitectures: Exclude<ArchitectureSelection['requestedArchitecture'], 'auto'>[] = [];
    if (memory64Supported && simdSupported) {
      preferredArchitectures.push('wasm64-simd');
    }
    if (memory64Supported) {
      preferredArchitectures.push('wasm64');
    }
    if (simdSupported) {
      preferredArchitectures.push('wasm32-simd');
    }
    preferredArchitectures.push('wasm32');

    for (const candidate of preferredArchitectures) {
      if (getManifestArchitectureEntry(manifest, candidate) !== null) {
        selectedArchitecture = candidate;
        break;
      }
    }

    if (selectedArchitecture === null) {
      throw new Error('Manifest does not expose any wasm bundle architectures compatible with this browser.');
    }

    if (selectedArchitecture === 'wasm64-simd') {
      selectionReason = 'Browser supports Memory64 + SIMD, so EffinDom selected the wasm64-simd bundle set.';
    } else if (selectedArchitecture === 'wasm64') {
      selectionReason = 'Browser supports Memory64, so EffinDom selected the wasm64 bundle set.';
    } else if (selectedArchitecture === 'wasm32-simd') {
      selectionReason = 'Browser supports SIMD, so EffinDom selected the wasm32-simd bundle set.';
    } else {
      selectionReason = 'Browser selected the wasm32 bundle set.';
    }
  }

  const manifestEntry = getManifestArchitectureEntry(manifest, selectedArchitecture);
  if (manifestEntry === null) {
    throw new Error(`Manifest entry for ${selectedArchitecture} is missing.`);
  }

  return {
    requestedArchitecture: normalizedRequest,
    selectedArchitecture,
    availableArchitectures,
    memory64Supported,
    simdSupported,
    selectionReason,
    manifestEntry,
  };
}

function readManifestUrls(): readonly string[] {
  const runtimeConfig = window.__effindomRuntime;
  if (runtimeConfig === undefined) {
    throw new Error('Missing effindom-runtime-config.js. Expected window.__effindomRuntime.manifestUrl before bridge.js loads.');
  }
  const configuredManifestUrls: unknown = runtimeConfig.manifestUrls;
  if (Array.isArray(configuredManifestUrls)) {
    const manifestUrls: string[] = [];
    for (const value of configuredManifestUrls as unknown[]) {
      if (typeof value === 'string' && value.length > 0) {
        manifestUrls.push(value);
      }
    }
    if (manifestUrls.length > 0) {
      return manifestUrls;
    }
  }
  if (typeof runtimeConfig.manifestUrl !== 'string' || runtimeConfig.manifestUrl.length === 0) {
    throw new Error('Malformed effindom-runtime-config.js. Expected window.__effindomRuntime.manifestUrl to be a non-empty string.');
  }
  return [runtimeConfig.manifestUrl];
}

function resolveManifestAssetUrl(manifestUrl: string, assetUrl: string): string {
  return new URL(assetUrl, manifestUrl).toString();
}

async function loadRuntimeManifest(manifestCandidate: string): Promise<LoadedRuntimeManifest> {
  const manifestUrl = resolveAssetUrl(manifestCandidate);
  const manifest = await fetchWithRetry<RuntimeManifest>(
    manifestUrl,
    ASSET_FETCH_ATTEMPTS,
    async (response) => await response.json() as RuntimeManifest,
    { cache: 'force-cache' },
  );
  return {
    manifest,
    manifestUrl,
  };
}

async function instantiatePreparedWasm(
  preparedAsset: PreparedWasmAsset,
  imports: WebAssembly.Imports,
): Promise<{ readonly instance: WebAssembly.Instance; readonly module: WebAssembly.Module; readonly compileMode: 'cached-module' }> {
  const module = await preparedAsset.modulePromise;
  const instance = await WebAssembly.instantiate(module, imports);
  return { instance, module, compileMode: 'cached-module' };
}

export async function loadIcuData(ui: UiModule, preparedAssets: PreparedRuntimeAssets): Promise<void> {
  const bytes = await preparedAssets.icu.bytesPromise;
  const heapBytes = writeBytesToHeap(ui, bytes);
  try {
    ui._ui_register_icu_data(heapBytes.ptr, heapBytes.len);
  } finally {
    heapBytes.dispose();
  }
}

function describeAbortReason(value: unknown, fallback: string): string {
  if (typeof value === 'string' && value.length > 0) {
    return value;
  }
  if (value instanceof Error && value.message.length > 0) {
    return value.message;
  }
  return fallback;
}

function prepareWasmAsset(url: string, integrity: string | null): PreparedWasmAsset {
  const requestInit: RequestInit = {
    credentials: 'same-origin',
    cache: 'force-cache',
  };
  if (integrity !== null) {
    requestInit.integrity = integrity;
  }
  const bytesPromise = fetchWithRetry<ArrayBuffer>(
    url,
    ASSET_FETCH_ATTEMPTS,
    async (response) => await response.arrayBuffer(),
    requestInit,
  );
  return {
    url,
    integrity,
    bytesPromise,
    modulePromise: bytesPromise.then(async (buffer) => await WebAssembly.compile(buffer)),
  };
}

const coreScriptRunnerCache = new Map<string, Promise<(module: CoreModule) => void>>();

async function getCoreScriptRunner(scriptUrl: string, integrity: string | null | undefined): Promise<(module: CoreModule) => void> {
  const absoluteUrl = resolveAssetUrl(scriptUrl);
  const cacheKey = `${absoluteUrl}::${integrity ?? ''}`;
  let runnerPromise = coreScriptRunnerCache.get(cacheKey);
  if (runnerPromise === undefined) {
    runnerPromise = fetchScriptSource(absoluteUrl, integrity).then((sourceText) => {
      const wrappedSource = `${sourceText}\n//# sourceURL=${absoluteUrl.replace(/\s/g, '%20')}`;
      const createRunner = globalThis.Function;
      return createRunner('Module', wrappedSource) as (module: CoreModule) => void;
    });
    coreScriptRunnerCache.set(cacheKey, runnerPromise);
  }
  return await runnerPromise;
}

export async function loadCoreModule(
  bundle: PreparedRuntimeAssets['coreBundle'],
  preparedWasm: PreparedWasmAsset,
  canvas: HTMLCanvasElement,
  loaderInfo: BridgeLoaderInfo,
): Promise<CoreModule> {
  return await new Promise<CoreModule>((resolve, reject) => {
    const module: CoreModule = {
      HEAPU8: new Uint8Array(),
      HEAPU32: new Uint32Array(),
      usesMemory64: loaderInfo.selectedWasmArchitecture.startsWith('wasm64'),
      locateFile: (path) => {
        if (path.endsWith('.wasm')) {
          return resolveAssetUrl(bundle.wasm);
        }
        return resolveAssetUrl(path);
      },
      instantiateWasm: (imports, receiveInstance) => {
        void instantiatePreparedWasm(preparedWasm, imports).then((result) => {
          loaderInfo.coreCompileMode = result.compileMode;
          receiveInstance(result.instance, result.module);
        }).catch((error: unknown) => {
          reject(createErrorWithCause('Failed to instantiate Core wasm.', error));
        });
        return {};
      },
      onAbort: (what) => {
        reject(new Error(describeAbortReason(what, 'Core wasm aborted.')));
      },
      refreshHeapViews: () => {
        if (module.wasmMemory !== undefined) {
          const buffer = module.wasmMemory.buffer;
          module.HEAPU8 = new Uint8Array(buffer);
          module.HEAPU32 = new Uint32Array(buffer);
        } else if (typeof HEAPU8 !== 'undefined') {
          // Fallback for older Emscripten builds that don't expose Module["memory"].
          module.HEAPU8 = HEAPU8;
          if (typeof HEAPU32 !== 'undefined') {
            module.HEAPU32 = HEAPU32;
          }
        }
      },
      canvas,
      onRuntimeInitialized: () => {
        // Emscripten assigns Module["memory"] = wasmMemory when the WASM instance
        // is processed, before onRuntimeInitialized fires.
        const emMemory = (module as { memory?: unknown }).memory;
        if (emMemory instanceof WebAssembly.Memory) {
          module.wasmMemory = emMemory;
        }
        module.refreshHeapViews?.();
        resolve(module);
      },
      _malloc: () => 0,
      _free: () => undefined,
      _ed_init: () => undefined,
      _ed_init_webgl: () => undefined,
      _ed_init_sw: () => undefined,
      _ed_resize: () => undefined,
      _ed_set_viewport_size: () => undefined,
      _ed_set_viewport_transform: () => undefined,
      _ed_get_viewport_scale: () => 1.0,
      _ed_get_viewport_offset_x: () => 0.0,
      _ed_get_viewport_offset_y: () => 0.0,
      _ed_set_viewport_zoom_from_scene_anchor: () => undefined,
      _ed_pan_viewport_by: () => undefined,
      _ed_begin_viewport_pan: () => undefined,
      _ed_update_viewport_pan: () => undefined,
      _ed_end_viewport_pan: () => undefined,
      _ed_tick_viewport_pan_momentum: () => 0,
      _ed_clear_viewport_pan_momentum: () => undefined,
      _ed_register_font: () => undefined,
      _ed_unregister_font: () => undefined,
      _ed_register_svg: () => undefined,
      _ed_register_texture_rgba: () => undefined,
      _ed_register_texture_sub_rgba: () => undefined,
      _ed_unregister_texture: () => undefined,
      _ed_execute_command_buffer: () => undefined,
      _ed_reset_scene: () => undefined,
      _ed_render_frame: () => undefined,
      _ed_clear_focus_state: () => undefined,
      _ed_clear_text_input_state: () => undefined,
      _ed_recover_device: () => undefined,
      _ed_hit_test: () => 0,
      _ed_get_sw_framebuffer: () => 0,
      _ed_get_backend_type: () => EdBackendType.NONE,
      _ed_get_device_state: () => EdDeviceState.OK,
      _ed_notify_webgl_context_lost: () => undefined,
      _ed_debug_simulate_device_lost: () => undefined,
      _ed_canvas_save: () => undefined,
      _ed_canvas_restore: () => undefined,
      _ed_canvas_translate: () => undefined,
      _ed_canvas_scale: () => undefined,
      _ed_canvas_rotate: () => undefined,
      _ed_canvas_clip_rect: () => undefined,
      _ed_canvas_clip_round_rect: () => undefined,
      _ed_canvas_draw_rect: () => undefined,
      _ed_canvas_draw_circle: () => undefined,
      _ed_canvas_draw_line: () => undefined,
      _ed_canvas_draw_round_rect: () => undefined,
      _ed_path_create: () => 0,
      _ed_path_destroy: () => undefined,
      _ed_path_move_to: () => undefined,
      _ed_path_line_to: () => undefined,
      _ed_path_quad_to: () => undefined,
      _ed_path_cubic_to: () => undefined,
      _ed_path_close: () => undefined,
      _ed_path_add_rect: () => undefined,
      _ed_path_add_circle: () => undefined,
      _ed_canvas_draw_path: () => undefined,
      _ed_canvas_draw_text_node: () => undefined,
      _ed_canvas_draw_image: () => undefined,
      _ed_canvas_draw_svg: () => undefined,
      _ed_canvas_draw_batch: () => undefined,
      _ed_canvas_create_offscreen: () => 0,
      _ed_canvas_get_offscreen_canvas: () => 0,
      _ed_canvas_read_offscreen_pixels: () => undefined,
      _ed_canvas_destroy_offscreen: () => undefined,
      _ed_render_node_to_rgba: () => 0,
    };
    void getCoreScriptRunner(bundle.js, bundle.js_integrity ?? null).then((runCoreScript) => {
      runCoreScript(module);
    }).catch(reject);
  });
}

export async function loadUiModule(
  bundle: PreparedRuntimeAssets['uiBundle'],
  preparedWasm: PreparedWasmAsset,
  loaderInfo: BridgeLoaderInfo,
): Promise<UiModule> {
  if (window.EffinDomUiV2ModuleFactory === undefined) {
    await loadScriptResource(bundle.js, bundle.js_integrity ?? null);
  }
  if (window.EffinDomUiV2ModuleFactory === undefined) {
    throw new Error('EffinDomUiV2ModuleFactory did not load.');
  }
  let rejectInstantiation: ((reason: unknown) => void) | null = null;
  const instantiationFailure = new Promise<never>((_, reject) => {
    rejectInstantiation = reject;
  });
  const modulePromise = window.EffinDomUiV2ModuleFactory({
    locateFile: (path: string) => {
      if (path.endsWith('.wasm')) {
        return resolveAssetUrl(bundle.wasm);
      }
      return resolveAssetUrl(path);
    },
    instantiateWasm: (imports: WebAssembly.Imports, receiveInstance: (instance: WebAssembly.Instance, module?: WebAssembly.Module) => void) => {
      void instantiatePreparedWasm(preparedWasm, imports).then((result) => {
        loaderInfo.uiCompileMode = result.compileMode;
        receiveInstance(result.instance, result.module);
      }).catch((error: unknown) => {
        rejectInstantiation?.(createErrorWithCause('Failed to instantiate Ui wasm.', error));
      });
      return {};
    },
    onAbort: (what: unknown) => {
      rejectInstantiation?.(new Error(describeAbortReason(what, 'Ui wasm aborted.')));
    },
  });
  const ui = await Promise.race([modulePromise, instantiationFailure]);
  ui.usesMemory64 = loaderInfo.selectedWasmArchitecture.startsWith('wasm64');
  const uiEmMemory = (ui as { memory?: unknown }).memory;
  if (uiEmMemory instanceof WebAssembly.Memory) {
    ui.wasmMemory = uiEmMemory;
  }
  ui.refreshHeapViews = () => {
    if (ui.wasmMemory !== undefined) {
      const buffer = ui.wasmMemory.buffer;
      ui.HEAPU8 = new Uint8Array(buffer);
      ui.HEAPU32 = new Uint32Array(buffer);
    }
  };
  ui.refreshHeapViews();
  return ui;
}

function readRequestedArchitecture(): string | null {
  return new URLSearchParams(window.location.search).get('arch');
}

function readRequestedRendererBackend(): RequestedRendererBackend {
  const value = new URLSearchParams(window.location.search).get('backend')?.toLowerCase() ?? 'auto';
  if (value === 'webgpu' || value === 'graphite') {
    return 'webgpu';
  }
  if (value === 'webgl2' || value === 'ganesh') {
    return 'webgl2';
  }
  if (value === 'software' || value === 'raster' || value === 'cpu') {
    return 'cpu';
  }
  return 'auto';
}

export function buildBackendLadder(requestedBackend: RequestedRendererBackend): readonly EdBackendTypeValue[] {
  if (requestedBackend === 'webgpu') {
    // TEMPORARY: explicit ?backend=webgpu currently downgrades to WebGL2/CPU.
    return [EdBackendType.WEBGL2, EdBackendType.CPU];
  }
  if (requestedBackend === 'webgl2') {
    return [EdBackendType.WEBGL2, EdBackendType.CPU];
  }
  if (requestedBackend === 'cpu') {
    return [EdBackendType.CPU];
  }
  return DEFAULT_BACKEND_LADDER;
}

async function prepareRuntimeCandidate(manifestCandidate: string): Promise<PreparedRuntimeAssets> {
  const loadedManifest = await loadRuntimeManifest(manifestCandidate);
  const manifest = loadedManifest.manifest;
  const manifestUrl = loadedManifest.manifestUrl;
  const expectedRuntimeSetHash = window.__effindomRuntime?.expectedRuntimeSetHash;
  if (
    typeof expectedRuntimeSetHash === 'string' &&
    expectedRuntimeSetHash.length > 0 &&
    manifest.runtime_set_hash !== expectedRuntimeSetHash
  ) {
    throw new Error(`Runtime manifest set hash mismatch for ${manifestUrl}.`);
  }
  const selection = selectManifestArchitecture(manifest, readRequestedArchitecture());
  const requestedRendererBackend = readRequestedRendererBackend();
  const coreBundle = {
    ...selection.manifestEntry.core,
    js: resolveManifestAssetUrl(manifestUrl, selection.manifestEntry.core.js),
    wasm: resolveManifestAssetUrl(manifestUrl, selection.manifestEntry.core.wasm),
  };
  const uiBundle = {
    ...selection.manifestEntry.ui,
    js: resolveManifestAssetUrl(manifestUrl, selection.manifestEntry.ui.js),
    wasm: resolveManifestAssetUrl(manifestUrl, selection.manifestEntry.ui.wasm),
  };
  const icuAsset = manifest.assets?.icu;
  if (icuAsset === undefined) {
    throw new Error('Manifest is missing the ICU asset descriptor.');
  }

  const loaderInfo: BridgeLoaderInfo = {
    manifestHash: manifest.manifest_hash ?? null,
    requestedWasmArchitecture: selection.requestedArchitecture,
    requestedRendererBackend,
    selectedWasmArchitecture: selection.selectedArchitecture,
    availableWasmArchitectures: selection.availableArchitectures,
    memory64Supported: selection.memory64Supported,
    simdSupported: selection.simdSupported,
    coreCompileMode: 'buffer',
    uiCompileMode: 'buffer',
    icuDataUrl: resolveManifestAssetUrl(manifestUrl, icuAsset.url),
    activeRenderer: 'none',
    deviceRecoveryCount: 0,
  };

  const preparedAssets: PreparedRuntimeAssets = {
    manifest,
    selection,
    loaderInfo,
    coreBundle,
    uiBundle,
    coreWasm: {
      ...prepareWasmAsset(coreBundle.wasm, coreBundle.wasm_integrity ?? null),
    },
    uiWasm: {
      ...prepareWasmAsset(uiBundle.wasm, uiBundle.wasm_integrity ?? null),
    },
    icu: {
      url: resolveManifestAssetUrl(manifestUrl, icuAsset.url),
      integrity: icuAsset.integrity ?? null,
      bytesPromise: fetchBinaryAsset(resolveManifestAssetUrl(manifestUrl, icuAsset.url), icuAsset.integrity ?? null),
    },
  };
  await Promise.all([
    preparedAssets.coreWasm.modulePromise,
    preparedAssets.uiWasm.modulePromise,
    fetchScriptSource(coreBundle.js, coreBundle.js_integrity ?? null),
    fetchScriptSource(uiBundle.js, uiBundle.js_integrity ?? null),
  ]);
  try {
    await preparedAssets.icu.bytesPromise;
  } catch (error: unknown) {
    const detail = error instanceof Error ? ` ${error.message}` : '';
    throw createErrorWithCause(`Failed to load ICU data from ${preparedAssets.icu.url}.${detail}`, error);
  }
  return preparedAssets;
}

export async function prepareRuntimeAssets(): Promise<PreparedRuntimeAssets> {
  const failures: string[] = [];
  for (const manifestCandidate of readManifestUrls()) {
    try {
      const preparedAssets = await prepareRuntimeCandidate(manifestCandidate);
      const manifestUrl = resolveAssetUrl(manifestCandidate);
      const fontUrls: Record<string, string> = {};
      for (const [fileName, descriptor] of Object.entries(preparedAssets.manifest.assets?.fonts ?? {})) {
        fontUrls[fileName] = resolveManifestAssetUrl(manifestUrl, descriptor.url);
      }
      window.__effindomResolvedRuntimeAssets = { manifestUrl, fontUrls };
      return preparedAssets;
    } catch (error: unknown) {
      const message = error instanceof Error ? error.message : String(error);
      failures.push(`${manifestCandidate}: ${message}`);
    }
  }
  const error = new Error(`No EffinDOM runtime candidate could be loaded.\n${failures.join('\n')}`);
  if (failures.some((failure) => failure.includes('Failed to load ICU data'))) {
    showIcuError(error.message);
  }
  throw error;
}
