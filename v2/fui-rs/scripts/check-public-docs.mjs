import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs';
import { dirname, extname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const packageRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const repositoryRoot = resolve(packageRoot, '..', '..');
const docsRoot = join(repositoryRoot, 'docs', 'v2', 'fui-rs');

function collectMarkdown(directory) {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) return collectMarkdown(path);
    return extname(entry.name) === '.md' && entry.name !== 'plan.md' ? [path] : [];
  });
}

function fail(message) {
  process.stderr.write(`FUI-RS documentation check failed: ${message}\n`);
  process.exitCode = 1;
}

const markdownFiles = collectMarkdown(docsRoot);
const markdownLink = /\[[^\]]*\]\(([^)\s]+)(?:\s+"[^"]*")?\)/g;
for (const file of markdownFiles) {
  const source = readFileSync(file, 'utf8');
  for (const forbidden of [
    'NATIVE_WORKER_PARITY_PLAN.md',
    'browser Worker start/cancel/progress/yield/completion/failure is a browser capability',
    'Fetch, Worker, browser-file',
  ]) {
    if (source.includes(forbidden)) fail(`${file} contains stale or internal Worker guidance: ${forbidden}`);
  }
  for (const match of source.matchAll(markdownLink)) {
    const target = match[1];
    if (/^(?:https?:|mailto:|#|\/)/.test(target)) continue;
    const decoded = decodeURIComponent(target.split('#', 1)[0]);
    if (decoded.length === 0) continue;
    const resolved = resolve(dirname(file), decoded);
    if (!existsSync(resolved)) fail(`${file} links to missing ${target}`);
  }
}

const requiredPaths = [join(repositoryRoot, 'v2', 'fui-rs', 'native-demo', 'src', 'lib.rs')];
if (existsSync(join(repositoryRoot, 'v2', 'native'))) {
  requiredPaths.push(
    join(repositoryRoot, 'v2', 'native', 'common', 'tests', 'test_native_custom_drawing.cpp'),
    join(repositoryRoot, 'v2', 'native', 'common', 'tests', 'test_native_demo_drawing_showcase.cpp'),
  );
}
if (existsSync(join(repositoryRoot, 'skills'))) {
  requiredPaths.push(join(repositoryRoot, 'skills', 'fui-rs-custom-drawing', 'SKILL.md'));
}
for (const path of requiredPaths) {
  if (!existsSync(path) || !statSync(path).isFile()) fail(`required source is missing: ${path}`);
}

const guide = readFileSync(join(docsRoot, 'CUSTOM_DRAWING_AND_BITMAPS.md'), 'utf8');
const normalizedGuide = guide.replace(/\s+/g, ' ');
const requiredGuidance = [
  'native macOS, Windows, and Linux applications',
  'release the mutable borrow',
  'Do not mix direct writes',
  '`Bitmap::render(&node, x, y, scale)`',
  'FUI-RS timers are one-shot',
  'Blank `CustomDrawable`',
];
for (const text of requiredGuidance) {
  if (!normalizedGuide.includes(text)) fail(`custom drawing guide is missing required guidance: ${text}`);
}

if (process.exitCode) process.exit(process.exitCode);

const workerGuide = readFileSync(join(docsRoot, 'HOST_SERVICES_AND_WORKERS.md'), 'utf8')
  .replace(/\s+/g, ' ');
for (const guidance of [
  'native build links the worker crate into the application',
  'dedicated thread',
  'application UI thread',
  'Cancellation is cooperative',
  'Worker troubleshooting',
  'browser file-processing helper is separate',
]) {
  if (!workerGuide.includes(guidance)) fail(`Worker guide is missing required guidance: ${guidance}`);
}

if (process.exitCode) process.exit(process.exitCode);
process.stdout.write(`FUI-RS public documentation check passed (${markdownFiles.length} Markdown files).\n`);
