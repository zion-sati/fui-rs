import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
import path from 'node:path';
import process from 'node:process';

const packageDir = path.resolve(import.meta.dirname, '..');
const workspaceManifest = path.join(packageDir, 'crates', 'Cargo.toml');
const demoOutput = path.resolve(process.argv[2] ?? path.join(packageDir, '..', '..', 'public', 'v2', 'fui-rs', 'demo'));
const routes = [
  ['fui-rs-demo-home', 'home.wasm'],
  ['fui-rs-demo-workbench', 'workbench.wasm'],
  ['fui-rs-demo-stage4', 'stage4.wasm'],
  ['fui-rs-demo-stage5', 'stage5.wasm'],
  ['fui-rs-demo-immediate-drawing', 'immediate-drawing.wasm'],
];
const routePackages = new Set(routes.map(([packageName]) => packageName));
const universalPagePackages = new Set([...routePackages, 'fui-rs-demo-platform']);

const metadataResult = spawnSync(
  'cargo',
  ['metadata', '--format-version', '1', '--no-deps', '--manifest-path', workspaceManifest],
  { encoding: 'utf8' },
);
if (metadataResult.status !== 0) {
  process.stderr.write(metadataResult.stderr);
  throw new Error('Unable to inspect the FUI-RS demo workspace.');
}

const metadata = JSON.parse(metadataResult.stdout);
const packages = new Map(metadata.packages.map((item) => [item.name, item]));
for (const [packageName] of routes) {
  const routePackage = packages.get(packageName);
  if (!routePackage) {
    throw new Error(`Missing routed demo package ${packageName}.`);
  }
  const siblingDependencies = routePackage.dependencies
    .map((dependency) => dependency.name)
    .filter((dependency) => routePackages.has(dependency));
  if (siblingDependencies.length !== 0) {
    throw new Error(`${packageName} links sibling routes: ${siblingDependencies.join(', ')}`);
  }
}

const registry = packages.get('fui-rs-demo-page-registry');
if (!registry) {
  throw new Error('Missing native universal page registry package.');
}
const registryDependencies = new Set(registry.dependencies.map((dependency) => dependency.name));
for (const packageName of universalPagePackages) {
  if (!registryDependencies.has(packageName)) {
    throw new Error(`Native page registry does not link ${packageName}.`);
  }
}

const hashes = new Set();
for (const [, artifactName] of routes) {
  const artifactPath = path.join(demoOutput, artifactName);
  const bytes = await readFile(artifactPath);
  if (bytes.length < 8 || bytes.subarray(0, 4).toString('hex') !== '0061736d') {
    throw new Error(`${artifactName} is not a valid non-empty WebAssembly route artifact.`);
  }
  hashes.add(createHash('sha256').update(bytes).digest('hex'));
}
if (hashes.size !== routes.length) {
  throw new Error('Two routed demo pages emitted identical WebAssembly artifacts.');
}

console.log('FUI-RS demo architecture acceptance passed.');
