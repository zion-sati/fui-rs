import type { ManagedHistoryState } from './types';

interface NavigationApiLike {
  readonly canGoBack?: boolean;
  readonly canGoForward?: boolean;
  back?(): void | Promise<void>;
  forward?(): void | Promise<void>;
}

let managedHistoryInitialized = false;
let managedHistoryEntries: string[] = [];
let managedHistoryIndex = 0;

function normalizeManagedHistoryState(rawState: unknown, currentUrl: URL): ManagedHistoryState {
  const rawRecord = typeof rawState === 'object' && rawState !== null
    ? rawState as { href?: unknown; uiSnapshotId?: unknown }
    : null;
  const href = typeof rawRecord?.href === 'string' && rawRecord.href.length > 0
    ? rawRecord.href
    : currentUrl.href;
  const snapshotId = typeof rawRecord?.uiSnapshotId === 'string' && rawRecord.uiSnapshotId.length > 0
    ? rawRecord.uiSnapshotId
    : undefined;
  return snapshotId === undefined
    ? { href }
    : { href, uiSnapshotId: snapshotId };
}

export function readManagedHistoryState(currentUrl: URL = new URL(window.location.href)): ManagedHistoryState {
  return normalizeManagedHistoryState(window.history.state, currentUrl);
}

function writeManagedHistoryState(state: ManagedHistoryState, mode: 'push' | 'replace'): void {
  if (mode === 'push') {
    window.history.pushState(state, '', state.href);
    return;
  }
  window.history.replaceState(state, '', state.href);
}

export function setCurrentManagedHistorySnapshotId(
  snapshotId: string | undefined,
  currentUrl: URL = new URL(window.location.href),
): ManagedHistoryState {
  const state = snapshotId === undefined
    ? { href: currentUrl.href }
    : { href: currentUrl.href, uiSnapshotId: snapshotId };
  writeManagedHistoryState(state, 'replace');
  return state;
}

export function ensureManagedHistoryInitialized(): void {
  const currentUrl = new URL(window.location.href);
  if (managedHistoryInitialized) {
    const normalizedState = readManagedHistoryState(currentUrl);
    writeManagedHistoryState(
      normalizedState.href === currentUrl.href
        ? normalizedState
        : (normalizedState.uiSnapshotId === undefined
          ? { href: currentUrl.href }
          : { href: currentUrl.href, uiSnapshotId: normalizedState.uiSnapshotId }),
      'replace',
    );
    return;
  }
  managedHistoryEntries = [currentUrl.href];
  managedHistoryIndex = 0;
  managedHistoryInitialized = true;
  const normalizedState = readManagedHistoryState(currentUrl);
  writeManagedHistoryState(
    normalizedState.href === currentUrl.href
      ? normalizedState
      : (normalizedState.uiSnapshotId === undefined
        ? { href: currentUrl.href }
        : { href: currentUrl.href, uiSnapshotId: normalizedState.uiSnapshotId }),
    'replace',
  );
}

export function pushManagedHistoryEntry(target: URL): void {
  ensureManagedHistoryInitialized();
  managedHistoryEntries = managedHistoryEntries.slice(0, managedHistoryIndex + 1);
  managedHistoryEntries.push(target.href);
  managedHistoryIndex = managedHistoryEntries.length - 1;
  writeManagedHistoryState({ href: target.href }, 'push');
}

export function replaceManagedHistoryEntry(target: URL): void {
  ensureManagedHistoryInitialized();
  managedHistoryEntries[managedHistoryIndex] = target.href;
  writeManagedHistoryState({ href: target.href }, 'replace');
}

export function syncManagedHistoryPop(target: URL): void {
  ensureManagedHistoryInitialized();
  if (managedHistoryIndex > 0 && managedHistoryEntries[managedHistoryIndex - 1] === target.href) {
    managedHistoryIndex -= 1;
    return;
  }
  if (managedHistoryIndex + 1 < managedHistoryEntries.length && managedHistoryEntries[managedHistoryIndex + 1] === target.href) {
    managedHistoryIndex += 1;
    return;
  }
  const existingIndex = managedHistoryEntries.lastIndexOf(target.href);
  if (existingIndex >= 0) {
    managedHistoryIndex = existingIndex;
    return;
  }
  managedHistoryEntries = [target.href];
  managedHistoryIndex = 0;
}

export function canManagedNavigateBack(): boolean {
  ensureManagedHistoryInitialized();
  return managedHistoryIndex > 0;
}

export function canManagedNavigateForward(): boolean {
  ensureManagedHistoryInitialized();
  return managedHistoryIndex + 1 < managedHistoryEntries.length;
}

function getBrowserNavigationApi(): NavigationApiLike | null {
  const windowWithNavigation = window as Window & {
    navigation?: NavigationApiLike;
  };
  return windowWithNavigation.navigation ?? null;
}

export function canBrowserNavigateBack(): boolean {
  const navigationApi = getBrowserNavigationApi();
  if (navigationApi?.canGoBack !== undefined) {
    return navigationApi.canGoBack;
  }
  return canManagedNavigateBack();
}

export function canBrowserNavigateForward(): boolean {
  const navigationApi = getBrowserNavigationApi();
  if (navigationApi?.canGoForward !== undefined) {
    return navigationApi.canGoForward;
  }
  return canManagedNavigateForward();
}

export function navigateBrowserBack(): void {
  if (!canBrowserNavigateBack()) {
    return;
  }
  const navigationApi = getBrowserNavigationApi();
  if (navigationApi?.back !== undefined) {
    void navigationApi.back();
    return;
  }
  window.history.back();
}

export function navigateBrowserForward(): void {
  if (!canBrowserNavigateForward()) {
    return;
  }
  const navigationApi = getBrowserNavigationApi();
  if (navigationApi?.forward !== undefined) {
    void navigationApi.forward();
    return;
  }
  window.history.forward();
}
