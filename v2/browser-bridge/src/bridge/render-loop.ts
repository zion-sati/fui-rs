import type { BridgeLoaderInfo,BridgeRuntime,CoreModule,EdBackendType as EdBackendTypeValue } from '../core-types';
import { EdBackendType,EdDeviceState } from '../core-types';
import type { SoftwarePresenter } from './local-types';
import { delay } from './utils/assets';
import { backendLabel,DEFAULT_BACKEND_LADDER,setActiveRenderer,tryReviveBackend } from './utils/backends';
import { normalizeBackendType,normalizeDeviceState,normalizePointerForWasm,pointerToHeapOffset } from './utils/encoding';

const DEVICE_LOST_RETRY_DELAYS_MS = [500, 1_000, 2_000, 4_000] as const;

function ensureSoftwarePresenter(
  presenter: SoftwarePresenter | null,
  canvas: HTMLCanvasElement,
): SoftwarePresenter {
  if (presenter !== null) {
    return presenter;
  }
  const overlay = document.createElement('canvas');
  overlay.dataset.effindomSoftwareOverlay = 'true';
  overlay.setAttribute('aria-hidden', 'true');
  overlay.style.position = 'absolute';
  overlay.style.pointerEvents = 'none';
  overlay.style.display = 'none';
  overlay.style.zIndex = '1';

  const parent = canvas.parentElement;
  if (parent !== null) {
    if (getComputedStyle(parent).position === 'static') {
      parent.style.position = 'relative';
    }
    parent.appendChild(overlay);
  } else {
    document.body.appendChild(overlay);
  }

  const ctx = overlay.getContext('2d');
  if (ctx === null) {
    throw new Error('Canvas 2D context is unavailable for software rendering.');
  }
  return {
    canvas: overlay,
    ctx,
    imageData: null,
    width: 0,
    height: 0,
  };
}

function presentSoftwareFrame(
  core: CoreModule,
  canvas: HTMLCanvasElement,
  presenter: SoftwarePresenter,
): void {
  const ptr = normalizePointerForWasm(core, core._ed_get_sw_framebuffer());
  const offset = pointerToHeapOffset(ptr);
  if (offset === 0) {
    return;
  }
  presenter.canvas.style.left = `${String(canvas.offsetLeft)}px`;
  presenter.canvas.style.top = `${String(canvas.offsetTop)}px`;
  presenter.canvas.style.width = canvas.style.width || `${String(canvas.clientWidth)}px`;
  presenter.canvas.style.height = canvas.style.height || `${String(canvas.clientHeight)}px`;
  presenter.canvas.style.borderRadius = getComputedStyle(canvas).borderRadius;
  presenter.canvas.style.display = '';
  if (presenter.imageData === null || presenter.width !== canvas.width || presenter.height !== canvas.height) {
    presenter.canvas.width = canvas.width;
    presenter.canvas.height = canvas.height;
    presenter.imageData = presenter.ctx.createImageData(canvas.width, canvas.height);
    presenter.width = canvas.width;
    presenter.height = canvas.height;
  }
  const byteLength = canvas.width * canvas.height * 4;
  const src = core.HEAPU8.subarray(offset, offset + byteLength);
  presenter.imageData.data.set(src);
  presenter.ctx.putImageData(presenter.imageData, 0, 0);
}

