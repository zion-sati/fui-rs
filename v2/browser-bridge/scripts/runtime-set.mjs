import { createHash } from 'node:crypto';

export function stableJson(value) {
  if (Array.isArray(value)) {
    return `[${value.map(stableJson).join(',')}]`;
  }
  if (value !== null && typeof value === 'object') {
    return `{${Object.keys(value).sort().map((key) => `${JSON.stringify(key)}:${stableJson(value[key])}`).join(',')}}`;
  }
  return JSON.stringify(value);
}

export function contentHash(data) {
  return createHash('sha256').update(data).digest('base64url');
}

export function contentIntegrity(data) {
  return `sha256-${createHash('sha256').update(data).digest('base64')}`;
}

export function computeRuntimeSetHash(manifest, fontIntegrities) {
  const architectures = {};
  for (const architecture of Object.keys(manifest.architectures).sort()) {
    architectures[architecture] = {};
    for (const bundleName of Object.keys(manifest.architectures[architecture]).sort()) {
      const bundle = manifest.architectures[architecture][bundleName];
      architectures[architecture][bundleName] = {
        js_integrity: bundle.js_integrity,
        wasm_integrity: bundle.wasm_integrity,
      };
    }
  }
  return contentHash(Buffer.from(stableJson({
    manifest_version: manifest.version,
    architectures,
    assets: {
      icu: manifest.assets.icu.integrity,
      fonts: fontIntegrities,
    },
  })));
}
