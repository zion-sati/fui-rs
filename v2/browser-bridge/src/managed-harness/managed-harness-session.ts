import type { WorkerHostServicesBundleConfig } from './worker-types';

import type { HarnessExports, HarnessState } from './types';

export interface HarnessAppSession {
  readonly exports: HarnessExports;
  readonly memory: WebAssembly.Memory;
  readonly keyBufferPtr: number;
  readonly textBufferPtr: number;
  readonly textBufferSize: number;
  readonly hostEventDisposers: (() => void)[];
  readonly workerHostServices?: WorkerHostServicesBundleConfig;
  readonly onStateUpdated?: (state: HarnessState) => void;
  readonly onDispose?: (exports: HarnessExports) => void;
}