export function installRenderLoop(
  runtime: BridgeRuntime,
  loaderInfo: BridgeLoaderInfo,
  fallbackLadder: readonly EdBackendTypeValue[] = DEFAULT_BACKEND_LADDER,
): () => void {
  const { core, canvas } = runtime;
  let activeBackend = normalizeBackendType(core._ed_get_backend_type());
  let softwarePresenter: SoftwarePresenter | null = null;
  let frameScheduled = false;
  let disposed = false;

  // Recovery state:
  // - lastAttemptedBackend  the backend we're trying to revive (set on first loss,
  //                         kept after a permanent fallback so wake-up can try again)
  // - recoveryAttempts      how many retries of lastAttemptedBackend have been tried
  // - recoveryExhausted     true after all retries failed and we permanently fell back
  // - recoveryPromise       non-null while an async recovery cycle is in progress
  let lastAttemptedBackend: EdBackendTypeValue = activeBackend;
  let recoveryAttempts = 0;
  let recoveryExhausted = false;
  let recoveryPromise: Promise<void> | null = null;

  // Retry the same backend with exponential backoff, then permanently fall back.
  // Mutates recoveryAttempts, recoveryExhausted, activeBackend.
  async function runRecovery(): Promise<void> {
    const dpr = Math.max(1, window.devicePixelRatio || 1);

    while (recoveryAttempts < DEVICE_LOST_RETRY_DELAYS_MS.length) {
      const delayMs = DEVICE_LOST_RETRY_DELAYS_MS[recoveryAttempts] as number;
      await delay(delayMs);

      if (await tryReviveBackend(core, canvas, dpr, lastAttemptedBackend)) {
        recoveryAttempts = 0;
        recoveryExhausted = false;
        activeBackend = lastAttemptedBackend;
        loaderInfo.deviceRecoveryCount += 1;
        setActiveRenderer(loaderInfo, lastAttemptedBackend);
        await runtime.replayLoadedAssets();
        console.info(`RENDERER RECOVERY: ${backendLabel(lastAttemptedBackend)} recovered successfully.`);
        runtime.commitFrame();
        return;
      }

      recoveryAttempts += 1;
    }

    // All retries exhausted — permanently fall to the next available backend.
    recoveryExhausted = true;
    const nextIndex = fallbackLadder.indexOf(lastAttemptedBackend) + 1;
    for (const fallback of fallbackLadder.slice(nextIndex)) {
      if (await tryReviveBackend(core, canvas, dpr, fallback)) {
        activeBackend = fallback;
        loaderInfo.deviceRecoveryCount += 1;
        setActiveRenderer(loaderInfo, fallback);
        await runtime.replayLoadedAssets();
        if (fallback === EdBackendType.CPU) {
          console.error(
            `RENDERER FALLBACK: ${backendLabel(lastAttemptedBackend)} device lost and recovery failed!` +
            ' Fell back to Software/Raster - performance will be painfully slow',
          );
        } else {
          console.warn(
            `RENDERER FALLBACK: ${backendLabel(lastAttemptedBackend)} device lost and recovery failed!` +
            ` Fell back to ${backendLabel(fallback)}`,
          );
        }
        runtime.commitFrame();
        return;
      }
    }

    throw new Error(
      `Renderer recovery failed: all backends exhausted after ${backendLabel(lastAttemptedBackend)} device loss.`,
    );
  }

  function scheduleRecovery(lostBackend: EdBackendTypeValue): void {
    if (recoveryPromise !== null) return;
    lastAttemptedBackend = lostBackend;
    recoveryAttempts = 0;
    recoveryExhausted = false;
    setActiveRenderer(loaderInfo, EdBackendType.NONE);
    recoveryPromise = runRecovery()
      .catch((error: unknown) => {
        const message = error instanceof Error ? error.message : String(error);
        window.__bridgeError = message;
      })
      .finally(() => {
        recoveryPromise = null;
      });
  }

  const scheduleFrame = (): void => {
    if (disposed) {
      return;
    }
    if (frameScheduled) {
      return;
    }
    frameScheduled = true;
    requestAnimationFrame(frame);
  };

  runtime.setFrameRequester(scheduleFrame);

  const handleWebGlContextLost = (event: Event): void => {
    event.preventDefault();
    core._ed_notify_webgl_context_lost?.();
    scheduleRecovery(activeBackend);
    scheduleFrame();
  };
  canvas.addEventListener('webglcontextlost', handleWebGlContextLost, false);

  // When the page becomes visible after being hidden (e.g. lid open after sleep),
  // the GPU may have recovered. If a previous recovery cycle exhausted all retries
  // and permanently fell back, optimistically try to revive the original backend.
  const handleVisibilityChange = (): void => {
    if (disposed) return;
    if (document.visibilityState !== 'visible') return;
    if (!recoveryExhausted) return;
    if (recoveryPromise !== null) return;
    console.info(
      `RENDERER RECOVERY: Page visible — attempting to revive ${backendLabel(lastAttemptedBackend)} after sleep/wake.`,
    );
    recoveryAttempts = 0;
    recoveryPromise = runRecovery()
      .catch((error: unknown) => {
        const message = error instanceof Error ? error.message : String(error);
        window.__bridgeError = message;
      })
      .finally(() => {
        recoveryPromise = null;
      });
  };
  document.addEventListener('visibilitychange', handleVisibilityChange);

  const frame = (now: number): void => {
    if (disposed) {
      frameScheduled = false;
      return;
    }
    frameScheduled = false;
    if (recoveryPromise !== null) {
      scheduleFrame();
      return;
    }
    // Rebuild heap views from the live WebAssembly.Memory buffer each frame.
    // Emscripten updates closure-scoped heap vars on memory growth but not
    // Module['HEAPU8'], leaving it pointing to a detached ArrayBuffer.
    // Reading module.wasmMemory.buffer always returns the current live buffer.
    core.refreshHeapViews?.();
    runtime.ui.refreshHeapViews?.();
    runtime.runAppFrameHandler(now);
    if (!runtime.hasPendingCommit() && runtime.uiNeedsAnimationFrame() && runtime.uiHasPendingVisualWork()) {
      runtime.commitFrame(now);
    }
    runtime.flushPendingCommit();
    core._ed_render_frame(now);
    core.refreshHeapViews?.();   // Ensure heap views are up‑to‑date after potential memory growth
    const deviceState = normalizeDeviceState(core._ed_get_device_state());
    const backendType = normalizeBackendType(core._ed_get_backend_type());
    if (deviceState === EdDeviceState.LOST) {
      scheduleRecovery(activeBackend);
      scheduleFrame();
      return;
    }
    activeBackend = backendType;
    setActiveRenderer(loaderInfo, backendType);
    if (backendType === EdBackendType.CPU) {
      softwarePresenter = ensureSoftwarePresenter(softwarePresenter, canvas);
      presentSoftwareFrame(core, canvas, softwarePresenter);
    } else if (softwarePresenter !== null) {
      softwarePresenter.canvas.style.display = 'none';
    }
    if (runtime.hasPendingCommit() || runtime.uiNeedsAnimationFrame()) {
      scheduleFrame();
    }
  };
  scheduleFrame();
  return () => {
    disposed = true;
    frameScheduled = false;
    runtime.setFrameRequester(null);
    canvas.removeEventListener('webglcontextlost', handleWebGlContextLost, false);
    document.removeEventListener('visibilitychange', handleVisibilityChange);
    if (softwarePresenter !== null) {
      softwarePresenter.canvas.remove();
      softwarePresenter = null;
    }
  };
}
