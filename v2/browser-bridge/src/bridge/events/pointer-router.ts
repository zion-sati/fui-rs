import type { BridgeRuntime } from '../../core-types';
import { commitIfVisualWork } from '../commit-policy';
import type { BridgeInteractionState } from '../local-types';
import { PointerMoveCoalescer } from '../pointer-move-coalescer';
import { PULL_TO_REFRESH_THRESHOLD,type PullToRefreshOverlay } from '../pull-to-refresh';
import {
type TouchGestureState,
transitionTouchGesture,
} from '../touch-gesture';
import { computeModifiers,handleToBigInt } from '../utils/encoding';
import {
getPointerPosition,
isPointerInsideCanvas,
normalizeWheelDelta,
} from './canvas-geometry';
import { findEditorTextHandleAtPoint } from './semantic-hit-testing';

const UI_EVENT_POINTER_DOWN = 1;
const UI_EVENT_POINTER_UP = 2;
const UI_EVENT_POINTER_MOVE = 3;
const UI_EVENT_POINTER_LEAVE = 5;
const UI_EVENT_POINTER_CANCEL = 6;
const POINTER_TYPE_UNKNOWN = 0;
const POINTER_TYPE_MOUSE = 1;
const POINTER_TYPE_TOUCH = 2;
const POINTER_TYPE_PEN = 3;
const GESTURE_INTENT_NONE = 0;
const GESTURE_INTENT_PAN = 1;
const GESTURE_INTENT_PINCH = 2;
const GESTURE_PHASE_BEGIN = 1;
const GESTURE_PHASE_UPDATE = 2;
const GESTURE_PHASE_END = 3;
const GESTURE_PHASE_CANCEL = 4;
const GESTURE_KIND_PAN = 1;
const GESTURE_KIND_PINCH = 2;
const EDGE_AUTOSCROLL_THRESHOLD = 30;
const TOUCH_SCROLL_THRESHOLD = 8;
const TWO_FINGER_PAN_THRESHOLD = 10;
const TWO_FINGER_PINCH_SCALE_THRESHOLD = 0.04;
const LONG_PRESS_DELAY_MS = 500;
const LONG_PRESS_MOVEMENT_TOLERANCE = 10;
const TOUCH_AXIS_LOCK_DOMINANCE_RATIO = 1.25;
const TOUCH_AXIS_BREAKOUT_DISTANCE = 32;
const TOUCH_AXIS_BREAKOUT_RATIO = 0.72;
const TOUCH_AXIS_BREAKOUT_STEP_THRESHOLD = 8;
const TRACKPAD_PINCH_WHEEL_SCALE = 0.01;
const REPEAT_CLICK_THRESHOLD_MS = 500;
const REPEAT_CLICK_DISTANCE = 6;
const MAX_CLICK_COUNT = 3;

interface PendingPointerMove {
  handle: bigint;
  x: number;
  y: number;
  clientX: number;
  clientY: number;
  pointerInsideCanvas: boolean;
  modifiers: number;
  pointerId: number;
  pointerType: number;
  button: number;
  buttons: number;
  pressure: number;
  width: number;
  height: number;
}

interface ActivePageZoomGesture {
  readonly pointerIds: readonly [number, number];
  anchorSceneX: number;
  anchorSceneY: number;
  initialDistance: number;
  initialScale: number;
}

interface ActiveControlGesture {
  readonly pointerIds: readonly [number, number];
  readonly ownerHandle: bigint;
  readonly intent: number;
  initialDistance: number;
  lastMidpoint: TouchPoint;
  movedPointerIds: Set<number>;
  started: boolean;
  kind: number;
}

interface TouchPoint {
  readonly x: number;
  readonly y: number;
}

interface ActiveLongPressGesture {
  readonly pointerId: number;
  readonly ownerHandle: bigint;
  readonly startX: number;
  readonly startY: number;
  readonly modifiers: number;
  readonly pointerType: number;
  readonly durationMs: number;
  readonly movementTolerance: number;
  readonly timeoutId: number;
  fired: boolean;
  handled: boolean;
  continuesWithPointerCapture: boolean;
}

function resolvePrimaryTouchAxis(deltaX: number, deltaY: number): 'x' | 'y' {
  const absX = Math.abs(deltaX);
  const absY = Math.abs(deltaY);
  if (absX >= absY * TOUCH_AXIS_LOCK_DOMINANCE_RATIO) {
    return 'x';
  }
  if (absY >= absX * TOUCH_AXIS_LOCK_DOMINANCE_RATIO) {
    return 'y';
  }
  return absX >= absY ? 'x' : 'y';
}

function triggerLongPressHapticFeedback(): void {
  const vibrate = navigator.vibrate.bind(navigator);
  if (typeof vibrate !== 'function') {
    return;
  }
  try {
    vibrate(25);
  } catch {
    // Some browsers expose the API but reject vibration from the current context.
  }
}

function distanceBetween(a: TouchPoint, b: TouchPoint): number {
  const dx = b.x - a.x;
  const dy = b.y - a.y;
  return Math.hypot(dx, dy);
}

function midpointBetween(a: TouchPoint, b: TouchPoint): TouchPoint {
  return {
    x: (a.x + b.x) * 0.5,
    y: (a.y + b.y) * 0.5,
  };
}

function normalizePointerType(pointerType: string): number {
  if (pointerType === 'mouse') {
    return POINTER_TYPE_MOUSE;
  }
  if (pointerType === 'touch') {
    return POINTER_TYPE_TOUCH;
  }
  if (pointerType === 'pen') {
    return POINTER_TYPE_PEN;
  }
  return POINTER_TYPE_UNKNOWN;
}

function shouldUnlockTouchAxis(axisMode: 'x' | 'y' | 'xy' | null, travelX: number, travelY: number): boolean {
  if (axisMode === null || axisMode === 'xy') {
    return false;
  }
  const primaryTravel = axisMode === 'x' ? travelX : travelY;
  const secondaryTravel = axisMode === 'x' ? travelY : travelX;
  if (primaryTravel < TOUCH_AXIS_BREAKOUT_DISTANCE || secondaryTravel < TOUCH_AXIS_BREAKOUT_DISTANCE) {
    return false;
  }
  return secondaryTravel >= TOUCH_AXIS_BREAKOUT_DISTANCE &&
    secondaryTravel >= primaryTravel * TOUCH_AXIS_BREAKOUT_RATIO;
}

function currentInteractionTimeMs(): bigint {
  return BigInt(Math.floor(performance.now()));
}

function isEditorTextHandle(runtime: BridgeRuntime, handle: bigint): boolean {
  const handleKey = handle.toString();
  return runtime.getDebugTree().nodesByHandle[handleKey]?.behavior.textEditor === true;
}

