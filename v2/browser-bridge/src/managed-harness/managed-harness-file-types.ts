import type { HarnessAppSession } from './managed-harness-session';

export const EXTERNAL_DRAG_EVENT_ENTER = 1;
export const EXTERNAL_DRAG_EVENT_OVER = 2;
export const EXTERNAL_DRAG_EVENT_LEAVE = 3;
export const EXTERNAL_DRAG_EVENT_DROP = 4;
export const EXTERNAL_DROP_ITEM_KIND_FILE = 1;
export const FILE_STATUS_SUCCESS = 1;
export const FILE_STATUS_CANCELLED = 2;
export const FILE_STATUS_ERROR = 3;
export const FILE_SAVE_MODE_DOWNLOAD = 1;
export const FILE_SAVE_MODE_NATIVE_PICKER = 2;
export const FILE_CAPABILITY_OPEN = 1 << 0;
export const FILE_CAPABILITY_READ = 1 << 1;
export const FILE_CAPABILITY_SAVE = 1 << 2;
export const FILE_CAPABILITY_CHUNKED_READ = 1 << 3;
export const FILE_CAPABILITY_CHUNKED_WRITE = 1 << 4;
export const FILE_CAPABILITY_NATIVE_SAVE_PICKER = 1 << 5;
export const FILE_CAPABILITY_PROCESS_WORKER_SAVE = 1 << 6;

export interface ExternalHarnessDropItem {
  readonly id: string;
  readonly kind: number;
  readonly name: string;
  readonly mimeType: string | null;
  readonly sizeBytes: number;
}

export interface WritableFileStreamLike {
  write(data: BufferSource | Blob | string): Promise<void>;
  close(): Promise<void>;
  abort?(reason?: unknown): Promise<void>;
}

export interface SaveFileHandleLike {
  readonly name?: string;
  createWritable(): Promise<WritableFileStreamLike>;
}

export interface SavePickerWindow extends Window {
  showSaveFilePicker?: (options?: {
    suggestedName?: string;
  }) => Promise<SaveFileHandleLike>;
}

export interface StoredFileRecord {
  readonly id: string;
  readonly file: File;
}

export interface ActiveFileWriterRecord {
  readonly id: string;
  readonly session: HarnessAppSession;
  readonly fileName: string;
  readonly mode: number;
  readonly stream: WritableFileStreamLike;
  writtenBytes: number;
}

export interface ActiveFileProcessingRecord {
  readonly requestId: number;
  readonly session: HarnessAppSession;
  readonly sourceFileName: string;
  readonly targetFileName: string | null;
  readonly totalBytes: number;
  readonly stream: WritableFileStreamLike | null;
  readonly saveToPickedFile: boolean;
  readonly worker: Worker;
  cancelled: boolean;
  failed: boolean;
  pendingWrites: Promise<void>[];
  processedBytes: number;
}
