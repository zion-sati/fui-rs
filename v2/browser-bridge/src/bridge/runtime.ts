import type {
  BridgeFontRegistration,
  BridgeFontStackRegistration,
  BridgeLoaderInfo,
  BridgeRuntime,
  CoreModule,
  UiModule,
} from '../core-types';
import type { BridgeInteractionState } from './local-types';
import { resetBridgeLogs } from './utils/assets';
import { normalizeBackendType, handleToBigInt } from './utils/encoding';
import { executeCommandBuffer, extractCommandBuffer } from './utils/heap';
import { setActiveRenderer } from './utils/backends';
import { PageZoomMode, resolveDevToolsDomMirrorConfig } from '../runtime-config';
import { getCanvasSizeSource, readCanvasLogicalSize } from './events';
import { AssetManager } from './runtime/asset-manager';
import { IncrementalFontManager } from './runtime/font-manager';
import { FindController } from './runtime/find-controller';
import { OpenCanvasApiAdapter } from './runtime/open-canvas-api';
import { SemanticController } from './runtime/semantic-controller';
import { DebugTreeController } from './runtime/debug-tree-controller';
import { DevToolsDomMirror } from './runtime/devtools-dom-mirror';
import { TextDocumentController } from './runtime/text-documents';

const DEFAULT_LOGICAL_WIDTH = 320;
const DEFAULT_LOGICAL_HEIGHT = 220;
const UI_EVENT_POINTER_ENTER = 4;
const UI_EVENT_POINTER_LEAVE = 5;

