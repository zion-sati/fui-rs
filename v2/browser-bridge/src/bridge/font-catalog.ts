export interface BridgeFontDefinition {
  readonly id: number;
  readonly assetFile: string;
  readonly fallbackIds: readonly number[];
  readonly preload: boolean;
}

export const BUILT_IN_FONT_BODY = 1;
export const BUILT_IN_FONT_HEADING = 2;
export const BUILT_IN_FONT_SYMBOLS = 3;
export const BUILT_IN_FONT_EMOJI = 4;
export const BUILT_IN_FONT_BODY_ITALIC = 5;
export const BUILT_IN_FONT_BODY_BOLD_ITALIC = 6;
export const BUILT_IN_FONT_MONO = 7;
export const BUILT_IN_FONT_MONO_BOLD = 8;

const BUILT_IN_BRIDGE_FONTS: readonly BridgeFontDefinition[] = [
  { id: BUILT_IN_FONT_BODY, assetFile: 'NotoSans-Regular.ttf', fallbackIds: [BUILT_IN_FONT_EMOJI, BUILT_IN_FONT_SYMBOLS], preload: true },
  { id: BUILT_IN_FONT_HEADING, assetFile: 'NotoSans-Bold.ttf', fallbackIds: [BUILT_IN_FONT_EMOJI, BUILT_IN_FONT_SYMBOLS], preload: true },
  { id: BUILT_IN_FONT_BODY_ITALIC, assetFile: 'NotoSans-Italic.ttf', fallbackIds: [BUILT_IN_FONT_EMOJI, BUILT_IN_FONT_SYMBOLS], preload: true },
  { id: BUILT_IN_FONT_BODY_BOLD_ITALIC, assetFile: 'NotoSans-BoldItalic.ttf', fallbackIds: [BUILT_IN_FONT_EMOJI, BUILT_IN_FONT_SYMBOLS], preload: true },
  { id: BUILT_IN_FONT_SYMBOLS, assetFile: 'NotoSansSymbols2-Regular.ttf', fallbackIds: [], preload: true },
  { id: BUILT_IN_FONT_EMOJI, assetFile: 'NotoEmoji-Regular.ttf', fallbackIds: [], preload: true },
  { id: BUILT_IN_FONT_MONO, assetFile: 'NotoSansMono-Regular.ttf', fallbackIds: [BUILT_IN_FONT_EMOJI, BUILT_IN_FONT_SYMBOLS], preload: false },
  { id: BUILT_IN_FONT_MONO_BOLD, assetFile: 'NotoSansMono-Bold.ttf', fallbackIds: [BUILT_IN_FONT_EMOJI, BUILT_IN_FONT_SYMBOLS], preload: false },
];

export const STARTUP_BRIDGE_FONTS: readonly BridgeFontDefinition[] = BUILT_IN_BRIDGE_FONTS.filter((font) => font.preload);

export function getBuiltInBridgeFont(fontId: number): BridgeFontDefinition | undefined {
  return BUILT_IN_BRIDGE_FONTS.find((font) => font.id === fontId);
}

export function getBridgeAssetBaseUrl(): string {
  const resolvedManifestUrl = window.__effindomResolvedRuntimeAssets?.manifestUrl;
  if (typeof resolvedManifestUrl === 'string') {
    return new URL('./', resolvedManifestUrl).toString();
  }
  const runtimeConfig = window as Window & {
    __effindomRuntime?: { manifestUrls?: readonly string[] };
  };
  const manifestUrl = runtimeConfig.__effindomRuntime?.manifestUrls[0];
  if (typeof manifestUrl === 'string' && manifestUrl.length > 0) {
    return new URL('./', new URL(manifestUrl, document.baseURI)).toString();
  }
  const currentScript = document.currentScript;
  if (currentScript instanceof HTMLScriptElement && currentScript.src.length > 0) {
    return new URL('./', currentScript.src).toString();
  }
  return new URL('./', document.baseURI).toString();
}

export function getBridgeAssetUrl(assetFile: string): string {
  const resolvedUrl = window.__effindomResolvedRuntimeAssets?.fontUrls[assetFile];
  if (resolvedUrl !== undefined) {
    return resolvedUrl;
  }
  return new URL(`../fonts/${assetFile}`, getBridgeAssetBaseUrl()).toString();
}
