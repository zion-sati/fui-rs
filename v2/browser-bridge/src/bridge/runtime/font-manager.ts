import type {
BridgeFontRegistration,
BridgeFontStackRegistration,
BridgeLogs,
CoreModule,
IncrementalFontAutoGrowBlockReason,
IncrementalFontCacheState,
IncrementalFontPolicy,
IncrementalFontRuntimeState,
UiModule,
} from '../../core-types';
import { getBridgeAssetUrl,getBuiltInBridgeFont } from '../font-catalog';
import { fetchGoogleFontShardBytes } from '../google-fonts';
import {
resolveIncrementalFontPackageRequests,
type ResolvedIncrementalFontShardRequest,
} from '../incremental-font-packages';
import { copyBytesFromHeap, withHeapAllocation, writeBytesToHeap } from '../utils/heap';
import type { BridgePlatformHost } from '../host/platform-host';

type InternalFontLoadState = 'known' | 'loading' | 'loaded' | 'failed';

interface InternalIncrementalFontState {
  sourceUrl: string | null;
  sourceState: 'unknown' | InternalFontLoadState;
  requestedSegmentIds: Set<string>;
  pendingSegmentIds: Set<string>;
  appliedSegmentIds: Set<string>;
  evictedSegmentIds: Set<string>;
  requestedCharactersByFamily: Map<string, Set<string>>;
  revision: number;
  blockedPackageIds: Set<string>;
  lastBlockedReason: IncrementalFontAutoGrowBlockReason | null;
}

interface InternalIncrementalShardCacheEntry {
  shardKey: string;
  fontId: number;
  sizeBytes: number;
  primaryFontIds: Set<number>;
  lastAccessTick: number;
}

interface NormalizedIncrementalFontPolicy {
  autoGrow: boolean;
  maxCachedShardFonts: number;
  allowedFontIds: Set<number> | null;
  blockedPackageIds: Set<string> | null;
}

const DEFAULT_INCREMENTAL_FONT_POLICY: IncrementalFontPolicy = {
  autoGrow: true,
  maxCachedShardFonts: 8,
  allowedFontIds: null,
  blockedPackageIds: null,
};

export class IncrementalFontManager {
  private readonly loadedFonts = new Map<number, string>();
  private readonly fontLoadStates = new Map<number, { readonly url: string; readonly state: InternalFontLoadState }>();
  private readonly incrementalFontStates = new Map<number, InternalIncrementalFontState>();
  private readonly fontFallbacks = new Map<number, number[]>();
  private readonly lazyFontSources = new Map<number, string>();
  private readonly pendingFontLoads = new Map<number, Promise<void>>();
  private readonly pendingIncrementalShardLoads = new Map<string, Promise<number>>();
  private readonly pendingIncrementalFamilyRequests = new Map<string, {
    packageId: string;
    coverageKind: number;
    familyKey: string;
    googleFamily: string;
    characters: Set<string>;
  }>();
  private readonly scheduledIncrementalFamilyFlushes = new Set<string>();
  private readonly incrementalShardCache = new Map<string, InternalIncrementalShardCacheEntry>();
  private readonly shardKeysByPrimaryFont = new Map<number, Set<string>>();
  private readonly evictedShardKeys: string[] = [];
  private nextIncrementalFontId = 0x7F000100;
  private shardAccessTick = 0;
  private incrementalFontPolicy: IncrementalFontPolicy = DEFAULT_INCREMENTAL_FONT_POLICY;
  private normalizedIncrementalFontPolicy: NormalizedIncrementalFontPolicy = {
    autoGrow: DEFAULT_INCREMENTAL_FONT_POLICY.autoGrow,
    maxCachedShardFonts: DEFAULT_INCREMENTAL_FONT_POLICY.maxCachedShardFonts,
    allowedFontIds: null,
    blockedPackageIds: null,
  };

  public constructor(
    private readonly core: CoreModule,
    private readonly ui: UiModule,
    private readonly logs: BridgeLogs,
    private readonly host: BridgePlatformHost,
    private readonly onCommitFrame: () => void,
  ) {}

