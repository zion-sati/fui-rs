import {
  createPersistedUiStateStore,
  PERSISTED_SCROLL_ENTRY_KIND,
  PERSISTED_SCROLL_ENTRY_VERSION,
  PERSISTED_UI_STATE_SNAPSHOT_SCHEMA_VERSION,
  type PersistedScrollPayload,
  type PersistedSnapshotEntry,
  type PersistedSnapshotRecord,
  type PersistedUiStateStore,
} from './persisted-ui-state';
import { readBrowserNavigationType, shouldRestoreInitialHistorySnapshot } from './persisted-restore-policy';
import { readManagedHistoryState, setCurrentManagedHistorySnapshotId } from './managed-history';

function createPersistedSnapshotId(): string {
  if (typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID();
  }
  const bytes = new Uint8Array(16);
  crypto.getRandomValues(bytes);
  return Array.from(bytes, (value) => value.toString(16).padStart(2, '0')).join('');
}

function buildPersistedSnapshotRecord(currentUrl: URL): PersistedSnapshotRecord {
  const timestamp = Date.now();
  return {
    snapshotId: createPersistedSnapshotId(),
    appKey: window.location.origin,
    routeHref: currentUrl.href,
    createdAt: timestamp,
    lastAccessedAt: timestamp,
    schemaVersion: PERSISTED_UI_STATE_SNAPSHOT_SCHEMA_VERSION,
    entries: [],
  };
}

export class PersistedUiStateController {
  private readonly persistedUiState: PersistedUiStateStore | null = createPersistedUiStateStore();
  private readonly currentPersistedEntries = new Map<string, PersistedSnapshotEntry>();
  private persistedUiStateWork: Promise<unknown> = Promise.resolve();

  private persistedEntryKey(kind: string, nodeId: string): string {
    return `${kind}\n${nodeId}`;
  }

  private reportPersistedUiStateError(context: string, error: unknown): void {
    const message = error instanceof Error ? error.stack ?? error.message : String(error);
    console.error(`[fui_persist] ${context}: ${message}`);
  }

  private collectGarbage(context: string): void {
    if (this.persistedUiState === null) {
      return;
    }
    void this.persistedUiState.collectGarbage(Date.now()).catch((error: unknown) => {
      this.reportPersistedUiStateError(`collecting garbage after ${context}`, error);
    });
  }

  private async saveSnapshotForUrl(
    url: URL,
    context: string,
    capture: (() => void) | undefined,
  ): Promise<PersistedSnapshotRecord | null> {
    if (this.persistedUiState === null) {
      return null;
    }
    this.captureCurrentPersistedUiState(context, capture);
    const record: PersistedSnapshotRecord = {
      ...buildPersistedSnapshotRecord(url),
      entries: Array.from(this.currentPersistedEntries.values()),
    };
    try {
      await this.persistedUiState.saveSnapshot(record);
    } catch (error: unknown) {
      this.reportPersistedUiStateError(`saving snapshot while ${context}`, error);
      return null;
    }
    this.collectGarbage(context);
    return record;
  }

  clearCurrentPersistedEntries(): void {
    this.currentPersistedEntries.clear();
  }

  hydrateCurrentPersistedEntries(snapshot: PersistedSnapshotRecord | null): void {
    this.clearCurrentPersistedEntries();
    if (snapshot === null) {
      return;
    }
    for (const entry of snapshot.entries) {
      this.currentPersistedEntries.set(this.persistedEntryKey(entry.kind, entry.nodeId), entry);
    }
  }

  setCurrentPersistedEntry(entry: PersistedSnapshotEntry): void {
    this.currentPersistedEntries.set(this.persistedEntryKey(entry.kind, entry.nodeId), entry);
  }

  setCurrentPersistedScrollEntry(nodeId: string, x: number, y: number): void {
    this.setCurrentPersistedEntry({
      nodeId,
      kind: PERSISTED_SCROLL_ENTRY_KIND,
      version: PERSISTED_SCROLL_ENTRY_VERSION,
      payload: { x, y } satisfies PersistedScrollPayload,
    });
  }

  setCurrentPersistedTextEntry(nodeId: string, kind: string, version: number, payload: string): void {
    this.setCurrentPersistedEntry({
      nodeId,
      kind,
      version,
      payload,
    });
  }

  getCurrentPersistedScrollEntry(nodeId: string): PersistedScrollPayload | null {
    const entry = this.currentPersistedEntries.get(this.persistedEntryKey(PERSISTED_SCROLL_ENTRY_KIND, nodeId));
    if (entry?.kind !== PERSISTED_SCROLL_ENTRY_KIND) {
      return null;
    }
    const payload = entry.payload as { x?: unknown; y?: unknown } | null;
    if (payload === null || typeof payload !== 'object') {
      return null;
    }
    if (typeof payload.x !== 'number' || typeof payload.y !== 'number') {
      return null;
    }
    return {
      x: payload.x,
      y: payload.y,
    };
  }

  getCurrentPersistedTextEntry(nodeId: string, kind: string): { version: number; payload: string; } | null {
    const entry = this.currentPersistedEntries.get(this.persistedEntryKey(kind, nodeId));
    if (entry?.kind !== kind || typeof entry.payload !== 'string') {
      return null;
    }
    return {
      version: entry.version,
      payload: entry.payload,
    };
  }

