import type { BridgeRuntime } from '../core-types';
import type { PreparedRuntimeAssets, BridgeInteractionState } from './local-types';
import { buildBackendLadder, createErrorWithCause, loadCoreModule, loadIcuData, loadUiModule, showIcuError } from './utils/assets';
import { initRenderer } from './utils/backends';
import { ensureCanvasLogicalSize, installEventHandlers } from './events';
import { getBridgeAssetUrl, STARTUP_BRIDGE_FONTS } from './font-catalog';
import { createBridgeRuntime } from './runtime';
import { installRenderLoop } from './render-loop';
import { browserBridgePlatformHost } from './host/platform-host';

export interface BridgeSession {
  readonly runtime: BridgeRuntime;
  destroy(): void;
}

export interface BridgeSessionOptions {
  readonly interactionState: BridgeInteractionState;
  readonly preparedAssets: PreparedRuntimeAssets;
  readonly runtimeRef: { current: BridgeRuntime | null };
}

function requireCanvas(id: string): HTMLCanvasElement {
  const element = document.getElementById(id);
  if (!(element instanceof HTMLCanvasElement)) {
    throw new Error(`Expected #${id} canvas.`);
  }
  return element;
}

export async function createBridgeSession(options: BridgeSessionOptions): Promise<BridgeSession> {
  const canvas = requireCanvas('fui-canvas');
  ensureCanvasLogicalSize(canvas);

  const { interactionState, preparedAssets, runtimeRef } = options;
  window.__bridgeLoaderInfo = preparedAssets.loaderInfo;
  const [core, ui] = await Promise.all([
    loadCoreModule(preparedAssets.coreBundle, preparedAssets.coreWasm, canvas, preparedAssets.loaderInfo),
    loadUiModule(preparedAssets.uiBundle, preparedAssets.uiWasm, preparedAssets.loaderInfo),
  ]);
  const host = browserBridgePlatformHost;
  const runtimeState = createBridgeRuntime(core, ui, canvas, interactionState, preparedAssets.loaderInfo, host);
  const runtime = runtimeState.runtime;
  runtimeRef.current = runtime;

  const dpr = host.getDevicePixelRatio();
  const fallbackLadder = buildBackendLadder(preparedAssets.loaderInfo.requestedRendererBackend);
  await initRenderer(core, canvas, dpr, preparedAssets.loaderInfo, fallbackLadder);
  ui._ui_reset();
  try {
    await loadIcuData(ui, preparedAssets);
  } catch (error: unknown) {
    const message = error instanceof Error ? error.message : 'Failed to load text engine.';
    showIcuError(message);
    throw createErrorWithCause(message, error);
  }
  runtime.updateCanvasSize();
  const disposeEventHandlers = installEventHandlers(runtime, interactionState, host);
  const disposeRenderLoop = installRenderLoop(runtime, preparedAssets.loaderInfo, fallbackLadder, host);
  await Promise.all(
    STARTUP_BRIDGE_FONTS.map((font) => runtime.registerFont({
      id: font.id,
      url: getBridgeAssetUrl(font.assetFile),
      fallbackIds: font.fallbackIds,
    })),
  );

  delete window.__bridgeError;
  return {
    runtime,
    destroy: () => {
      disposeEventHandlers();
      disposeRenderLoop();
      runtimeState.destroy();
      if (runtimeRef.current === runtime) {
        runtimeRef.current = null;
      }
    },
  };
}
