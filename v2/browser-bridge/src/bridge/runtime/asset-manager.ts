import type { AssetLoadResult,CoreModule } from '../../core-types';
import { writeBytesToHeap } from '../utils/heap';
import type { IncrementalFontManager } from './font-manager';
import { normalizeSvgBytesForCore,parseSvgIntrinsicSize } from './svg-intrinsic-size';

interface BitmapLike {
  readonly width: number;
  readonly height: number;
  close?(): void;
}

function createDecodeCanvas(width: number, height: number): HTMLCanvasElement {
  const canvas = document.createElement('canvas');
  canvas.width = width;
  canvas.height = height;
  return canvas;
}

async function decodeBlobToBitmap(blob: Blob): Promise<ImageBitmap> {
  if (typeof createImageBitmap !== 'function') {
    throw new Error('createImageBitmap is unavailable for texture decoding.');
  }
  return await createImageBitmap(blob);
}

function extractBitmapRgba(bitmap: CanvasImageSource & BitmapLike): Uint8Array {
  const canvas = createDecodeCanvas(bitmap.width, bitmap.height);
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

export class AssetManager {
  private readonly loadedSvgs = new Map<number, string>();
  private readonly loadedTextures = new Map<number, string>();

  public constructor(
    private readonly core: CoreModule,
    private readonly fontManager: IncrementalFontManager,
    private readonly onCommitFrame: () => void,
  ) {}

  public async loadSvg(svgId: number, url: string): Promise<AssetLoadResult> {
    this.loadedSvgs.set(svgId, url);
    const response = await fetch(url);
    if (!response.ok) {
      throw new Error(`Failed to fetch SVG ${url}: ${String(response.status)}`);
    }
    const bytes = new Uint8Array(await response.arrayBuffer());
    const size = parseSvgIntrinsicSize(bytes);
    const svgBytes = writeBytesToHeap(this.core, normalizeSvgBytesForCore(bytes));
    try {
      this.core._ed_register_svg(svgId, svgBytes.ptr, svgBytes.len);
    } finally {
      svgBytes.dispose();
    }
    this.onCommitFrame();
    return size;
  }

  public async loadTexture(textureId: number, url: string): Promise<AssetLoadResult> {
    this.loadedTextures.set(textureId, url);
    const response = await fetch(url);
    if (!response.ok) {
      throw new Error(`Failed to fetch texture ${url}: ${String(response.status)}`);
    }
    const blob = await response.blob();
    const bitmap = await decodeBlobToBitmap(blob);
    const width = bitmap.width;
    const height = bitmap.height;
    try {
      const rgba = extractBitmapRgba(bitmap);
      const textureBytes = writeBytesToHeap(this.core, rgba);
      try {
        this.core._ed_register_texture_rgba(textureId, textureBytes.ptr, width, height, textureBytes.len);
      } finally {
        textureBytes.dispose();
      }
    } finally {
      if (typeof bitmap.close === 'function') {
        bitmap.close();
      }
    }
    this.onCommitFrame();
    return {
      width,
      height,
    };
  }

  public releaseSvg(svgId: number): void {
    this.loadedSvgs.delete(svgId);
  }

  public releaseTexture(textureId: number): void {
    this.loadedTextures.delete(textureId);
    this.core._ed_unregister_texture(textureId);
    this.onCommitFrame();
  }

  public async replayLoadedAssets(): Promise<void> {
    await Promise.all([
      this.fontManager.replayLoadedFonts(),
      ...Array.from(this.loadedSvgs.entries(), async ([svgId, url]) => {
        const response = await fetch(url);
        if (!response.ok) {
          throw new Error(`Failed to refetch SVG ${url}: ${String(response.status)}`);
        }
        const bytes = new Uint8Array(await response.arrayBuffer());
        const svgBytes = writeBytesToHeap(this.core, normalizeSvgBytesForCore(bytes));
        try {
          this.core._ed_register_svg(svgId, svgBytes.ptr, svgBytes.len);
        } finally {
          svgBytes.dispose();
        }
      }),
      ...Array.from(this.loadedTextures.entries(), async ([textureId, url]) => {
        const response = await fetch(url);
        if (!response.ok) {
          throw new Error(`Failed to refetch texture ${url}: ${String(response.status)}`);
        }
        const blob = await response.blob();
        const bitmap = await decodeBlobToBitmap(blob);
        try {
          const rgba = extractBitmapRgba(bitmap);
          const textureBytes = writeBytesToHeap(this.core, rgba);
          try {
            this.core._ed_register_texture_rgba(textureId, textureBytes.ptr, bitmap.width, bitmap.height, textureBytes.len);
          } finally {
            textureBytes.dispose();
          }
        } finally {
          if (typeof bitmap.close === 'function') {
            bitmap.close();
          }
        }
      }),
    ]);
  }
}
