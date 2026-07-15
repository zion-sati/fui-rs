import type { EffinDomCallbacks, SemanticNode } from '../../core-types';
import { detectPlatformFamily } from '../platform';

export interface HostByteResponse {
  readonly ok: boolean;
  readonly status: number;
  bytes(): Promise<Uint8Array>;
}

export interface HostTextureResponse {
  readonly ok: boolean;
  readonly status: number;
  decodeRgba(): Promise<{
    readonly width: number;
    readonly height: number;
    readonly rgba: Uint8Array;
  }>;
}

export interface BridgePlatformHost {
  nowMilliseconds(): number;
  getDevicePixelRatio(): number;
  getPlatformFamily(): number;
  isCoarsePointer(): boolean;
  observeCoarsePointer(callback: () => void): () => void;
  requestFrame(callback: FrameRequestCallback): number;
  setTimer(callback: () => void, delayMs: number): number;
  clearTimer(timerId: number): void;
  loadBytes(url: string): Promise<HostByteResponse>;
  loadTexture(url: string): Promise<HostTextureResponse>;
  notifyFontLoaded(fontId: number): void;
  publishSemanticTree(tree: readonly SemanticNode[]): void;
}

interface BitmapLike {
  readonly width: number;
  readonly height: number;
  close?(): void;
}

function extractBitmapRgba(bitmap: CanvasImageSource & BitmapLike): Uint8Array {
  const canvas = document.createElement('canvas');
  canvas.width = bitmap.width;
  canvas.height = bitmap.height;
  const context = canvas.getContext('2d', { willReadFrequently: true });
  if (context === null) {
    throw new Error('Failed to allocate a 2D canvas for texture decoding.');
  }
  context.clearRect(0, 0, bitmap.width, bitmap.height);
  context.drawImage(bitmap, 0, 0);
  const imageData = context.getImageData(0, 0, bitmap.width, bitmap.height);
  return new Uint8Array(
    imageData.data.buffer.slice(
      imageData.data.byteOffset,
      imageData.data.byteOffset + imageData.data.byteLength,
    ),
  );
}

export class BrowserBridgePlatformHost implements BridgePlatformHost {
  public nowMilliseconds(): number {
    return performance.now();
  }

  public getDevicePixelRatio(): number {
    return Math.max(1, window.devicePixelRatio || 1);
  }

  public getPlatformFamily(): number {
    return detectPlatformFamily();
  }

  public isCoarsePointer(): boolean {
    return window.matchMedia('(pointer: coarse)').matches || navigator.maxTouchPoints > 0;
  }

  public observeCoarsePointer(callback: () => void): () => void {
    const query = window.matchMedia('(pointer: coarse)');
    query.addEventListener('change', callback);
    return () => {
      query.removeEventListener('change', callback);
    };
  }

  public requestFrame(callback: FrameRequestCallback): number {
    return window.requestAnimationFrame(callback);
  }

  public setTimer(callback: () => void, delayMs: number): number {
    return window.setTimeout(callback, delayMs);
  }

  public clearTimer(timerId: number): void {
    window.clearTimeout(timerId);
  }

  public async loadBytes(url: string): Promise<HostByteResponse> {
    const response = await fetch(url);
    return {
      ok: response.ok,
      status: response.status,
      bytes: async () => new Uint8Array(await response.arrayBuffer()),
    };
  }

  public async loadTexture(url: string): Promise<HostTextureResponse> {
    const response = await fetch(url);
    return {
      ok: response.ok,
      status: response.status,
      decodeRgba: async () => {
        if (typeof createImageBitmap !== 'function') {
          throw new Error('createImageBitmap is unavailable for texture decoding.');
        }
        const bitmap = await createImageBitmap(await response.blob());
        try {
          return {
            width: bitmap.width,
            height: bitmap.height,
            rgba: extractBitmapRgba(bitmap),
          };
        } finally {
          bitmap.close();
        }
      },
    };
  }

  public notifyFontLoaded(fontId: number): void {
    const callbacks: EffinDomCallbacks | undefined = window.__effindomCallbacks;
    callbacks?.onFontLoaded?.(fontId);
  }

  public publishSemanticTree(tree: readonly SemanticNode[]): void {
    window.__bridgeSemanticTree = tree;
  }
}

export const browserBridgePlatformHost: BridgePlatformHost = new BrowserBridgePlatformHost();
