import type { BridgeRuntime } from '../core-types';

export function commitIfVisualWork(runtime: BridgeRuntime): boolean {
  if (!runtime.uiHasPendingVisualWork()) {
    return false;
  }
  runtime.commitFrame();
  return true;
}

