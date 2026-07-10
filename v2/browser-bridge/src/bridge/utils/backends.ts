import { EdBackendType, EdDeviceState } from '../../core-types';
import type { BridgeLoaderInfo, CoreModule, EdBackendType as EdBackendTypeValue } from '../../core-types';
import { normalizeBackendType, normalizeDeviceState } from './encoding';

const WEBGPU_INIT_TIMEOUT_MS = 1_500;

// TEMPORARY: keep WebGPU disabled from the bridge until resize/device-loss stability
// issues are resolved. Default to WebGL2 with software fallback.
export const DEFAULT_BACKEND_LADDER = [EdBackendType.WEBGL2, EdBackendType.CPU] as const;

export function backendTypeToRenderer(backendType: EdBackendTypeValue): BridgeLoaderInfo['activeRenderer'] {
  switch (backendType) {
    case EdBackendType.WEBGPU:
      return 'webgpu';
    case EdBackendType.WEBGL2:
      return 'webgl2';
    case EdBackendType.CPU:
      return 'cpu';
    default:
      return 'none';
  }
}

export function setActiveRenderer(loaderInfo: BridgeLoaderInfo, backendType: EdBackendTypeValue): void {
  loaderInfo.activeRenderer = backendTypeToRenderer(backendType);
  window.__bridgeLoaderInfo = loaderInfo;
}

async function waitForAnimationFrame(): Promise<void> {
  await new Promise<void>((resolve) => {
    requestAnimationFrame(() => {
      resolve();
    });
  });
}

export async function waitForWebGpuInit(core: CoreModule): Promise<EdBackendTypeValue> {
  const deadline = performance.now() + WEBGPU_INIT_TIMEOUT_MS;
  while (performance.now() < deadline) {
    const backendType = normalizeBackendType(core._ed_get_backend_type());
    const deviceState = normalizeDeviceState(core._ed_get_device_state());
    if (backendType === EdBackendType.WEBGPU) {
      return backendType;
    }
    if (deviceState !== EdDeviceState.RECOVERING) {
      return backendType;
    }
    await waitForAnimationFrame();
  }
  return normalizeBackendType(core._ed_get_backend_type());
}

export async function probeWebGpuAdapter(): Promise<boolean> {
  const nav = navigator as Navigator & { gpu?: { requestAdapter?: () => Promise<unknown> } };
  if (nav.gpu?.requestAdapter === undefined) {
    return false;
  }
  try {
    const adapter = await nav.gpu.requestAdapter();
    return adapter !== null && adapter !== undefined;
  } catch {
    return false;
  }
}

export function backendLabel(backend: EdBackendTypeValue): string {
  if (backend === EdBackendType.WEBGPU) return 'WebGPU';
  if (backend === EdBackendType.WEBGL2) return 'WebGL2';
  if (backend === EdBackendType.CPU) return 'Software/Raster';
  return 'None';
}

// Attempts to initialise exactly one backend. Returns true on success.
// Safe to call after a device loss — wraps all init calls in try/catch.
export async function tryReviveBackend(
  core: CoreModule,
  canvas: HTMLCanvasElement,
  dpr: number,
  backend: EdBackendTypeValue,
): Promise<boolean> {
  const w = canvas.width;
  const h = canvas.height;
  try {
    if (backend === EdBackendType.WEBGPU) {
      if (!await probeWebGpuAdapter()) return false;
      core._ed_init(w, h, dpr);
      return await waitForWebGpuInit(core) === EdBackendType.WEBGPU;
    }
    if (backend === EdBackendType.WEBGL2) {
      core._ed_init_webgl(w, h, dpr);
      return normalizeBackendType(core._ed_get_backend_type()) === EdBackendType.WEBGL2;
    }
    core._ed_init_sw(w, h, dpr);
    return normalizeBackendType(core._ed_get_backend_type()) === EdBackendType.CPU;
  } catch {
    return false;
  }
}

export async function initRenderer(
  core: CoreModule,
  canvas: HTMLCanvasElement,
  dpr: number,
  loaderInfo: BridgeLoaderInfo,
  backendLadder: readonly EdBackendTypeValue[] = DEFAULT_BACKEND_LADDER,
): Promise<EdBackendTypeValue> {
  const physicalWidth = canvas.width;
  const physicalHeight = canvas.height;
  const ladder = backendLadder.length > 0 ? backendLadder : DEFAULT_BACKEND_LADDER;
  const firstBackend = ladder[0] ?? EdBackendType.WEBGPU;
  let webGpuAttempted = false;
  let webGl2Attempted = false;

  for (const backend of ladder) {
    if (backend === EdBackendType.WEBGPU) {
      // Probe the adapter before calling ed_init. On platforms like Ubuntu where
      // navigator.gpu exists but has no adapter (Vulkan blocklist, no GPU, etc.),
      // requestAdapter() returns null. Calling ed_init without checking locks the
      // canvas to a WebGPU context (even on failure), preventing WebGL2 init.
      if (!await probeWebGpuAdapter()) {
        continue;
      }
      webGpuAttempted = true;
      core._ed_init(physicalWidth, physicalHeight, dpr);
      const resolvedBackend = await waitForWebGpuInit(core);
      if (resolvedBackend === EdBackendType.WEBGPU) {
        setActiveRenderer(loaderInfo, resolvedBackend);
        return resolvedBackend;
      }
      continue;
    }

    if (backend === EdBackendType.WEBGL2) {
      webGl2Attempted = true;
      core._ed_init_webgl(physicalWidth, physicalHeight, dpr);
      if (normalizeBackendType(core._ed_get_backend_type()) === EdBackendType.WEBGL2) {
        if (webGpuAttempted || firstBackend === EdBackendType.WEBGPU) {
          console.warn('RENDERER FALLBACK: WebGPU failed to initialize or unavailable! Fell back to WebGL2');
        }
        setActiveRenderer(loaderInfo, EdBackendType.WEBGL2);
        return EdBackendType.WEBGL2;
      }
      continue;
    }

    core._ed_init_sw(physicalWidth, physicalHeight, dpr);
    if (normalizeBackendType(core._ed_get_backend_type()) === EdBackendType.CPU) {
      const triedBackends = [
        webGpuAttempted ? 'WebGPU' : null,
        webGl2Attempted ? 'WebGL2' : null,
      ].filter(Boolean).join(' and ');
      const reason = triedBackends.length > 0
        ? `${triedBackends} failed to initialize or unavailable!`
        : 'No GPU backend available.';
      console.error(`RENDERER FALLBACK: ${reason} Fell back to Software/Raster - performance will be painfully slow`);
      setActiveRenderer(loaderInfo, EdBackendType.CPU);
      return EdBackendType.CPU;
    }
  }

  setActiveRenderer(loaderInfo, EdBackendType.NONE);
  throw new Error('Failed to initialize any renderer backend.');
}
