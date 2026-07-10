import type { OpenCanvasFindMatch,OpenCanvasFindState,OpenCanvasTextDocument,UiModule } from '../../core-types';
import { FindOnPageProjector,type ResolvedFindSelection } from '../../find-on-page';
import { DEFAULT_OPEN_CANVAS_FIND_OPTIONS,normalizeOpenCanvasFindOptions } from '../find-session';
import type { TextDocumentController } from './text-documents';

const OPEN_CANVAS_FIND_BACKGROUND_COLOR = 0xFFEB3B38;

interface FindControllerOptions {
  readonly canvas: HTMLCanvasElement;
  readonly ui: UiModule;
  readonly textDocuments: TextDocumentController;
  readonly commitFrame: () => void;
  readonly flushPendingCommit: () => Uint32Array | null;
}

export class FindController {
  private readonly findProjector: FindOnPageProjector;
  private activeFindMatch: OpenCanvasFindMatch | null = null;
  private activeFindState: OpenCanvasFindState | null = null;
  private selectionPollTimer: ReturnType<typeof setInterval> | null = null;
  private selectionClearTimer: number | null = null;
  private readonly findDialogObserver: MutationObserver | null = null;

  public constructor(private readonly options: FindControllerOptions) {
    const runtimeCanvasId = `ed-canvas-${Math.random().toString(36).slice(2, 10)}`;
    this.findProjector = new FindOnPageProjector(options.canvas, runtimeCanvasId);
    window.__bridgeFindMatch = null;
    window.__bridgeFindState = null;

    const handleDocumentSelectionChange = () => {
      this.handleSelectionChange();
    };
    document.addEventListener('selectionchange', handleDocumentSelectionChange);
    const handleVisibilityChange = () => {
      this.syncSelectionPolling();
    };
    document.addEventListener('visibilitychange', handleVisibilityChange);
    this.findDialogObserver = typeof MutationObserver === 'undefined'
      ? null
      : new MutationObserver(() => {
          this.syncSelectionPolling();
        });
    this.findDialogObserver?.observe(document.documentElement, {
      subtree: true,
      childList: true,
      attributes: true,
      attributeFilter: ['data-ed-open'],
    });
    this.syncSelectionPolling();

    this.destroy = (() => {
      this.stopSelectionPolling();
      this.cancelPendingFindClear();
      document.removeEventListener('selectionchange', handleDocumentSelectionChange);
      document.removeEventListener('visibilitychange', handleVisibilityChange);
      this.findDialogObserver?.disconnect();
      this.storeFindState(null);
      this.storeFindMatch(null);
      this.findProjector.destroy();
    }).bind(this);
  }

  public syncSize(logicalWidth: number, logicalHeight: number): void {
    this.findProjector.syncSize(logicalWidth, logicalHeight);
  }

  public syncViewportTransform(scale: number, offsetX: number, offsetY: number): void {
    this.findProjector.syncViewportTransform(scale, offsetX, offsetY);
  }

  public syncDocuments(): void {
    this.findProjector.update(this.options.textDocuments.readFindDocuments());
  }

  public getFindDocuments(): readonly OpenCanvasTextDocument[] {
    return this.options.textDocuments.readFindDocuments().map((document) => ({ ...document }));
  }

  public getFindState(): OpenCanvasFindState | null {
    return this.activeFindState === null
      ? null
      : {
          query: this.activeFindState.query,
          options: { ...this.activeFindState.options },
          matches: this.activeFindState.matches.map((match) => ({ ...match })),
          activeMatchIndex: this.activeFindState.activeMatchIndex,
        };
  }

  public activateFindMatch(match: OpenCanvasFindMatch | null, reveal = true): boolean {
    return this.applyRetainedFindMatch(match, reveal);
  }

  public setFindState(state: OpenCanvasFindState | null, revealActive = false): boolean {
    return this.applyRetainedFindState(state, revealActive);
  }

  public syncFindSelection(clearOnMissing = false): boolean {
    return this.handleSelectionChange(clearOnMissing);
  }

  public clearFindMatch(): boolean {
    this.findProjector.selectMatch(null);
    return this.applyRetainedFindMatch(null, false);
  }

  public destroy(): void {
    this.cancelPendingFindClear();
  }

  private isBridgeFindDialogOpen(): boolean {
    return document.querySelector('[data-ed-find-dialog="1"][data-ed-open="1"]') !== null;
  }

  private stopSelectionPolling(): void {
    if (this.selectionPollTimer === null) {
      return;
    }
    clearInterval(this.selectionPollTimer);
    this.selectionPollTimer = null;
  }

  private syncSelectionPolling(): void {
    if (document.hidden || !this.isBridgeFindDialogOpen()) {
      this.stopSelectionPolling();
      return;
    }
    if (this.selectionPollTimer !== null) {
      return;
    }
    this.selectionPollTimer = setInterval(() => {
      if (document.hidden || !this.isBridgeFindDialogOpen()) {
        this.stopSelectionPolling();
        return;
      }
      this.handleSelectionChange();
    }, 100);
  }

