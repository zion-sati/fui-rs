/**
 * Touch gesture state machine.
 * Models the lifecycle of touch interactions from pointer-down through scroll/tap completion.
 */

export type TouchGesturePhase = 'pressed' | 'scrolling';

export interface TouchGestureState {
  pointerId: number;
  phase: TouchGesturePhase;
  startX: number;
  startY: number;
  lastX: number;
  lastY: number;
  startScreenX: number;
  startScreenY: number;
  lastScreenX: number;
  lastScreenY: number;
  startedOnTextbox: boolean;
  pendingTapTextHandle: bigint | null;
  axisMode: 'x' | 'y' | 'xy' | null;
  pullToRefreshEligible: boolean;
  pullToRefreshCaptured: boolean;
  pullToRefreshDistance: number;
  // Note: breakoutTravel is tracked separately outside the reducer to avoid churn;
  // see touchGestureBreakoutTravel in events.ts
}

export type TouchGestureEvent =
  | {
      type: 'press-start';
      pointerId: number;
      x: number;
      y: number;
      screenX: number;
      screenY: number;
      startedOnTextbox: boolean;
      pendingTextHandle: bigint | null;
    }
  | {
      type: 'move';
      x: number;
      y: number;
      screenX: number;
      screenY: number;
    }
  | {
      type: 'scroll-threshold-crossed';
      axis: 'x' | 'y';
    }
  | {
      type: 'axis-unlocked';
    }
  | {
      type: 'pull-to-refresh-captured';
    }
  | {
      type: 'pull-to-refresh-released';
    }
  | {
      type: 'cancel';
    }
  | {
      type: 'ended';
      triggered: boolean;
    };

/**
 * Transition reducer for touch gesture state.
 * Returns the new state after applying an event, or null if the gesture ended.
 */
export function transitionTouchGesture(
  state: TouchGestureState | null,
  event: TouchGestureEvent,
): TouchGestureState | null {
  if (state === null) {
    if (event.type === 'press-start') {
      return {
        pointerId: event.pointerId,
        phase: 'pressed',
        startX: event.x,
        startY: event.y,
        lastX: event.x,
        lastY: event.y,
        startScreenX: event.screenX,
        startScreenY: event.screenY,
        lastScreenX: event.screenX,
        lastScreenY: event.screenY,
        startedOnTextbox: event.startedOnTextbox,
        pendingTapTextHandle: event.pendingTextHandle,
        axisMode: null,
        pullToRefreshEligible: false,
        pullToRefreshCaptured: false,
        pullToRefreshDistance: 0.0,
      };
    }
    return null;
  }

  switch (state.phase) {
    case 'pressed': {
      switch (event.type) {
        case 'scroll-threshold-crossed': {
          const primaryAxis = event.axis;
          return {
            ...state,
            phase: 'scrolling',
            axisMode: primaryAxis,
            pullToRefreshEligible: primaryAxis === 'y',
            pendingTapTextHandle: null,
          };
        }
        case 'move': {
          return {
            ...state,
            lastX: event.x,
            lastY: event.y,
            lastScreenX: event.screenX,
            lastScreenY: event.screenY,
          };
        }
        case 'cancel':
        case 'ended': {
          return null;
        }
        default: {
          return state;
        }
      }
    }

    case 'scrolling': {
      switch (event.type) {
        case 'move': {
          return {
            ...state,
            lastX: event.x,
            lastY: event.y,
            lastScreenX: event.screenX,
            lastScreenY: event.screenY,
          };
        }
        case 'axis-unlocked': {
          return {
            ...state,
            axisMode: 'xy',
            pullToRefreshEligible: false,
            pullToRefreshCaptured: false,
            pullToRefreshDistance: 0.0,
          };
        }
        case 'pull-to-refresh-captured': {
          return {
            ...state,
            pullToRefreshCaptured: true,
          };
        }
        case 'pull-to-refresh-released': {
          return {
            ...state,
            pullToRefreshCaptured: false,
            pullToRefreshDistance: 0.0,
          };
        }
        case 'cancel':
        case 'ended': {
          return null;
        }
        default: {
          return state;
        }
      }
    }
  }
}