  captureCurrentPersistedUiState(context: string, capture: (() => void) | undefined): void {
    this.clearCurrentPersistedEntries();
    if (capture === undefined) {
      return;
    }
    try {
      capture();
    } catch (error: unknown) {
      this.clearCurrentPersistedEntries();
      this.reportPersistedUiStateError(`capturing persisted state while ${context}`, error);
    }
  }

  restoreCurrentPersistedUiState(context: string, restore: (() => void) | undefined): void {
    if (restore === undefined) {
      return;
    }
    try {
      restore();
    } catch (error: unknown) {
      this.reportPersistedUiStateError(`restoring persisted state while ${context}`, error);
    }
  }

  queuePersistedUiStateWork<T>(work: () => Promise<T>): Promise<T> {
    const next = this.persistedUiStateWork.then(work, work);
    this.persistedUiStateWork = next.then(
      () => undefined,
      () => undefined,
    );
    return next;
  }

  async loadPersistedSnapshotById(snapshotId: string, context: string): Promise<PersistedSnapshotRecord | null> {
    if (this.persistedUiState === null) {
      return null;
    }
    try {
      const snapshot = await this.persistedUiState.loadSnapshot(snapshotId);
      if (snapshot === null) {
        console.error(`[fui_persist] Missing snapshot ${snapshotId} while ${context}.`);
      }
      return snapshot;
    } catch (error: unknown) {
      this.reportPersistedUiStateError(`loading snapshot while ${context}`, error);
      return null;
    }
  }

  async loadCurrentHistoryEntrySnapshot(context: string): Promise<PersistedSnapshotRecord | null> {
    const state = readManagedHistoryState();
    if (state.uiSnapshotId === undefined) {
      return null;
    }
    return this.loadPersistedSnapshotById(state.uiSnapshotId, context);
  }

  async loadRouteHeadSnapshot(routeHref: string, context: string): Promise<PersistedSnapshotRecord | null> {
    if (this.persistedUiState === null) {
      return null;
    }
    try {
      const routeHead = await this.persistedUiState.loadRouteHead(window.location.origin, routeHref);
      if (routeHead === null) {
        return null;
      }
      return await this.loadPersistedSnapshotById(routeHead.snapshotId, `${context} via route head`);
    } catch (error: unknown) {
      this.reportPersistedUiStateError(`loading route head while ${context}`, error);
      return null;
    }
  }

  async loadSelectedPersistedSnapshot(
    context: string,
    routeHref: string = window.location.href,
  ): Promise<PersistedSnapshotRecord | null> {
    const fromHistory = await this.loadCurrentHistoryEntrySnapshot(context);
    if (fromHistory !== null) {
      return fromHistory;
    }
    return this.loadRouteHeadSnapshot(routeHref, context);
  }

  async loadPopPersistedSnapshot(
    context: string,
    routeHref: string = window.location.href,
  ): Promise<PersistedSnapshotRecord | null> {
    const [fromHistory, fromRouteHead] = await Promise.all([
      this.loadCurrentHistoryEntrySnapshot(context),
      this.loadRouteHeadSnapshot(routeHref, context),
    ]);
    if (fromHistory === null) {
      return fromRouteHead;
    }
    if (fromRouteHead === null) {
      return fromHistory;
    }
    return fromRouteHead.createdAt > fromHistory.createdAt
      ? fromRouteHead
      : fromHistory;
  }

  async loadInitialPersistedSnapshot(context: string): Promise<PersistedSnapshotRecord | null> {
    const state = readManagedHistoryState();
    const navigationType = readBrowserNavigationType();
    const shouldRestore = shouldRestoreInitialHistorySnapshot(
      navigationType,
      state.uiSnapshotId !== undefined,
    );
    if (!shouldRestore) {
      if (state.uiSnapshotId !== undefined) {
        setCurrentManagedHistorySnapshotId(undefined);
      }
      return null;
    }

    const fromHistory = await this.loadCurrentHistoryEntrySnapshot(`${context} via ${navigationType}`);
    if (fromHistory !== null) {
      return fromHistory;
    }
    return navigationType === 'back_forward'
      ? this.loadRouteHeadSnapshot(window.location.href, `${context} via ${navigationType}`)
      : null;
  }

  async saveCurrentHistoryEntrySnapshot(
    context: string,
    capture: (() => void) | undefined,
  ): Promise<string | null> {
    const currentUrl = new URL(window.location.href);
    const record = await this.saveSnapshotForUrl(currentUrl, context, capture);
    if (record === null) {
      return null;
    }
    setCurrentManagedHistorySnapshotId(record.snapshotId, currentUrl);
    return record.snapshotId;
  }

  async saveRouteHeadSnapshotForHref(
    routeHref: string,
    context: string,
    capture: (() => void) | undefined,
  ): Promise<string | null> {
    const record = await this.saveSnapshotForUrl(new URL(routeHref), context, capture);
    return record?.snapshotId ?? null;
  }

  async ensureCurrentHistoryEntrySnapshot(
    context: string,
    capture: (() => void) | undefined,
  ): Promise<string | null> {
    const state = readManagedHistoryState();
    if (state.uiSnapshotId === undefined) {
      return this.saveCurrentHistoryEntrySnapshot(context, capture);
    }
    const loaded = await this.loadCurrentHistoryEntrySnapshot(context);
    if (loaded !== null) {
      return state.uiSnapshotId;
    }
    return this.saveCurrentHistoryEntrySnapshot(context, capture);
  }
}
