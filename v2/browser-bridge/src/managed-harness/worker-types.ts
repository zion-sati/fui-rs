export interface WorkerHostServicesBundleConfig {
  readonly scriptUrl: string;
  readonly exportName: string;
}

export interface WorkerBootstrapStartMessage {
  readonly type: "start";
  readonly workerId: number;
  readonly wasmUrl: string;
  readonly entryName: string;
  readonly input: string;
  readonly workerHostServices?: WorkerHostServicesBundleConfig;
}

export interface WorkerBootstrapFileProcessStartMessage {
  readonly type: "start-file-process";
  readonly workerId: number;
  readonly file: File;
  readonly wasmUrl: string;
  readonly entryName: string;
  readonly chunkSize: number;
  readonly workerHostServices?: WorkerHostServicesBundleConfig;
}

export interface WorkerBootstrapCancelMessage {
  readonly type: "cancel";
  readonly workerId: number;
}

export type WorkerBootstrapInboundMessage =
  | WorkerBootstrapStartMessage
  | WorkerBootstrapFileProcessStartMessage
  | WorkerBootstrapCancelMessage;

export interface WorkerBootstrapProgressMessage {
  readonly type: "progress";
  readonly workerId: number;
  readonly text: string;
}

export interface WorkerBootstrapCompleteMessage {
  readonly type: "complete";
  readonly workerId: number;
  readonly text: string;
}

export interface WorkerBootstrapFileProcessChunkMessage {
  readonly type: "file-process-chunk";
  readonly workerId: number;
  readonly bytes: ArrayBuffer;
}

export interface WorkerBootstrapErrorMessage {
  readonly type: "error";
  readonly workerId: number;
  readonly text: string;
}

export type WorkerBootstrapOutboundMessage =
  | WorkerBootstrapProgressMessage
  | WorkerBootstrapCompleteMessage
  | WorkerBootstrapFileProcessChunkMessage
  | WorkerBootstrapErrorMessage;