  private normalizePolicy(policy: IncrementalFontPolicy): NormalizedIncrementalFontPolicy {
    return {
      autoGrow: policy.autoGrow,
      maxCachedShardFonts: Math.max(1, Math.floor(policy.maxCachedShardFonts)),
      allowedFontIds: policy.allowedFontIds === null ? null : new Set(policy.allowedFontIds),
      blockedPackageIds: policy.blockedPackageIds === null ? null : new Set(policy.blockedPackageIds),
    };
  }

  private registerFontBytes(fontId: number, bytes: Uint8Array): void {
    const coreBytes = writeBytesToHeap(this.core, bytes);
    const uiBytes = writeBytesToHeap(this.ui, bytes);
    try {
      this.core._ed_register_font(fontId, coreBytes.ptr, coreBytes.len);
      if (this.ui._ui_register_font(fontId, uiBytes.ptr, uiBytes.len) === 0) {
        throw new Error(`Ui rejected font ${String(fontId)}.`);
      }
      this.ui._ui_font_loaded(fontId);
      this.host.notifyFontLoaded(fontId);
    } finally {
      coreBytes.dispose();
      uiBytes.dispose();
    }
  }

  private unregisterFont(fontId: number): void {
    this.core._ed_unregister_font(fontId);
    this.ui._ui_unregister_font(fontId);
    this.loadedFonts.delete(fontId);
    this.fontLoadStates.delete(fontId);
  }

  private updateShardAccess(shardKey: string): void {
    const shard = this.incrementalShardCache.get(shardKey);
    if (shard === undefined) {
      return;
    }
    this.shardAccessTick += 1;
    shard.lastAccessTick = this.shardAccessTick;
  }

  private getAutoGrowBlockReason(
    fontId: number,
    packageId: string | null,
  ): IncrementalFontAutoGrowBlockReason | null {
    if (!this.normalizedIncrementalFontPolicy.autoGrow) {
      return 'auto-grow-disabled';
    }
    if (
      this.normalizedIncrementalFontPolicy.allowedFontIds !== null
      && !this.normalizedIncrementalFontPolicy.allowedFontIds.has(fontId)
    ) {
      return 'font-not-allowed';
    }
    if (
      packageId !== null
      && this.normalizedIncrementalFontPolicy.blockedPackageIds?.has(packageId) === true
    ) {
      return 'package-blocked';
    }
    return null;
  }

  private releasePrimaryFontShardReference(primaryFontId: number, shardKey: string, shardFontId: number): void {
    const fontState = this.ensureIncrementalFontState(primaryFontId);
    fontState.appliedSegmentIds.delete(shardKey);
    fontState.evictedSegmentIds.add(shardKey);
    fontState.revision += 1;

    const trackedShardKeys = this.shardKeysByPrimaryFont.get(primaryFontId);
    trackedShardKeys?.delete(shardKey);
    if (trackedShardKeys?.size === 0) {
      this.shardKeysByPrimaryFont.delete(primaryFontId);
    }

    const fallbacks = this.fontFallbacks.get(primaryFontId);
    if (fallbacks !== undefined) {
      this.fontFallbacks.set(
        primaryFontId,
        fallbacks.filter((fallbackId) => fallbackId !== shardFontId),
      );
    }
    this.ui._ui_unregister_font_fallback(primaryFontId, shardFontId);
    this.ui._ui_font_loaded(primaryFontId);
    this.host.notifyFontLoaded(primaryFontId);
  }

  private detachPrimaryFontShardReference(primaryFontId: number, shardFontId: number): void {
    const fallbacks = this.fontFallbacks.get(primaryFontId);
    if (fallbacks?.includes(shardFontId) === true) {
      this.fontFallbacks.set(
        primaryFontId,
        fallbacks.filter((fallbackId) => fallbackId !== shardFontId),
      );
    }
    this.ui._ui_unregister_font_fallback(primaryFontId, shardFontId);
    this.ui._ui_font_loaded(primaryFontId);
  }

