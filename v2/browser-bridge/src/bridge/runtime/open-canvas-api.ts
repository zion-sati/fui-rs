import { cloneSemanticTree } from '../../semantic';
import type {
  OpenCanvasApi,
  OpenCanvasFindOptions,
  UiModule,
} from '../../core-types';
import { findTextInOpenCanvasDocuments } from '../find-session';
import type { BridgeInteractionState } from '../local-types';
import type { DebugTreeSnapshot } from '../../debug-tree';
import type { FindController } from './find-controller';
import type { SemanticController } from './semantic-controller';
import type { TextDocumentController } from './text-documents';
import { utf8ByteLength } from '../interaction/text-encoding';
import {
  buildForms,
  resolveFormHandle,
  resolveTextInputMetadata,
  resolveStableFieldName,
} from './editable-form-model';

interface OpenCanvasApiAdapterOptions {
  readonly ui: UiModule;
  readonly semantic: SemanticController;
  readonly find: FindController;
  readonly textDocuments: TextDocumentController;
  readonly interactionState: BridgeInteractionState;
  readonly getDebugTree: () => DebugTreeSnapshot;
  readonly getTextInputMetadata: (handle: string) => { readonly kind: 'text' | 'password' | 'email'; readonly hostAutofillHint: string | null } | null;
  readonly commitFrame: () => void;
  readonly flushPendingCommit: () => Uint32Array | null;
}

export class OpenCanvasApiAdapter {
  private readonly api: OpenCanvasApi;

  public constructor(private readonly options: OpenCanvasApiAdapterOptions) {
    this.api = {
      getSemanticTree: () => cloneSemanticTree(this.options.semantic.getSemanticTree()),
      getForms: () => {
        const semanticTree = this.options.semantic.getSemanticTree();
        const debugTree = this.options.getDebugTree();
        return buildForms(debugTree, semanticTree);
      },
      getForm: (handle: string) => {
        const semanticTree = this.options.semantic.getSemanticTree();
        const debugTree = this.options.getDebugTree();
        const forms = buildForms(debugTree, semanticTree);
        return forms.find((form) => form.handle === handle) ?? null;
      },
      getFocusedHandle: () => this.options.interactionState.getFocusedHandle(),
      getActiveTextHandle: () => this.options.interactionState.getActiveTextHandle()?.toString() ?? null,
      getBoundingBox: (handle: string) => this.options.semantic.getBoundingBox(handle),
      getTextVisibleBounds: (handle: string) => {
        const bounds = this.options.textDocuments.readVisibleTextBounds(handle);
        return bounds === null ? null : { ...bounds };
      },
      getTextDocument: (handle: string) => {
        const snapshot = this.options.textDocuments.readTextDocumentSnapshot(handle);
        return snapshot === null ? null : { ...snapshot.document };
      },
      getEditableTextDocument: (handle: string) => {
        const semanticTree = this.options.semantic.getSemanticTree();
        const semantic = semanticTree.find((entry) => entry.handle === handle);
        const debugTree = this.options.getDebugTree();
        const debugNode = debugTree.nodesByHandle[handle];
        if (debugNode?.behavior.textEditor !== true) {
          return null;
        }
        const snapshot = this.options.textDocuments.readTextDocumentSnapshot(handle);
        const text = this.options.interactionState.textByHandle[handle] ?? snapshot?.document.text ?? '';
        const semanticByHandle = new Map(semanticTree.map((node) => [node.handle, node]));
        const metadata = resolveTextInputMetadata(debugTree, handle, this.options.getTextInputMetadata);
        const byteLength = this.options.interactionState.textByHandle[handle] === undefined
          ? snapshot?.byteLength ?? utf8ByteLength(text)
          : utf8ByteLength(text);
        const selection = this.options.interactionState.selectionsByHandle[handle] ?? { start: byteLength, end: byteLength };
        return {
          handle,
          text,
          selectionStart: selection.start,
          selectionEnd: selection.end,
          multiline: semantic?.state.multiline === true,
          readOnly: !debugNode.behavior.editable,
          disabled: semantic?.state.disabled === true,
          kind: metadata?.kind ?? 'text',
          autofillHint: metadata?.hostAutofillHint ?? null,
          stableFieldName: resolveStableFieldName(debugTree, handle),
          formHandle: resolveFormHandle(debugTree, semanticByHandle, handle),
        };
      },
      getRangeRects: (handle: string, start: number, end: number) =>
        this.options.textDocuments.readRangeRects(handle, start, end).map((rect) => ({ ...rect })),
      findText: (query: string, options?: OpenCanvasFindOptions) => {
        const results = findTextInOpenCanvasDocuments(this.options.textDocuments.readFindDocuments(), query, options);
        return {
          query: results.query,
          options: { ...results.options },
          matches: results.matches.map((match) => ({ ...match })),
        };
      },
      setFindState: (state, revealActive = false) => this.options.find.setFindState(state, revealActive),
      getFindState: () => this.options.find.getFindState(),
      setFindMatch: (match) => this.options.find.activateFindMatch(match, false),
      revealRange: (handle: string, start: number, end: number) => {
        const range = this.options.textDocuments.resolveTextRange(handle, start, end);
        if (range === null) {
          return false;
        }
        if (this.options.ui._ui_reveal_text_range(range.handleArg, range.start, range.end) === 0) {
          return false;
        }
        this.options.commitFrame();
        this.options.flushPendingCommit();
        return true;
      },
    };
    window.__OPEN_CANVAS_API__ = this.api;
  }

  public getApi(): OpenCanvasApi {
    return this.api;
  }

  public destroy(): void {
    delete window.__OPEN_CANVAS_API__;
  }
}
