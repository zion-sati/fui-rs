import type { BridgeRuntime } from '../core-types';
import type { BridgeInteractionState } from './local-types';
import { DesktopFindDialogController } from './find-dialog';
import { detectPlatformFamily } from './platform';
import { createPullToRefreshOverlay } from './pull-to-refresh';
export {
  ensureCanvasLogicalSize,
  getCanvasSizeSource,
  readCanvasLogicalSize,
} from './events/canvas-geometry';
import { installKeyAndWindowHandlers } from './events/key-router';
import { installPointerHandlers } from './events/pointer-router';

function detectCoarsePointerMode(): boolean {
  return window.matchMedia('(pointer: coarse)').matches || navigator.maxTouchPoints > 0;
}

export function installEventHandlers(runtime: BridgeRuntime, interactionState: BridgeInteractionState): () => void {
  const { canvas, ui } = runtime;
  const desktopFindDialog = new DesktopFindDialogController(runtime);
  const pullToRefresh = createPullToRefreshOverlay();
  const coarsePointerQuery = window.matchMedia('(pointer: coarse)');

  canvas.tabIndex = 0;
  canvas.style.touchAction = 'none';
  canvas.style.outline = 'none';

  const updateCoarsePointerMode = (): void => {
    ui._ui_set_coarse_pointer_mode(detectCoarsePointerMode() ? 1 : 0);
  };
  const updatePlatformFamily = (): void => {
    ui._ui_set_platform_family(detectPlatformFamily());
  };

  updateCoarsePointerMode();
  updatePlatformFamily();
  coarsePointerQuery.addEventListener('change', updateCoarsePointerMode);

  const removePointerHandlers = installPointerHandlers(runtime, interactionState, pullToRefresh);
  const removeKeyAndWindowHandlers = installKeyAndWindowHandlers(runtime, interactionState, desktopFindDialog);

  return () => {
    removeKeyAndWindowHandlers();
    removePointerHandlers();
    desktopFindDialog.destroy();
    pullToRefresh.destroy();
    coarsePointerQuery.removeEventListener('change', updateCoarsePointerMode);
  };
}
