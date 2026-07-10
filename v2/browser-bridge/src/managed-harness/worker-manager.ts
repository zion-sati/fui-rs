import type {
  WorkerBootstrapInboundMessage,
  WorkerBootstrapOutboundMessage,
  WorkerHostServicesBundleConfig,
} from './worker-types';

interface WorkerHarnessExports {
  __fui_on_worker_progress(workerId: number, textPtr: number, textLen: number): void;
  __fui_on_worker_complete(workerId: number, textPtr: number, textLen: number): void;
  __fui_on_worker_error(workerId: number, textPtr: number, textLen: number): void;
}

export interface WorkerHarnessSession {
  readonly exports: WorkerHarnessExports;
  readonly memory: WebAssembly.Memory;
  readonly textBufferPtr: number;
  readonly textBufferSize: number;
}

interface WorkerTextSession {
  readonly memory: WebAssembly.Memory;
  readonly textBufferPtr: number;
  readonly textBufferSize: number;
}

interface WorkerRecord {
  worker: Worker | null;
  cancelled: boolean;
}

export interface WorkerManager {
  startString(workerId: number, wasmPath: string, entryName: string, input: string): void;
  cancel(workerId: number): void;
  terminateAll(): void;
}

interface WorkerManagerOptions {
  readonly scriptBaseUrl: string;
  readonly getCurrentSession: () => WorkerHarnessSession | null;
  readonly getCurrentWorkerHostServices: () => WorkerHostServicesBundleConfig | undefined;
  readonly writeTextCallbackPayload: (session: WorkerTextSession, text: string, context: string) => number;
}

function describeError(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export function createWorkerManager(options: WorkerManagerOptions): WorkerManager {
  const workerBootstrapUrl = new URL('./worker-bootstrap.js', options.scriptBaseUrl).toString();
  const records = new Map<number, WorkerRecord>();

  function emitToSession(workerId: number, kind: 'progress' | 'complete' | 'error', text: string): void {
    const session = options.getCurrentSession();
    if (session === null) {
      return;
    }
    try {
      const textLength = options.writeTextCallbackPayload(session, text, `Worker ${kind} payload`);
      const textPtr = textLength > 0 ? session.textBufferPtr : 0;
      if (kind === 'progress') {
        session.exports.__fui_on_worker_progress(workerId, textPtr, textLength);
        return;
      }
      if (kind === 'complete') {
        session.exports.__fui_on_worker_complete(workerId, textPtr, textLength);
        return;
      }
      session.exports.__fui_on_worker_error(workerId, textPtr, textLength);
    } catch (error: unknown) {
      console.error(`[fui-worker] failed to deliver ${kind} payload for worker ${String(workerId)}: ${describeError(error)}`);
      if (kind !== 'error') {
        emitToSession(workerId, 'error', `Worker ${kind} delivery failed: ${describeError(error)}`);
      }
    }
  }

  function finishWorker(workerId: number): void {
    const record = records.get(workerId);
    if (record === undefined) {
      return;
    }
    record.worker?.terminate();
    records.delete(workerId);
  }

  function handleWorkerMessage(workerId: number, message: WorkerBootstrapOutboundMessage): void {
    const record = records.get(workerId);
    if (record === undefined) {
      return;
    }
    if (message.type === 'progress') {
      if (!record.cancelled) {
        emitToSession(workerId, 'progress', message.text);
      }
      return;
    }
    if (message.type === 'complete') {
      emitToSession(workerId, 'complete', message.text);
      finishWorker(workerId);
      return;
    }
    // File-process chunk messages are routed through the file host, not the worker manager.
    if (message.type === 'file-process-chunk') {
      return;
    }
    emitToSession(workerId, 'error', message.text);
    finishWorker(workerId);
  }

  function startWorker(workerId: number, wasmPath: string, entryName: string, input: string): void {
    const record = records.get(workerId);
    if (record === undefined) {
      return;
    }
    try {
      const workerModuleUrl = new URL(wasmPath, options.scriptBaseUrl).toString();
      const worker = new Worker(workerBootstrapUrl);
      if (records.get(workerId) !== record) {
        worker.terminate();
        return;
      }
      record.worker = worker;
      worker.addEventListener('message', (event: MessageEvent<WorkerBootstrapOutboundMessage>) => {
        handleWorkerMessage(workerId, event.data);
      });
      worker.addEventListener('error', (event: ErrorEvent) => {
        const active = records.get(workerId);
        if (active === undefined) {
          return;
        }
        const message = typeof event.message === 'string' && event.message.length > 0
          ? event.message
          : 'Worker bootstrap crashed.';
        emitToSession(workerId, 'error', message);
        finishWorker(workerId);
      });
      const workerHostServices = options.getCurrentWorkerHostServices();
      const message: WorkerBootstrapInboundMessage = workerHostServices === undefined
        ? {
          type: 'start',
          workerId,
          wasmUrl: workerModuleUrl,
          entryName,
          input,
        }
        : {
          type: 'start',
          workerId,
          wasmUrl: workerModuleUrl,
          entryName,
          input,
          workerHostServices,
        };
      worker.postMessage(message);
      if (record.cancelled) {
        worker.postMessage({
          type: 'cancel',
          workerId,
        });
      }
    } catch (error: unknown) {
      emitToSession(workerId, 'error', describeError(error));
      finishWorker(workerId);
    }
  }

  return {
    startString(workerId: number, wasmPath: string, entryName: string, input: string): void {
      if (records.has(workerId)) {
        emitToSession(workerId, 'error', 'Worker already started.');
        return;
      }
      records.set(workerId, {
        worker: null,
        cancelled: false,
      });
      startWorker(workerId, wasmPath, entryName, input);
    },
    cancel(workerId: number): void {
      const record = records.get(workerId);
      if (record === undefined) {
        return;
      }
      record.cancelled = true;
      if (record.worker !== null) {
        record.worker.postMessage({
          type: 'cancel',
          workerId,
        });
      }
    },
    terminateAll(): void {
      for (const record of records.values()) {
        record.worker?.terminate();
      }
      records.clear();
    },
  };
}
