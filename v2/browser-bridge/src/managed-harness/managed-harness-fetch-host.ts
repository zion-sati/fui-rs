import type { HarnessAppSession } from './managed-harness-session';

interface ActiveFetchRequestRecord {
  readonly requestId: number;
  readonly session: HarnessAppSession;
  readonly controller: AbortController;
}

interface ManagedHarnessFetchHostDependencies {
  getCurrentSession(): HarnessAppSession | null;
  readAppUtf8(ptr: number, len: number): string;
  readAppBytes(ptr: number, len: number): Uint8Array;
  readAppTextParts(ptr: number, len: number): string[];
  writeTextCallbackPayload(session: HarnessAppSession, text: string, context: string): number;
  describeHarnessError(error: unknown): string;
}

const encoder = new TextEncoder();

function encodeLengthPrefixedText(value: string): Uint8Array {
  return encoder.encode(value);
}

function measureLengthPrefixedText(encoded: Uint8Array): number {
  return 4 + encoded.length;
}

function writeLengthPrefixedText(
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

function writeTextPartsPayload(session: HarnessAppSession, values: readonly string[], context: string): number {
  const encodedValues = new Array<Uint8Array>(values.length);
  let totalBytes = 4;
  for (let index = 0; index < values.length; index += 1) {
    const encoded = encodeLengthPrefixedText(values[index] ?? '');
    encodedValues[index] = encoded;
    totalBytes += measureLengthPrefixedText(encoded);
  }
  if (totalBytes > session.textBufferSize) {
    throw new Error(`${context} exceeds the shared AssemblyScript text buffer.`);
  }
  const dataView = new DataView(session.memory.buffer, session.textBufferPtr, totalBytes);
  let byteOffset = 0;
  dataView.setUint32(byteOffset, values.length >>> 0, true);
  byteOffset += 4;
  for (const encoded of encodedValues) {
    byteOffset = writeLengthPrefixedText(session.memory, session.textBufferPtr, byteOffset, encoded);
  }
  return totalBytes;
}

function copyBytesToArrayBuffer(bytes: Uint8Array): ArrayBuffer {
  const copied = new Uint8Array(bytes.byteLength);
  copied.set(bytes);
  return copied.buffer;
}

export function createManagedHarnessFetchHost(dependencies: ManagedHarnessFetchHostDependencies) {
  const activeFetchRequests = new Map<number, ActiveFetchRequestRecord>();

  function emitFetchComplete(
    session: HarnessAppSession | null,
    requestId: number,
    ok: boolean,
    status: number,
    statusText: string,
    url: string,
  ): void {
    if (session === null) {
      return;
    }
    const payloadLength = writeTextPartsPayload(
      session,
      [statusText, url],
      'Fetch completion payload',
    );
    session.exports.__fui_on_fetch_complete(
      requestId,
      ok,
      status,
      payloadLength > 0 ? session.textBufferPtr : 0,
      payloadLength,
    );
  }

  function emitFetchError(session: HarnessAppSession | null, requestId: number, message: string): void {
    if (session === null) {
      return;
    }
    const payloadLength = dependencies.writeTextCallbackPayload(session, message, 'Fetch failure payload');
    session.exports.__fui_on_fetch_error(
      requestId,
      payloadLength > 0 ? session.textBufferPtr : 0,
      payloadLength,
    );
  }

  function cancelAllForSession(session: HarnessAppSession | null): void {
    for (const [requestId, record] of activeFetchRequests.entries()) {
      if (session !== null && record.session !== session) {
        continue;
      }
      activeFetchRequests.delete(requestId);
      record.controller.abort();
    }
  }

  return {
    cancelAllForSession,
    imports: {
      fui_fetch_start(
        requestId: number,
        methodPtr: number,
        methodLen: number,
        urlPtr: number,
        urlLen: number,
        headersPtr: number,
        headersLen: number,
        bodyPtr: number,
        bodyLen: number,
      ): void {
        const session = dependencies.getCurrentSession();
        if (session === null) {
          return;
        }
        const method = dependencies.readAppUtf8(methodPtr, methodLen);
        const url = dependencies.readAppUtf8(urlPtr, urlLen);
        const headerParts = dependencies.readAppTextParts(headersPtr, headersLen);
        if ((headerParts.length & 1) != 0) {
          emitFetchError(session, requestId, 'Fetch request headers were malformed.');
          return;
        }
        const controller = new AbortController();
        const headers = new Headers();
        for (let index = 0; index < headerParts.length; index += 2) {
          headers.append(headerParts[index] ?? '', headerParts[index + 1] ?? '');
        }
        const bodyBytes = dependencies.readAppBytes(bodyPtr, bodyLen);
        activeFetchRequests.set(requestId, {
          requestId,
          session,
          controller,
        });
        const init: RequestInit = {
          method,
          headers,
          signal: controller.signal,
        };
        if (bodyBytes.length > 0) {
          init.body = copyBytesToArrayBuffer(bodyBytes);
        }
        void fetch(url, init).then((response) => {
          const active = activeFetchRequests.get(requestId);
          if (active?.session !== session) {
            return;
          }
          activeFetchRequests.delete(requestId);
          if (dependencies.getCurrentSession() !== session) {
            return;
          }
          emitFetchComplete(
            session,
            requestId,
            response.ok,
            response.status,
            response.statusText,
            response.url,
          );
        }).catch((error: unknown) => {
          const active = activeFetchRequests.get(requestId);
          if (active === undefined) {
            return;
          }
          activeFetchRequests.delete(requestId);
          if (controller.signal.aborted || dependencies.getCurrentSession() !== session) {
            return;
          }
          emitFetchError(session, requestId, dependencies.describeHarnessError(error));
        });
      },
      fui_fetch_cancel(requestId: number): void {
        const record = activeFetchRequests.get(requestId);
        if (record === undefined) {
          return;
        }
        activeFetchRequests.delete(requestId);
        record.controller.abort();
      },
    },
  };
}
