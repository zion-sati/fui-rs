import type { BridgeRuntime } from '@effindomv2/runtime';
import { computeModifiers,getPointerPosition } from '@effindomv2/runtime';

import type { WorkerHostServicesBundleConfig } from './worker-types';
import { writeExternalDropPayload,writeFileListPayload,writeWriterPayload } from './managed-harness-file-payloads';
import {
EXTERNAL_DRAG_EVENT_DROP,
EXTERNAL_DRAG_EVENT_LEAVE,
EXTERNAL_DROP_ITEM_KIND_FILE,
FILE_CAPABILITY_CHUNKED_READ,
FILE_CAPABILITY_CHUNKED_WRITE,
FILE_CAPABILITY_NATIVE_SAVE_PICKER,
FILE_CAPABILITY_OPEN,
FILE_CAPABILITY_PROCESS_WORKER_SAVE,
FILE_CAPABILITY_READ,
FILE_CAPABILITY_SAVE,
FILE_SAVE_MODE_DOWNLOAD,
FILE_SAVE_MODE_NATIVE_PICKER,
FILE_STATUS_CANCELLED,
FILE_STATUS_ERROR,
FILE_STATUS_SUCCESS,
type ActiveFileProcessingRecord,
type ActiveFileWriterRecord,
type ExternalHarnessDropItem,
type SavePickerWindow,
type StoredFileRecord,
type WritableFileStreamLike
} from './managed-harness-file-types';
import type { HarnessAppSession } from './managed-harness-session';

interface ManagedHarnessFileHostDependencies {
  getCurrentSession(): HarnessAppSession | null;
  getRuntime(): BridgeRuntime;
  readAppUtf8(ptr: number, len: number): string;
  readAppBytes(ptr: number, len: number): Uint8Array;
  writeTextCallbackPayload(session: HarnessAppSession, text: string, context: string): number;
  describeHarnessError(error: unknown): string;
  workerBootstrapUrl: string;
  getCurrentWorkerHostServices(): WorkerHostServicesBundleConfig | undefined;
}

type FileProcessWorkerMessage =
  | { readonly type: 'file-process-chunk'; readonly bytes: Uint8Array | ArrayBuffer }
  | { readonly type: 'progress'; readonly text: string }
  | { readonly type: 'complete'; readonly text: string }
  | { readonly type: 'error'; readonly text: string };

const encoder = new TextEncoder();

function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === 'object';
}

function parseFileProcessWorkerMessage(value: unknown): FileProcessWorkerMessage | null {
  if (!isRecord(value) || typeof value.type !== 'string') {
    return null;
  }
  if (
    value.type === 'file-process-chunk' &&
    (value.bytes instanceof Uint8Array || value.bytes instanceof ArrayBuffer)
  ) {
    return { type: value.type, bytes: value.bytes };
  }
  if (
    (value.type === 'progress' || value.type === 'complete' || value.type === 'error') &&
    typeof value.text === 'string'
  ) {
    return { type: value.type, text: value.text };
  }
  return null;
}

function copyBytesToArrayBuffer(bytes: Uint8Array): ArrayBuffer {
  const copied = new Uint8Array(bytes.byteLength);
  copied.set(bytes);
  return copied.buffer;
}

function supportsNativeSavePicker(): boolean {
  return typeof (window as SavePickerWindow).showSaveFilePicker === 'function';
}

function getFileCapabilities(): number {
  let flags = FILE_CAPABILITY_OPEN | FILE_CAPABILITY_READ | FILE_CAPABILITY_SAVE | FILE_CAPABILITY_CHUNKED_READ;
  if (supportsNativeSavePicker()) {
    flags |= FILE_CAPABILITY_CHUNKED_WRITE | FILE_CAPABILITY_NATIVE_SAVE_PICKER;
    flags |= FILE_CAPABILITY_PROCESS_WORKER_SAVE;
  }
  return flags;
}

function resolveSuggestedName(suggestedName: string, fileExtension: string): string {
  const trimmedName = suggestedName.trim();
  const trimmedExtension = fileExtension.trim();
  if (trimmedName.length === 0) {
    if (trimmedExtension.length > 0) {
      return `export${trimmedExtension.startsWith('.') ? trimmedExtension : `.${trimmedExtension}`}`;
    }
    return 'export.bin';
  }
  if (trimmedExtension.length === 0) {
    return trimmedName;
  }
  const normalizedExtension = trimmedExtension.startsWith('.') ? trimmedExtension : `.${trimmedExtension}`;
  return trimmedName.endsWith(normalizedExtension) ? trimmedName : `${trimmedName}${normalizedExtension}`;
}

async function abortWritableStream(stream: WritableFileStreamLike): Promise<void> {
  if (typeof stream.abort === 'function') {
    await stream.abort();
    return;
  }
  await stream.close();
}

