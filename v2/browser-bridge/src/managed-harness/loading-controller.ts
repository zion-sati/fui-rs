export interface LoadingIndicatorOptions {
  readonly delayMs?: number;
  readonly minimumVisibleMs?: number;
  readonly debugDelayMs?: number;
}

export interface LoadingOverlayPresenter {
  showLoading(state: 'loading' | 'error', title: string, detail: string): void;
  hideLoading(): void;
}

export interface LoadingControllerClock {
  now(): number;
  setTimer(callback: () => void, delayMs: number): number;
  clearTimer(timerId: number): void;
  delay(delayMs: number): Promise<void>;
}

export interface LoadingOperation {
  readonly generation: number;
}

interface ActiveLoadingOperation extends LoadingOperation {
  readonly startedAtMs: number;
}

const DEFAULT_DELAY_MS = 300;
const DEFAULT_MINIMUM_VISIBLE_MS = 300;

function normalizeDuration(value: number | undefined, fallback: number): number {
  return value === undefined || !Number.isFinite(value) ? fallback : Math.max(0, value);
}

export function createBrowserLoadingClock(): LoadingControllerClock {
  return {
    now: () => performance.now(),
    setTimer: (callback, delayMs) => window.setTimeout(callback, delayMs),
    clearTimer: (timerId) => {
      window.clearTimeout(timerId);
    },
    delay: (delayMs) => new Promise<void>((resolve) => window.setTimeout(resolve, delayMs)),
  };
}

export class HarnessLoadingController {
  private readonly delayMs: number;
  private readonly minimumVisibleMs: number;
  private readonly debugDelayMs: number;
  private generation = 0;
  private active: ActiveLoadingOperation | null = null;
  private revealTimer: number | null = null;
  private visibleAtMs: number | null = null;
  private title: string;
  private detail: string;

  constructor(
    private readonly presenter: LoadingOverlayPresenter,
    private readonly clock: LoadingControllerClock,
    title: string,
    detail: string,
    options: LoadingIndicatorOptions = {},
    initialVisibleAtMs: number | null = null,
  ) {
    this.title = title;
    this.detail = detail;
    this.delayMs = normalizeDuration(options.delayMs, DEFAULT_DELAY_MS);
    this.minimumVisibleMs = normalizeDuration(options.minimumVisibleMs, DEFAULT_MINIMUM_VISIBLE_MS);
    this.debugDelayMs = normalizeDuration(options.debugDelayMs, 0);
    this.visibleAtMs = initialVisibleAtMs;
  }

  begin(detail: string, startedAtMs = this.clock.now()): LoadingOperation {
    this.cancelRevealTimer();
    this.generation += 1;
    this.detail = detail;
    const operation: ActiveLoadingOperation = {
      generation: this.generation,
      startedAtMs,
    };
    this.active = operation;
    if (this.visibleAtMs === null) {
      const revealDelayMs = Math.max(0, this.delayMs - (this.clock.now() - startedAtMs));
      this.revealTimer = this.clock.setTimer(() => {
        if (!this.isCurrent(operation)) {
          return;
        }
        this.revealTimer = null;
        this.visibleAtMs = this.clock.now();
        this.presenter.showLoading('loading', this.title, this.detail);
      }, revealDelayMs);
    } else {
      this.presenter.showLoading('loading', this.title, this.detail);
    }
    return operation;
  }

  update(operation: LoadingOperation, detail: string): void {
    if (!this.isCurrent(operation)) {
      return;
    }
    this.detail = detail;
    if (this.visibleAtMs !== null) {
      this.presenter.showLoading('loading', this.title, this.detail);
    }
  }

  async complete(operation: LoadingOperation): Promise<void> {
    if (!this.isCurrent(operation)) {
      return;
    }
    const active = this.active;
    if (active === null) {
      return;
    }
    const debugRemainingMs = this.debugDelayMs - (this.clock.now() - active.startedAtMs);
    if (debugRemainingMs > 0) {
      await this.clock.delay(debugRemainingMs);
    }
    if (!this.isCurrent(operation)) {
      return;
    }
    if (this.visibleAtMs === null) {
      this.cancelRevealTimer();
      this.active = null;
      this.presenter.hideLoading();
      return;
    }
    const visibleRemainingMs = this.minimumVisibleMs - (this.clock.now() - this.visibleAtMs);
    if (visibleRemainingMs > 0) {
      await this.clock.delay(visibleRemainingMs);
    }
    if (!this.isCurrent(operation)) {
      return;
    }
    this.cancelRevealTimer();
    this.active = null;
    this.visibleAtMs = null;
    this.presenter.hideLoading();
  }

  fail(detail: string): void {
    this.cancelRevealTimer();
    this.generation += 1;
    this.active = null;
    this.visibleAtMs = this.clock.now();
    this.detail = detail;
    this.presenter.showLoading('error', this.title, detail);
  }

  private isCurrent(operation: LoadingOperation): boolean {
    return this.active?.generation === operation.generation;
  }

  private cancelRevealTimer(): void {
    if (this.revealTimer === null) {
      return;
    }
    this.clock.clearTimer(this.revealTimer);
    this.revealTimer = null;
  }
}