  private deactivateSupersededPrimaryShards(primaryFontId: number, familyKey: string, activeShardKey: string): void {
    const activeText = activeShardKey.startsWith(`${familyKey}:`)
      ? activeShardKey.slice(familyKey.length + 1)
      : '';
    if (activeText.length === 0) {
      return;
    }
    const primaryShardKeys = this.shardKeysByPrimaryFont.get(primaryFontId);
    if (primaryShardKeys === undefined) {
      return;
    }
    for (const shardKey of primaryShardKeys) {
      if (shardKey === activeShardKey || !shardKey.startsWith(`${familyKey}:`)) {
        continue;
      }
      const shardText = shardKey.slice(familyKey.length + 1);
      if (shardText.length === 0 || !Array.from(shardText).every((character) => activeText.includes(character))) {
        continue;
      }
      const shard = this.incrementalShardCache.get(shardKey);
      if (shard !== undefined) {
        this.detachPrimaryFontShardReference(primaryFontId, shard.fontId);
      }
    }
  }

  private hasActivePrimaryShardCovering(primaryFontId: number, familyKey: string, shardKey: string): boolean {
    const shardText = shardKey.startsWith(`${familyKey}:`)
      ? shardKey.slice(familyKey.length + 1)
      : '';
    if (shardText.length === 0) {
      return false;
    }
    const primaryShardKeys = this.shardKeysByPrimaryFont.get(primaryFontId);
    if (primaryShardKeys === undefined) {
      return false;
    }
    const activeFallbacks = this.fontFallbacks.get(primaryFontId) ?? [];
    for (const activeShardKey of primaryShardKeys) {
      if (activeShardKey === shardKey || !activeShardKey.startsWith(`${familyKey}:`)) {
        continue;
      }
      const activeShard = this.incrementalShardCache.get(activeShardKey);
      if (activeShard === undefined || !activeFallbacks.includes(activeShard.fontId)) {
        continue;
      }
      const activeText = activeShardKey.slice(familyKey.length + 1);
      if (Array.from(shardText).every((character) => activeText.includes(character))) {
        return true;
      }
    }
    return false;
  }

  private readLiveFallbackFontIds(): Set<number> {
    return withHeapAllocation(this.ui, 4, (allocation) => {
      const ptr = this.ui._ui_get_live_fallback_font_buffer(allocation.ptr);
      const lengthBytes = copyBytesFromHeap(this.ui, allocation.ptr, 4);
      const length = new DataView(lengthBytes.buffer, lengthBytes.byteOffset, lengthBytes.byteLength).getUint32(0, true);
      if (length === 0) {
        return new Set<number>();
      }
      const bytes = copyBytesFromHeap(this.ui, ptr, length * 4);
      const words = new Uint32Array(bytes.buffer, bytes.byteOffset, length);
      return new Set<number>(words);
    });
  }

  private evictIncrementalShard(shardKey: string): void {
    const shard = this.incrementalShardCache.get(shardKey);
    if (shard === undefined) {
      return;
    }
    for (const primaryFontId of shard.primaryFontIds) {
      this.releasePrimaryFontShardReference(primaryFontId, shardKey, shard.fontId);
    }
    this.incrementalShardCache.delete(shardKey);
    this.evictedShardKeys.push(shardKey);
    if (this.evictedShardKeys.length > 32) {
      this.evictedShardKeys.splice(0, this.evictedShardKeys.length - 32);
    }
    this.unregisterFont(shard.fontId);
    this.onCommitFrame();
  }