export function createManagedHarnessFileHost(dependencies: ManagedHarnessFileHostDependencies) {
  let nextStoredBrowserFileId = 1;
  let nextFileWriterId = 1;
  let nextExternalDropItemId = 1;
  const storedBrowserFiles = new Map<string, File>();
  const activeFileWriters = new Map<string, ActiveFileWriterRecord>();
  const activeFileProcessingRequests = new Map<number, ActiveFileProcessingRecord>();
  const cancelledFileProcessingRequestIds = new Set<number>();
  let activeExternalDropItems: ExternalHarnessDropItem[] = [];

  function emitFilePickResult(
    session: HarnessAppSession | null,
    requestId: number,
    status: number,
    files: readonly StoredFileRecord[] = [],
    message = '',
  ): void {
    if (session === null) {
      return;
    }
    const payloadLength = status === FILE_STATUS_SUCCESS
      ? writeFileListPayload(session, files)
      : dependencies.writeTextCallbackPayload(session, message, 'File picker result');
    session.exports.__fui_on_file_pick_result(
      requestId,
      status,
      payloadLength > 0 ? session.textBufferPtr : 0,
      payloadLength,
    );
  }

  function emitFileReadResult(
    session: HarnessAppSession | null,
    requestId: number,
    status: number,
    offsetBytes: bigint,
    fileSizeBytes: bigint,
    bytes: Uint8Array | null = null,
    message = '',
  ): void {
    if (session === null) {
      return;
    }
    let payloadLength: number;
    if (status === FILE_STATUS_SUCCESS) {
      payloadLength = bytes?.length ?? 0;
      if (payloadLength > session.textBufferSize) {
        throw new Error('File read result exceeds the shared AssemblyScript text buffer.');
      }
      if (payloadLength > 0 && bytes !== null) {
        new Uint8Array(session.memory.buffer, session.textBufferPtr, payloadLength).set(bytes);
      }
    } else {
      payloadLength = dependencies.writeTextCallbackPayload(session, message, 'File read failure');
    }
    session.exports.__fui_on_file_read_result(
      requestId,
      status,
      offsetBytes,
      fileSizeBytes,
      payloadLength > 0 ? session.textBufferPtr : 0,
      payloadLength,
    );
  }

  function emitFileSaveResult(
    session: HarnessAppSession | null,
    requestId: number,
    status: number,
    writtenBytes: bigint,
    fileName = '',
    mode = FILE_SAVE_MODE_DOWNLOAD,
    message = '',
  ): void {
    if (session === null) {
      return;
    }
    const payloadLength = status === FILE_STATUS_SUCCESS
      ? writeWriterPayload(session, mode, fileName)
      : dependencies.writeTextCallbackPayload(session, message, 'File save failure');
    session.exports.__fui_on_file_save_result(
      requestId,
      status,
      writtenBytes,
      payloadLength > 0 ? session.textBufferPtr : 0,
      payloadLength,
    );
  }

  function emitFileWriterCreated(
    session: HarnessAppSession | null,
    requestId: number,
    status: number,
    writerId = '',
    fileName = '',
    mode = FILE_SAVE_MODE_NATIVE_PICKER,
    message = '',
  ): void {
    if (session === null) {
      return;
    }
    const payloadLength = status === FILE_STATUS_SUCCESS
      ? writeWriterPayload(session, mode, writerId, fileName)
      : dependencies.writeTextCallbackPayload(session, message, 'File writer creation failure');
    session.exports.__fui_on_file_writer_created(
      requestId,
      status,
      payloadLength > 0 ? session.textBufferPtr : 0,
      payloadLength,
    );
  }

  function emitFileWriteResult(
    session: HarnessAppSession | null,
    requestId: number,
    status: number,
    writtenBytes: bigint,
    totalWrittenBytes: bigint,
    message = '',
  ): void {
    if (session === null) {
      return;
    }
    const payloadLength = status === FILE_STATUS_SUCCESS
      ? 0
      : dependencies.writeTextCallbackPayload(session, message, 'File write failure');
    session.exports.__fui_on_file_write_result(
      requestId,
      status,
      writtenBytes,
      totalWrittenBytes,
      payloadLength > 0 ? session.textBufferPtr : 0,
      payloadLength,
    );
  }

  function emitFileFinishResult(
    session: HarnessAppSession | null,
    requestId: number,
    status: number,
    writtenBytes: bigint,
    fileName = '',
    mode = FILE_SAVE_MODE_NATIVE_PICKER,
    message = '',
  ): void {
    if (session === null) {
      return;
    }
    const payloadLength = status === FILE_STATUS_SUCCESS
      ? writeWriterPayload(session, mode, fileName)
      : dependencies.writeTextCallbackPayload(session, message, 'File writer finish failure');
    session.exports.__fui_on_file_finish_result(
      requestId,
      status,
      writtenBytes,
      payloadLength > 0 ? session.textBufferPtr : 0,
      payloadLength,
    );
  }

  function emitFileWorkerProcessProgress(
    session: HarnessAppSession | null,
    requestId: number,
    processedBytes: bigint,
    totalBytes: bigint,
    outputFileName: string | null,
  ): void {
    if (session === null) {
      return;
    }
    const payloadLength = dependencies.writeTextCallbackPayload(
      session,
      outputFileName ?? '',
      'File worker process progress',
    );
    session.exports.__fui_on_file_worker_process_progress(
      requestId,
      processedBytes,
      totalBytes,
      payloadLength > 0 ? session.textBufferPtr : 0,
      payloadLength,
    );
  }

  function emitFileWorkerProcessComplete(
    session: HarnessAppSession | null,
    requestId: number,
    processedBytes: bigint,
    outputFileName: string | null,
    workerResult: string | null = null,
  ): void {
    if (session === null) {
      return;
    }
    // Encode outputFileName\0workerResult so the wasm handler splits them.
    const combined = (outputFileName ?? '') + '\0' + (workerResult ?? '');
    const payloadLength = dependencies.writeTextCallbackPayload(
      session,
      combined,
      'File worker process completion',
    );
    session.exports.__fui_on_file_worker_process_complete(
      requestId,
      processedBytes,
      payloadLength > 0 ? session.textBufferPtr : 0,
      payloadLength,
    );
  }

  function emitFileWorkerProcessError(
    session: HarnessAppSession | null,
    requestId: number,
    status: number,
    message: string,
  ): void {
    if (session === null) {
      return;
    }
    const payloadLength = dependencies.writeTextCallbackPayload(session, message, 'File worker process failure');
    session.exports.__fui_on_file_worker_process_error(
      requestId,
      status,
      payloadLength > 0 ? session.textBufferPtr : 0,
      payloadLength,
    );
  }

  function cleanupFileProcessingRequest(requestId: number): void {
    cancelledFileProcessingRequestIds.delete(requestId);
    activeFileProcessingRequests.delete(requestId);
  }

  function failFileProcessingRequest(record: ActiveFileProcessingRecord, status: number, message: string): void {
    record.failed = true;
    cleanupFileProcessingRequest(record.requestId);
    if (dependencies.getCurrentSession() === record.session) {
      emitFileWorkerProcessError(record.session, record.requestId, status, message);
    }
  }

  function isActiveFileProcessingRecord(record: ActiveFileProcessingRecord): boolean {
    return activeFileProcessingRequests.get(record.requestId) === record
      && dependencies.getCurrentSession() === record.session
      && !record.cancelled
      && !record.failed;
  }

  function trackFileProcessingWrite(record: ActiveFileProcessingRecord, write: Promise<void>): void {
    record.pendingWrites.push(write);
    void write.finally(() => {
      const index = record.pendingWrites.indexOf(write);
      if (index >= 0) {
        void record.pendingWrites.splice(index, 1);
      }
    }).catch(() => {
      // The write promise handler below reports the failure.
    });
  }

  async function startFileProcessing(
    requestId: number,
    session: HarnessAppSession,
    workerWasmPath: string,
    workerEntryName: string,
    sourceFile: File,
    suggestedName: string,
    chunkBytes: number,
    saveToPickedFile: boolean,
    workerHostServices: WorkerHostServicesBundleConfig | undefined,
  ): Promise<void> {
    const workerId = requestId;
    const resolvedWasmUrl = new URL(workerWasmPath, dependencies.workerBootstrapUrl).toString();
    const startProcessor = (targetFileName: string | null, stream: WritableFileStreamLike | null): void => {
      const worker = new Worker(dependencies.workerBootstrapUrl);
      const record: ActiveFileProcessingRecord = {
        requestId,
        session,
        sourceFileName: sourceFile.name,
        targetFileName,
        totalBytes: sourceFile.size,
        stream,
        saveToPickedFile,
        cancelled: false,
        failed: false,
        worker,
        pendingWrites: [],
        processedBytes: 0,
      };
      activeFileProcessingRequests.set(requestId, record);
      worker.addEventListener('message', (event: MessageEvent) => {
        const msg = parseFileProcessWorkerMessage(event.data);
        if (msg === null) {
          return;
        }
        if (msg.type === 'file-process-chunk' && stream !== null) {
          const sourceBytes = msg.bytes instanceof Uint8Array ? msg.bytes : new Uint8Array(msg.bytes);
          const bytes = new Uint8Array(sourceBytes.byteLength);
          bytes.set(sourceBytes);
          const write = stream.write(bytes).catch((error: unknown) => {
            if (isActiveFileProcessingRecord(record)) {
              failFileProcessingRequest(record, FILE_STATUS_ERROR, dependencies.describeHarnessError(error));
            }
          });
          trackFileProcessingWrite(record, write);
        } else if (msg.type === 'progress') {
          if (dependencies.getCurrentSession() === record.session) {
            const parts = msg.text.split(' ');
            const processed = parseInt(parts[0] ?? '0', 10);
            record.processedBytes = processed;
            emitFileWorkerProcessProgress(
              record.session,
              record.requestId,
              BigInt(processed),
              BigInt(record.totalBytes),
              record.targetFileName,
            );
          }
        } else if (msg.type === 'complete') {
          void Promise.all(record.pendingWrites).then(async () => {
            if (!isActiveFileProcessingRecord(record)) {
              return;
            }
            if (stream !== null) {
              await stream.close();
            }
            if (!isActiveFileProcessingRecord(record)) {
              return;
            }
            const hashText = msg.text;
            emitFileWorkerProcessComplete(
              record.session,
              record.requestId,
              BigInt(record.processedBytes),
              record.targetFileName,
              hashText,
            );
            cleanupFileProcessingRequest(record.requestId);
          }).catch((error: unknown) => {
            if (isActiveFileProcessingRecord(record)) {
              failFileProcessingRequest(record, FILE_STATUS_ERROR, dependencies.describeHarnessError(error));
            }
          });
        } else if (msg.type === 'error') {
          failFileProcessingRequest(record, FILE_STATUS_ERROR, msg.text);
        }
      });
      worker.addEventListener('error', () => {
        failFileProcessingRequest(record, FILE_STATUS_ERROR, 'Worker crashed.');
      });
      worker.postMessage({
        type: 'start-file-process',
        workerId,
        file: sourceFile,
        wasmUrl: resolvedWasmUrl,
        entryName: workerEntryName,
        chunkSize: Math.max(1, Math.floor(chunkBytes)),
        workerHostServices,
      });
    };

    if (!saveToPickedFile) {
      startProcessor(null, null);
      return;
    }
    if (!supportsNativeSavePicker()) {
      emitFileWorkerProcessError(session, requestId, FILE_STATUS_ERROR, 'Worker file processing requires the native save picker.');
      return;
    }
    const savePicker = (window as SavePickerWindow).showSaveFilePicker;
    if (typeof savePicker !== 'function') {
      emitFileWorkerProcessError(session, requestId, FILE_STATUS_ERROR, 'Worker file processing requires the native save picker.');
      return;
    }
    await savePicker({ suggestedName }).then((handle) =>
      handle.createWritable().then((writableStream) => {
        if (dependencies.getCurrentSession() !== session || cancelledFileProcessingRequestIds.has(requestId)) {
          void abortWritableStream(writableStream).catch(() => {
            // Ignore cleanup failures after the session moved on.
          });
          cancelledFileProcessingRequestIds.delete(requestId);
          return;
        }
        startProcessor(handle.name ?? suggestedName, writableStream);
      }),
    ).catch((error: unknown) => {
      cancelledFileProcessingRequestIds.delete(requestId);
      if (dependencies.getCurrentSession() !== session) {
        return;
      }
      emitFileWorkerProcessError(
        session,
        requestId,
        error instanceof DOMException && error.name === 'AbortError'
          ? FILE_STATUS_CANCELLED
          : FILE_STATUS_ERROR,
        dependencies.describeHarnessError(error),
      );
    });
  }

  function cancelFileProcessingRequest(requestId: number): void {
    cancelledFileProcessingRequestIds.add(requestId);
    const record = activeFileProcessingRequests.get(requestId);
    if (record === undefined) {
      return;
    }
    record.cancelled = true;
    record.worker.terminate();
  }

  function storeBrowserFile(file: File, prefix: string): StoredFileRecord {
    const id = `${prefix}-${String(nextStoredBrowserFileId++)}`;
    storedBrowserFiles.set(id, file);
    return {
      id,
      file,
    };
  }

  function snapshotStoredBrowserFile(file: File, prefix: string): ExternalHarnessDropItem {
    const stored = storeBrowserFile(file, prefix);
    return {
      id: stored.id,
      kind: EXTERNAL_DROP_ITEM_KIND_FILE,
      name: file.name,
      mimeType: file.type.length > 0 ? file.type : null,
      sizeBytes: file.size,
    };
  }

  function clearActiveExternalDropItems(): void {
    activeExternalDropItems = [];
  }

  function snapshotExternalDropItems(dataTransfer: DataTransfer | null): ExternalHarnessDropItem[] {
    if (dataTransfer === null) {
      return [];
    }
    const files = Array.from(dataTransfer.files);
    if (files.length > 0) {
      return files.map((file) => snapshotStoredBrowserFile(file, 'external-drop'));
    }
    const itemEntries = Array.from(dataTransfer.items);
    const fileEntries = itemEntries.filter((item) => item.kind === 'file');
    if (fileEntries.length > 0) {
      return fileEntries.map((item, index) => ({
        id: `external-drop-${String(nextExternalDropItemId++)}`,
        kind: EXTERNAL_DROP_ITEM_KIND_FILE,
        name: `Dropped file ${String(index + 1)}`,
        mimeType: item.type.length > 0 ? item.type : null,
        sizeBytes: 0,
      }));
    }
    const dragTypes = Array.from(dataTransfer.types);
    if (dragTypes.includes('Files')) {
      return [{
        id: `external-drop-${String(nextExternalDropItemId++)}`,
        kind: EXTERNAL_DROP_ITEM_KIND_FILE,
        name: 'Dropped file',
        mimeType: null,
        sizeBytes: 0,
      }];
    }
    return [];
  }

  function getExternalDropItems(dataTransfer: DataTransfer | null, reuseActive: boolean): ExternalHarnessDropItem[] {
    if (reuseActive && activeExternalDropItems.length > 0) {
      return activeExternalDropItems;
    }
    const items = snapshotExternalDropItems(dataTransfer);
    activeExternalDropItems = items;
    return items;
  }

  function mapExternalDropEffect(effect: number): DataTransfer['dropEffect'] {
    if ((effect & 2) !== 0) {
      return 'move';
    }
    if ((effect & 1) !== 0) {
      return 'copy';
    }
    if ((effect & 4) !== 0) {
      return 'link';
    }
    return 'none';
  }

  function dispatchExternalDragEvent(
    eventType: number,
    event: DragEvent,
    options: { readonly handle?: bigint; readonly reuseActiveItems?: boolean } = {},
  ): number {
    const session = dependencies.getCurrentSession();
    if (
      session === null ||
      session.textBufferPtr === 0 ||
      session.textBufferSize === 0
    ) {
      if (eventType === EXTERNAL_DRAG_EVENT_LEAVE || eventType === EXTERNAL_DRAG_EVENT_DROP) {
        clearActiveExternalDropItems();
      }
      return 0;
    }
    const items = eventType === EXTERNAL_DRAG_EVENT_LEAVE
      ? activeExternalDropItems
      : getExternalDropItems(
        event.dataTransfer,
        eventType === EXTERNAL_DRAG_EVENT_DROP ? false : (options.reuseActiveItems !== false),
      );
    if (items.length === 0 && eventType !== EXTERNAL_DRAG_EVENT_LEAVE) {
      return 0;
    }
    const runtime = dependencies.getRuntime();
    const position = getPointerPosition(runtime.canvas, event);
    const handle = options.handle ?? runtime.getHandleFromPoint(position.x, position.y);
    const payloadLength = items.length > 0 ? writeExternalDropPayload(session, items) : 0;
    const effect = session.exports.__fui_on_external_drag_event(
      eventType,
      handle,
      position.x,
      position.y,
      computeModifiers(event),
      payloadLength > 0 ? session.textBufferPtr : 0,
      payloadLength,
    );
    if (eventType === EXTERNAL_DRAG_EVENT_LEAVE || eventType === EXTERNAL_DRAG_EVENT_DROP) {
      clearActiveExternalDropItems();
    }
    return effect;
  }

  function cancelAllForSession(session: HarnessAppSession | null): void {
    clearActiveExternalDropItems();
    for (const [writerId, record] of activeFileWriters.entries()) {
      if (session !== null && record.session !== session) {
        continue;
      }
      activeFileWriters.delete(writerId);
      void abortWritableStream(record.stream).catch(() => {
        // Ignore disposal cleanup failures.
      });
    }
    for (const [requestId, record] of activeFileProcessingRequests.entries()) {
      if (session !== null && record.session !== session) {
        continue;
      }
      cancelFileProcessingRequest(requestId);
    }
  }

  return {
    cancelAllForSession,
    dispatchExternalDragEvent,
    mapExternalDropEffect,
    imports: {
      fui_file_capabilities(): number {
        return getFileCapabilities();
      },
      fui_file_pick(requestId: number, acceptPtr: number, acceptLen: number, multiple: boolean): void {
        const session = dependencies.getCurrentSession();
        if (session === null) {
          return;
        }
        const accept = dependencies.readAppUtf8(acceptPtr, acceptLen);
        const host = document.body;
        const input = document.createElement('input');
        input.type = 'file';
        input.multiple = multiple;
        if (accept.length > 0) {
          input.accept = accept;
        }
        input.tabIndex = -1;
        input.style.position = 'fixed';
        input.style.left = '-10000px';
        input.style.top = '0';
        input.style.opacity = '0';
        host.appendChild(input);
        let finished = false;
        const complete = (status: number, files: readonly StoredFileRecord[] = [], message = '') => {
          if (finished) {
            return;
          }
          finished = true;
          input.remove();
          if (dependencies.getCurrentSession() !== session) {
            return;
          }
          emitFilePickResult(session, requestId, status, files, message);
        };
        input.addEventListener('change', () => {
          const selected = Array.from(input.files ?? []).map((file) => storeBrowserFile(file, 'picked-file'));
          if (selected.length > 0) {
            complete(FILE_STATUS_SUCCESS, selected);
            return;
          }
          complete(FILE_STATUS_CANCELLED, [], 'File picker cancelled.');
        }, { once: true });
        input.addEventListener('cancel', () => {
          complete(FILE_STATUS_CANCELLED, [], 'File picker cancelled.');
        }, { once: true });
        input.click();
      },
      fui_file_read_chunk(requestId: number, fileIdPtr: number, fileIdLen: number, offsetBytes: bigint | number, maxBytes: number): void {
        const session = dependencies.getCurrentSession();
        if (session === null) {
          return;
        }
        const fileId = dependencies.readAppUtf8(fileIdPtr, fileIdLen);
        const sourceFile = storedBrowserFiles.get(fileId);
        if (sourceFile === undefined) {
          emitFileReadResult(session, requestId, FILE_STATUS_ERROR, 0n, 0n, null, `Unknown browser file "${fileId}".`);
          return;
        }
        const numericOffset = typeof offsetBytes === 'bigint' ? Number(offsetBytes) : offsetBytes;
        if (!Number.isFinite(numericOffset) || numericOffset < 0) {
          emitFileReadResult(session, requestId, FILE_STATUS_ERROR, 0n, BigInt(sourceFile.size), null, 'File read offset was invalid.');
          return;
        }
        const safeOffset = Math.min(sourceFile.size, Math.floor(numericOffset));
        const clampedMaxBytes = Math.max(1, Math.min(Math.floor(maxBytes), session.textBufferSize));
        void sourceFile.slice(safeOffset, safeOffset + clampedMaxBytes).arrayBuffer().then((buffer) => {
          if (dependencies.getCurrentSession() !== session) {
            return;
          }
          emitFileReadResult(
            session,
            requestId,
            FILE_STATUS_SUCCESS,
            BigInt(safeOffset),
            BigInt(sourceFile.size),
            new Uint8Array(buffer),
          );
        }).catch((error: unknown) => {
          if (dependencies.getCurrentSession() !== session) {
            return;
          }
          emitFileReadResult(
            session,
            requestId,
            FILE_STATUS_ERROR,
            BigInt(safeOffset),
            BigInt(sourceFile.size),
            null,
            error instanceof Error ? error.message : String(error),
          );
        });
      },
      fui_file_save_text(
        requestId: number,
        suggestedNamePtr: number,
        suggestedNameLen: number,
        mimeTypePtr: number,
        mimeTypeLen: number,
        fileExtensionPtr: number,
        fileExtensionLen: number,
        textPtr: number,
        textLen: number,
      ): void {
        const session = dependencies.getCurrentSession();
        if (session === null) {
          return;
        }
        const suggestedName = resolveSuggestedName(
          dependencies.readAppUtf8(suggestedNamePtr, suggestedNameLen),
          dependencies.readAppUtf8(fileExtensionPtr, fileExtensionLen),
        );
        const mimeType = dependencies.readAppUtf8(mimeTypePtr, mimeTypeLen);
        const text = dependencies.readAppUtf8(textPtr, textLen);
        const encoded = encoder.encode(text);
        const finishDownload = () => {
          const blob = mimeType.length > 0
            ? new Blob([encoded], { type: mimeType })
            : new Blob([encoded]);
          const url = URL.createObjectURL(blob);
          const anchor = document.createElement('a');
          anchor.href = url;
          anchor.download = suggestedName;
          document.body.appendChild(anchor);
          anchor.click();
          anchor.remove();
          window.setTimeout(() => { URL.revokeObjectURL(url); }, 0);
          if (dependencies.getCurrentSession() === session) {
            emitFileSaveResult(session, requestId, FILE_STATUS_SUCCESS, BigInt(encoded.length), suggestedName, FILE_SAVE_MODE_DOWNLOAD);
          }
        };
        const savePicker = (window as SavePickerWindow).showSaveFilePicker;
        if (typeof savePicker !== 'function') {
          finishDownload();
          return;
        }
        void savePicker({ suggestedName }).then((handle) => handle.createWritable().then(async (stream) => {
          await stream.write(text);
          await stream.close();
          if (dependencies.getCurrentSession() === session) {
            emitFileSaveResult(
              session,
              requestId,
              FILE_STATUS_SUCCESS,
              BigInt(encoded.length),
              handle.name ?? suggestedName,
              FILE_SAVE_MODE_NATIVE_PICKER,
            );
          }
        })).catch((error: unknown) => {
          if (dependencies.getCurrentSession() !== session) {
            return;
          }
          emitFileSaveResult(
            session,
            requestId,
            error instanceof DOMException && error.name === 'AbortError' ? FILE_STATUS_CANCELLED : FILE_STATUS_ERROR,
            0n,
            '',
            FILE_SAVE_MODE_NATIVE_PICKER,
            error instanceof Error ? error.message : String(error),
          );
        });
      },
      fui_file_save_bytes(
        requestId: number,
        suggestedNamePtr: number,
        suggestedNameLen: number,
        mimeTypePtr: number,
        mimeTypeLen: number,
        fileExtensionPtr: number,
        fileExtensionLen: number,
        bytesPtr: number,
        bytesLen: number,
      ): void {
        const session = dependencies.getCurrentSession();
        if (session === null) {
          return;
        }
        const suggestedName = resolveSuggestedName(
          dependencies.readAppUtf8(suggestedNamePtr, suggestedNameLen),
          dependencies.readAppUtf8(fileExtensionPtr, fileExtensionLen),
        );
        const mimeType = dependencies.readAppUtf8(mimeTypePtr, mimeTypeLen);
        const bytes = dependencies.readAppBytes(bytesPtr, bytesLen);
        const copiedBytes = copyBytesToArrayBuffer(bytes);
        const finishDownload = () => {
          const blob = mimeType.length > 0
            ? new Blob([copiedBytes], { type: mimeType })
            : new Blob([copiedBytes]);
          const url = URL.createObjectURL(blob);
          const anchor = document.createElement('a');
          anchor.href = url;
          anchor.download = suggestedName;
          document.body.appendChild(anchor);
          anchor.click();
          anchor.remove();
          window.setTimeout(() => { URL.revokeObjectURL(url); }, 0);
          if (dependencies.getCurrentSession() === session) {
            emitFileSaveResult(session, requestId, FILE_STATUS_SUCCESS, BigInt(bytes.length), suggestedName, FILE_SAVE_MODE_DOWNLOAD);
          }
        };
        const savePicker = (window as SavePickerWindow).showSaveFilePicker;
        if (typeof savePicker !== 'function') {
          finishDownload();
          return;
        }
        void savePicker({ suggestedName }).then((handle) => handle.createWritable().then(async (stream) => {
          await stream.write(copiedBytes);
          await stream.close();
          if (dependencies.getCurrentSession() === session) {
            emitFileSaveResult(
              session,
              requestId,
              FILE_STATUS_SUCCESS,
              BigInt(bytes.length),
              handle.name ?? suggestedName,
              FILE_SAVE_MODE_NATIVE_PICKER,
            );
          }
        })).catch((error: unknown) => {
          if (dependencies.getCurrentSession() !== session) {
            return;
          }
          emitFileSaveResult(
            session,
            requestId,
            error instanceof DOMException && error.name === 'AbortError' ? FILE_STATUS_CANCELLED : FILE_STATUS_ERROR,
            0n,
            '',
            FILE_SAVE_MODE_NATIVE_PICKER,
            error instanceof Error ? error.message : String(error),
          );
        });
      },
      fui_file_create_writer(
        requestId: number,
        suggestedNamePtr: number,
        suggestedNameLen: number,
        _mimeTypePtr: number,
        _mimeTypeLen: number,
        fileExtensionPtr: number,
        fileExtensionLen: number,
      ): void {
        const session = dependencies.getCurrentSession();
        if (session === null) {
          return;
        }
        const savePicker = (window as SavePickerWindow).showSaveFilePicker;
        if (typeof savePicker !== 'function') {
          emitFileWriterCreated(session, requestId, FILE_STATUS_ERROR, '', '', FILE_SAVE_MODE_NATIVE_PICKER, 'Chunked file writers require the native save picker.');
          return;
        }
        const suggestedName = resolveSuggestedName(
          dependencies.readAppUtf8(suggestedNamePtr, suggestedNameLen),
          dependencies.readAppUtf8(fileExtensionPtr, fileExtensionLen),
        );
        void savePicker({ suggestedName }).then((handle) => handle.createWritable().then((stream) => {
          const writerId = `writer-${String(nextFileWriterId++)}`;
          activeFileWriters.set(writerId, {
            id: writerId,
            session,
            fileName: handle.name ?? suggestedName,
            mode: FILE_SAVE_MODE_NATIVE_PICKER,
            stream,
            writtenBytes: 0,
          });
          if (dependencies.getCurrentSession() === session) {
            emitFileWriterCreated(
              session,
              requestId,
              FILE_STATUS_SUCCESS,
              writerId,
              handle.name ?? suggestedName,
              FILE_SAVE_MODE_NATIVE_PICKER,
            );
          }
        })).catch((error: unknown) => {
          if (dependencies.getCurrentSession() !== session) {
            return;
          }
          emitFileWriterCreated(
            session,
            requestId,
            error instanceof DOMException && error.name === 'AbortError' ? FILE_STATUS_CANCELLED : FILE_STATUS_ERROR,
            '',
            '',
            FILE_SAVE_MODE_NATIVE_PICKER,
            error instanceof Error ? error.message : String(error),
          );
        });
      },
      fui_file_writer_write_text(requestId: number, writerIdPtr: number, writerIdLen: number, textPtr: number, textLen: number): void {
        const session = dependencies.getCurrentSession();
        if (session === null) {
          return;
        }
        const writerId = dependencies.readAppUtf8(writerIdPtr, writerIdLen);
        const record = activeFileWriters.get(writerId);
        if (record === undefined) {
          emitFileWriteResult(session, requestId, FILE_STATUS_ERROR, 0n, 0n, `Unknown file writer "${writerId}".`);
          return;
        }
        const text = dependencies.readAppUtf8(textPtr, textLen);
        const encodedLength = encoder.encode(text).length;
        void record.stream.write(text).then(() => {
          record.writtenBytes += encodedLength;
          if (dependencies.getCurrentSession() === session) {
            emitFileWriteResult(
              session,
              requestId,
              FILE_STATUS_SUCCESS,
              BigInt(encodedLength),
              BigInt(record.writtenBytes),
            );
          }
        }).catch((error: unknown) => {
          if (dependencies.getCurrentSession() === session) {
            emitFileWriteResult(session, requestId, FILE_STATUS_ERROR, 0n, BigInt(record.writtenBytes), error instanceof Error ? error.message : String(error));
          }
        });
      },
      fui_file_writer_write_bytes(requestId: number, writerIdPtr: number, writerIdLen: number, bytesPtr: number, bytesLen: number): void {
        const session = dependencies.getCurrentSession();
        if (session === null) {
          return;
        }
        const writerId = dependencies.readAppUtf8(writerIdPtr, writerIdLen);
        const record = activeFileWriters.get(writerId);
        if (record === undefined) {
          emitFileWriteResult(session, requestId, FILE_STATUS_ERROR, 0n, 0n, `Unknown file writer "${writerId}".`);
          return;
        }
        const bytes = dependencies.readAppBytes(bytesPtr, bytesLen);
        const copiedBytes = copyBytesToArrayBuffer(bytes);
        void record.stream.write(copiedBytes).then(() => {
          record.writtenBytes += bytes.length;
          if (dependencies.getCurrentSession() === session) {
            emitFileWriteResult(
              session,
              requestId,
              FILE_STATUS_SUCCESS,
              BigInt(bytes.length),
              BigInt(record.writtenBytes),
            );
          }
        }).catch((error: unknown) => {
          if (dependencies.getCurrentSession() === session) {
            emitFileWriteResult(session, requestId, FILE_STATUS_ERROR, 0n, BigInt(record.writtenBytes), error instanceof Error ? error.message : String(error));
          }
        });
      },
      fui_file_writer_finish(requestId: number, writerIdPtr: number, writerIdLen: number): void {
        const session = dependencies.getCurrentSession();
        if (session === null) {
          return;
        }
        const writerId = dependencies.readAppUtf8(writerIdPtr, writerIdLen);
        const record = activeFileWriters.get(writerId);
        if (record === undefined) {
          emitFileFinishResult(session, requestId, FILE_STATUS_ERROR, 0n, '', FILE_SAVE_MODE_NATIVE_PICKER, `Unknown file writer "${writerId}".`);
          return;
        }
        activeFileWriters.delete(writerId);
        void record.stream.close().then(() => {
          if (dependencies.getCurrentSession() === session) {
            emitFileFinishResult(
              session,
              requestId,
              FILE_STATUS_SUCCESS,
              BigInt(record.writtenBytes),
              record.fileName,
              record.mode,
            );
          }
        }).catch((error: unknown) => {
          if (dependencies.getCurrentSession() === session) {
            emitFileFinishResult(
              session,
              requestId,
              FILE_STATUS_ERROR,
              BigInt(record.writtenBytes),
              '',
              record.mode,
              error instanceof Error ? error.message : String(error),
            );
          }
        });
      },
      fui_file_process_worker_start(
        requestId: number,
        workerWasmPathPtr: number,
        workerWasmPathLen: number,
        workerEntryPtr: number,
        workerEntryLen: number,
        fileIdPtr: number,
        fileIdLen: number,
        suggestedNamePtr: number,
        suggestedNameLen: number,
        chunkBytes: number,
        saveToPickedFile: boolean,
      ): void {
        const session = dependencies.getCurrentSession();
        if (session === null) {
          return;
        }
        const workerWasmPath = dependencies.readAppUtf8(workerWasmPathPtr, workerWasmPathLen);
        const workerEntryName = dependencies.readAppUtf8(workerEntryPtr, workerEntryLen);
        const fileId = dependencies.readAppUtf8(fileIdPtr, fileIdLen);
        const sourceFile = storedBrowserFiles.get(fileId);
        if (sourceFile === undefined) {
          emitFileWorkerProcessError(session, requestId, FILE_STATUS_ERROR, `Unknown browser file "${fileId}".`);
          return;
        }
        const suggestedName = resolveSuggestedName(dependencies.readAppUtf8(suggestedNamePtr, suggestedNameLen), '');
        const workerHostServices = dependencies.getCurrentWorkerHostServices();
        void startFileProcessing(requestId, session, workerWasmPath, workerEntryName, sourceFile, suggestedName, chunkBytes, saveToPickedFile, workerHostServices);
      },
      fui_file_process_worker_cancel(requestId: number): void {
        cancelFileProcessingRequest(requestId);
      },
    },
  };
}
