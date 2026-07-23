import { expect, test } from '@playwright/test';

import {
  HarnessLoadingController,
  type LoadingControllerClock,
  type LoadingOverlayPresenter,
} from '../src/managed-harness/loading-controller';

class ManualClock implements LoadingControllerClock {
  private timeMs = 0;
  private nextTimerId = 1;
  private readonly timers = new Map<number, { dueMs: number; callback: () => void }>();

  now(): number {
    return this.timeMs;
  }

  setTimer(callback: () => void, delayMs: number): number {
    const timerId = this.nextTimerId;
    this.nextTimerId += 1;
    this.timers.set(timerId, { dueMs: this.timeMs + delayMs, callback });
    return timerId;
  }

  clearTimer(timerId: number): void {
    this.timers.delete(timerId);
  }

  delay(delayMs: number): Promise<void> {
    return new Promise<void>((resolve) => {
      this.setTimer(resolve, delayMs);
    });
  }

  advance(delayMs: number): void {
    const targetMs = this.timeMs + delayMs;
    for (;;) {
      const ready = [...this.timers.entries()]
        .filter(([, timer]) => timer.dueMs <= targetMs)
        .sort((left, right) => left[1].dueMs - right[1].dueMs);
      const next = ready.shift();
      if (next === undefined) {
        break;
      }
      const [timerId, timer] = next;
      this.timers.delete(timerId);
      this.timeMs = timer.dueMs;
      timer.callback();
    }
    this.timeMs = targetMs;
  }
}

class RecordingPresenter implements LoadingOverlayPresenter {
  readonly shown: { state: 'loading' | 'error'; title: string; detail: string }[] = [];
  hiddenCount = 0;

  showLoading(state: 'loading' | 'error', title: string, detail: string): void {
    this.shown.push({ state, title, detail });
  }

  hideLoading(): void {
    this.hiddenCount += 1;
  }
}

test('fast loading never reveals the delayed overlay', async () => {
  const clock = new ManualClock();
  const presenter = new RecordingPresenter();
  const controller = new HarnessLoadingController(presenter, clock, 'Loading', 'Starting', {
    delayMs: 300,
    minimumVisibleMs: 300,
  });

  await controller.complete(controller.begin('Loading runtime'));

  expect(presenter.shown).toEqual([]);
  expect(presenter.hiddenCount).toBe(1);
  clock.advance(1000);
  expect(presenter.shown).toEqual([]);
});

test('initial loading delay is measured from page bootstrap rather than harness startup', () => {
  const clock = new ManualClock();
  const presenter = new RecordingPresenter();
  const controller = new HarnessLoadingController(presenter, clock, 'Loading', 'Starting', {
    delayMs: 300,
    minimumVisibleMs: 300,
  });

  clock.advance(250);
  controller.begin('Loading runtime', 0);
  clock.advance(49);
  expect(presenter.shown).toEqual([]);
  clock.advance(1);
  expect(presenter.shown).toHaveLength(1);
});

test('controller adopts an overlay already revealed by static bootstrap', () => {
  const clock = new ManualClock();
  clock.advance(500);
  const presenter = new RecordingPresenter();
  const controller = new HarnessLoadingController(
    presenter,
    clock,
    'Loading',
    'Starting',
    { delayMs: 300, minimumVisibleMs: 300 },
    300,
  );

  controller.begin('Loading runtime', 0);
  expect(presenter.shown).toEqual([{ state: 'loading', title: 'Loading', detail: 'Loading runtime' }]);
});

test('slow loading reveals the overlay and honors its minimum visible duration', async () => {
  const clock = new ManualClock();
  const presenter = new RecordingPresenter();
  const controller = new HarnessLoadingController(presenter, clock, 'Loading', 'Starting', {
    delayMs: 300,
    minimumVisibleMs: 300,
  });
  const operation = controller.begin('Loading application');

  clock.advance(300);
  expect(presenter.shown[presenter.shown.length - 1]).toEqual({
    state: 'loading',
    title: 'Loading',
    detail: 'Loading application',
  });
  const completion = controller.complete(operation);
  clock.advance(299);
  expect(presenter.hiddenCount).toBe(0);
  clock.advance(1);
  await completion;
  expect(presenter.hiddenCount).toBe(1);
});

test('debug delay deterministically exposes the loader on fast local builds', async () => {
  const clock = new ManualClock();
  const presenter = new RecordingPresenter();
  const controller = new HarnessLoadingController(presenter, clock, 'Loading', 'Starting', {
    delayMs: 10,
    minimumVisibleMs: 0,
    debugDelayMs: 100,
  });
  const completion = controller.complete(controller.begin('Debug loading'));

  clock.advance(10);
  expect(presenter.shown).toHaveLength(1);
  clock.advance(90);
  await completion;
  expect(presenter.hiddenCount).toBe(1);
});

test('completion from an obsolete load cannot hide a newer load', async () => {
  const clock = new ManualClock();
  const presenter = new RecordingPresenter();
  const controller = new HarnessLoadingController(presenter, clock, 'Loading', 'Starting', {
    delayMs: 10,
    minimumVisibleMs: 100,
  });
  const first = controller.begin('First route');
  clock.advance(10);
  const firstCompletion = controller.complete(first);
  const second = controller.begin('Second route');
  clock.advance(100);
  await firstCompletion;
  expect(presenter.hiddenCount).toBe(0);

  await controller.complete(second);
  expect(presenter.hiddenCount).toBe(1);
});

test('loading failures are displayed immediately', () => {
  const clock = new ManualClock();
  const presenter = new RecordingPresenter();
  const controller = new HarnessLoadingController(presenter, clock, 'Loading', 'Starting');

  controller.begin('Loading runtime');
  controller.fail('Runtime unavailable');

  expect(presenter.shown[presenter.shown.length - 1]).toEqual({
    state: 'error',
    title: 'Loading',
    detail: 'Runtime unavailable',
  });
});
