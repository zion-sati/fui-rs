import type { ExternalHarnessDropItem,StoredFileRecord } from './managed-harness-file-types';
import type { HarnessAppSession } from './managed-harness-session';

const encoder = new TextEncoder();

export function encodeLengthPrefixedText(value: string): Uint8Array {
  return encoder.encode(value);
}

export function measureLengthPrefixedText(encoded: Uint8Array): number {
  return 4 + encoded.length;
}

export function writeLengthPrefixedText(
  memory: WebAssembly.Memory,
  basePtr: number,
  byteOffset: number,
  encoded: Uint8Array,
): number {
  const view = new DataView(memory.buffer, basePtr, byteOffset + 4 + encoded.length);
  view.setUint32(byteOffset, encoded.length >>> 0, true);
  let nextOffset = byteOffset + 4;
  if (encoded.length > 0) {
    new Uint8Array(memory.buffer, basePtr + nextOffset, encoded.length).set(encoded);
    nextOffset += encoded.length;
  }
  return nextOffset;
}

export function writeFileListPayload(session: HarnessAppSession, files: readonly StoredFileRecord[]): number {
  let totalBytes = 4;
  const encodedEntries = new Array<{
    entry: StoredFileRecord;
    encodedId: Uint8Array;
    encodedName: Uint8Array;
    encodedMimeType: Uint8Array;
  }>();
  for (const entry of files) {
    const encodedId = encodeLengthPrefixedText(entry.id);
    const encodedName = encodeLengthPrefixedText(entry.file.name);
    const encodedMimeType = encodeLengthPrefixedText(entry.file.type);
    encodedEntries.push({ entry, encodedId, encodedName, encodedMimeType });
    totalBytes += measureLengthPrefixedText(encodedId) + 8 + 8 + measureLengthPrefixedText(encodedName) + measureLengthPrefixedText(encodedMimeType);
  }
  if (totalBytes > session.textBufferSize) {
    throw new Error('File picker payload exceeds the shared AssemblyScript text buffer.');
  }
  const dataView = new DataView(session.memory.buffer, session.textBufferPtr, totalBytes);
  let byteOffset = 0;
  dataView.setUint32(byteOffset, files.length >>> 0, true);
  byteOffset += 4;
  for (const { entry, encodedId, encodedName, encodedMimeType } of encodedEntries) {
    byteOffset = writeLengthPrefixedText(session.memory, session.textBufferPtr, byteOffset, encodedId);
    dataView.setBigUint64(byteOffset, BigInt(entry.file.size), true);
    byteOffset += 8;
    dataView.setBigUint64(byteOffset, BigInt(Math.max(0, Math.trunc(entry.file.lastModified))), true);
    byteOffset += 8;
    byteOffset = writeLengthPrefixedText(session.memory, session.textBufferPtr, byteOffset, encodedName);
    byteOffset = writeLengthPrefixedText(session.memory, session.textBufferPtr, byteOffset, encodedMimeType);
  }
  return totalBytes;
}

export function writeWriterPayload(
  session: HarnessAppSession,
  mode: number,
  first: string,
  second: string | null = null,
): number {
  const encodedFirst = encodeLengthPrefixedText(first);
  const encodedSecond = second === null ? null : encodeLengthPrefixedText(second);
  const totalBytes = 4 + measureLengthPrefixedText(encodedFirst) + (encodedSecond === null ? 0 : measureLengthPrefixedText(encodedSecond));
  if (totalBytes > session.textBufferSize) {
    throw new Error('File bridge metadata exceeds the shared AssemblyScript text buffer.');
  }
  const dataView = new DataView(session.memory.buffer, session.textBufferPtr, totalBytes);
  let byteOffset = 0;
  dataView.setUint32(byteOffset, mode >>> 0, true);
  byteOffset += 4;
  byteOffset = writeLengthPrefixedText(session.memory, session.textBufferPtr, byteOffset, encodedFirst);
  if (encodedSecond !== null) {
    writeLengthPrefixedText(session.memory, session.textBufferPtr, byteOffset, encodedSecond);
  }
  return totalBytes;
}

export function writeExternalDropPayload(session: HarnessAppSession, items: readonly ExternalHarnessDropItem[]): number {
  let totalBytes = 4;
  const encodedItems = new Array<{
    item: ExternalHarnessDropItem;
    encodedId: Uint8Array;
    encodedName: Uint8Array;
    encodedMimeType: Uint8Array;
  }>();
  for (const item of items) {
    const encodedId = encoder.encode(item.id);
    const encodedName = encoder.encode(item.name);
    const encodedMimeType = encoder.encode(item.mimeType ?? '');
    encodedItems.push({ item, encodedId, encodedName, encodedMimeType });
    totalBytes += 4 + 8 + 4 + encodedId.length + 4 + encodedName.length + 4 + encodedMimeType.length;
  }
  if (totalBytes > session.textBufferSize) {
    throw new Error('External drop payload exceeds the shared AssemblyScript text buffer.');
  }
  const dataView = new DataView(session.memory.buffer, session.textBufferPtr, totalBytes);
  let byteOffset = 0;
  dataView.setUint32(byteOffset, items.length >>> 0, true);
  byteOffset += 4;
  for (const { item, encodedId, encodedName, encodedMimeType } of encodedItems) {
    dataView.setUint32(byteOffset, item.kind >>> 0, true);
    byteOffset += 4;
    dataView.setFloat64(byteOffset, item.sizeBytes, true);
    byteOffset += 8;
    dataView.setUint32(byteOffset, encodedId.length >>> 0, true);
    byteOffset += 4;
    if (encodedId.length > 0) {
      new Uint8Array(session.memory.buffer, session.textBufferPtr + byteOffset, encodedId.length).set(encodedId);
      byteOffset += encodedId.length;
    }
    dataView.setUint32(byteOffset, encodedName.length >>> 0, true);
    byteOffset += 4;
    if (encodedName.length > 0) {
      new Uint8Array(session.memory.buffer, session.textBufferPtr + byteOffset, encodedName.length).set(encodedName);
      byteOffset += encodedName.length;
    }
    dataView.setUint32(byteOffset, encodedMimeType.length >>> 0, true);
    byteOffset += 4;
    if (encodedMimeType.length > 0) {
      new Uint8Array(session.memory.buffer, session.textBufferPtr + byteOffset, encodedMimeType.length).set(encodedMimeType);
      byteOffset += encodedMimeType.length;
    }
  }
  return totalBytes;
}
