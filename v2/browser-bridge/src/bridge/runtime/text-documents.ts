import type {
  OpenCanvasTextDocument,
  SemanticNode,
  UiModule,
} from '../../core-types';
import type { FindOnPageDocument } from '../../find-on-page';
import { handleToBigInt } from '../utils/encoding';
import { copyBytesFromHeap, withHeapAllocation } from '../utils/heap';

const INVALID_TEXT_DOCUMENT_LENGTH = 0xFFFFFFFF;
const openCanvasTextDecoder = new TextDecoder();

export interface TextDocumentMeta {
  readonly handleArg: number | bigint;
  readonly byteLength: number;
}

export interface ResolvedTextRange extends TextDocumentMeta {
  readonly start: number;
  readonly end: number;
}

export class TextDocumentController {
  public constructor(private readonly ui: UiModule) {}

  public readTextDocumentMeta(handle: string): TextDocumentMeta | null {
    const handleArg = this.toUiHandleArgument(handle);
    if (handleArg === null) {
      return null;
    }
    const byteLength = this.ui._ui_get_text_document_utf8_length(handleArg);
    if (byteLength < 0 || (byteLength >>> 0) === INVALID_TEXT_DOCUMENT_LENGTH) {
      return null;
    }
    return { handleArg, byteLength };
  }

  public resolveTextRange(handle: string, start: number, end: number): ResolvedTextRange | null {
    const meta = this.readTextDocumentMeta(handle);
    const range = this.normalizeByteRange(start, end);
    if (meta === null || range === null || range.end > meta.byteLength) {
      return null;
    }
    return {
      handleArg: meta.handleArg,
      byteLength: meta.byteLength,
      start: range.start,
      end: range.end,
    };
  }

  public readTextDocumentSnapshot(
    handle: string,
  ): { readonly document: OpenCanvasTextDocument; readonly byteLength: number } | null {
    const meta = this.readTextDocumentMeta(handle);
    if (meta === null) {
      return null;
    }
    if (meta.byteLength === 0) {
      return {
        document: { handle, text: '' },
        byteLength: 0,
      };
    }

    return withHeapAllocation(this.ui, meta.byteLength, (allocation) => {
      const copied = this.ui._ui_copy_text_document_utf8(meta.handleArg, allocation.ptr, meta.byteLength);
      if (copied === 0) {
        return null;
      }
      const text = openCanvasTextDecoder.decode(copyBytesFromHeap(this.ui, allocation.ptr, meta.byteLength));
      return {
        document: { handle, text },
        byteLength: meta.byteLength,
      };
    });
  }

  public readVisibleTextBounds(handle: string): SemanticNode['bounds'] | null {
    const handleArg = this.toUiHandleArgument(handle);
    if (handleArg === null) {
      return null;
    }
    return withHeapAllocation(this.ui, 16, (allocation) => {
      const xPtr = allocation.ptr;
      const yPtr = this.addPointerOffset(allocation.ptr, 4);
      const widthPtr = this.addPointerOffset(allocation.ptr, 8);
      const heightPtr = this.addPointerOffset(allocation.ptr, 12);
      const copied = this.ui._ui_get_text_visible_bounds(handleArg, xPtr, yPtr, widthPtr, heightPtr);
      if (copied === 0) {
        return null;
      }
      const words = new Float32Array(copyBytesFromHeap(this.ui, allocation.ptr, allocation.len).buffer);
      return {
        x: words[0] ?? 0,
        y: words[1] ?? 0,
        width: words[2] ?? 0,
        height: words[3] ?? 0,
      };
    });
  }

  public readRangeRects(handle: string, start: number, end: number): SemanticNode['bounds'][] {
    const range = this.resolveTextRange(handle, start, end);
    if (range === null) {
      return [];
    }

    const rectCount = this.ui._ui_get_text_range_rect_count(range.handleArg, range.start, range.end);
    if (rectCount === 0) {
      return [];
    }

    return withHeapAllocation(this.ui, rectCount * 16, (allocation) => {
      const copiedCount = this.ui._ui_copy_text_range_rects(
        range.handleArg,
        range.start,
        range.end,
        allocation.ptr,
        rectCount,
      );
      if (copiedCount === 0) {
        return [];
      }
      const words = new Float32Array(copyBytesFromHeap(this.ui, allocation.ptr, copiedCount * 16).buffer);
      const rects: SemanticNode['bounds'][] = [];
      for (let index = 0; index < copiedCount; index += 1) {
        const base = index * 4;
        rects.push({
          x: words[base] ?? 0,
          y: words[base + 1] ?? 0,
          width: words[base + 2] ?? 0,
          height: words[base + 3] ?? 0,
        });
      }
      return rects;
    });
  }

  public readFindDocuments(): FindOnPageDocument[] {
    const documents: FindOnPageDocument[] = [];
    for (const handle of this.readTextSnapshotHandles()) {
      const snapshot = this.readTextDocumentSnapshot(handle);
      if (snapshot === null) {
        continue;
      }
      documents.push(snapshot.document);
    }
    return documents;
  }

  private readTextSnapshotHandles(): string[] {
    const handleCount = this.ui._ui_get_text_snapshot_handle_count();
    if (handleCount === 0) {
      return [];
    }

    return withHeapAllocation(this.ui, handleCount * 8, (allocation) => {
      const copiedCount = this.ui._ui_copy_text_snapshot_handles(allocation.ptr, handleCount);
      if (copiedCount === 0) {
        return [];
      }
      const words = new Uint32Array(copyBytesFromHeap(this.ui, allocation.ptr, copiedCount * 8).buffer);
      const handles: string[] = [];
      for (let index = 0; index < copiedCount; index += 1) {
        const low = words[index * 2] ?? 0;
        const high = words[(index * 2) + 1] ?? 0;
        handles.push(((BigInt(high) << 32n) | BigInt(low)).toString());
      }
      return handles;
    });
  }

  private toUiHandleArgument(handle: string): bigint | null {
    try {
      return handleToBigInt(handle);
    } catch {
      return null;
    }
  }

  private normalizeByteRange(
    start: number,
    end: number,
  ): { readonly start: number; readonly end: number } | null {
    if (!Number.isInteger(start) || !Number.isInteger(end) || start < 0 || end < 0) {
      return null;
    }
    return {
      start: Math.min(start, end),
      end: Math.max(start, end),
    };
  }

  private addPointerOffset(pointer: number | bigint, offset: number): number | bigint {
    return typeof pointer === 'bigint' ? pointer + BigInt(offset) : pointer + offset;
  }
}
