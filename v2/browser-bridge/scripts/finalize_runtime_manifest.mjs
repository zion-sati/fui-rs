#!/usr/bin/env node

import { readFileSync, readdirSync, statSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';

import { computeRuntimeSetHash, contentIntegrity } from './runtime-set.mjs';

const distDirectory = resolve(process.argv[2] ?? 'dist');
const fontDirectory = resolve(process.argv[3] ?? join(distDirectory, 'fonts'));
const fontUrlPrefix = process.argv[4] ?? './fonts';
const manifestPath = join(distDirectory, 'effindom.v2.manifest.json');
const manifest = JSON.parse(readFileSync(manifestPath, 'utf8'));
const fonts = {};
const fontIntegrities = {};
const runtimeFontFiles = new Set([
  'NotoColorEmoji.ttf',
  'NotoEmoji-Regular.ttf',
  'NotoSans-Bold.ttf',
  'NotoSans-BoldItalic.ttf',
  'NotoSans-Italic.ttf',
  'NotoSans-Regular.ttf',
  'NotoSansMono-Bold.ttf',
  'NotoSansMono-Regular.ttf',
  'NotoSansSymbols2-Regular.ttf',
]);

for (const fileName of readdirSync(fontDirectory).sort()) {
  if (!runtimeFontFiles.has(fileName)) {
    continue;
  }
  const path = join(fontDirectory, fileName);
  if (!statSync(path).isFile()) {
    continue;
  }
  const integrity = contentIntegrity(readFileSync(path));
  fonts[fileName] = { url: `${fontUrlPrefix}/${fileName}`, integrity };
  fontIntegrities[fileName] = integrity;
}

manifest.assets.fonts = fonts;
manifest.runtime_set_hash = computeRuntimeSetHash(manifest, fontIntegrities);
writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`, 'utf8');