  private trimShardCacheToPolicyLimit(protectedFontIds: ReadonlySet<number> = new Set<number>()): void {
    const liveFallbackFontIds = this.readLiveFallbackFontIds();
    while (this.incrementalShardCache.size > this.normalizedIncrementalFontPolicy.maxCachedShardFonts) {
      let leastRecentlyUsed: InternalIncrementalShardCacheEntry | null = null;
      for (const shard of this.incrementalShardCache.values()) {
        if (protectedFontIds.has(shard.fontId) || liveFallbackFontIds.has(shard.fontId)) {
          continue;
        }
        if (leastRecentlyUsed === null || shard.lastAccessTick < leastRecentlyUsed.lastAccessTick) {
          leastRecentlyUsed = shard;
        }
      }
      if (leastRecentlyUsed === null) {
        return;
      }
      this.evictIncrementalShard(leastRecentlyUsed.shardKey);
    }
  }

  private ensureIncrementalFontState(fontId: number): InternalIncrementalFontState {
    let state = this.incrementalFontStates.get(fontId);
    if (state !== undefined) {
      return state;
    }
    state = {
      sourceUrl: null,
      sourceState: 'unknown',
      requestedSegmentIds: new Set<string>(),
      pendingSegmentIds: new Set<string>(),
      appliedSegmentIds: new Set<string>(),
      evictedSegmentIds: new Set<string>(),
      requestedCharactersByFamily: new Map<string, Set<string>>(),
      revision: 0,
      blockedPackageIds: new Set<string>(),
      lastBlockedReason: null,
    };
    this.incrementalFontStates.set(fontId, state);
    return state;
  }

  private rememberFontSource(fontId: number, url: string): void {
    const current = this.fontLoadStates.get(fontId);
    if (current?.url === url && (current.state === 'loading' || current.state === 'loaded')) {
      return;
    }
    this.fontLoadStates.set(fontId, { url, state: 'known' });
    const state = this.ensureIncrementalFontState(fontId);
    const previousUrl = state.sourceUrl;
    state.sourceUrl = url;
    if (state.sourceState === 'unknown' || state.sourceState === 'failed' || previousUrl !== url) {
      state.sourceState = 'known';
    }
  }

  private markFontLoading(fontId: number, url: string): void {
    this.fontLoadStates.set(fontId, { url, state: 'loading' });
    const state = this.ensureIncrementalFontState(fontId);
    state.sourceUrl = url;
    state.sourceState = 'loading';
  }

  private markFontLoaded(fontId: number, url: string): void {
    this.fontLoadStates.set(fontId, { url, state: 'loaded' });
    const state = this.ensureIncrementalFontState(fontId);
    state.sourceUrl = url;
    state.sourceState = 'loaded';
  }

  private markFontLoadFailed(fontId: number, url: string): void {
    this.fontLoadStates.set(fontId, { url, state: 'failed' });
    const state = this.ensureIncrementalFontState(fontId);
    state.sourceUrl = url;
    state.sourceState = 'failed';
  }

  public isFontLoaded(fontId: number, url?: string): boolean {
    const current = this.fontLoadStates.get(fontId);
    return current?.state === 'loaded'
      && (url === undefined || current.url === url);
  }

  private async fetchFontBytes(url: string, replay = false): Promise<Uint8Array> {
    const response = await this.host.loadBytes(url);
    if (!response.ok) {
      const verb = replay ? 'refetch' : 'fetch';
      throw new Error(`Failed to ${verb} font ${url}: ${String(response.status)}`);
    }
    return await response.bytes();
  }

  private reportNonFatalFontError(context: string, error: unknown): void {
    const message = error instanceof Error ? error.message : String(error);
    console.error(`[fui_host] ${context}: ${message}`);
  }

  private applyFontFallbacks(fontId: number, fallbackIds: readonly number[]): void {
    this.fontFallbacks.set(fontId, [...fallbackIds]);
    for (const fallbackId of fallbackIds) {
      this.ui._ui_register_font_fallback(fontId, fallbackId);
    }
    this.ui._ui_font_loaded(fontId);
    this.host.notifyFontLoaded(fontId);
  }

