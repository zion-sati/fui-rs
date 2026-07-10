export class RoutedAppHeadTag {
  kind: string;
  name: string;
  content: string;

  constructor(kind = '', name = '', content = '') {
    this.kind = kind;
    this.name = name;
    this.content = content;
  }
}

export class RoutedAppRoute {
  key: string;
  title: string;
  headTags: RoutedAppHeadTag[];
  entrypoint: string;
  wasmFile: string;
  shellDir: string;
  sourceRoutePath: string;
  publishedRoutePath: string;

  constructor(
    key = '',
    title = '',
    headTags: RoutedAppHeadTag[] = new Array<RoutedAppHeadTag>(),
    entrypoint = '',
    wasmFile = '',
    shellDir = '',
    sourceRoutePath = '',
    publishedRoutePath = '',
  ) {
    this.key = key;
    this.title = title;
    this.headTags = headTags;
    this.entrypoint = entrypoint;
    this.wasmFile = wasmFile;
    this.shellDir = shellDir;
    this.sourceRoutePath = sourceRoutePath;
    this.publishedRoutePath = publishedRoutePath;
  }
}

export class RoutedAppRouteManifest {
  sourceRouteBase: string;
  routes: RoutedAppRoute[];

  constructor(sourceRouteBase = '', routes: RoutedAppRoute[] = new Array<RoutedAppRoute>()) {
    this.sourceRouteBase = sourceRouteBase;
    this.routes = routes;
  }
}

export class ResolvedRoutedAppRoute {
  key: string;
  title: string;
  headTags: RoutedAppHeadTag[];
  shellDir: string;
  wasmFile: string;
  entrypoint: string;
  sourceRoutePath: string;
  publishedRoutePath: string;

  constructor(
    key = '',
    title = '',
    headTags: RoutedAppHeadTag[] = new Array<RoutedAppHeadTag>(),
    shellDir = '',
    wasmFile = '',
    entrypoint = '',
    sourceRoutePath = '',
    publishedRoutePath = '',
  ) {
    this.key = key;
    this.title = title;
    this.headTags = headTags;
    this.shellDir = shellDir;
    this.wasmFile = wasmFile;
    this.entrypoint = entrypoint;
    this.sourceRoutePath = sourceRoutePath;
    this.publishedRoutePath = publishedRoutePath;
  }
}

export class ResolvedRoutedAppRouteManifest {
  sourceRouteBase: string;
  routes: ResolvedRoutedAppRoute[];

  constructor(sourceRouteBase = '', routes: ResolvedRoutedAppRoute[] = new Array<ResolvedRoutedAppRoute>()) {
    this.sourceRouteBase = sourceRouteBase;
    this.routes = routes;
  }
}

export class RoutedHarnessRouteSpec {
  routePath: string;
  wasmPath: string;
  title: string;

  constructor(routePath = '', wasmPath = '', title = '') {
    this.routePath = routePath;
    this.wasmPath = wasmPath;
    this.title = title;
  }
}

export class RoutedAppRouteDefinition {
  key: string;
  title: string;
  entrypoint: string;
  wasmFile: string;
  shellDir: string;
  sourceRoutePath: string;
  publishedRoutePath: string;

  constructor(
    key = '',
    title = '',
    entrypoint = '',
    wasmFile = '',
    shellDir = '',
    sourceRoutePath = '',
    publishedRoutePath = '',
  ) {
    this.key = key;
    this.title = title;
    this.entrypoint = entrypoint;
    this.wasmFile = wasmFile;
    this.shellDir = shellDir;
    this.sourceRoutePath = sourceRoutePath;
    this.publishedRoutePath = publishedRoutePath;
  }
}

function trimSlashes(path: string): string {
  let normalized = path;
  while (normalized.startsWith('/')) {
    normalized = normalized.slice(1);
  }
  while (normalized.endsWith('/')) {
    normalized = normalized.slice(0, -1);
  }
  return normalized;
}

function normalizeRouteBase(path: string): string {
  const trimmed = trimSlashes(path);
  return trimmed.length === 0 ? '' : `/${trimmed}`;
}

function toPascalCase(value: string): string {
  let result = '';
  let capitalizeNext = true;
  const normalized = trimSlashes(value);
  for (let index = 0; index < normalized.length; index += 1) {
    const char = normalized.charAt(index);
    const code = char.charCodeAt(0);
    const isAlphaNum =
      (code >= 48 && code <= 57) ||
      (code >= 65 && code <= 90) ||
      (code >= 97 && code <= 122);
    if (!isAlphaNum) {
      capitalizeNext = true;
      continue;
    }
    result += capitalizeNext ? char.toUpperCase() : char;
    capitalizeNext = false;
  }
  return result;
}

