/// <reference lib="webworker" />

interface FileProcessingWorkerStartMessage {
  readonly type: 'start';
  readonly file: File;
  readonly chunkSize: number;
}

interface FileProcessingWorkerNextMessage {
  readonly type: 'next';
}

interface FileProcessingWorkerCancelMessage {
  readonly type: 'cancel';
}

type FileProcessingWorkerInboundMessage =
  | FileProcessingWorkerStartMessage
  | FileProcessingWorkerNextMessage
  | FileProcessingWorkerCancelMessage;

interface FileProcessingWorkerChunkMessage {
  readonly type: 'chunk';
  readonly offsetBytes: number;
  readonly bytes: ArrayBuffer;
  readonly copiedBytes: number;
  readonly totalBytes: number;
}

interface FileProcessingWorkerErrorMessage {
  readonly type: 'error';
  readonly message: string;
}

let activeFile: File | null = null;
let activeChunkSize = 0;
let activeOffsetBytes = 0;
const workerScope = self as DedicatedWorkerGlobalScope;

function describeError(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function postNextChunk(): void {
  const file = activeFile;
  if (file === null) {
    return;
  }
  try {
    const offsetBytes = activeOffsetBytes;
    const nextOffset = Math.min(file.size, activeOffsetBytes + activeChunkSize);
    const reader = new FileReaderSync();
    const bytes = reader.readAsArrayBuffer(file.slice(activeOffsetBytes, nextOffset));
    activeOffsetBytes = nextOffset;
    const message: FileProcessingWorkerChunkMessage = {
      type: 'chunk',
      offsetBytes,
      bytes,
      copiedBytes: activeOffsetBytes,
      totalBytes: file.size,
    };
    workerScope.postMessage(message, [bytes]);
  } catch (error: unknown) {
    const failure: FileProcessingWorkerErrorMessage = {
      type: 'error',
      message: describeError(error),
    };
    workerScope.postMessage(failure);
  }
}

workerScope.onmessage = (event: MessageEvent<FileProcessingWorkerInboundMessage>) => {
  const message = event.data;
  if (message.type === 'cancel') {
    activeFile = null;
    activeOffsetBytes = 0;
    return;
  }
  if (message.type === 'start') {
    activeFile = message.file;
    activeChunkSize = Math.max(1, Math.floor(message.chunkSize));
    activeOffsetBytes = 0;
    postNextChunk();
    return;
  }
  if (activeFile !== null) {
    postNextChunk();
  }
};