  private async ensureRegisteredFont(
    fontId: number,
    url: string,
    fallbackIds: readonly number[] = [],
  ): Promise<void> {
    this.rememberFontSource(fontId, url);
    const pending = this.pendingFontLoads.get(fontId);
    if (pending !== undefined) {
      await pending;
      return;
    }
    const request = this.registerFont(
      fallbackIds.length > 0
        ? {
            id: fontId,
            url,
            fallbackIds,
          }
        : {
            id: fontId,
            url,
          },
    ).finally(() => {
      this.pendingFontLoads.delete(fontId);
    });
    this.pendingFontLoads.set(fontId, request);
    await request;
  }

  private async ensureFontWithFallbacks(fontId: number, visited = new Set<number>()): Promise<void> {
    if (visited.has(fontId)) {
      return;
    }
    visited.add(fontId);
    if (!this.isFontLoaded(fontId)) {
      const lazyUrl = this.lazyFontSources.get(fontId);
      if (lazyUrl !== undefined) {
        await this.ensureRegisteredFont(fontId, lazyUrl);
      } else {
        const builtInFont = getBuiltInBridgeFont(fontId);
        if (builtInFont !== undefined) {
          await this.ensureRegisteredFont(
            builtInFont.id,
            getBridgeAssetUrl(builtInFont.assetFile),
            builtInFont.fallbackIds,
          );
        }
      }
    }
    const fallbackIds = this.fontFallbacks.get(fontId) ?? [];
    await Promise.all(fallbackIds.map((fallbackId) => this.ensureFontWithFallbacks(fallbackId, visited)));
  }

  private registerSingleFontFallback(fontId: number, fallbackFontId: number, notifyApp = true): void {
    const existingFallbacks = this.fontFallbacks.get(fontId) ?? [];
    if (!existingFallbacks.includes(fallbackFontId)) {
      this.fontFallbacks.set(fontId, [...existingFallbacks, fallbackFontId]);
    }
    this.ui._ui_register_font_fallback(fontId, fallbackFontId);
    this.ui._ui_font_loaded(fontId);
    if (notifyApp) {
      this.host.notifyFontLoaded(fontId);
    }
  }

  private async ensureIncrementalShardFont(
    request: ResolvedIncrementalFontShardRequest & { readonly shardKey: string },
  ): Promise<number> {
    const cachedShard = this.incrementalShardCache.get(request.shardKey);
    if (cachedShard !== undefined) {
      this.updateShardAccess(request.shardKey);
      return cachedShard.fontId;
    }
    const existing = this.pendingIncrementalShardLoads.get(request.shardKey);
    if (existing !== undefined) {
      return await existing;
    }
    const pending = (async () => {
      const fontId = this.nextIncrementalFontId;
      this.nextIncrementalFontId += 1;
      const shard = await fetchGoogleFontShardBytes(request.googleFamily, request.text);
      this.registerFontBytes(fontId, shard.bytes);
      this.loadedFonts.set(fontId, shard.url);
      this.markFontLoaded(fontId, shard.url);
      this.shardAccessTick += 1;
      this.incrementalShardCache.set(request.shardKey, {
        shardKey: request.shardKey,
        fontId,
        sizeBytes: shard.bytes.byteLength,
        primaryFontIds: new Set<number>(),
        lastAccessTick: this.shardAccessTick,
      });
      return fontId;
    })().finally(() => {
      this.pendingIncrementalShardLoads.delete(request.shardKey);
    });
    this.pendingIncrementalShardLoads.set(request.shardKey, pending);
    return await pending;
  }

