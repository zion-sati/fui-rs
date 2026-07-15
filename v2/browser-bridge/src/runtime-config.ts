export const EFFINDOM_RUNTIME_DIST_DIR = 'dist';
export const EFFINDOM_RUNTIME_MANIFEST_FILE = 'effindom.v2.manifest.json';
export const EFFINDOM_RUNTIME_BRIDGE_SCRIPT = 'bridge.js';
export const EFFINDOM_RUNTIME_HARNESS_SCRIPT = 'harness.js';
export const EFFINDOM_RUNTIME_ARTIFACT_DIR = 'runtime';
export const EFFINDOM_RUNTIME_FONTS_DIR = 'fonts';

export const BuildMode = {
  Debug: 'debug',
  Release: 'release',
} as const;

export type BuildMode = (typeof BuildMode)[keyof typeof BuildMode];

export const DevToolsDomMirrorMode = {
  Disabled: 'disabled',
  Enabled: 'enabled',
  OnRequested: 'on-requested',
} as const;

export type DevToolsDomMirrorMode = (typeof DevToolsDomMirrorMode)[keyof typeof DevToolsDomMirrorMode];

export const PageZoomMode = {
  Disabled: 'disabled',
  Enabled: 'enabled',
} as const;

export type PageZoomMode = (typeof PageZoomMode)[keyof typeof PageZoomMode];

export interface EffinDomRuntimeConfig {
  readonly manifestUrls: readonly string[];
  readonly expectedRuntimeSetHash?: string;
  readonly buildMode?: BuildMode;
  readonly devToolsDomMirror?: DevToolsDomMirrorMode;
  readonly pageZoom?: PageZoomMode;
}

export interface EffinDomRuntimeAssetUrls {
  readonly packageBaseUrl: string;
  readonly distBaseUrl: string;
  readonly manifestUrl: string;
  readonly bridgeScriptUrl: string;
  readonly harnessScriptUrl: string;
  readonly runtimeBaseUrl: string;
  readonly fontsBaseUrl: string;
}

interface RuntimeWindowLike {
  __effindomRuntime?: Partial<EffinDomRuntimeConfig>;
}

export interface ResolvedDevToolsDomMirrorConfig {
  readonly buildMode: BuildMode;
  readonly devToolsDomMirror: DevToolsDomMirrorMode;
  readonly pageZoom: PageZoomMode;
}

function ensureTrailingSlash(url: string): string {
  return url.endsWith('/') ? url : `${url}/`;
}

function resolveFromBase(
  value: string | URL,
  base?: string | URL,
): URL {
  if (value instanceof URL) {
    return new URL(value.toString());
  }
  if (base !== undefined) {
    return new URL(value, base);
  }
  if (typeof document !== 'undefined') {
    return new URL(value, document.baseURI);
  }
  return new URL(value);
}

export function resolveRuntimeAssetUrls(
  packageBaseUrl: string | URL,
  relativeTo?: string | URL,
): EffinDomRuntimeAssetUrls {
  const packageBase = resolveFromBase(packageBaseUrl, relativeTo);
  packageBase.pathname = ensureTrailingSlash(packageBase.pathname);
  const distBase = new URL(`${EFFINDOM_RUNTIME_DIST_DIR}/`, packageBase);
  const runtimeBase = new URL(`${EFFINDOM_RUNTIME_ARTIFACT_DIR}/`, distBase);
  const fontsBase = new URL(`${EFFINDOM_RUNTIME_FONTS_DIR}/`, distBase);

  return {
    packageBaseUrl: packageBase.toString(),
    distBaseUrl: distBase.toString(),
    manifestUrl: new URL(EFFINDOM_RUNTIME_MANIFEST_FILE, distBase).toString(),
    bridgeScriptUrl: new URL(EFFINDOM_RUNTIME_BRIDGE_SCRIPT, distBase).toString(),
    harnessScriptUrl: new URL(EFFINDOM_RUNTIME_HARNESS_SCRIPT, distBase).toString(),
    runtimeBaseUrl: runtimeBase.toString(),
    fontsBaseUrl: fontsBase.toString(),
  };
}

export function createRuntimeConfig(
  packageBaseUrl: string | URL,
  relativeTo?: string | URL,
  overrides: Partial<EffinDomRuntimeConfig> = {},
): EffinDomRuntimeConfig {
  const urls = resolveRuntimeAssetUrls(packageBaseUrl, relativeTo);
  const normalized = normalizeRuntimeConfig(overrides);
  return {
    ...normalized,
    manifestUrls: normalized.manifestUrls ?? [urls.manifestUrl],
  };
}

