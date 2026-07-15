import type { AssetLoadResult,CoreModule } from '../../core-types';
import { writeBytesToHeap } from '../utils/heap';
import type { IncrementalFontManager } from './font-manager';
import { normalizeSvgBytesForCore,parseSvgIntrinsicSize } from './svg-intrinsic-size';
import type { BridgePlatformHost } from '../host/platform-host';

export class AssetManager {
  private readonly loadedSvgs = new Map<number, string>();
  private readonly loadedTextures = new Map<number, string>();

  public constructor(
    private readonly core: CoreModule,
    private readonly fontManager: IncrementalFontManager,
    private readonly host: BridgePlatformHost,
    private readonly onCommitFrame: () => void,
  ) {}

  public async loadSvg(svgId: number, url: string): Promise<AssetLoadResult> {
    this.loadedSvgs.set(svgId, url);
    const response = await this.host.loadBytes(url);
    if (!response.ok) {
      throw new Error(`Failed to fetch SVG ${url}: ${String(response.status)}`);
    }
    const bytes = await response.bytes();
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
    const response = await this.host.loadTexture(url);
    if (!response.ok) {
      throw new Error(`Failed to fetch texture ${url}: ${String(response.status)}`);
    }
    const decoded = await response.decodeRgba();
    const textureBytes = writeBytesToHeap(this.core, decoded.rgba);
    try {
      this.core._ed_register_texture_rgba(textureId, textureBytes.ptr, decoded.width, decoded.height, textureBytes.len);
    } finally {
      textureBytes.dispose();
    }
    this.onCommitFrame();
    return {
      width: decoded.width,
      height: decoded.height,
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
        const response = await this.host.loadBytes(url);
        if (!response.ok) {
          throw new Error(`Failed to refetch SVG ${url}: ${String(response.status)}`);
        }
        const bytes = await response.bytes();
        const svgBytes = writeBytesToHeap(this.core, normalizeSvgBytesForCore(bytes));
        try {
          this.core._ed_register_svg(svgId, svgBytes.ptr, svgBytes.len);
        } finally {
          svgBytes.dispose();
        }
      }),
      ...Array.from(this.loadedTextures.entries(), async ([textureId, url]) => {
        const response = await this.host.loadTexture(url);
        if (!response.ok) {
          throw new Error(`Failed to refetch texture ${url}: ${String(response.status)}`);
        }
        const decoded = await response.decodeRgba();
        const textureBytes = writeBytesToHeap(this.core, decoded.rgba);
        try {
          this.core._ed_register_texture_rgba(
            textureId,
            textureBytes.ptr,
            decoded.width,
            decoded.height,
            textureBytes.len,
          );
        } finally {
          textureBytes.dispose();
        }
      }),
    ]);
  }
}