  private async flushPendingIncrementalFamilyRequest(primaryFontId: number, familyKey: string): Promise<void> {
    const pendingKey = `${String(primaryFontId)}:${familyKey}`;
    this.scheduledIncrementalFamilyFlushes.delete(pendingKey);
    const pendingRequest = this.pendingIncrementalFamilyRequests.get(pendingKey);
    if (pendingRequest === undefined) {
      return;
    }
    this.pendingIncrementalFamilyRequests.delete(pendingKey);

    const fontState = this.ensureIncrementalFontState(primaryFontId);
    const requestedCharacters = fontState.requestedCharactersByFamily.get(pendingRequest.familyKey);
    const text = Array.from((requestedCharacters ?? pendingRequest.characters).values()).join('');
    if (text.length === 0) {
      return;
    }
    const shardKey = `${pendingRequest.familyKey}:${text}`;
    fontState.requestedSegmentIds.add(shardKey);
    fontState.pendingSegmentIds.add(shardKey);
    this.logs.incrementalFontPackageRequests.push({
      primaryFontId,
      coverageKind: pendingRequest.coverageKind,
      packageId: pendingRequest.packageId,
      segmentIds: [shardKey],
      sampleText: text,
    });

    try {
      const shardFontId = await this.ensureIncrementalShardFont({
        packageId: pendingRequest.packageId,
        coverageKind: pendingRequest.coverageKind,
        familyKey: pendingRequest.familyKey,
        googleFamily: pendingRequest.googleFamily,
        text,
        shardKey,
      });
      if (this.hasActivePrimaryShardCovering(primaryFontId, pendingRequest.familyKey, shardKey)) {
        fontState.pendingSegmentIds.delete(shardKey);
        this.trimShardCacheToPolicyLimit();
        return;
      }
      this.registerSingleFontFallback(primaryFontId, shardFontId, false);
      const shard = this.incrementalShardCache.get(shardKey);
      if (shard !== undefined) {
        shard.primaryFontIds.add(primaryFontId);
        this.updateShardAccess(shardKey);
      }
      let primaryShardKeys = this.shardKeysByPrimaryFont.get(primaryFontId);
      if (primaryShardKeys === undefined) {
        primaryShardKeys = new Set<string>();
        this.shardKeysByPrimaryFont.set(primaryFontId, primaryShardKeys);
      }
      primaryShardKeys.add(shardKey);
      this.deactivateSupersededPrimaryShards(primaryFontId, pendingRequest.familyKey, shardKey);
      if (!fontState.appliedSegmentIds.has(shardKey)) {
        fontState.appliedSegmentIds.add(shardKey);
        fontState.revision += 1;
      }
      fontState.pendingSegmentIds.delete(shardKey);
      fontState.evictedSegmentIds.delete(shardKey);
      this.host.notifyFontLoaded(primaryFontId);
      this.trimShardCacheToPolicyLimit(new Set<number>([shardFontId]));
      this.onCommitFrame();
    } catch (error: unknown) {
      fontState.requestedSegmentIds.delete(shardKey);
      fontState.pendingSegmentIds.delete(shardKey);
      const requestedCharacters = fontState.requestedCharactersByFamily.get(pendingRequest.familyKey);
      if (requestedCharacters !== undefined) {
        for (const character of text) {
          requestedCharacters.delete(character);
        }
        if (requestedCharacters.size === 0) {
          fontState.requestedCharactersByFamily.delete(pendingRequest.familyKey);
        }
      }
      this.reportNonFatalFontError(
        `incremental font shard fetch failed for ${pendingRequest.googleFamily} (${shardKey})`,
        error,
      );
      throw error;
    }
  }

  public async ensureFont(fontId: number): Promise<void> {
    await this.ensureFontWithFallbacks(fontId);
  }

  public async ensureBuiltInFont(fontId: number): Promise<void> {
    const builtInFont = getBuiltInBridgeFont(fontId);
    if (builtInFont === undefined) {
      throw new Error(`Unknown built-in font id ${String(fontId)}.`);
    }
    await this.ensureRegisteredFont(
      builtInFont.id,
      getBridgeAssetUrl(builtInFont.assetFile),
      builtInFont.fallbackIds,
    );
  }

  public getIncrementalFontCacheState(): IncrementalFontCacheState {
    return {
      maxCachedShardFonts: this.incrementalFontPolicy.maxCachedShardFonts,
      cachedShardCount: this.incrementalShardCache.size,
      cachedShardKeys: Array.from(this.incrementalShardCache.keys()),
      evictedShardKeys: [...this.evictedShardKeys],
    };
  }

