import type { BridgeRuntime, WasmHandleLike } from '@effindomv2/runtime';
import { copyBytesFromHeap,pointerToHeapOffset,withHeapAllocation,writeBytesToHeap } from '@effindomv2/runtime';
import { toBigIntHandle } from './interop';

interface CustomBitmapRecord {
  readonly width: number;
  readonly height: number;
  readonly bytes: Uint8Array;
}

interface ManagedHarnessBitmapHostDependencies {
  getRuntime(): BridgeRuntime;
  readAppBytes(ptr: number, len: number): Uint8Array;
  writeAppBytes(ptr: number, capacity: number, bytes: Uint8Array, context: string): number;
  notifyBitmapChanged(): void;
}

export function createManagedHarnessBitmapHost(dependencies: ManagedHarnessBitmapHostDependencies) {
  const customBitmapTextures = new Map<number, CustomBitmapRecord>();
  const customBitmapReplayRuntimes = new WeakSet<BridgeRuntime>();

  function uploadCustomBitmap(targetRuntime: BridgeRuntime, textureId: number, record: CustomBitmapRecord): void {
    const textureBytes = writeBytesToHeap(targetRuntime.core, record.bytes);
    try {
      targetRuntime.core._ed_register_texture_rgba(
        textureId,
        textureBytes.ptr,
        record.width,
        record.height,
        textureBytes.len,
      );
    } finally {
      textureBytes.dispose();
    }
  }

  function installReplay(targetRuntime: BridgeRuntime): void {
    if (customBitmapReplayRuntimes.has(targetRuntime)) {
      return;
    }
    const replayLoadedAssets = targetRuntime.replayLoadedAssets.bind(targetRuntime);
    targetRuntime.replayLoadedAssets = async (): Promise<void> => {
      await replayLoadedAssets();
      for (const [textureId, record] of customBitmapTextures.entries()) {
        uploadCustomBitmap(targetRuntime, textureId, record);
      }
    };
    customBitmapReplayRuntimes.add(targetRuntime);
  }

  function clearTextures(targetRuntime: BridgeRuntime): void {
    for (const textureId of customBitmapTextures.keys()) {
      targetRuntime.core._ed_unregister_texture(textureId);
    }
    customBitmapTextures.clear();
  }

  return {
    installReplay,
    clearTextures,
    imports: {
      fui_bitmap_commit(textureId: number, ptr: WasmHandleLike, len: number, width: number, height: number): void {
        if (!Number.isInteger(textureId) || textureId <= 0) {
          throw new Error('Bitmap commit requires a non-zero texture ID.');
        }
        if (!Number.isInteger(width) || !Number.isInteger(height) || width <= 0 || height <= 0) {
          throw new Error('Bitmap commit requires positive integer dimensions.');
        }
        const expectedLength = width * height * 4;
        if (len !== expectedLength) {
          throw new Error(
            `Bitmap commit byte length mismatch: expected ${String(expectedLength)} bytes for ${String(width)}x${String(height)}, received ${String(len)}.`,
          );
        }
        const record: CustomBitmapRecord = {
          width,
          height,
          bytes: dependencies.readAppBytes(pointerToHeapOffset(ptr), len),
        };
        customBitmapTextures.set(textureId, record);
        uploadCustomBitmap(dependencies.getRuntime(), textureId, record);
        dependencies.notifyBitmapChanged();
      },
      fui_bitmap_commit_dirty(
        textureId: number,
        ptr: WasmHandleLike,
        len: number,
        fullW: number,
        fullH: number,
        subX: number,
        subY: number,
        subW: number,
        subH: number,
      ): void {
        if (!Number.isInteger(textureId) || textureId <= 0) return;
        if (subW <= 0 || subH <= 0) return;
        const expectedLen = subW * subH * 4;
        if (len !== expectedLen) return;
        const runtime = dependencies.getRuntime();
        const heap = writeBytesToHeap(runtime.core, dependencies.readAppBytes(pointerToHeapOffset(ptr), len));
        try {
          runtime.core._ed_register_texture_sub_rgba(
            textureId, heap.ptr, subX, subY, subW, subH, fullW, fullH,
          );
        } finally {
          heap.dispose();
        }
        dependencies.notifyBitmapChanged();
      },
      fui_bitmap_release(textureId: number): void {
        if (!Number.isInteger(textureId) || textureId <= 0) {
          return;
        }
        customBitmapTextures.delete(textureId);
        dependencies.getRuntime().core._ed_unregister_texture(textureId);
        dependencies.notifyBitmapChanged();
      },
      fui_render_node_to_rgba(handle: number | bigint, width: number, height: number, outPtr: WasmHandleLike, outCapacity: number, scale: number, x: number, y: number): number {
        const runtime = dependencies.getRuntime();
        const core = runtime.core;
        const byteCount = width * height * 4;
        if (outCapacity < byteCount) return 0;

        return withHeapAllocation(core, byteCount, (allocation) => {
          const written = core._ed_render_node_to_rgba(
            toBigIntHandle(handle),
            width, height, allocation.ptr, byteCount, scale, x, y,
          );
          if (written === 0) {
            return 0;
          }

          const bytes = copyBytesFromHeap(core, allocation.ptr, byteCount);
          dependencies.writeAppBytes(pointerToHeapOffset(outPtr), byteCount, bytes, 'bitmap-text-render');
          return written;
        });
      },
    },
  };
}