export function routeDef(
  key: string,
  title: string,
  headTags: RoutedAppHeadTag[] = new Array<RoutedAppHeadTag>(),
  entrypoint = '',
  wasmFile = '',
  shellDir = '',
  sourceRoutePath = '',
  publishedRoutePath = '',
): RoutedAppRoute {
  return new RoutedAppRoute(key, title, headTags, entrypoint, wasmFile, shellDir, sourceRoutePath, publishedRoutePath);
}

export function defineRoutedAppManifest(
  sourceRouteBase: string,
  routes: RoutedAppRoute[],
): RoutedAppRouteManifest {
  const normalizedBase = normalizeRouteBase(sourceRouteBase);
  const normalizedRoutes = new Array<RoutedAppRoute>();
  for (const route of routes) {
    const routeKey = trimSlashes(route.key);
    const entrypoint = route.entrypoint.length === 0 ? `src/routes/${toPascalCase(routeKey)}App.ts` : route.entrypoint;
    const wasmFile = route.wasmFile.length === 0 ? `${routeKey}.wasm` : route.wasmFile;
    const shellDir = route.shellDir.length === 0 ? routeKey : route.shellDir;
    const sourceRoutePath = route.sourceRoutePath.length === 0 ? `${normalizedBase}/${routeKey}/` : route.sourceRoutePath;
    const publishedRoutePath = route.publishedRoutePath.length === 0 ? `/${routeKey}/` : route.publishedRoutePath;
    normalizedRoutes.push(new RoutedAppRoute(routeKey, route.title, route.headTags, entrypoint, wasmFile, shellDir, sourceRoutePath, publishedRoutePath));
  }
  return new RoutedAppRouteManifest(normalizedBase, normalizedRoutes);
}

export function resolveRouteManifest(manifest: RoutedAppRouteManifest): ResolvedRoutedAppRouteManifest {
  const routes = new Array<ResolvedRoutedAppRoute>();
  for (const route of manifest.routes) {
    const routeKey = trimSlashes(route.key);
    routes.push(
      new ResolvedRoutedAppRoute(
        routeKey,
        route.title,
        route.headTags,
        routeKey.length === 0 ? '' : routeKey,
        `${routeKey}.wasm`,
        `src/routes/${toPascalCase(routeKey)}App.ts`,
        `${manifest.sourceRouteBase}/${routeKey}/`,
        `/${routeKey}/`,
      ),
    );
  }
  return new ResolvedRoutedAppRouteManifest(manifest.sourceRouteBase, routes);
}

function escapeHtml(value: string): string {
  return value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

function pushHeadTag(tags: string[], headTag: RoutedAppHeadTag): void {
  if (headTag.content.length === 0 || headTag.name.length === 0) {
    return;
  }
  const attribute = headTag.kind === 'property' ? 'property' : 'name';
  tags.push(`    <meta ${attribute}="${escapeHtml(headTag.name)}" content="${escapeHtml(headTag.content)}" />`);
}

export function routeHead(...entries: string[]): RoutedAppHeadTag[] {
  const headTags = new Array<RoutedAppHeadTag>();
  for (let index = 0; index + 1 < entries.length; index += 2) {
    const name = entries[index];
    const content = entries[index + 1];
    if (name === undefined || content === undefined) {
      continue;
    }
    const kind = name.startsWith('og:') || name.startsWith('fb:') ? 'property' : 'name';
    headTags.push(new RoutedAppHeadTag(kind, name, content));
  }
  return headTags;
}

export function renderRoutedPageHead(title: string, headTags: RoutedAppHeadTag[] = new Array<RoutedAppHeadTag>()): string {
  const tags = new Array<string>();
  const effectiveTitle = title.length === 0 ? 'FUI-AS' : title;
  tags.push(`    <title>${escapeHtml(effectiveTitle)}</title>`);
  for (const headTag of headTags) {
    pushHeadTag(tags, headTag);
  }
  return tags.join('\n');
}

export function buildRoutedHarnessRoutes(
  manifest: ResolvedRoutedAppRouteManifest,
): RoutedHarnessRouteSpec[] {
  const routes = new Array<RoutedHarnessRouteSpec>();
  for (const route of manifest.routes) {
    routes.push(new RoutedHarnessRouteSpec(route.publishedRoutePath, route.wasmFile, route.title));
    if (route.sourceRoutePath !== route.publishedRoutePath) {
      routes.push(new RoutedHarnessRouteSpec(route.sourceRoutePath, route.wasmFile, route.title));
    }
  }
  return routes;
}

export function resolveRoutePath(shellDir: string, path: string): string {
  const normalizedShellDir = trimSlashes(shellDir);
  const normalizedPath = trimSlashes(path);
  if (normalizedShellDir.length === 0) {
    return `/${normalizedPath}/`;
  }
  if (normalizedPath.length === 0) {
    return `/${normalizedShellDir}/`;
  }
  return `/${normalizedShellDir}/${normalizedPath}/`;
}
