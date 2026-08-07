import { execFileSync } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const packageDirectory = dirname(dirname(fileURLToPath(import.meta.url)));
const packagePath = join(packageDirectory, 'package.json');
const cargoPath = join(packageDirectory, 'Cargo.toml');
const npm = process.platform === 'win32' ? 'npm.cmd' : 'npm';
const versionPattern = /^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/;

const runtimeVersion = process.env.EFFINDOM_RUNTIME_VERSION ?? execFileSync(
  npm,
  ['view', '@effindomv2/runtime', 'dist-tags.latest'],
  { encoding: 'utf8' },
).trim();
if (!versionPattern.test(runtimeVersion)) {
  throw new Error(`npm returned an invalid EffinDOM runtime version: ${JSON.stringify(runtimeVersion)}`);
}

const packageJson = JSON.parse(readFileSync(packagePath, 'utf8'));
packageJson.dependencies['@effindomv2/runtime'] = runtimeVersion;
writeFileSync(packagePath, `${JSON.stringify(packageJson, null, 2)}\n`);

const cargoToml = readFileSync(cargoPath, 'utf8');
const runtimeField = /(^\[package\.metadata\.effindom\][\s\S]*?^runtime-version\s*=\s*)"[^"]+"/m;
if (!runtimeField.test(cargoToml)) {
  throw new Error('Cargo.toml is missing package.metadata.effindom.runtime-version.');
}
writeFileSync(cargoPath, cargoToml.replace(runtimeField, `$1"${runtimeVersion}"`));
execFileSync(npm, ['install', '--package-lock-only', '--ignore-scripts'], {
  cwd: packageDirectory,
  stdio: 'inherit',
});
console.log(`Pinned @effindomv2/runtime@${runtimeVersion} for npm and Cargo packaging metadata.`);
