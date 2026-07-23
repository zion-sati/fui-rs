import { execFileSync } from 'node:child_process';
import { appendFileSync } from 'node:fs';

const repository = process.env.GITHUB_REPOSITORY;
const releaseSha = process.env.RELEASE_SHA;
const token = process.env.GITHUB_TOKEN;
const outputPath = process.env.GITHUB_OUTPUT;

if (repository === undefined || releaseSha === undefined || token === undefined || outputPath === undefined) {
  throw new Error('GITHUB_REPOSITORY, RELEASE_SHA, GITHUB_TOKEN, and GITHUB_OUTPUT are required.');
}

const exactPaths = new Set([
  'package.json',
  'package-lock.json',
  'eslint.config.ts',
  '.github/workflows/fui-rs-ci.yml',
]);
const scopedPrefixes = [
  'scripts/ci/',
  'v2/abi/',
  'v2/core/include/',
  'v2/ui/include/',
  'v2/fui-rs/',
];

function affectsFuiRs(path) {
  return exactPaths.has(path) || scopedPrefixes.some((prefix) => path.startsWith(prefix));
}

async function api(path) {
  const response = await fetch(`https://api.github.com${path}`, {
    headers: {
      Accept: 'application/vnd.github+json',
      Authorization: `Bearer ${token}`,
      'X-GitHub-Api-Version': '2022-11-28',
    },
  });
  if (!response.ok) {
    throw new Error(`GitHub API ${path} failed: ${response.status} ${await response.text()}`);
  }
  return response.json();
}

function isAncestor(ancestor, descendant) {
  try {
    execFileSync('git', ['merge-base', '--is-ancestor', ancestor, descendant], { stdio: 'ignore' });
    return true;
  } catch {
    return false;
  }
}

function changedPaths(base, head) {
  return execFileSync('git', ['diff', '--name-only', base, head], { encoding: 'utf8' })
    .split('\n')
    .map((path) => path.trim())
    .filter(Boolean);
}

async function successfulFuiRsCiRuns() {
  const runs = [];
  for (let page = 1; page <= 10; page += 1) {
    const result = await api(`/repos/${repository}/actions/workflows/fui-rs-ci.yml/runs?event=push&status=success&per_page=100&page=${page}`);
    runs.push(...result.workflow_runs);
    if (result.workflow_runs.length < 100) break;
  }
  return runs;
}

async function hasReleaseInputs(runId) {
  const result = await api(`/repos/${repository}/actions/runs/${runId}/artifacts?per_page=100`);
  return result.artifacts.some((artifact) => artifact.name === 'fui-rs-release-inputs' && !artifact.expired);
}

const runs = await successfulFuiRsCiRuns();
let attestationRun = null;
for (const run of runs) {
  if (!isAncestor(run.head_sha, releaseSha)) continue;
  if (changedPaths(run.head_sha, releaseSha).some(affectsFuiRs)) continue;
  if (!(await hasReleaseInputs(run.id))) continue;
  attestationRun = run;
  break;
}

if (attestationRun === null) {
  throw new Error('No successful FUI-RS CI run attests all FUI-RS release inputs for this release commit.');
}

appendFileSync(outputPath, `fui_rs_ci_run_id=${attestationRun.id}\nrelease_inputs_run_id=${attestationRun.id}\n`);
console.log(`FUI-RS CI attestation: ${attestationRun.html_url}`);