export function installPointerHandlers(
  runtime: BridgeRuntime,
  interactionState: BridgeInteractionState,
  pullToRefresh: PullToRefreshOverlay,
): () => void {
  const { canvas, ui } = runtime;
  let primaryPointerDown = false;
  let activePrimaryPointerId: number | null = null;
  let edgeAutoScrollTickScheduled = false;
  let activePrimaryPointerType: string | null = null;
  let activeTouchGesture: TouchGestureState | null = null;
  let touchGestureBreakoutTravel = { x: 0.0, y: 0.0 };
  let touchGesturePanningPageZoom = false;
  const activeTouchPointerIds = new Set<number>();
  const activeTouchPoints = new Map<number, TouchPoint>();
  let activePageZoomGesture: ActivePageZoomGesture | null = null;
  let activeControlGesture: ActiveControlGesture | null = null;
  let activeLongPressGesture: ActiveLongPressGesture | null = null;
  let lastClickHandle: bigint | null = null;
  let lastClickX = 0;
  let lastClickY = 0;
  let lastClickTimeMs = 0;
  let lastClickCount = 0;

  const resolveClickCount = (handle: bigint, x: number, y: number): number => {
    const now = performance.now();
    const deltaX = x - lastClickX;
    const deltaY = y - lastClickY;
    const isRepeat =
      lastClickHandle === handle &&
      (now - lastClickTimeMs) <= REPEAT_CLICK_THRESHOLD_MS &&
      ((deltaX * deltaX) + (deltaY * deltaY)) <= REPEAT_CLICK_DISTANCE * REPEAT_CLICK_DISTANCE;
    lastClickCount = isRepeat && lastClickCount < MAX_CLICK_COUNT ? lastClickCount + 1 : 1;
    lastClickHandle = handle;
    lastClickX = x;
    lastClickY = y;
    lastClickTimeMs = now;
    return lastClickCount;
  };

  const dispatchPointerEvent = (
    eventType: number,
    handle: bigint,
    x: number,
    y: number,
    modifiers: number,
    pointerId: number,
    pointerType: number,
    button: number,
    buttons: number,
    pressure: number,
    width: number,
    height: number,
    clickCount: number,
  ): boolean => {
    const metadata = {
      eventType,
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
    };
    window.__effindomPendingPointerMetadata = metadata;
    window.__effindomLastPointerEventHandled = false;
    ui._ui_on_pointer_event(
      eventType,
      handle,
      x,
      y,
      pointerId,
      pointerType,
      button,
      buttons,
      pressure,
      width,
      height,
      clickCount,
      modifiers,
    );
    if (window.__effindomPendingPointerMetadata === metadata) {
      delete window.__effindomPendingPointerMetadata;
    }
    const handled = (window.__effindomLastPointerEventHandled as boolean | undefined) === true;
    delete window.__effindomLastPointerEventHandled;
    return handled;
  };

  const applySelectionAutoScroll = (x: number, y: number, allowTouch = false): void => {
    if (!primaryPointerDown || (activePrimaryPointerType === 'touch' && !allowTouch)) {
      return;
    }
    if (handleToBigInt(ui._ui_selection_autoscroll(x, y, EDGE_AUTOSCROLL_THRESHOLD)) === 0n) {
      return;
    }
    runtime.flushPendingCommit();
    runtime.requestFrame();
  };

  const processPointerMove = (pending: PendingPointerMove): void => {
    interactionState.setPointerInsideCanvas(pending.pointerInsideCanvas);
    interactionState.setLastPointerClientPosition(pending.clientX, pending.clientY);
    interactionState.setLastPointerPosition(pending.x, pending.y);
    interactionState.setLastPointerModifiers(pending.modifiers);
    ui._ui_set_interaction_time(currentInteractionTimeMs());
    dispatchPointerEvent(
      UI_EVENT_POINTER_MOVE,
      pending.handle,
      pending.x,
      pending.y,
      pending.modifiers,
      pending.pointerId,
      pending.pointerType,
      pending.button,
      pending.buttons,
      pending.pressure,
      pending.width,
      pending.height,
      0,
    );
    commitIfVisualWork(runtime);
    if (pending.handle === 0n) {
      const appCapturedHandle = runtime.getCapturedPointerHandle();
      if (appCapturedHandle !== null) {
        window.__effindomCallbacks?.onPointerEventWithMetadata?.(
          UI_EVENT_POINTER_MOVE,
          appCapturedHandle,
          pending.x,
          pending.y,
          pending.modifiers,
          pending.pointerId,
          pending.pointerType,
          pending.button,
          pending.buttons,
          pending.pressure,
          pending.width,
          pending.height,
          0,
        );
      }
    }
    applySelectionAutoScroll(pending.x, pending.y);
    scheduleEdgeAutoScrollTick();
  };
  const pointerMoveCoalescer = new PointerMoveCoalescer<PendingPointerMove>(processPointerMove);

  const scheduleEdgeAutoScrollTick = (allowTouch = false): void => {
    if (edgeAutoScrollTickScheduled || !primaryPointerDown || (activePrimaryPointerType === 'touch' && !allowTouch)) {
      return;
    }
    edgeAutoScrollTickScheduled = true;
    requestAnimationFrame(() => {
      edgeAutoScrollTickScheduled = false;
      if (!primaryPointerDown) {
        return;
      }
      if (activePrimaryPointerType === 'touch' && (!allowTouch || ui._ui_has_pointer_autoscroll() === 0)) {
        return;
      }
      const { x, y } = interactionState.getLastPointerPosition();
      if (handleToBigInt(ui._ui_selection_autoscroll(x, y, EDGE_AUTOSCROLL_THRESHOLD)) === 0n) {
        return;
      }
      runtime.commitFrame();
      runtime.flushPendingCommit();
      runtime.requestFrame();
      scheduleEdgeAutoScrollTick(allowTouch);
    });
  };

  const releaseCanvasPointerCapture = (pointerId: number): void => {
    if (canvas.hasPointerCapture(pointerId)) {
      canvas.releasePointerCapture(pointerId);
    }
  };

  const captureCanvasPointer = (pointerId: number): void => {
    try {
      canvas.setPointerCapture(pointerId);
    } catch {
      // Synthetic touch events in tests and some browser/device combinations can reject explicit capture.
    }
  };

  const cancelPressedPointerInteraction = (x: number, y: number): void => {
    const capturedHandle = interactionState.getCapturedPointerHandle();
    interactionState.setCapturedPointerHandle(null);
    primaryPointerDown = false;
    activePrimaryPointerId = null;
    activePrimaryPointerType = null;
    pointerMoveCoalescer.clear();
    ui._ui_set_interaction_time(currentInteractionTimeMs());
    dispatchPointerEvent(
      UI_EVENT_POINTER_CANCEL,
      capturedHandle ?? 0n,
      x,
      y,
      0,
      -1,
      POINTER_TYPE_UNKNOWN,
      0,
      0,
      0,
      0,
      0,
      0,
    );
    runtime.commitFrame();
  };

  const cleanupCanceledTouchInteraction = (pointerId: number, timestampMs: number = performance.now()): void => {
    const wasActiveTouchGesture = activeTouchGesture?.pointerId === pointerId;
    if (wasActiveTouchGesture) {
      const scrolling = activeTouchGesture?.phase === 'scrolling';
      activeTouchGesture = transitionTouchGesture(activeTouchGesture, {
        type: 'cancel',
      });
      interactionState.cancelTouchTextFocusDeferral();
      if (scrolling) {
        ui._ui_touch_scroll_end(timestampMs);
      }
      if (touchGesturePanningPageZoom) {
        runtime.endPageZoomPan(timestampMs);
        touchGesturePanningPageZoom = false;
      }
      pullToRefresh.hide(true);
    }
    if (activePrimaryPointerId === pointerId) {
      interactionState.setCapturedPointerHandle(null);
      primaryPointerDown = false;
      activePrimaryPointerId = null;
      activePrimaryPointerType = null;
      pointerMoveCoalescer.clear();
    }
    releaseCanvasPointerCapture(pointerId);
  };

  const cancelActiveTouchGestureForPageZoom = (): void => {
    const timestampMs = performance.now();
    const previousPrimaryPointerId = activePrimaryPointerId;
    interactionState.cancelTouchTextFocusDeferral();
    if (activeTouchGesture !== null) {
      const scrolling = activeTouchGesture.phase === 'scrolling';
      activeTouchGesture = transitionTouchGesture(activeTouchGesture, {
        type: 'cancel',
      });
      if (scrolling) {
        ui._ui_touch_scroll_end(timestampMs);
      }
      if (touchGesturePanningPageZoom) {
        runtime.endPageZoomPan(timestampMs);
        touchGesturePanningPageZoom = false;
      }
    }
    interactionState.setCapturedPointerHandle(null);
    primaryPointerDown = false;
    activePrimaryPointerId = null;
    activePrimaryPointerType = null;
    pointerMoveCoalescer.clear();
    pullToRefresh.hide(true);
    runtime.clearPageZoomPanMomentum();
    if (previousPrimaryPointerId !== null) {
      releaseCanvasPointerCapture(previousPrimaryPointerId);
    }
  };

  const cancelActiveLongPressGesture = (pointerId?: number): void => {
    if (activeLongPressGesture === null) {
      return;
    }
    if (pointerId !== undefined && activeLongPressGesture.pointerId !== pointerId) {
      return;
    }
    window.clearTimeout(activeLongPressGesture.timeoutId);
    activeLongPressGesture = null;
  };

  const fireActiveLongPressGesture = (pointerId?: number): boolean => {
    const gesture = activeLongPressGesture;
    if (gesture === null || (pointerId !== undefined && gesture.pointerId !== pointerId)) {
      return false;
    }
    if (gesture.fired) {
      return gesture.handled;
    }
    gesture.fired = true;
    const scenePoint = runtime.screenToScenePoint(gesture.startX, gesture.startY);
    runtime.setCapturedPointerHandle(null);
    gesture.handled = window.__effindomCallbacks?.onLongPressEventWithCoords?.(
      gesture.ownerHandle,
      scenePoint.x,
      scenePoint.y,
      gesture.pointerId,
      gesture.pointerType,
      gesture.modifiers,
      gesture.durationMs,
    ) === true;
    gesture.continuesWithPointerCapture =
      window.__effindomCallbacks?.longPressContinuesPointerEvents?.(gesture.ownerHandle) === true;
    if (gesture.handled) {
      interactionState.cancelTouchTextFocusDeferral();
      triggerLongPressHapticFeedback();
    }
    commitIfVisualWork(runtime);
    return gesture.handled;
  };

  const tryScheduleLongPressGesture = (
    hitHandle: bigint,
    pointerId: number,
    pointerType: number,
    screenPosition: TouchPoint,
    modifiers: number,
  ): void => {
    cancelActiveLongPressGesture();
    if (pointerType !== POINTER_TYPE_TOUCH && pointerType !== POINTER_TYPE_PEN) {
      return;
    }
    const ownerHandleLike = window.__effindomCallbacks?.resolveLongPressOwner?.(hitHandle);
    if (ownerHandleLike === null || ownerHandleLike === undefined) {
      return;
    }
    const ownerHandle = handleToBigInt(ownerHandleLike);
    if (ownerHandle === 0n) {
      return;
    }
    const durationMs = Math.max(
      0,
      window.__effindomCallbacks?.getLongPressMinimumDurationMs?.(ownerHandle) ?? LONG_PRESS_DELAY_MS,
    );
    const movementTolerance = Math.max(
      0,
      window.__effindomCallbacks?.getLongPressMovementTolerance?.(ownerHandle) ?? LONG_PRESS_MOVEMENT_TOLERANCE,
    );
    const timeoutId = window.setTimeout(() => {
      fireActiveLongPressGesture(pointerId);
    }, durationMs);
    activeLongPressGesture = {
      pointerId,
      ownerHandle,
      startX: screenPosition.x,
      startY: screenPosition.y,
      modifiers,
      pointerType,
      durationMs,
      movementTolerance,
      timeoutId,
      fired: false,
      handled: false,
      continuesWithPointerCapture: false,
    };
  };

  const cleanupHandledLongPressTouch = (pointerId: number): void => {
    activeTouchGesture = null;
    activeTouchPointerIds.delete(pointerId);
    activeTouchPoints.delete(pointerId);
    interactionState.cancelTouchTextFocusDeferral();
    interactionState.setCapturedPointerHandle(null);
    primaryPointerDown = false;
    activePrimaryPointerId = null;
    activePrimaryPointerType = null;
    pointerMoveCoalescer.clear();
    releaseCanvasPointerCapture(pointerId);
  };

  const cancelLongPressIfMovedPastTolerance = (pointerId: number, screenPosition: TouchPoint): void => {
    const gesture = activeLongPressGesture;
    if (gesture?.pointerId !== pointerId || gesture.fired) {
      return;
    }
    const deltaX = screenPosition.x - gesture.startX;
    const deltaY = screenPosition.y - gesture.startY;
    if ((deltaX * deltaX) + (deltaY * deltaY) >= gesture.movementTolerance * gesture.movementTolerance) {
      cancelActiveLongPressGesture(pointerId);
    }
  };

  const tryStartPageZoomGesture = (): void => {
    if (activePageZoomGesture !== null || activeTouchPoints.size < 2) {
      return;
    }
    cancelActiveLongPressGesture();
    const entries = Array.from(activeTouchPoints.entries()).slice(0, 2);
    const first = entries[0];
    const second = entries[1];
    if (first === undefined || second === undefined) {
      return;
    }
    const initialDistance = Math.max(1.0, distanceBetween(first[1], second[1]));
    const midpoint = midpointBetween(first[1], second[1]);
    const anchor = runtime.screenToScenePoint(midpoint.x, midpoint.y);
    const zoom = runtime.getPageZoom();
    activePageZoomGesture = {
      pointerIds: [first[0], second[0]],
      anchorSceneX: anchor.x,
      anchorSceneY: anchor.y,
      initialDistance,
      initialScale: zoom.scale,
    };
    cancelActiveTouchGestureForPageZoom();
  };

  const startPageZoomGestureFromControlFallback = (gesture: ActiveControlGesture): void => {
    if (activePageZoomGesture !== null) {
      return;
    }
    const zoom = runtime.getPageZoom();
    const anchor = runtime.screenToScenePoint(gesture.lastMidpoint.x, gesture.lastMidpoint.y);
    activePageZoomGesture = {
      pointerIds: gesture.pointerIds,
      anchorSceneX: anchor.x,
      anchorSceneY: anchor.y,
      initialDistance: gesture.initialDistance,
      initialScale: zoom.scale,
    };
    cancelActiveTouchGestureForPageZoom();
    updatePageZoomGesture();
  };

  const resolveControlGestureCandidate = (): ActiveControlGesture | null => {
    if (activeTouchPoints.size < 2) {
      return null;
    }
    const entries = Array.from(activeTouchPoints.entries()).slice(0, 2);
    const first = entries[0];
    const second = entries[1];
    if (first === undefined || second === undefined) {
      return null;
    }
    const midpoint = midpointBetween(first[1], second[1]);
    const hitHandle = runtime.getHandleFromPoint(midpoint.x, midpoint.y);
    const ownerHandleLike = window.__effindomCallbacks?.resolveGestureOwner?.(hitHandle);
    if (ownerHandleLike === null || ownerHandleLike === undefined) {
      return null;
    }
    const ownerHandle = handleToBigInt(ownerHandleLike);
    if (ownerHandle === 0n) {
      return null;
    }
    const intent = window.__effindomCallbacks?.getGestureIntent?.(ownerHandle) ?? GESTURE_INTENT_NONE;
    if (intent === GESTURE_INTENT_NONE) {
      return null;
    }
    return {
      pointerIds: [first[0], second[0]],
      ownerHandle,
      intent,
      initialDistance: Math.max(1.0, distanceBetween(first[1], second[1])),
      lastMidpoint: midpoint,
      movedPointerIds: new Set<number>(),
      started: false,
      kind: GESTURE_KIND_PAN,
    };
  };

  const dispatchControlGesture = (
    gesture: ActiveControlGesture,
    phase: number,
    midpoint: TouchPoint,
    deltaX: number,
    deltaY: number,
    scale: number,
  ): boolean => {
    const sceneMidpoint = runtime.screenToScenePoint(midpoint.x, midpoint.y);
    return window.__effindomCallbacks?.onGestureEventWithCoords?.(
      gesture.ownerHandle,
      phase,
      gesture.kind,
      sceneMidpoint.x,
      sceneMidpoint.y,
      deltaX,
      deltaY,
      scale,
      2,
    ) === true;
  };

  const dispatchTrackpadPinchGesture = (
    handle: bigint,
    midpoint: TouchPoint,
    deltaY: number,
    scale: number,
  ): boolean => {
    const ownerHandleLike = window.__effindomCallbacks?.resolveGestureOwner?.(handle);
    if (ownerHandleLike === null || ownerHandleLike === undefined) {
      return false;
    }
    const ownerHandle = handleToBigInt(ownerHandleLike);
    if (ownerHandle === 0n) {
      return false;
    }
    const intent = window.__effindomCallbacks?.getGestureIntent?.(ownerHandle) ?? GESTURE_INTENT_NONE;
    if ((intent & GESTURE_INTENT_PINCH) === 0) {
      return false;
    }
    const sceneMidpoint = runtime.screenToScenePoint(midpoint.x, midpoint.y);
    return window.__effindomCallbacks?.onGestureEventWithCoords?.(
      ownerHandle,
      GESTURE_PHASE_UPDATE,
      GESTURE_KIND_PINCH,
      sceneMidpoint.x,
      sceneMidpoint.y,
      0.0,
      deltaY,
      scale,
      2,
    ) === true;
  };

  const startControlGesture = (
    gesture: ActiveControlGesture,
    midpoint: TouchPoint,
    deltaX: number,
    deltaY: number,
    scale: number,
  ): boolean => {
    gesture.started = true;
    cancelActiveLongPressGesture();
    cancelActiveTouchGestureForPageZoom();
    let handled = dispatchControlGesture(gesture, GESTURE_PHASE_BEGIN, midpoint, 0, 0, 1.0);
    if (deltaX !== 0.0 || deltaY !== 0.0 || scale !== 1.0) {
      handled = dispatchControlGesture(gesture, GESTURE_PHASE_UPDATE, midpoint, deltaX, deltaY, scale) || handled;
    }
    gesture.lastMidpoint = midpoint;
    if (handled) {
      return true;
    }
    if (runtime.isPageZoomEnabled()) {
      startPageZoomGestureFromControlFallback(gesture);
    }
    return true;
  };

  const updateControlGesture = (pointerId: number): boolean => {
    if (activeControlGesture === null) {
      return false;
    }
    const first = activeTouchPoints.get(activeControlGesture.pointerIds[0]);
    const second = activeTouchPoints.get(activeControlGesture.pointerIds[1]);
    if (first === undefined || second === undefined) {
      const midpoint = activeControlGesture.lastMidpoint;
      if (activeControlGesture.started) {
        dispatchControlGesture(activeControlGesture, GESTURE_PHASE_END, midpoint, 0, 0, 1.0);
      }
      activeControlGesture = null;
      return true;
    }
    const midpoint = midpointBetween(first, second);
    const distance = Math.max(1.0, distanceBetween(first, second));
    const scale = distance / activeControlGesture.initialDistance;
    const panDeltaX = midpoint.x - activeControlGesture.lastMidpoint.x;
    const panDeltaY = midpoint.y - activeControlGesture.lastMidpoint.y;
    if (!activeControlGesture.started) {
      activeControlGesture.movedPointerIds.add(pointerId);
      const totalPanX = midpoint.x - activeControlGesture.lastMidpoint.x;
      const totalPanY = midpoint.y - activeControlGesture.lastMidpoint.y;
      const panReady = Math.hypot(totalPanX, totalPanY) >= TWO_FINGER_PAN_THRESHOLD;
      const pinchReady = Math.abs(scale - 1.0) >= TWO_FINGER_PINCH_SCALE_THRESHOLD;
      const wantsPan = (activeControlGesture.intent & GESTURE_INTENT_PAN) !== 0;
      const wantsPinch = (activeControlGesture.intent & GESTURE_INTENT_PINCH) !== 0;
      const bothPointersMoved = activeControlGesture.movedPointerIds.size >= 2;
      if (pinchReady && wantsPinch) {
        activeControlGesture.kind = GESTURE_KIND_PINCH;
        startControlGesture(activeControlGesture, midpoint, totalPanX, totalPanY, scale);
        return true;
      }
      if (panReady && wantsPan && (!pinchReady || bothPointersMoved)) {
        activeControlGesture.kind = GESTURE_KIND_PAN;
        startControlGesture(activeControlGesture, midpoint, totalPanX, totalPanY, scale);
        return true;
      }
      if ((pinchReady && !wantsPinch && bothPointersMoved) || (panReady && !wantsPan && bothPointersMoved)) {
        const gesture = activeControlGesture;
        activeControlGesture = null;
        if (runtime.isPageZoomEnabled()) {
          startPageZoomGestureFromControlFallback(gesture);
        }
        return true;
      }
      return true;
    }
    const handled = dispatchControlGesture(activeControlGesture, GESTURE_PHASE_UPDATE, midpoint, panDeltaX, panDeltaY, scale);
    activeControlGesture.lastMidpoint = midpoint;
    if (handled) {
      activePageZoomGesture = null;
    } else if (activePageZoomGesture === null && runtime.isPageZoomEnabled()) {
      startPageZoomGestureFromControlFallback(activeControlGesture);
    } else {
      updatePageZoomGesture();
    }
    return true;
  };

  const finishControlGesture = (pointerId: number, cancelled: boolean): boolean => {
    if (activeControlGesture?.pointerIds.includes(pointerId) !== true) {
      return false;
    }
    const gesture = activeControlGesture;
    activeControlGesture = null;
    if (gesture.started) {
      dispatchControlGesture(
        gesture,
        cancelled ? GESTURE_PHASE_CANCEL : GESTURE_PHASE_END,
        gesture.lastMidpoint,
        0,
        0,
        1.0,
      );
    }
    return true;
  };

  const updatePageZoomGesture = (): boolean => {
    if (activePageZoomGesture === null) {
      return false;
    }
    const first = activeTouchPoints.get(activePageZoomGesture.pointerIds[0]);
    const second = activeTouchPoints.get(activePageZoomGesture.pointerIds[1]);
    if (first === undefined || second === undefined) {
      activePageZoomGesture = null;
      return true;
    }
    const midpoint = midpointBetween(first, second);
    const distance = Math.max(1.0, distanceBetween(first, second));
    const scale = activePageZoomGesture.initialScale *
      (distance / activePageZoomGesture.initialDistance);
    const zoom = runtime.setPageZoomFromSceneAnchor(
      scale,
      activePageZoomGesture.anchorSceneX,
      activePageZoomGesture.anchorSceneY,
      midpoint.x,
      midpoint.y,
    );
    if (zoom.scale !== scale) {
      const anchor = runtime.screenToScenePoint(midpoint.x, midpoint.y);
      activePageZoomGesture.anchorSceneX = anchor.x;
      activePageZoomGesture.anchorSceneY = anchor.y;
      activePageZoomGesture.initialDistance = distance;
      activePageZoomGesture.initialScale = zoom.scale;
    }
    return true;
  };

  const handleTouchPointerScroll = (
    event: PointerEvent,
    screenPosition: { readonly x: number; readonly y: number },
    position: { readonly x: number; readonly y: number },
    modifiers: number,
  ): boolean => {
    if (event.pointerType !== 'touch' || activeTouchGesture?.pointerId !== event.pointerId) {
      return false;
    }

    const deltaFromStartX = screenPosition.x - activeTouchGesture.startScreenX;
    const deltaFromStartY = screenPosition.y - activeTouchGesture.startScreenY;
    const distanceSquared = (deltaFromStartX * deltaFromStartX) + (deltaFromStartY * deltaFromStartY);

    if (activeTouchGesture.phase === 'pressed') {
      cancelLongPressIfMovedPastTolerance(event.pointerId, screenPosition);
      interactionState.setPointerInsideCanvas(isPointerInsideCanvas(canvas, event));
      interactionState.setLastPointerClientPosition(event.clientX, event.clientY);
      interactionState.setLastPointerPosition(position.x, position.y);
      interactionState.setLastPointerModifiers(modifiers);
      ui._ui_set_interaction_time(currentInteractionTimeMs());
      const pressedMoveHitHandle = runtime.getHandleFromPoint(screenPosition.x, screenPosition.y);
      const pressedMoveHandle = activeTouchGesture.startedOnTextbox
        ? (interactionState.getCapturedPointerHandle() ?? pressedMoveHitHandle)
        : (pressedMoveHitHandle !== 0n
          ? pressedMoveHitHandle
          : (interactionState.getCapturedPointerHandle() ?? 0n));
      const handled = dispatchPointerEvent(
        UI_EVENT_POINTER_MOVE,
        pressedMoveHandle,
        position.x,
        position.y,
        modifiers,
        event.pointerId,
        normalizePointerType(event.pointerType),
        event.button,
        event.buttons,
        event.pressure,
        event.width,
        event.height,
        0,
      );
      if (
        handled ||
        activeTouchGesture.startedOnTextbox ||
        activeLongPressGesture?.continuesWithPointerCapture === true
      ) {
        activeTouchGesture = transitionTouchGesture(activeTouchGesture, {
          type: 'move',
          x: position.x,
          y: position.y,
          screenX: screenPosition.x,
          screenY: screenPosition.y,
        });
        runtime.commitFrame();
        applySelectionAutoScroll(position.x, position.y, true);
        scheduleEdgeAutoScrollTick(true);
        event.preventDefault();
        return true;
      }
      if (distanceSquared < TOUCH_SCROLL_THRESHOLD * TOUCH_SCROLL_THRESHOLD) {
        event.preventDefault();
        return true;
      }
      const primaryAxis = resolvePrimaryTouchAxis(deltaFromStartX, deltaFromStartY);
      activeTouchGesture = transitionTouchGesture(activeTouchGesture, {
        type: 'scroll-threshold-crossed',
        axis: primaryAxis,
      });
      cancelActiveLongPressGesture(event.pointerId);
      if (activeTouchGesture === null) {
        return true;
      }
      interactionState.cancelTouchTextFocusDeferral();
      cancelPressedPointerInteraction(position.x, position.y);
      captureCanvasPointer(event.pointerId);
      const scrollStartHandle = interactionState.getCapturedPointerHandle() ??
        runtime.getHandleFromPoint(screenPosition.x, screenPosition.y);
      ui._ui_touch_scroll_begin(
        scrollStartHandle,
        activeTouchGesture.startX,
        activeTouchGesture.startY,
        event.timeStamp,
      );
      touchGestureBreakoutTravel = { x: 0.0, y: 0.0 };
    }

    const prevState = activeTouchGesture;
    activeTouchGesture = transitionTouchGesture(activeTouchGesture, {
      type: 'move',
      x: position.x,
      y: position.y,
      screenX: screenPosition.x,
      screenY: screenPosition.y,
    });
    if (activeTouchGesture === null) {
      return true;
    }

    const deltaX = prevState.lastX - position.x;
    const deltaY = prevState.lastY - position.y;
    const screenDeltaX = prevState.lastScreenX - screenPosition.x;
    const screenDeltaY = prevState.lastScreenY - screenPosition.y;
    const absDeltaX = Math.abs(screenDeltaX);
    const absDeltaY = Math.abs(screenDeltaY);

    if (absDeltaX >= TOUCH_AXIS_BREAKOUT_STEP_THRESHOLD && absDeltaY >= TOUCH_AXIS_BREAKOUT_STEP_THRESHOLD) {
      touchGestureBreakoutTravel.x += absDeltaX;
      touchGestureBreakoutTravel.y += absDeltaY;
    } else {
      touchGestureBreakoutTravel = { x: 0.0, y: 0.0 };
    }

    if (shouldUnlockTouchAxis(
      activeTouchGesture.axisMode,
      touchGestureBreakoutTravel.x,
      touchGestureBreakoutTravel.y,
    )) {
      activeTouchGesture = transitionTouchGesture(activeTouchGesture, {
        type: 'axis-unlocked',
      });
      if (activeTouchGesture === null) {
        return true;
      }
      touchGestureBreakoutTravel = { x: 0.0, y: 0.0 };
    }

    const scrollDeltaX = activeTouchGesture.axisMode === 'x' || activeTouchGesture.axisMode === 'xy' ? deltaX : 0.0;
    const scrollDeltaY = activeTouchGesture.axisMode === 'y' || activeTouchGesture.axisMode === 'xy' ? deltaY : 0.0;
    const contentCanConsumeScroll = ui._ui_touch_scroll_can_consume(scrollDeltaX, scrollDeltaY) !== 0;

    interactionState.setPointerInsideCanvas(isPointerInsideCanvas(canvas, event));
    interactionState.setLastPointerClientPosition(event.clientX, event.clientY);
    interactionState.setLastPointerPosition(position.x, position.y);
    interactionState.setLastPointerModifiers(modifiers);
    ui._ui_set_interaction_time(currentInteractionTimeMs());
    const pageZoom = runtime.getPageZoom();
    if (
      !activeTouchGesture.pullToRefreshCaptured &&
      !contentCanConsumeScroll &&
      runtime.isPageZoomEnabled() &&
      pageZoom.scale > 1.0
    ) {
      if (!touchGesturePanningPageZoom) {
        runtime.beginPageZoomPan(event.timeStamp);
        touchGesturePanningPageZoom = true;
      }
      const pageZoomDeltaX = activeTouchGesture.axisMode === 'x' || activeTouchGesture.axisMode === 'xy' ? screenDeltaX : 0.0;
      const pageZoomDeltaY = activeTouchGesture.axisMode === 'y' || activeTouchGesture.axisMode === 'xy' ? screenDeltaY : 0.0;
      runtime.updatePageZoomPan(pageZoomDeltaX, pageZoomDeltaY, event.timeStamp);
      pullToRefresh.hide(true);
      event.preventDefault();
      return true;
    }

    if (touchGesturePanningPageZoom) {
      runtime.clearPageZoomPanMomentum();
      touchGesturePanningPageZoom = false;
    }

    if (!activeTouchGesture.startedOnTextbox && !activeTouchGesture.pullToRefreshCaptured) {
      const canCapturePullToRefresh =
        !contentCanConsumeScroll &&
        activeTouchGesture.pullToRefreshEligible &&
        deltaFromStartY > 0.0 &&
        ui._ui_touch_scroll_allows_pull_to_refresh() !== 0;
      if (canCapturePullToRefresh) {
        activeTouchGesture = transitionTouchGesture(activeTouchGesture, {
          type: 'pull-to-refresh-captured',
        }) ?? activeTouchGesture;
      }
    }

    const appliedScrollDeltaY = activeTouchGesture.pullToRefreshCaptured ? 0.0 : scrollDeltaY;

    if (!activeTouchGesture.pullToRefreshCaptured && contentCanConsumeScroll) {
      runtime.deferSemanticProjectionUntilScrollIdle();
      ui._ui_touch_scroll_update(scrollDeltaX, appliedScrollDeltaY, event.timeStamp);
    }

    const pullToRefreshDistance = activeTouchGesture.pullToRefreshCaptured
      ? Math.max(0.0, deltaFromStartY)
      : 0.0;

    activeTouchGesture.pullToRefreshDistance = pullToRefreshDistance;

    if (activeTouchGesture.pullToRefreshCaptured) {
      pullToRefresh.show(pullToRefreshDistance);
    } else {
      pullToRefresh.hide();
    }
    runtime.commitFrame();
    event.preventDefault();
    return true;
  };

  const forwardPointerEvent = (type: number, useHitTest = true) => (event: PointerEvent): void => {
    const modifiers = computeModifiers(event);
    const pointerInsideCanvas = type === UI_EVENT_POINTER_LEAVE ? false : isPointerInsideCanvas(canvas, event);
    const screenPosition = getPointerPosition(canvas, event);
    const position = runtime.screenToScenePoint(screenPosition.x, screenPosition.y);
    const pointerType = normalizePointerType(event.pointerType);
    const pointerId = event.pointerId;
    const button = event.button;
    const buttons = event.buttons;
    const pressure = event.pressure;
    const pointerWidth = event.width;
    const pointerHeight = event.height;
    let clickCount = 0;
    if (event.pointerType === 'touch' && event.cancelable) {
      event.preventDefault();
    }
    const isTouchEvent = event.pointerType === 'touch';
    const isPointerCancel = event.type === 'pointercancel';
    let touchTapCandidateHandle: bigint | null = null;
    let touchTapDiscarded = false;

    if (
      isTouchEvent &&
      activeLongPressGesture?.pointerId === event.pointerId &&
      activeLongPressGesture.handled &&
      !activeLongPressGesture.continuesWithPointerCapture
    ) {
      if (type === UI_EVENT_POINTER_MOVE) {
        event.preventDefault();
        return;
      }
      if (type === UI_EVENT_POINTER_UP || type === UI_EVENT_POINTER_LEAVE || type === UI_EVENT_POINTER_CANCEL) {
        cleanupHandledLongPressTouch(event.pointerId);
        cancelActiveLongPressGesture(event.pointerId);
        event.preventDefault();
        return;
      }
    }

    if (isTouchEvent && isPointerCancel) {
      const cancelHandle = interactionState.getCapturedPointerHandle() ??
        runtime.getHandleFromPoint(screenPosition.x, screenPosition.y);
      activeTouchPointerIds.delete(event.pointerId);
      activeTouchPoints.delete(event.pointerId);
      cancelActiveLongPressGesture(event.pointerId);
      finishControlGesture(event.pointerId, true);
      if (activePageZoomGesture?.pointerIds.includes(event.pointerId)) {
        activePageZoomGesture = null;
      }
      cleanupCanceledTouchInteraction(event.pointerId, event.timeStamp);
      if (event.cancelable) {
        event.preventDefault();
      }
      dispatchPointerEvent(
        UI_EVENT_POINTER_CANCEL,
        cancelHandle,
        position.x,
        position.y,
        modifiers,
        pointerId,
        pointerType,
        button,
        buttons,
        pressure,
        pointerWidth,
        pointerHeight,
        0,
      );
      runtime.commitFrame();
      return;
    }

    if (isTouchEvent && type !== UI_EVENT_POINTER_DOWN &&
      activePrimaryPointerId !== event.pointerId &&
      activeTouchGesture?.pointerId !== event.pointerId &&
      activeControlGesture?.pointerIds.includes(event.pointerId) !== true &&
      activePageZoomGesture?.pointerIds.includes(event.pointerId) !== true) {
      if (type === UI_EVENT_POINTER_UP || type === UI_EVENT_POINTER_LEAVE || type === UI_EVENT_POINTER_CANCEL) {
        activeTouchPointerIds.delete(event.pointerId);
        activeTouchPoints.delete(event.pointerId);
      }
      return;
    }

    if (isTouchEvent && type === UI_EVENT_POINTER_DOWN) {
      activeTouchPointerIds.add(event.pointerId);
      activeTouchPoints.set(event.pointerId, screenPosition);
      if (activeTouchPointerIds.size > 1) {
        cancelActiveLongPressGesture();
        activeControlGesture = resolveControlGestureCandidate();
        if (activeControlGesture !== null) {
          cancelActiveTouchGestureForPageZoom();
        } else if (runtime.isPageZoomEnabled()) {
          tryStartPageZoomGesture();
        }
        return;
      }
    }

    if (isTouchEvent && activeControlGesture?.pointerIds.includes(event.pointerId) === true) {
      if (type === UI_EVENT_POINTER_MOVE) {
        activeTouchPoints.set(event.pointerId, screenPosition);
        cancelActiveLongPressGesture(event.pointerId);
        updateControlGesture(event.pointerId);
        event.preventDefault();
        return;
      }
      if (type === UI_EVENT_POINTER_UP || type === UI_EVENT_POINTER_LEAVE || type === UI_EVENT_POINTER_CANCEL) {
        activeTouchPointerIds.delete(event.pointerId);
        activeTouchPoints.delete(event.pointerId);
        cancelActiveLongPressGesture(event.pointerId);
        finishControlGesture(event.pointerId, type === UI_EVENT_POINTER_CANCEL || type === UI_EVENT_POINTER_LEAVE);
        if (activePageZoomGesture?.pointerIds.includes(event.pointerId) === true) {
          activePageZoomGesture = null;
        }
        releaseCanvasPointerCapture(event.pointerId);
        event.preventDefault();
        return;
      }
    }

    if (isTouchEvent && activePageZoomGesture?.pointerIds.includes(event.pointerId) === true) {
      if (type === UI_EVENT_POINTER_MOVE) {
        activeTouchPoints.set(event.pointerId, screenPosition);
        cancelActiveLongPressGesture(event.pointerId);
        updatePageZoomGesture();
        return;
      }
      if (type === UI_EVENT_POINTER_UP || type === UI_EVENT_POINTER_LEAVE || type === UI_EVENT_POINTER_CANCEL) {
        activeTouchPointerIds.delete(event.pointerId);
        activeTouchPoints.delete(event.pointerId);
        cancelActiveLongPressGesture(event.pointerId);
        activePageZoomGesture = null;
        releaseCanvasPointerCapture(event.pointerId);
        return;
      }
    }

    if (type === UI_EVENT_POINTER_DOWN) {
      captureCanvasPointer(event.pointerId);
      primaryPointerDown = true;
      activePrimaryPointerId = event.pointerId;
      activePrimaryPointerType = event.pointerType;
      if (isTouchEvent) {
        ui._ui_clear_momentum_scroll();
        runtime.clearPageZoomPanMomentum();
        touchGesturePanningPageZoom = false;

        activeTouchGesture = transitionTouchGesture(null, {
          type: 'press-start',
          pointerId: event.pointerId,
          x: position.x,
          y: position.y,
          screenX: screenPosition.x,
          screenY: screenPosition.y,
          startedOnTextbox: false,
          pendingTextHandle: null,
        });
        touchGestureBreakoutTravel = { x: 0.0, y: 0.0 };
      }
    } else if (activeTouchGesture !== null && activeTouchGesture.pointerId === event.pointerId) {
      if (type === UI_EVENT_POINTER_MOVE) {
        cancelLongPressIfMovedPastTolerance(event.pointerId, screenPosition);
      }
      if (type === UI_EVENT_POINTER_MOVE && handleTouchPointerScroll(event, screenPosition, position, modifiers)) {
        return;
      }
      if (type === UI_EVENT_POINTER_UP || type === UI_EVENT_POINTER_LEAVE || type === UI_EVENT_POINTER_CANCEL) {
        const pendingTapTextHandle = activeTouchGesture.pendingTapTextHandle;
        const wasCancelled = isPointerCancel || type === UI_EVENT_POINTER_LEAVE || type === UI_EVENT_POINTER_CANCEL;
        const scrolling = activeTouchGesture.phase === 'scrolling';
        const pullToRefreshDistance = activeTouchGesture.pullToRefreshDistance;

        activeTouchGesture = transitionTouchGesture(activeTouchGesture, {
          type: wasCancelled ? 'cancel' : 'ended',
          triggered: false,
        });
        cancelActiveLongPressGesture(event.pointerId);

        const triggerRefresh = scrolling && pullToRefreshDistance >= PULL_TO_REFRESH_THRESHOLD;
        if (scrolling) {
          interactionState.cancelTouchTextFocusDeferral();
          interactionState.setCapturedPointerHandle(null);
          primaryPointerDown = false;
          activePrimaryPointerId = null;
          activePrimaryPointerType = null;
          pointerMoveCoalescer.clear();
          ui._ui_touch_scroll_end(event.timeStamp);
          if (touchGesturePanningPageZoom) {
            runtime.endPageZoomPan(event.timeStamp);
            touchGesturePanningPageZoom = false;
          }
          if (triggerRefresh) {
            pullToRefresh.hide(true);
            window.location.reload();
          } else {
            pullToRefresh.hide();
          }
          releaseCanvasPointerCapture(event.pointerId);
          activeTouchPointerIds.delete(event.pointerId);
          activeTouchPoints.delete(event.pointerId);
          event.preventDefault();
          return;
        }
        if (wasCancelled) {
          interactionState.cancelTouchTextFocusDeferral();
          touchTapDiscarded = true;
        } else {
          touchTapCandidateHandle = pendingTapTextHandle;
        }
        pullToRefresh.hide(true);
        activeTouchPointerIds.delete(event.pointerId);
        activeTouchPoints.delete(event.pointerId);
      }
    }
    const capturedHandle = interactionState.getCapturedPointerHandle();
    const activeTextHandle = interactionState.getActiveTextHandle();
    const rawHitHandle = useHitTest ? runtime.getHandleFromPoint(screenPosition.x, screenPosition.y) : 0n;
    const editorTextHandle =
      (type === UI_EVENT_POINTER_DOWN || activeTextHandle !== null) && useHitTest
        ? findEditorTextHandleAtPoint(runtime, position.x, position.y)
        : 0n;
    const rawHitPreservesSelection =
      rawHitHandle !== 0n && runtime.ui._ui_preserves_selection_on_pointer_down?.(rawHitHandle) === 1;
    const shouldPreferEditorText =
      editorTextHandle !== 0n &&
      rawHitHandle !== 0n &&
      !rawHitPreservesSelection &&
      !isEditorTextHandle(runtime, rawHitHandle) &&
      (type === UI_EVENT_POINTER_DOWN ||
        editorTextHandle === activeTextHandle ||
        editorTextHandle === capturedHandle);
    const hitHandle = shouldPreferEditorText ? editorTextHandle : rawHitHandle;
    const handle = type === UI_EVENT_POINTER_DOWN
      ? hitHandle
      : ((useHitTest && pointerInsideCanvas) ? hitHandle : (capturedHandle ?? hitHandle));
    const refocusActiveTextInputAfterPointerDown =
      type === UI_EVENT_POINTER_DOWN &&
      activeTextHandle !== null &&
      handle === activeTextHandle &&
      !isTouchEvent;
    const delayCanvasFocusUntilAfterPointerDown =
      type === UI_EVENT_POINTER_DOWN &&
      activeTextHandle !== null &&
      handle !== activeTextHandle;
    const keepTouchEditorFocusedOnPointerDown =
      type === UI_EVENT_POINTER_DOWN &&
      isTouchEvent &&
      activeTextHandle !== null &&
      handle === activeTextHandle &&
      interactionState.isActiveTextInputFocused();
    const refocusActiveTextInputAfterPointerUp =
      type === UI_EVENT_POINTER_UP &&
      activeTextHandle !== null &&
      handle === activeTextHandle &&
      !isTouchEvent;
    const shouldCommitDeferredTouchFocus =
      isTouchEvent &&
      type === UI_EVENT_POINTER_UP &&
      !touchTapDiscarded &&
      touchTapCandidateHandle !== null &&
      handle === touchTapCandidateHandle;
    if (type === UI_EVENT_POINTER_UP || type === UI_EVENT_POINTER_LEAVE || type === UI_EVENT_POINTER_CANCEL) {
      const pending = pointerMoveCoalescer.takePending();
      if (pending !== null) {
        processPointerMove(pending);
      }
    }
    if (type === UI_EVENT_POINTER_DOWN) {
      if (isTouchEvent) {
        const touchDownTextboxHandle = isEditorTextHandle(runtime, handle) ? handle : 0n;
        if (touchDownTextboxHandle !== 0n) {
          const touchDownOnAlreadyFocusedText =
            activeTextHandle !== null &&
            touchDownTextboxHandle === activeTextHandle &&
            interactionState.isActiveTextInputFocused();
          if (activeTouchGesture !== null) {
            activeTouchGesture.startedOnTextbox = true;
            activeTouchGesture.pendingTapTextHandle = touchDownOnAlreadyFocusedText ? null : touchDownTextboxHandle;
          }
          interactionState.beginTouchTextFocusDeferral(touchDownTextboxHandle);
        } else {
          interactionState.cancelTouchTextFocusDeferral();
        }
      }
      interactionState.setPointerInsideCanvas(pointerInsideCanvas);
      interactionState.setLastPointerClientPosition(event.clientX, event.clientY);
      interactionState.setLastPointerPosition(position.x, position.y);
      interactionState.setLastPointerModifiers(modifiers);
      if (!refocusActiveTextInputAfterPointerDown &&
        !delayCanvasFocusUntilAfterPointerDown &&
        !keepTouchEditorFocusedOnPointerDown) {
        canvas.focus({ preventScroll: true });
      }
      interactionState.setCapturedPointerHandle(handle === 0n ? null : handle);
      ui._ui_set_interaction_time(currentInteractionTimeMs());
      clickCount = resolveClickCount(handle, position.x, position.y);
      const handled = dispatchPointerEvent(
        type,
        handle,
        position.x,
        position.y,
        modifiers,
        pointerId,
        pointerType,
        button,
        buttons,
        pressure,
        pointerWidth,
        pointerHeight,
        clickCount,
      );
      if (isTouchEvent) {
        tryScheduleLongPressGesture(handle, pointerId, pointerType, screenPosition, modifiers);
      }
      runtime.commitFrame();
      if (event.button === 2) {
        const canShowContextMenu = window.__effindomCallbacks?.canShowContextMenu?.(handle) !== false;
        if (!handled && canShowContextMenu) {
          window.__effindomCallbacks?.onBeforeContextMenuHitTest?.();
          window.__effindomCallbacks?.onContextMenu?.(
            handle,
            position.x,
            position.y,
          );
          runtime.commitFrame();
        }
        event.preventDefault();
      }
      if (delayCanvasFocusUntilAfterPointerDown && interactionState.getActiveTextHandle() === null) {
        canvas.focus({ preventScroll: true });
      }
      if (refocusActiveTextInputAfterPointerDown) {
        interactionState.refocusActiveTextInput();
      }
      scheduleEdgeAutoScrollTick();
    } else if (type === UI_EVENT_POINTER_MOVE) {
      if (isTouchEvent) {
        cancelLongPressIfMovedPastTolerance(event.pointerId, screenPosition);
      }
      pointerMoveCoalescer.enqueue({
        handle,
        x: position.x,
        y: position.y,
        clientX: event.clientX,
        clientY: event.clientY,
        pointerInsideCanvas,
        modifiers,
        pointerId,
        pointerType,
        button,
        buttons,
        pressure,
        width: pointerWidth,
        height: pointerHeight,
      });
      return;
    } else {
      interactionState.setPointerInsideCanvas(pointerInsideCanvas);
      interactionState.setLastPointerClientPosition(event.clientX, event.clientY);
      interactionState.setLastPointerPosition(position.x, position.y);
      interactionState.setLastPointerModifiers(modifiers);
      ui._ui_set_interaction_time(currentInteractionTimeMs());
      dispatchPointerEvent(
        type,
        handle,
        position.x,
        position.y,
        modifiers,
        pointerId,
        pointerType,
        button,
        buttons,
        pressure,
        pointerWidth,
        pointerHeight,
        0,
      );
      runtime.commitFrame();
      if (shouldCommitDeferredTouchFocus) {
        interactionState.commitTouchTextFocusDeferral(handle);
      } else if (refocusActiveTextInputAfterPointerUp) {
        interactionState.refocusActiveTextInput();
      }
      if (isTouchEvent && type === UI_EVENT_POINTER_UP && !shouldCommitDeferredTouchFocus) {
        interactionState.cancelTouchTextFocusDeferral();
      }
      if (handle === 0n) {
        const appCapturedHandle = runtime.getCapturedPointerHandle();
        if (appCapturedHandle !== null) {
          window.__effindomCallbacks?.onPointerEventWithMetadata?.(
            type,
            appCapturedHandle,
            position.x,
            position.y,
            modifiers,
            pointerId,
            pointerType,
            button,
            buttons,
          pressure,
          pointerWidth,
          pointerHeight,
          clickCount,
        );
      }
      }
      scheduleEdgeAutoScrollTick();
    }
    if (type === UI_EVENT_POINTER_UP || type === UI_EVENT_POINTER_LEAVE || type === UI_EVENT_POINTER_CANCEL) {
      primaryPointerDown = false;
      activePrimaryPointerId = null;
      activePrimaryPointerType = null;
      interactionState.setCapturedPointerHandle(null);
    }
    if (type === UI_EVENT_POINTER_UP || type === UI_EVENT_POINTER_LEAVE || type === UI_EVENT_POINTER_CANCEL) {
      releaseCanvasPointerCapture(event.pointerId);
    }
    if (isTouchEvent && (type === UI_EVENT_POINTER_UP || type === UI_EVENT_POINTER_LEAVE || type === UI_EVENT_POINTER_CANCEL)) {
      activeTouchPointerIds.delete(event.pointerId);
      activeTouchPoints.delete(event.pointerId);
      cancelActiveLongPressGesture(event.pointerId);
    }
  };

  const handleContextMenu = (event: Event): void => {
    if (activeLongPressGesture !== null) {
      fireActiveLongPressGesture();
    }
    event.preventDefault();
  };
  const handlePointerDown = forwardPointerEvent(UI_EVENT_POINTER_DOWN);
  const handlePointerUp = forwardPointerEvent(UI_EVENT_POINTER_UP);
  const handlePointerMove = forwardPointerEvent(UI_EVENT_POINTER_MOVE);
  const handleCapturedPointerExit = (event: PointerEvent): void => {
    if (canvas.hasPointerCapture(event.pointerId)) {
      if (primaryPointerDown) {
        forwardPointerEvent(UI_EVENT_POINTER_MOVE, false)(event);
      }
      return;
    }
    forwardPointerEvent(UI_EVENT_POINTER_LEAVE, false)(event);
  };
  const handlePointerLeave = (event: PointerEvent): void => {
    handleCapturedPointerExit(event);
  };
  const handlePointerOut = (event: PointerEvent): void => {
    handleCapturedPointerExit(event);
  };
  const handlePointerCancel = (event: PointerEvent): void => {
    forwardPointerEvent(UI_EVENT_POINTER_CANCEL, false)(event);
  };
  const shouldHandleWindowPointerEvent = (event: PointerEvent): boolean => {
    return primaryPointerDown &&
      activePrimaryPointerId !== null &&
      event.pointerId === activePrimaryPointerId &&
      event.target !== canvas;
  };
  const handleWindowPointerMove = (event: PointerEvent): void => {
    if (!shouldHandleWindowPointerEvent(event)) {
      return;
    }
    handlePointerMove(event);
  };
  const handleWindowPointerUp = (event: PointerEvent): void => {
    if (!shouldHandleWindowPointerEvent(event)) {
      return;
    }
    handlePointerUp(event);
  };
  const handleWindowPointerCancel = (event: PointerEvent): void => {
    if (!shouldHandleWindowPointerEvent(event)) {
      return;
    }
    handlePointerCancel(event);
  };
  const handleWheel = (event: WheelEvent): void => {
    event.preventDefault();
    runtime.clearPageZoomPanMomentum();
    const position = getPointerPosition(canvas, event);
    interactionState.setPointerInsideCanvas(isPointerInsideCanvas(canvas, event));
    interactionState.setLastPointerClientPosition(event.clientX, event.clientY);
    interactionState.setLastPointerPosition(position.x, position.y);
    interactionState.setLastPointerModifiers(computeModifiers(event));
    ui._ui_set_interaction_time(currentInteractionTimeMs());
    const handle = runtime.getHandleFromPoint(position.x, position.y);
    if (event.ctrlKey && runtime.isPageZoomEnabled()) {
      const multiplier = Math.exp(-event.deltaY * TRACKPAD_PINCH_WHEEL_SCALE);
      if (dispatchTrackpadPinchGesture(handle, position, event.deltaY, multiplier)) {
        commitIfVisualWork(runtime);
        return;
      }
      const sceneAnchor = runtime.screenToScenePoint(position.x, position.y);
      runtime.setPageZoomFromSceneAnchor(
        runtime.getPageZoom().scale * multiplier,
        sceneAnchor.x,
        sceneAnchor.y,
        position.x,
        position.y,
      );
      return;
    }
    ui._ui_on_pointer_event(
      UI_EVENT_POINTER_MOVE,
      handle,
      position.x,
      position.y,
      -1,
      POINTER_TYPE_MOUSE,
      0,
      1,
      0,
      0,
      0,
      0,
      computeModifiers(event),
    );
    const delta = normalizeWheelDelta(event, canvas);
    const handled = window.__effindomCallbacks?.onWheelEventWithCoords?.(
      handle,
      position.x,
      position.y,
      delta.x,
      delta.y,
      0,
      computeModifiers(event),
    ) === true;
    if (handled) {
      commitIfVisualWork(runtime);
      return;
    }
    if (ui._ui_wheel_scroll_can_consume(delta.x, delta.y) !== 0) {
      runtime.deferSemanticProjectionUntilScrollIdle();
      ui._ui_on_wheel_event(delta.x, delta.y);
      runtime.commitFrame();
      return;
    }
    runtime.panPageZoomBy(delta.x, delta.y);
  };

  canvas.addEventListener('contextmenu', handleContextMenu);
  canvas.addEventListener('pointerdown', handlePointerDown, { passive: false });
  canvas.addEventListener('pointerup', handlePointerUp, { passive: false });
  canvas.addEventListener('pointermove', handlePointerMove, { passive: false });
  canvas.addEventListener('pointerleave', handlePointerLeave, { passive: false });
  canvas.addEventListener('pointerout', handlePointerOut, { passive: false });
  canvas.addEventListener('pointercancel', handlePointerCancel, { passive: false });
  window.addEventListener('pointermove', handleWindowPointerMove, { passive: false });
  window.addEventListener('pointerup', handleWindowPointerUp, { passive: false });
  window.addEventListener('pointercancel', handleWindowPointerCancel, { passive: false });
  canvas.addEventListener('wheel', handleWheel, { passive: false });

  return () => {
    canvas.removeEventListener('contextmenu', handleContextMenu);
    canvas.removeEventListener('pointerdown', handlePointerDown);
    canvas.removeEventListener('pointerup', handlePointerUp);
    canvas.removeEventListener('pointermove', handlePointerMove);
    canvas.removeEventListener('pointerleave', handlePointerLeave);
    canvas.removeEventListener('pointerout', handlePointerOut);
    canvas.removeEventListener('pointercancel', handlePointerCancel);
    window.removeEventListener('pointermove', handleWindowPointerMove);
    window.removeEventListener('pointerup', handleWindowPointerUp);
    window.removeEventListener('pointercancel', handleWindowPointerCancel);
    canvas.removeEventListener('wheel', handleWheel);
    cancelActiveLongPressGesture();
  };
}