  private cancelPendingFindClear(): void {
    if (this.selectionClearTimer !== null) {
      clearTimeout(this.selectionClearTimer);
      this.selectionClearTimer = null;
    }
  }

  private schedulePendingFindClear(): void {
    this.cancelPendingFindClear();
    this.selectionClearTimer = window.setTimeout(() => {
      this.selectionClearTimer = null;
      if (this.isBridgeFindDialogOpen()) {
        return;
      }
      if (this.findProjector.resolveSelection(document.getSelection()) !== null) {
        return;
      }
      if (this.activeFindMatch !== null) {
        this.applyRetainedFindState(null, false);
      }
    }, 150);
  }

  private storeFindMatch(match: ResolvedFindSelection | null): void {
    this.activeFindMatch = match === null ? null : { ...match };
    window.__bridgeFindMatch = this.activeFindMatch;
  }

  private storeFindState(state: OpenCanvasFindState | null): void {
    this.activeFindState = state === null
      ? null
      : {
          query: state.query,
          options: { ...state.options },
          matches: state.matches.map((match) => ({ ...match })),
          activeMatchIndex: state.activeMatchIndex,
        };
    window.__bridgeFindState = this.activeFindState;
  }

  private applyRetainedFindState(
    state: OpenCanvasFindState | null,
    revealActive: boolean,
  ): boolean {
    if (state === null) {
      this.options.ui._ui_clear_text_find_highlights();
      this.options.ui._ui_clear_text_find_match();
      this.options.commitFrame();
      this.options.flushPendingCommit();
      this.storeFindState(null);
      this.storeFindMatch(null);
      return true;
    }

    const normalizedOptions = normalizeOpenCanvasFindOptions(state.options);
    if (
      state.activeMatchIndex < -1 ||
      state.activeMatchIndex >= state.matches.length ||
      (state.matches.length === 0 && state.activeMatchIndex !== -1)
    ) {
      return false;
    }

    const normalizedMatches: { readonly handleArg: number | bigint; readonly match: ResolvedFindSelection }[] = [];
    for (const match of state.matches) {
      const range = this.options.textDocuments.resolveTextRange(match.handle, match.start, match.end);
      if (range === null) {
        return false;
      }
      normalizedMatches.push({
        handleArg: range.handleArg,
        match: {
          handle: match.handle,
          start: range.start,
          end: range.end,
        },
      });
    }

    this.options.ui._ui_clear_text_find_highlights();
    if (normalizedOptions.highlightAll) {
      for (let index = 0; index < normalizedMatches.length; index += 1) {
        if (index === state.activeMatchIndex) {
          continue;
        }
        const entry = normalizedMatches[index];
        if (entry === undefined) {
          continue;
        }
        if (
          this.options.ui._ui_push_text_find_highlight(
            entry.handleArg,
            entry.match.start,
            entry.match.end,
            OPEN_CANVAS_FIND_BACKGROUND_COLOR,
          ) === 0
        ) {
          this.options.ui._ui_clear_text_find_highlights();
          return false;
        }
      }
    }

    const activeEntry = state.activeMatchIndex >= 0
      ? normalizedMatches[state.activeMatchIndex] ?? null
      : null;
    if (activeEntry === null) {
      this.options.ui._ui_clear_text_find_match();
    } else {
      if (this.options.ui._ui_set_text_find_match(activeEntry.handleArg, activeEntry.match.start, activeEntry.match.end) === 0) {
        this.options.ui._ui_clear_text_find_highlights();
        return false;
      }
      if (
        revealActive &&
        this.options.ui._ui_reveal_text_range(activeEntry.handleArg, activeEntry.match.start, activeEntry.match.end) === 0
      ) {
        this.options.ui._ui_clear_text_find_highlights();
        return false;
      }
    }

    this.options.commitFrame();
    this.options.flushPendingCommit();
    this.storeFindState({
      query: state.query,
      options: normalizedOptions,
      matches: normalizedMatches.map((entry) => ({ ...entry.match })),
      activeMatchIndex: activeEntry === null ? -1 : state.activeMatchIndex,
    });
    this.storeFindMatch(activeEntry?.match ?? null);
    return true;
  }

  private applyRetainedFindMatch(
    match: ResolvedFindSelection | null,
    reveal: boolean,
  ): boolean {
    return this.applyRetainedFindState(
      match === null
        ? null
        : {
            query: '',
            options: { ...DEFAULT_OPEN_CANVAS_FIND_OPTIONS },
            matches: [{ ...match }],
            activeMatchIndex: 0,
          },
      reveal,
    );
  }

  private handleSelectionChange(clearOnMissing = false): boolean {
    const match = this.findProjector.resolveSelection(document.getSelection());
    if (match === null) {
      if (clearOnMissing && !this.isBridgeFindDialogOpen() && this.activeFindMatch !== null) {
        this.schedulePendingFindClear();
      }
      return false;
    }
    this.cancelPendingFindClear();
    if (
      this.activeFindMatch !== null &&
      this.activeFindMatch.handle === match.handle &&
      this.activeFindMatch.start === match.start &&
      this.activeFindMatch.end === match.end
    ) {
      return true;
    }
    return this.applyRetainedFindMatch(match, true);
  }
}
