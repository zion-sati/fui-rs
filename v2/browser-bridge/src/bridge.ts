import type { BridgeRuntime, BridgeState } from './core-types';
import { createBridgeSession, type BridgeSession } from './bridge/init';
import { installCallbacks } from './bridge/interaction';
import { prepareRuntimeAssets } from './bridge/utils/assets';
import {
  handleToBigInt,
  handleToString,
  normalizePointerForWasm,
  pointerToHeapOffset,
  toHeapPointer,
} from './bridge/utils/encoding';

let currentRuntime: BridgeRuntime | null = null;
let currentSession: BridgeSession | null = null;
let bridgeReadyPromise: Promise<BridgeRuntime> | null = null;
let sessionChain: Promise<void> = Promise.resolve();

const runtimeRef: { current: BridgeRuntime | null } = { current: null };
const interactionState = installCallbacks(runtimeRef);
const preparedAssetsPromise = prepareRuntimeAssets();

async function bootRuntimeSession(): Promise<BridgeRuntime> {
  currentRuntime?.resetLogs();
  currentSession?.destroy();
  currentSession = null;

  const preparedAssets = await preparedAssetsPromise;
  const session = await createBridgeSession({
    interactionState,
    preparedAssets,
    runtimeRef,
  });
  currentSession = session;
  currentRuntime = session.runtime;
  window.__bridgeReady = true;
  window.__bridgeDebug = {
    forceDeviceLost() {
      session.runtime.core._ed_debug_simulate_device_lost?.();
      session.runtime.requestFrame();
    },
  };
  delete window.__bridgeError;
  return session.runtime;
}

function queueRuntimeBoot(): Promise<BridgeRuntime> {
  sessionChain = sessionChain
    .catch(() => undefined)
    .then(async () => {
      await bootRuntimeSession();
    });
  return sessionChain.then(() => {
    if (currentRuntime === null) {
      throw new Error('Bridge runtime failed to boot.');
    }
    return currentRuntime;
  });
}

bridgeReadyPromise = queueRuntimeBoot().catch((error: unknown) => {
  const message = error instanceof Error ? error.message : String(error);
  window.__bridgeReady = false;
  window.__bridgeError = message;
  throw error;
});

const bridgeState: BridgeState = {
  get ready(): Promise<BridgeRuntime> {
    return bridgeReadyPromise ?? Promise.reject(new Error('Bridge runtime is not booting.'));
  },
  devTools: {
    enableDomMirror: () => currentRuntime?.devTools.enableDomMirror() ?? false,
    disableDomMirror: () => currentRuntime?.devTools.disableDomMirror() ?? false,
    toggleDomMirror: () => currentRuntime?.devTools.toggleDomMirror() ?? false,
    isDomMirrorEnabled: () => currentRuntime?.devTools.isDomMirrorEnabled() ?? false,
    selectHandle: (handle) => currentRuntime?.devTools.selectHandle(handle) ?? false,
    clearSelection: () => {
      currentRuntime?.devTools.clearSelection();
    },
    getSelectedHandle: () => currentRuntime?.devTools.getSelectedHandle() ?? null,
    openDebugDialog: () => currentRuntime?.devTools.openDebugDialog() ?? false,
    closeDebugDialog: () => currentRuntime?.devTools.closeDebugDialog() ?? false,
    toggleDebugDialog: () => currentRuntime?.devTools.toggleDebugDialog() ?? false,
    isDebugDialogOpen: () => currentRuntime?.devTools.isDebugDialogOpen() ?? false,
  },
  getRuntime: () => currentRuntime,
  recreateRuntime: async () => {
    bridgeReadyPromise = queueRuntimeBoot();
    return await bridgeReadyPromise;
  },
  resetLogs: () => {
    if (currentRuntime !== null) {
      currentRuntime.resetLogs();
    }
  },
  getPageZoom: () => currentRuntime?.getPageZoom() ?? { scale: 1.0, offsetX: 0.0, offsetY: 0.0 },
  setPageZoom: (scale, offsetX = 0.0, offsetY = 0.0) => {
    currentRuntime?.setPageZoom(scale, offsetX, offsetY);
  },
  resetPageZoom: () => {
    currentRuntime?.resetPageZoom();
  },
  handleToBigInt,
  handleToString,
  pointerToHeapOffset,
  normalizePointerForWasm,
  toHeapPointer,
};

window.__bridgeReady = false;
window.EffinDomBrowserBridge = bridgeState;

void bridgeReadyPromise.catch(() => undefined);

export {};
