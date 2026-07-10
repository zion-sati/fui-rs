import type { BridgeRuntime } from './core-types';
import { handleToBigInt } from './bridge/utils/encoding';

const UI_SIZE_UNIT_PIXEL = 0;
const UI_NODE_FLEX_BOX = 0;

function waitForFrame(): Promise<void> {
  return new Promise<void>((resolve) => {
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        resolve();
      });
    });
  });
}

async function runSmokeHarness(runtime: BridgeRuntime): Promise<void> {
  const { canvas, ui } = runtime;
  const rect = canvas.getBoundingClientRect();
  const logicalWidth = rect.width;
  const logicalHeight = rect.height;
  const rootHandle = handleToBigInt(ui._ui_create_node(UI_NODE_FLEX_BOX));
  if (rootHandle === 0n) {
    throw new Error('ui_create_node returned UI_INVALID_HANDLE.');
  }

  ui._ui_set_root(rootHandle);
  ui._ui_set_width(rootHandle, logicalWidth, UI_SIZE_UNIT_PIXEL);
  ui._ui_set_height(rootHandle, logicalHeight, UI_SIZE_UNIT_PIXEL);
  ui._ui_set_bg_color(rootHandle, 0xff0000ff);
  ui._ui_commit_frame();

  const commandWords = runtime.syncCommandBufferToCore();
  await waitForFrame();

  window.__bridgeState = {
    commandWordCount: commandWords.length,
    commandWords: Array.from(commandWords),
    rootHandle: rootHandle.toString(),
  };
  window.__bridgeReady = true;
  delete window.__bridgeError;
}

void window.EffinDomBrowserBridge?.ready.then(async (runtime) => {
  await runSmokeHarness(runtime);
}).catch((error: unknown) => {
  const message = error instanceof Error ? error.message : String(error);
  window.__bridgeError = message;
  throw error;
});

export {};
