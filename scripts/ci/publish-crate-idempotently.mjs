#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

export function classifyCargoPublish(status, output, crateName, version) {
  if (status === 0) return 'published';
  const duplicate = `crate ${crateName}@${version} already exists on crates.io index`;
  return output.includes(duplicate) ? 'already-published' : 'failed';
}

function option(name) {
  const index = process.argv.indexOf(name);
  return index < 0 ? '' : process.argv[index + 1] ?? '';
}

function main() {
  const manifestPath = option('--manifest-path');
  const crateName = option('--crate');
  const version = option('--version');
  if (!manifestPath || !crateName || !version) {
    console.error('usage: publish-crate-idempotently.mjs --manifest-path <path> --crate <name> --version <version>');
    process.exit(2);
  }

  const result = spawnSync(
    process.env.CARGO_BIN || 'cargo',
    ['publish', '--manifest-path', manifestPath, '--allow-dirty'],
    { encoding: 'utf8' },
  );
  const stdout = result.stdout ?? '';
  const stderr = result.stderr ?? '';
  process.stdout.write(stdout);
  process.stderr.write(stderr);

  const outcome = classifyCargoPublish(result.status, `${stdout}\n${stderr}`, crateName, version);
  if (outcome === 'already-published') {
    console.log(`${crateName}@${version} is already published; treating this release replay as successful.`);
    process.exit(0);
  }
  if (outcome === 'failed') process.exit(result.status ?? 1);
}

if (process.argv[1] && fileURLToPath(import.meta.url) === resolve(process.argv[1])) main();
