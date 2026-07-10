export const PERSISTED_UI_STATE_DB_NAME = 'effindom-ui-state';
export const PERSISTED_UI_STATE_DB_VERSION = 1;
export const PERSISTED_UI_STATE_SNAPSHOT_SCHEMA_VERSION = 1;
export const PERSISTED_UI_STATE_SNAPSHOTS_STORE = 'snapshots';
export const PERSISTED_UI_STATE_ROUTE_HEADS_STORE = 'route-heads';
export const PERSISTED_SCROLL_ENTRY_KIND = 'scroll-position';
export const PERSISTED_SCROLL_ENTRY_VERSION = 1;

const SNAPSHOT_MAX_AGE_MS = 30 * 24 * 60 * 60 * 1000;

export interface PersistedSnapshotEntry {
  readonly nodeId: string;
  readonly kind: string;
  readonly version: number;
  readonly payload: unknown;
}

export interface PersistedScrollPayload {
  readonly x: number;
  readonly y: number;
}

export interface PersistedSnapshotRecord {
  readonly snapshotId: string;
  readonly appKey: string;
  readonly routeHref: string;
  readonly createdAt: number;
  readonly lastAccessedAt: number;
  readonly schemaVersion: number;
  readonly entries: readonly PersistedSnapshotEntry[];
}

export interface PersistedRouteHeadRecord {
  readonly routeKey: string;
  readonly appKey: string;
  readonly routeHref: string;
  readonly snapshotId: string;
  readonly updatedAt: number;
}

export interface PersistedUiStateStore {
  saveSnapshot(record: PersistedSnapshotRecord): Promise<void>;
  loadSnapshot(snapshotId: string): Promise<PersistedSnapshotRecord | null>;
  loadRouteHead(appKey: string, routeHref: string): Promise<PersistedRouteHeadRecord | null>;
  deleteSnapshot(snapshotId: string): Promise<void>;
  collectGarbage(now: number): Promise<void>;
}

export function buildPersistedUiRouteKey(appKey: string, routeHref: string): string {
  return `${appKey}\n${routeHref}`;
}

function requestToPromise<T>(request: IDBRequest<T>): Promise<T> {
  return new Promise<T>((resolve, reject) => {
    request.onsuccess = () => { resolve(request.result); };
    request.onerror = () => { reject(request.error ?? new Error('IndexedDB request failed.')); };
  });
}

function transactionToPromise(transaction: IDBTransaction): Promise<void> {
  return new Promise<void>((resolve, reject) => {
    transaction.oncomplete = () => { resolve(); };
    transaction.onabort = () => { reject(transaction.error ?? new Error('IndexedDB transaction aborted.')); };
    transaction.onerror = () => { reject(transaction.error ?? new Error('IndexedDB transaction failed.')); };
  });
}

function openDatabase(factory: IDBFactory): Promise<IDBDatabase> {
  return new Promise<IDBDatabase>((resolve, reject) => {
    const request = factory.open(PERSISTED_UI_STATE_DB_NAME, PERSISTED_UI_STATE_DB_VERSION);
    request.onupgradeneeded = () => {
      const database = request.result;
      let snapshots = database.objectStoreNames.contains(PERSISTED_UI_STATE_SNAPSHOTS_STORE)
        ? request.transaction?.objectStore(PERSISTED_UI_STATE_SNAPSHOTS_STORE) ?? null
        : null;
      snapshots ??= database.createObjectStore(PERSISTED_UI_STATE_SNAPSHOTS_STORE, {
          keyPath: 'snapshotId',
        });
      if (!snapshots.indexNames.contains('byLastAccessedAt')) {
        snapshots.createIndex('byLastAccessedAt', 'lastAccessedAt');
      }

      let routeHeads = database.objectStoreNames.contains(PERSISTED_UI_STATE_ROUTE_HEADS_STORE)
        ? request.transaction?.objectStore(PERSISTED_UI_STATE_ROUTE_HEADS_STORE) ?? null
        : null;
      routeHeads ??= database.createObjectStore(PERSISTED_UI_STATE_ROUTE_HEADS_STORE, {
          keyPath: 'routeKey',
        });
      if (!routeHeads.indexNames.contains('byUpdatedAt')) {
        routeHeads.createIndex('byUpdatedAt', 'updatedAt');
      }
      if (!routeHeads.indexNames.contains('bySnapshotId')) {
        routeHeads.createIndex('bySnapshotId', 'snapshotId');
      }
    };
    request.onsuccess = () => { resolve(request.result); };
    request.onerror = () => { reject(request.error ?? new Error('Failed to open IndexedDB database.')); };
  });
}

class IndexedDbPersistedUiStateStore implements PersistedUiStateStore {
  private readonly databasePromise: Promise<IDBDatabase>;

  constructor(factory: IDBFactory) {
    this.databasePromise = openDatabase(factory);
  }