export function applyRuntimeConfig(
  config: EffinDomRuntimeConfig,
  target?: RuntimeWindowLike,
): EffinDomRuntimeConfig {
  let destination = target;
  if (destination === undefined && typeof window !== 'undefined') {
    // eslint-disable-next-line @typescript-eslint/no-unnecessary-type-assertion -- tsc does not see the browser bridge Window augmentation from this package entry point.
    destination = window as unknown as RuntimeWindowLike;
  }
  if (destination === undefined) {
    throw new Error('applyRuntimeConfig requires a browser window-like target outside browser contexts.');
  }
  destination.__effindomRuntime = Object.assign({}, destination.__effindomRuntime, config);
  const normalizedConfig = normalizeRuntimeConfig(destination.__effindomRuntime);
  const output: EffinDomRuntimeConfig = {
    manifestUrls: normalizedConfig.manifestUrls ?? config.manifestUrls,
    ...(normalizedConfig.expectedRuntimeSetHash === undefined
      ? {}
      : { expectedRuntimeSetHash: normalizedConfig.expectedRuntimeSetHash }),
  };
  const buildMode = normalizeBuildMode(destination.__effindomRuntime.buildMode);
  const devToolsDomMirror = normalizeDevToolsDomMirrorMode(destination.__effindomRuntime.devToolsDomMirror);
  const pageZoom = normalizePageZoomMode(destination.__effindomRuntime.pageZoom);
  if (buildMode !== undefined) {
    const withBuildMode = { ...output, buildMode };
    const withDevTools = devToolsDomMirror !== undefined
      ? { ...withBuildMode, devToolsDomMirror }
      : withBuildMode;
    return pageZoom !== undefined ? { ...withDevTools, pageZoom } : withDevTools;
  }
  const withDevTools = devToolsDomMirror !== undefined ? { ...output, devToolsDomMirror } : output;
  return pageZoom !== undefined ? { ...withDevTools, pageZoom } : withDevTools;
}

export function createRuntimeConfigScript(
  config: EffinDomRuntimeConfig,
): string {
  const normalized = normalizeRuntimeConfig(config);
  const entries = [
    `  manifestUrls: ${JSON.stringify(normalized.manifestUrls ?? config.manifestUrls)},`,
  ];
  if (normalized.expectedRuntimeSetHash !== undefined) {
    entries.push(`  expectedRuntimeSetHash: ${JSON.stringify(normalized.expectedRuntimeSetHash)},`);
  }
  if (normalized.buildMode !== undefined) {
    entries.push(`  buildMode: ${JSON.stringify(normalized.buildMode)},`);
  }
  if (normalized.devToolsDomMirror !== undefined) {
    entries.push(`  devToolsDomMirror: ${JSON.stringify(normalized.devToolsDomMirror)},`);
  }
  if (normalized.pageZoom !== undefined) {
    entries.push(`  pageZoom: ${JSON.stringify(normalized.pageZoom)},`);
  }
  return `window.__effindomRuntime = Object.assign({}, window.__effindomRuntime, {\n${entries.join('\n')}\n});\n`;
}

export function normalizeBuildMode(value: unknown): BuildMode | undefined {
  return value === BuildMode.Release || value === BuildMode.Debug ? value : undefined;
}

export function normalizeDevToolsDomMirrorMode(value: unknown): DevToolsDomMirrorMode | undefined {
  switch (value) {
    case DevToolsDomMirrorMode.Disabled:
    case DevToolsDomMirrorMode.Enabled:
    case DevToolsDomMirrorMode.OnRequested:
      return value;
    default:
      return undefined;
  }
}

export function normalizePageZoomMode(value: unknown): PageZoomMode | undefined {
  switch (value) {
    case PageZoomMode.Disabled:
    case PageZoomMode.Enabled:
      return value;
    default:
      return undefined;
  }
}

export function normalizeRuntimeConfig(config: Partial<EffinDomRuntimeConfig>): Partial<EffinDomRuntimeConfig> {
  const output: {
    manifestUrls?: readonly string[];
    expectedRuntimeSetHash?: string;
    buildMode?: BuildMode;
    devToolsDomMirror?: DevToolsDomMirrorMode;
    pageZoom?: PageZoomMode;
  } = {};
  const buildMode = normalizeBuildMode(config.buildMode);
  const devToolsDomMirror = normalizeDevToolsDomMirrorMode(config.devToolsDomMirror);
  const pageZoom = normalizePageZoomMode(config.pageZoom);
  if (Array.isArray(config.manifestUrls)) {
    const manifestUrls = config.manifestUrls.filter((value): value is string => typeof value === 'string' && value.length > 0);
    if (manifestUrls.length > 0) {
      output.manifestUrls = manifestUrls;
    }
  }
  if (typeof config.expectedRuntimeSetHash === 'string' && config.expectedRuntimeSetHash.length > 0) {
    output.expectedRuntimeSetHash = config.expectedRuntimeSetHash;
  }
  if (buildMode !== undefined) {
    output.buildMode = buildMode;
  }
  if (devToolsDomMirror !== undefined) {
    output.devToolsDomMirror = devToolsDomMirror;
  }
  if (pageZoom !== undefined) {
    output.pageZoom = pageZoom;
  }
  return output;
}

export function resolveDevToolsDomMirrorConfig(
  config: Partial<EffinDomRuntimeConfig> | undefined,
): ResolvedDevToolsDomMirrorConfig {
  const buildMode = normalizeBuildMode(config?.buildMode) ?? BuildMode.Debug;
  const devToolsDomMirror = normalizeDevToolsDomMirrorMode(config?.devToolsDomMirror)
    ?? (buildMode === BuildMode.Release ? DevToolsDomMirrorMode.Disabled : DevToolsDomMirrorMode.OnRequested);
  const pageZoom = normalizePageZoomMode(config?.pageZoom) ?? PageZoomMode.Enabled;
  return {
    buildMode,
    devToolsDomMirror,
    pageZoom,
  };
}