export function createBridgeRuntime(
  core: CoreModule,
  ui: UiModule,
  canvas: HTMLCanvasElement,
  interactionState: BridgeInteractionState,
  loaderInfo: BridgeLoaderInfo,
): { runtime: BridgeRuntime; destroy(): void } {
  let logicalWidth = DEFAULT_LOGICAL_WIDTH;
  let logicalHeight = DEFAULT_LOGICAL_HEIGHT;
  let devicePixelRatio = Math.max(1, window.devicePixelRatio || 1);
  let pageZoomMomentumFrameScheduled = false;
  let needsCommit = false;
  let appFrameHandler: ((timestampMs: number) => void) | null = null;
  let frameRequester: (() => void) | null = null;
  // eslint-disable-next-line prefer-const -- controllers below close over runtime before it is assigned.
  let runtime!: BridgeRuntime;
  const textInputMetadataByHandle = new Map<string, {
    readonly kind: 'text' | 'password' | 'email';
    readonly hostAutofillHint: string | null;
  }>();

  const fontManager = new IncrementalFontManager(core, ui, interactionState.logs, () => {
    runtime.commitFrame();
  });
  const assetManager = new AssetManager(core, fontManager, () => {
    runtime.commitFrame();
  });
  const textDocuments = new TextDocumentController(ui);
  const semanticController = new SemanticController(
    canvas,
    ui,
    interactionState,
    textDocuments,
    () => debugTreeController.getDebugTree(),
    (handle: string) => textInputMetadataByHandle.get(handle) ?? null,
  );
  const debugTreeController = new DebugTreeController(ui);
  const devToolsConfig = resolveDevToolsDomMirrorConfig(window.__effindomRuntime);
  const devToolsDomMirror = new DevToolsDomMirror(canvas, devToolsConfig.devToolsDomMirror, {
    hitTest: (x, y) => {
      const position = runtime.screenToScenePoint(x, y);
      const handle = handleToBigInt(core._ed_hit_test(position.x, position.y));
      return handle === 0n ? null : handle.toString();
    },
  });
  const findController = new FindController({
    canvas,
    ui,
    textDocuments,
    commitFrame: () => {
      runtime.commitFrame();
    },
    flushPendingCommit: () => runtime.flushPendingCommit(),
  });
  const openCanvasApiAdapter = new OpenCanvasApiAdapter({
    ui,
    semantic: semanticController,
    find: findController,
    textDocuments,
    interactionState,
    getDebugTree: () => debugTreeController.getDebugTree(),
    getTextInputMetadata: (handle: string) => textInputMetadataByHandle.get(handle) ?? null,
    commitFrame: () => {
      runtime.commitFrame();
    },
    flushPendingCommit: () => runtime.flushPendingCommit(),
  });

  const syncSemanticAndFindState = (): void => {
    semanticController.syncSemanticState();
    findController.syncDocuments();
    debugTreeController.syncDebugTreeState();
    devToolsDomMirror.sync(debugTreeController.getDebugTree());
  };

  const readNativePageZoom = (): { readonly scale: number; readonly offsetX: number; readonly offsetY: number } => ({
    scale: core._ed_get_viewport_scale(),
    offsetX: core._ed_get_viewport_offset_x(),
    offsetY: core._ed_get_viewport_offset_y(),
  });

  const syncViewportTransform = (): void => {
    const zoom = readNativePageZoom();
    semanticController.syncViewportTransform(zoom.scale, zoom.offsetX, zoom.offsetY);
    findController.syncViewportTransform(zoom.scale, zoom.offsetX, zoom.offsetY);
    devToolsDomMirror.syncViewportTransform(zoom.scale, zoom.offsetX, zoom.offsetY);
  };

  const schedulePageZoomMomentumFrame = (): void => {
    if (pageZoomMomentumFrameScheduled) {
      return;
    }
    pageZoomMomentumFrameScheduled = true;
    requestAnimationFrame((timestampMs) => {
      pageZoomMomentumFrameScheduled = false;
      if (core._ed_tick_viewport_pan_momentum(timestampMs) === 0) {
        syncViewportTransform();
        return;
      }
      syncViewportTransform();
      frameRequester?.();
      schedulePageZoomMomentumFrame();
    });
  };

  const commitUiFrame = (timestampMs: number = performance.now()): void => {
    ui._ui_commit_frame(timestampMs);
  };

  const dispatchAppPointerEvent = (eventType: number, handle: bigint, x: number, y: number, modifiers = 0): void => {
    if (handle === 0n) {
      return;
    }
    window.__effindomCallbacks?.onPointerEventWithMetadata?.(
      eventType,
      handle,
      x,
      y,
      modifiers,
      -1,
      0,
      0,
      0,
      0,
      0,
      0,
      0,
    );
  };

  const isLastPointerStillOverCanvas = (): boolean => {
    const { x, y } = interactionState.getLastPointerClientPosition();
    if (x === null || y === null) {
      return false;
    }
    const rect = canvas.getBoundingClientRect();
    return x >= rect.left && x <= rect.right && y >= rect.top && y <= rect.bottom;
  };

  const syncAppPointerHover = (): void => {
    const { x, y } = interactionState.getLastPointerPosition();
    const previousHandle = interactionState.getLastInteractivePointerHandle();
    const capturedHandle = interactionState.getCapturedPointerHandle();
    if (capturedHandle !== null) {
      if (previousHandle === capturedHandle) {
        return;
      }
      if (previousHandle !== null) {
        dispatchAppPointerEvent(UI_EVENT_POINTER_LEAVE, previousHandle, x, y);
      }
      interactionState.setLastInteractivePointerHandle(capturedHandle);
      dispatchAppPointerEvent(UI_EVENT_POINTER_ENTER, capturedHandle, x, y);
      return;
    }
    const pointerInsideCanvas = interactionState.isPointerInsideCanvas() || isLastPointerStillOverCanvas();
    if (!pointerInsideCanvas) {
      if (previousHandle !== null) {
        interactionState.setLastInteractivePointerHandle(null);
        dispatchAppPointerEvent(UI_EVENT_POINTER_LEAVE, previousHandle, x, y);
      }
      return;
    }

    const scenePosition = runtime.screenToScenePoint(x, y);
    const hitHandle = handleToBigInt(core._ed_hit_test(scenePosition.x, scenePosition.y));
    const currentHandle = hitHandle === 0n ? null : hitHandle;
    if (currentHandle === previousHandle) {
      return;
    }
    if (previousHandle !== null) {
      dispatchAppPointerEvent(UI_EVENT_POINTER_LEAVE, previousHandle, x, y);
    }
    interactionState.setLastInteractivePointerHandle(currentHandle);
    if (currentHandle !== null) {
      dispatchAppPointerEvent(UI_EVENT_POINTER_ENTER, currentHandle, x, y);
    }
  };

  const updateCanvasSize = (): void => {
    const dpr = Math.max(1, window.devicePixelRatio || 1);
    const size = readCanvasLogicalSize(canvas);
    logicalWidth = size.width;
    logicalHeight = size.height;
    devicePixelRatio = dpr;
    const physicalWidth = Math.round(logicalWidth * dpr);
    const physicalHeight = Math.round(logicalHeight * dpr);

    canvas.style.width = `${String(logicalWidth)}px`;
    canvas.style.height = `${String(logicalHeight)}px`;
    if (physicalWidth !== canvas.width || physicalHeight !== canvas.height) {
      canvas.width = physicalWidth;
      canvas.height = physicalHeight;
    }

    semanticController.syncSize(logicalWidth, logicalHeight);
    findController.syncSize(logicalWidth, logicalHeight);
    core._ed_clear_viewport_pan_momentum();
    core._ed_set_viewport_size(logicalWidth, logicalHeight);
    core._ed_resize(physicalWidth, physicalHeight, dpr);
    syncViewportTransform();
    ui._ui_resize_window(logicalWidth, logicalHeight);
    syncSemanticAndFindState();
    setActiveRenderer(loaderInfo, normalizeBackendType(core._ed_get_backend_type()));
  };

  runtime = {
    core,
    ui,
    canvas,
    buildMode: devToolsConfig.buildMode,
    devToolsDomMirrorMode: devToolsConfig.devToolsDomMirror,
    pageZoomMode: devToolsConfig.pageZoom,
    devTools: {
      enableDomMirror: () => {
        devToolsDomMirror.activate();
        return devToolsDomMirror.isActive();
      },
      disableDomMirror: () => {
        devToolsDomMirror.deactivate();
        return devToolsDomMirror.isActive();
      },
      toggleDomMirror: () => devToolsDomMirror.toggle(),
      isDomMirrorEnabled: () => devToolsDomMirror.isActive(),
      selectHandle: (handle) => devToolsDomMirror.selectHandle(handleToBigInt(handle).toString()),
      clearSelection: () => {
        devToolsDomMirror.clearSelection();
      },
      getSelectedHandle: () => devToolsDomMirror.getSelectedHandle(),
      openDebugDialog: () => devToolsDomMirror.openDebugDialog(),
      closeDebugDialog: () => devToolsDomMirror.closeDebugDialog(),
      toggleDebugDialog: () => devToolsDomMirror.toggleDebugDialog(),
      isDebugDialogOpen: () => devToolsDomMirror.isDebugDialogOpen(),
    },
    openCanvasApi: openCanvasApiAdapter.getApi(),
    logs: interactionState.logs,
    updateCanvasSize,
    extractCommandBuffer: () => extractCommandBuffer(ui),
    executeCommandBuffer: (words: Uint32Array) => {
      executeCommandBuffer(core, words);
    },
    syncCommandBufferToCore: () => {
      const words = extractCommandBuffer(ui);
      executeCommandBuffer(core, words);
      syncSemanticAndFindState();
      syncAppPointerHover();
      return words;
    },
    flushPendingCommit: () => {
      if (!needsCommit && !interactionState.hasPendingTextMutations()) {
        return null;
      }
      if (interactionState.hasPendingTextMutations()) {
        if (needsCommit) {
          needsCommit = false;
          runtime.syncCommandBufferToCore();
        }
        if (interactionState.materializePendingTextMutations()) {
          commitUiFrame();
          needsCommit = true;
        }
      }
      if (!needsCommit) {
        return null;
      }
      needsCommit = false;
      return runtime.syncCommandBufferToCore();
    },
    hasPendingCommit: () => needsCommit || interactionState.hasPendingTextMutations(),
    commitFrame: (timestampMs: number = performance.now()) => {
      if (interactionState.hasPendingTextMutations()) {
        if (needsCommit) {
          runtime.flushPendingCommit();
        }
        if (interactionState.materializePendingTextMutations()) {
          commitUiFrame(timestampMs);
          needsCommit = true;
          frameRequester?.();
          return;
        }
      }
      if (needsCommit) {
        runtime.flushPendingCommit();
      }
      commitUiFrame(timestampMs);
      needsCommit = true;
      frameRequester?.();
    },
    requestFrame: () => {
      frameRequester?.();
    },
    setFrameRequester: (requester: (() => void) | null) => {
      frameRequester = requester;
    },
    getSemanticTree: () => semanticController.getSemanticTree(),
    getDebugTree: () => debugTreeController.getDebugTree(),
    setTextInputMetadata: (handle, metadata) => {
      textInputMetadataByHandle.set(handle, metadata);
    },
    getTextInputMetadata: (handle) => textInputMetadataByHandle.get(handle) ?? null,
    clearTextInputMetadata: () => {
      textInputMetadataByHandle.clear();
    },
    getActiveTextHandle: () => interactionState.getActiveTextHandle(),
    getCapturedPointerHandle: () => interactionState.getCapturedPointerHandle(),
    setCapturedPointerHandle: (handle: bigint | null) => {
      interactionState.setCapturedPointerHandle(handle);
    },
    getPageZoom: () => readNativePageZoom(),
    isPageZoomEnabled: () => runtime.pageZoomMode === PageZoomMode.Enabled,
    setPageZoom: (scale: number, offsetX: number, offsetY: number) => {
      const before = readNativePageZoom();
      core._ed_set_viewport_transform(scale, offsetX, offsetY);
      const after = readNativePageZoom();
      if (before.scale === after.scale && before.offsetX === after.offsetX && before.offsetY === after.offsetY) {
        return;
      }
      core._ed_clear_viewport_pan_momentum();
      syncViewportTransform();
      frameRequester?.();
    },
    setPageZoomFromSceneAnchor: (
      scale: number,
      anchorSceneX: number,
      anchorSceneY: number,
      screenX: number,
      screenY: number,
    ) => {
      const before = readNativePageZoom();
      core._ed_set_viewport_zoom_from_scene_anchor(scale, anchorSceneX, anchorSceneY, screenX, screenY);
      const after = readNativePageZoom();
      if (before.scale !== after.scale || before.offsetX !== after.offsetX || before.offsetY !== after.offsetY) {
        core._ed_clear_viewport_pan_momentum();
        syncViewportTransform();
        frameRequester?.();
      }
      return runtime.getPageZoom();
    },
    panPageZoomBy: (deltaX: number, deltaY: number) => {
      if (!runtime.isPageZoomEnabled() || readNativePageZoom().scale <= 1.0) {
        return runtime.getPageZoom();
      }
      const before = readNativePageZoom();
      core._ed_pan_viewport_by(deltaX, deltaY);
      const after = readNativePageZoom();
      if (before.scale !== after.scale || before.offsetX !== after.offsetX || before.offsetY !== after.offsetY) {
        syncViewportTransform();
        frameRequester?.();
      }
      return runtime.getPageZoom();
    },
    beginPageZoomPan: (timestampMs: number) => {
      core._ed_begin_viewport_pan(timestampMs);
    },
    updatePageZoomPan: (deltaX: number, deltaY: number, timestampMs: number) => {
      if (!runtime.isPageZoomEnabled() || readNativePageZoom().scale <= 1.0) {
        return runtime.getPageZoom();
      }
      const before = readNativePageZoom();
      core._ed_update_viewport_pan(deltaX, deltaY, timestampMs);
      const after = readNativePageZoom();
      if (before.scale !== after.scale || before.offsetX !== after.offsetX || before.offsetY !== after.offsetY) {
        syncViewportTransform();
        frameRequester?.();
      }
      return runtime.getPageZoom();
    },
    endPageZoomPan: (timestampMs: number) => {
      core._ed_end_viewport_pan(timestampMs);
      schedulePageZoomMomentumFrame();
    },
    clearPageZoomPanMomentum: () => {
      core._ed_clear_viewport_pan_momentum();
    },
    resetPageZoom: () => {
      runtime.setPageZoom(1.0, 0.0, 0.0);
    },
    screenToScenePoint: (x: number, y: number) => {
      const zoom = readNativePageZoom();
      return {
        x: (x - zoom.offsetX) / zoom.scale,
        y: (y - zoom.offsetY) / zoom.scale,
      };
    },
    setAppFrameHandler: (handler: ((timestampMs: number) => void) | null) => {
      appFrameHandler = handler;
      frameRequester?.();
    },
    runAppFrameHandler: (timestampMs: number) => {
      appFrameHandler?.(timestampMs);
    },
    uiHasPendingVisualWork: () => ui._ui_has_pending_visual_work() !== 0,
    uiNeedsAnimationFrame: () => ui._ui_needs_animation_frame() !== 0,
    getHandleFromPoint: (x: number, y: number) => {
      const position = runtime.screenToScenePoint(x, y);
      return handleToBigInt(core._ed_hit_test(position.x, position.y));
    },
    clearPointerHover: () => {
      interactionState.setLastInteractivePointerHandle(null);
    },
    refreshPointerHover: () => {
      syncAppPointerHover();
    },
    getFindDocuments: () => findController.getFindDocuments(),
    activateFindMatch: (match, reveal = true) => findController.activateFindMatch(match, reveal),
    syncFindSelection: (clearOnMissing = false) => findController.syncFindSelection(clearOnMissing),
    clearFindMatch: () => findController.clearFindMatch(),
    ensureFont: async (fontId: number) => {
      await fontManager.ensureFont(fontId);
    },
    ensureBuiltInFont: async (fontId: number) => {
      await fontManager.ensureBuiltInFont(fontId);
    },
    isFontLoaded: (fontId: number, url?: string) => fontManager.isFontLoaded(fontId, url),
    getIncrementalFontState: (fontId: number) => fontManager.getIncrementalFontState(fontId),
    getIncrementalFontCacheState: () => fontManager.getIncrementalFontCacheState(),
    getIncrementalFontPolicy: () => fontManager.getIncrementalFontPolicy(),
    setIncrementalFontPolicy: (policy) => {
      fontManager.setIncrementalFontPolicy(policy);
    },
    getClipboardFontUrl: (fontId: number) => fontManager.getClipboardFontUrl(fontId),
    registerLazyFont: (fontId: number, url: string) => {
      fontManager.registerLazyFont(fontId, url);
    },
    registerFontFallback: (fontId: number, fallbackFontId: number) => {
      fontManager.registerFontFallback(fontId, fallbackFontId);
    },
    handleMissingFontCoverage: (fontId: number, coverageKind: number, sampleText: string) => {
      fontManager.handleMissingFontCoverage(fontId, coverageKind, sampleText);
    },
    loadFont: async (fontId: number, url: string) => {
      await fontManager.loadFont(fontId, url);
    },
    registerFont: async (font: BridgeFontRegistration) => {
      await fontManager.registerFont(font);
    },
    registerFontStack: async (stack: BridgeFontStackRegistration) => {
      await fontManager.registerFontStack(stack);
    },
    loadSvg: async (svgId: number, url: string) => {
      return await assetManager.loadSvg(svgId, url);
    },
    loadTexture: async (textureId: number, url: string) => {
      return await assetManager.loadTexture(textureId, url);
    },
    releaseSvg: (svgId: number) => {
      assetManager.releaseSvg(svgId);
    },
    releaseTexture: (textureId: number) => {
      assetManager.releaseTexture(textureId);
    },
    replayLoadedAssets: async () => {
      await assetManager.replayLoadedAssets();
    },
    resetLogs: () => {
      resetBridgeLogs(interactionState.logs);
    },
    resetAppSession: () => {
      textInputMetadataByHandle.clear();
      interactionState.resetAppSession();
    },
  };

  const refreshCanvas = (): void => {
    const dpr = Math.max(1, window.devicePixelRatio || 1);
    const size = readCanvasLogicalSize(canvas);
    const physicalWidth = Math.round(size.width * dpr);
    const physicalHeight = Math.round(size.height * dpr);
    if (
      size.width === logicalWidth &&
      size.height === logicalHeight &&
      dpr === devicePixelRatio &&
      physicalWidth === canvas.width &&
      physicalHeight === canvas.height
    ) {
      return;
    }
    runtime.updateCanvasSize();
    runtime.commitFrame();
  };

  const canvasSizeSource = getCanvasSizeSource(canvas);
  const resizeObserver = typeof ResizeObserver !== 'undefined'
    ? new ResizeObserver(() => {
        refreshCanvas();
      })
    : null;
  if (resizeObserver !== null) {
    resizeObserver.observe(canvasSizeSource);
  }

  return {
    runtime,
    destroy: () => {
      resizeObserver?.disconnect();
      runtime.setFrameRequester(null);
      runtime.clearPageZoomPanMomentum();
      runtime.clearPointerHover();
      openCanvasApiAdapter.destroy();
      findController.destroy();
      semanticController.destroy();
      debugTreeController.destroy();
      devToolsDomMirror.destroy();
      canvas.style.cursor = 'default';
    },
  };
}