  async saveSnapshot(record: PersistedSnapshotRecord): Promise<void> {
    const database = await this.databasePromise;
    const transaction = database.transaction(
      [PERSISTED_UI_STATE_SNAPSHOTS_STORE, PERSISTED_UI_STATE_ROUTE_HEADS_STORE],
      'readwrite',
    );
    const snapshots = transaction.objectStore(PERSISTED_UI_STATE_SNAPSHOTS_STORE);
    const routeHeads = transaction.objectStore(PERSISTED_UI_STATE_ROUTE_HEADS_STORE);
    snapshots.put(record);
    const updatedAt = Math.max(record.createdAt, record.lastAccessedAt);
    routeHeads.put({
      routeKey: buildPersistedUiRouteKey(record.appKey, record.routeHref),
      appKey: record.appKey,
      routeHref: record.routeHref,
      snapshotId: record.snapshotId,
      updatedAt,
    } satisfies PersistedRouteHeadRecord);
    await transactionToPromise(transaction);
  }

  async loadSnapshot(snapshotId: string): Promise<PersistedSnapshotRecord | null> {
    const database = await this.databasePromise;
    const loadTransaction = database.transaction(PERSISTED_UI_STATE_SNAPSHOTS_STORE, 'readonly');
    const snapshots = loadTransaction.objectStore(PERSISTED_UI_STATE_SNAPSHOTS_STORE);
    const existing = await requestToPromise(snapshots.get(snapshotId) as IDBRequest<PersistedSnapshotRecord | undefined>);
    await transactionToPromise(loadTransaction);
    if (existing === undefined) {
      return null;
    }

    const updatedRecord: PersistedSnapshotRecord = {
      ...existing,
      lastAccessedAt: Date.now(),
    };
    const saveTransaction = database.transaction(PERSISTED_UI_STATE_SNAPSHOTS_STORE, 'readwrite');
    saveTransaction.objectStore(PERSISTED_UI_STATE_SNAPSHOTS_STORE).put(updatedRecord);
    await transactionToPromise(saveTransaction);
    return updatedRecord;
  }

  async loadRouteHead(appKey: string, routeHref: string): Promise<PersistedRouteHeadRecord | null> {
    const database = await this.databasePromise;
    const transaction = database.transaction(PERSISTED_UI_STATE_ROUTE_HEADS_STORE, 'readonly');
    const routeHeads = transaction.objectStore(PERSISTED_UI_STATE_ROUTE_HEADS_STORE);
    const routeHead = await requestToPromise<PersistedRouteHeadRecord | undefined>(
      routeHeads.get(buildPersistedUiRouteKey(appKey, routeHref)) as IDBRequest<PersistedRouteHeadRecord | undefined>,
    );
    await transactionToPromise(transaction);
    return routeHead ?? null;
  }

  async deleteSnapshot(snapshotId: string): Promise<void> {
    const database = await this.databasePromise;
    const transaction = database.transaction(PERSISTED_UI_STATE_SNAPSHOTS_STORE, 'readwrite');
    transaction.objectStore(PERSISTED_UI_STATE_SNAPSHOTS_STORE).delete(snapshotId);
    await transactionToPromise(transaction);
  }

  async collectGarbage(now: number): Promise<void> {
    const database = await this.databasePromise;
    const transaction = database.transaction(
      [PERSISTED_UI_STATE_SNAPSHOTS_STORE, PERSISTED_UI_STATE_ROUTE_HEADS_STORE],
      'readwrite',
    );
    const snapshots = transaction.objectStore(PERSISTED_UI_STATE_SNAPSHOTS_STORE);
    const routeHeads = transaction.objectStore(PERSISTED_UI_STATE_ROUTE_HEADS_STORE);
    const snapshotRecords = await requestToPromise(snapshots.getAll() as IDBRequest<PersistedSnapshotRecord[]>);
    const routeHeadRecords = await requestToPromise(routeHeads.getAll() as IDBRequest<PersistedRouteHeadRecord[]>);
    const retainedSnapshotIds = new Set(routeHeadRecords.map((record) => record.snapshotId));
    const knownSnapshotIds = new Set(snapshotRecords.map((record) => record.snapshotId));
    const pruneBefore = now - SNAPSHOT_MAX_AGE_MS;

    for (const routeHead of routeHeadRecords) {
      if (!knownSnapshotIds.has(routeHead.snapshotId)) {
        routeHeads.delete(routeHead.routeKey);
      }
    }

    for (const snapshot of snapshotRecords) {
      if (snapshot.lastAccessedAt >= pruneBefore || retainedSnapshotIds.has(snapshot.snapshotId)) {
        continue;
      }
      snapshots.delete(snapshot.snapshotId);
    }

    await transactionToPromise(transaction);
  }
}

export function createPersistedUiStateStore(): PersistedUiStateStore | null {
  if (typeof globalThis.indexedDB === 'undefined') {
    return null;
  }
  return new IndexedDbPersistedUiStateStore(globalThis.indexedDB);
}