  public getIncrementalFontPolicy(): IncrementalFontPolicy {
    return {
      autoGrow: this.incrementalFontPolicy.autoGrow,
      maxCachedShardFonts: this.incrementalFontPolicy.maxCachedShardFonts,
      allowedFontIds: this.incrementalFontPolicy.allowedFontIds === null
        ? null
        : [...this.incrementalFontPolicy.allowedFontIds],
      blockedPackageIds: this.incrementalFontPolicy.blockedPackageIds === null
        ? null
        : [...this.incrementalFontPolicy.blockedPackageIds],
    };
  }

  public setIncrementalFontPolicy(policy: Partial<IncrementalFontPolicy>): void {
    const nextPolicy: IncrementalFontPolicy = {
      autoGrow: policy.autoGrow ?? this.incrementalFontPolicy.autoGrow,
      maxCachedShardFonts: policy.maxCachedShardFonts ?? this.incrementalFontPolicy.maxCachedShardFonts,
      allowedFontIds: 'allowedFontIds' in policy
        ? (policy.allowedFontIds ?? null)
        : this.incrementalFontPolicy.allowedFontIds,
      blockedPackageIds: 'blockedPackageIds' in policy
        ? (policy.blockedPackageIds ?? null)
        : this.incrementalFontPolicy.blockedPackageIds,
    };
    this.normalizedIncrementalFontPolicy = this.normalizePolicy(nextPolicy);
    this.incrementalFontPolicy = {
      ...nextPolicy,
      maxCachedShardFonts: this.normalizedIncrementalFontPolicy.maxCachedShardFonts,
    };
    this.trimShardCacheToPolicyLimit();
  }

  public getIncrementalFontState(fontId: number): IncrementalFontRuntimeState | null {
    const state = this.incrementalFontStates.get(fontId);
    if (state === undefined) {
      return null;
    }
    return {
      fontId,
      sourceUrl: state.sourceUrl,
      sourceState: state.sourceState,
      loaded: this.isFontLoaded(fontId),
      requestedSegmentIds: Array.from(state.requestedSegmentIds.values()),
      pendingSegmentIds: Array.from(state.pendingSegmentIds.values()),
      appliedSegmentIds: Array.from(state.appliedSegmentIds.values()),
      evictedSegmentIds: Array.from(state.evictedSegmentIds.values()),
      revision: state.revision,
      autoGrowAllowed: this.getAutoGrowBlockReason(fontId, null) === null,
      blockedPackageIds: Array.from(state.blockedPackageIds.values()),
      lastBlockedReason: state.lastBlockedReason,
    };
  }

  public getClipboardFontUrl(fontId: number): string | null {
    const loadedUrl = this.loadedFonts.get(fontId);
    if (loadedUrl !== undefined) {
      return loadedUrl;
    }
    const trackedUrl = this.fontLoadStates.get(fontId)?.url;
    if (trackedUrl !== undefined) {
      return trackedUrl;
    }
    const lazyUrl = this.lazyFontSources.get(fontId);
    if (lazyUrl !== undefined) {
      return lazyUrl;
    }
    const builtInFont = getBuiltInBridgeFont(fontId);
    return builtInFont === undefined ? null : getBridgeAssetUrl(builtInFont.assetFile);
  }

  public registerLazyFont(fontId: number, url: string): void {
    this.lazyFontSources.set(fontId, url);
    this.rememberFontSource(fontId, url);
    if (this.loadedFonts.get(fontId) !== url) {
      this.loadedFonts.delete(fontId);
    }
  }

  public registerFontFallback(fontId: number, fallbackFontId: number): void {
    this.registerSingleFontFallback(fontId, fallbackFontId);
  }

