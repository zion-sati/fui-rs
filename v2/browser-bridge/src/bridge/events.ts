import type { BridgeRuntime } from '../core-types';
import type { BridgeInteractionState } from './local-types';
import { DesktopFindDialogController } from './find-dialog';
import { createPullToRefreshOverlay } from './pull-to-refresh';
import { browserBridgePlatformHost, type BridgePlatformHost } from './host/platform-host';
export {
  ensureCanvasLogicalSize,
  getCanvasSizeSource,
  readCanvasLogicalSize,
} from './events/canvas-geometry';
import { installKeyAndWindowHandlers } from './events/key-router';
import { installPointerHandlers } from './events/pointer-router';

export function installEventHandlers(
  runtime: BridgeRuntime,
  interactionState: BridgeInteractionState,
  host: BridgePlatformHost = browserBridgePlatformHost,
): () => void {
  const { canvas, ui } = runtime;
  const desktopFindDialog = new DesktopFindDialogController(runtime);
  const pullToRefresh = createPullToRefreshOverlay();

  canvas.tabIndex = 0;
  canvas.style.touchAction = 'none';
  canvas.style.outline = 'none';

  const updateCoarsePointerMode = (): void => {
    ui._ui_set_coarse_pointer_mode(host.isCoarsePointer() ? 1 : 0);
  };
  const updatePlatformFamily = (): void => {
    ui._ui_set_platform_family(host.getPlatformFamily());
  };

  updateCoarsePointerMode();
  updatePlatformFamily();
  const stopObservingCoarsePointer = host.observeCoarsePointer(updateCoarsePointerMode);

  const removePointerHandlers = installPointerHandlers(runtime, interactionState, pullToRefresh);
  const removeKeyAndWindowHandlers = installKeyAndWindowHandlers(runtime, interactionState, desktopFindDialog);

  return () => {
    removeKeyAndWindowHandlers();
    removePointerHandlers();
    desktopFindDialog.destroy();
    pullToRefresh.destroy();
    stopObservingCoarsePointer();
  };
}
