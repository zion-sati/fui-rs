import { expect, test } from '@playwright/test';

import type { CoreModule, SemanticNode } from '../src/core-types';
import type {
  BridgePlatformHost,
  HostByteResponse,
  HostTextureResponse,
} from '../src/bridge/host/platform-host';
import { AssetManager } from '../src/bridge/runtime/asset-manager';

class RecordingBridgeHost implements BridgePlatformHost {
  public readonly byteRequests: string[] = [];
  public readonly textureRequests: string[] = [];
  public readonly publishedSemanticTrees: SemanticNode[][] = [];
  public readonly fontNotifications: number[] = [];

  public nowMilliseconds(): number {
    return 123.5;
  }

  public getDevicePixelRatio(): number {
    return 2;
  }

  public getPlatformFamily(): number {
    return 1;
  }

  public isCoarsePointer(): boolean {
    return false;
  }

  public observeCoarsePointer(): () => void {
    return () => {
      void this;
    };
  }

  public requestFrame(callback: FrameRequestCallback): number {
    callback(this.nowMilliseconds());
    return 1;
  }

  public setTimer(callback: () => void): number {
    callback();
    return 1;
  }

  public clearTimer(): void {
    void this;
  }

  public loadBytes(url: string): Promise<HostByteResponse> {
    this.byteRequests.push(url);
    const bytes = new TextEncoder().encode(
      "<svg xmlns='http://www.w3.org/2000/svg' width='12' height='9'></svg>",
    );
    return Promise.resolve({
      ok: true,
      status: 200,
      bytes: () => Promise.resolve(bytes),
    });
  }

  public loadTexture(url: string): Promise<HostTextureResponse> {
    this.textureRequests.push(url);
    return Promise.resolve({
      ok: true,
      status: 200,
      decodeRgba: () =>
        Promise.resolve({
          width: 2,
          height: 1,
          rgba: new Uint8Array([255, 0, 0, 255, 0, 255, 0, 255]),
        }),
    });
  }

  public notifyFontLoaded(fontId: number): void {
    this.fontNotifications.push(fontId);
  }

  public publishSemanticTree(tree: readonly SemanticNode[]): void {
    this.publishedSemanticTrees.push([...tree]);
  }
}

function createRecordingCore() {
  const heap = new Uint8Array(4096);
  let nextAllocation = 16;
  const svgRegistrations: number[] = [];
  const textureRegistrations: { id: number; width: number; height: number }[] = [];
  const textureReleases: number[] = [];
  const core = {
    HEAPU8: heap,
    wasmMemory: { buffer: heap.buffer },
    _malloc(size: number): number {
      const pointer = nextAllocation;
      nextAllocation += size;
      return pointer;
    },
    _free(): void {
      void heap;
    },
    _ed_register_svg(id: number): void {
      svgRegistrations.push(id);
    },
    _ed_register_texture_rgba(id: number, _ptr: number, width: number, height: number): void {
      textureRegistrations.push({ id, width, height });
    },
    _ed_unregister_texture(id: number): void {
      textureReleases.push(id);
    },
  } as unknown as CoreModule;
  return { core, svgRegistrations, textureRegistrations, textureReleases };
}

test('asset manager loads and replays resources through the injected platform host', async () => {
  const host = new RecordingBridgeHost();
  const recording = createRecordingCore();
  let commits = 0;
  const fontManager = {
    replayLoadedFonts: (): Promise<void> => Promise.resolve(),
  };
  const manager = new AssetManager(
    recording.core,
    fontManager as never,
    host,
    () => {
      commits += 1;
    },
  );

  await expect(manager.loadSvg(7, 'asset://icon.svg')).resolves.toEqual({ width: 12, height: 9 });
  await expect(manager.loadTexture(9, 'asset://photo.png')).resolves.toEqual({ width: 2, height: 1 });
  await manager.replayLoadedAssets();
  manager.releaseTexture(9);

  expect(host.byteRequests).toEqual(['asset://icon.svg', 'asset://icon.svg']);
  expect(host.textureRequests).toEqual(['asset://photo.png', 'asset://photo.png']);
  expect(recording.svgRegistrations).toEqual([7, 7]);
  expect(recording.textureRegistrations).toEqual([
    { id: 9, width: 2, height: 1 },
    { id: 9, width: 2, height: 1 },
  ]);
  expect(recording.textureReleases).toEqual([9]);
  expect(commits).toBe(3);
});