  public handleMissingFontCoverage(fontId: number, coverageKind: number, sampleText: string): void {
    const fontState = this.ensureIncrementalFontState(fontId);
    const resolvedRequests = resolveIncrementalFontPackageRequests(fontId, coverageKind, sampleText);
    for (const request of resolvedRequests) {
      const blockReason = this.getAutoGrowBlockReason(fontId, request.packageId);
      if (blockReason !== null) {
        fontState.blockedPackageIds.add(request.packageId);
        fontState.lastBlockedReason = blockReason;
        continue;
      }
      fontState.blockedPackageIds.delete(request.packageId);
      if (fontState.blockedPackageIds.size === 0) {
        fontState.lastBlockedReason = null;
      }
      let requestedCharacters = fontState.requestedCharactersByFamily.get(request.familyKey);
      if (requestedCharacters === undefined) {
        requestedCharacters = new Set<string>();
        fontState.requestedCharactersByFamily.set(request.familyKey, requestedCharacters);
      }
      const pendingKey = `${String(fontId)}:${request.familyKey}`;
      let pendingRequest = this.pendingIncrementalFamilyRequests.get(pendingKey);
      if (pendingRequest === undefined) {
        pendingRequest = {
          packageId: request.packageId,
          coverageKind: request.coverageKind,
          familyKey: request.familyKey,
          googleFamily: request.googleFamily,
          characters: new Set<string>(),
        };
        this.pendingIncrementalFamilyRequests.set(pendingKey, pendingRequest);
      }
      let hasNovelCharacter = false;
      for (const character of request.text) {
        if (requestedCharacters.has(character)) {
          continue;
        }
        requestedCharacters.add(character);
        pendingRequest.characters.add(character);
        hasNovelCharacter = true;
      }
      if (!hasNovelCharacter || this.scheduledIncrementalFamilyFlushes.has(pendingKey)) {
        continue;
      }
      this.scheduledIncrementalFamilyFlushes.add(pendingKey);
      queueMicrotask(() => {
        void this.flushPendingIncrementalFamilyRequest(fontId, request.familyKey).catch(() => undefined);
      });
    }
  }

  public async loadFont(fontId: number, url: string): Promise<void> {
    if (this.isFontLoaded(fontId, url)) {
      return;
    }
    this.rememberFontSource(fontId, url);
    this.markFontLoading(fontId, url);
    try {
      const bytes = await this.fetchFontBytes(url);
      this.registerFontBytes(fontId, bytes);
      this.loadedFonts.set(fontId, url);
      this.markFontLoaded(fontId, url);
      this.onCommitFrame();
    } catch (error) {
      if (this.loadedFonts.get(fontId) === url) {
        this.loadedFonts.delete(fontId);
      }
      this.markFontLoadFailed(fontId, url);
      throw error;
    }
  }

  public async registerFont(font: BridgeFontRegistration): Promise<void> {
    if (!this.isFontLoaded(font.id, font.url)) {
      await this.loadFont(font.id, font.url);
    }
    if (font.fallbackIds !== undefined) {
      this.applyFontFallbacks(font.id, font.fallbackIds);
    }
    this.onCommitFrame();
  }

  public async registerFontStack(stack: BridgeFontStackRegistration): Promise<void> {
    const fallbackFonts = stack.fallbacks ?? [];
    await Promise.all([
      this.loadFont(stack.primary.id, stack.primary.url),
      ...fallbackFonts.map((fallback) => this.loadFont(fallback.id, fallback.url)),
    ]);
    this.applyFontFallbacks(
      stack.primary.id,
      fallbackFonts.map((fallback) => fallback.id),
    );
    this.onCommitFrame();
  }

  public async replayLoadedFonts(): Promise<void> {
    const fontReloads = Array.from(this.loadedFonts.entries(), async ([fontId, url]) => {
      const bytes = await this.fetchFontBytes(url, true);
      this.registerFontBytes(fontId, bytes);
    });
    await Promise.all(fontReloads);
    for (const [fontId, fallbackIds] of this.fontFallbacks.entries()) {
      for (const fallbackId of fallbackIds) {
        this.ui._ui_register_font_fallback(fontId, fallbackId);
      }
    }
  }
}
