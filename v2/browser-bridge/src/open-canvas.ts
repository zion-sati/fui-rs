/**
 * Opaque host-side identity for a retained node or document snapshot.
 *
 * The runtime owns handles as 64-bit values. The browser bridge exposes them as strings so:
 * - JavaScript never loses precision on wasm64 / native 64-bit handles
 * - extensions and hosts can persist and compare identities without reaching into wasm internals
 *
 * A handle is:
 * - stable for the lifetime of the realized retained node
 * - local to the current runtime instance
 * - not a DOM node id, CSS selector, or application-level business id
 */
export type OpenCanvasHandle = string;

export interface SemanticBounds {
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
}

export interface SemanticState {
  readonly checked?: 'false' | 'true' | 'mixed';
  readonly selected?: boolean;
  readonly expanded?: boolean;
  readonly disabled?: boolean;
  readonly readonly?: boolean;
  readonly multiline?: boolean;
  readonly orientation?: 'horizontal' | 'vertical';
  readonly valueNow?: number;
  readonly valueMin?: number;
  readonly valueMax?: number;
}

/**
 * Flattened semantic snapshot of one retained node as exposed to hosts.
 *
 * This is intentionally a bridge-friendly read model rather than a DOM clone:
 * - `role` is the engine enum value
 * - `roleName` is the host-readable role label
 * - `handle` is the retained node identity token
 * - `bounds` are logical canvas bounds for the realized node snapshot
 * - `label` is the semantic label/text payload currently exposed for this node
 * - `state` carries role-relevant flags without forcing consumers to inspect bridge DOM internals
 */
export interface SemanticNode {
  readonly role: number;
  readonly roleName: string;
  readonly handle: OpenCanvasHandle;
  readonly bounds: SemanticBounds;
  readonly label: string;
  readonly state: SemanticState;
}

export interface OpenCanvasTextDocument {
  readonly handle: OpenCanvasHandle;
  readonly text: string;
}

export type OpenCanvasAutofillHint = string | null;

export type OpenCanvasEditableTextKind =
  | 'text'
  | 'password'
  | 'email';

export type OpenCanvasFormPurpose =
  | 'generic'
  | 'sign-in'
  | 'sign-up'
  | 'change-password';

export interface OpenCanvasForm {
  readonly handle: OpenCanvasHandle;
  readonly stableName: string | null;
  readonly purpose: OpenCanvasFormPurpose;
  readonly fieldHandles: readonly OpenCanvasHandle[];
  readonly submitHandle: OpenCanvasHandle | null;
}

export interface OpenCanvasEditableTextDocument extends OpenCanvasTextDocument {
  readonly selectionStart: number;
  readonly selectionEnd: number;
  readonly multiline: boolean;
  readonly readOnly: boolean;
  readonly disabled: boolean;
  readonly kind: OpenCanvasEditableTextKind;
  readonly autofillHint: OpenCanvasAutofillHint;
  readonly stableFieldName: string | null;
  readonly formHandle: OpenCanvasHandle | null;
}

export interface OpenCanvasFindMatch {
  readonly handle: OpenCanvasHandle;
  readonly start: number;
  readonly end: number;
}

export interface OpenCanvasFindOptions {
  readonly highlightAll?: boolean;
  readonly matchCase?: boolean;
  readonly matchDiacritics?: boolean;
  readonly wholeWords?: boolean;
}

export interface OpenCanvasResolvedFindOptions {
  readonly highlightAll: boolean;
  readonly matchCase: boolean;
  readonly matchDiacritics: boolean;
  readonly wholeWords: boolean;
}

export interface OpenCanvasFindResults {
  readonly query: string;
  readonly options: OpenCanvasResolvedFindOptions;
  readonly matches: readonly OpenCanvasFindMatch[];
}

export interface OpenCanvasFindState extends OpenCanvasFindResults {
  readonly activeMatchIndex: number;
}

/**
 * Current shipped browser-host contract.
 *
 * This interface only covers the runtime surface that is actually wired today.
 * Proposed mutation and callback/event contracts are documented in
 * `docs/v2/browser-bridge/OPEN_CANVAS_API.md` until they are implemented.
 */
export interface OpenCanvasApi {
  getSemanticTree(): SemanticNode[];
  getForms(): OpenCanvasForm[];
  getForm(handle: OpenCanvasHandle): OpenCanvasForm | null;
  getFocusedHandle(): OpenCanvasHandle | null;
  getActiveTextHandle(): OpenCanvasHandle | null;
  getBoundingBox(handle: OpenCanvasHandle): SemanticBounds | null;
  getTextVisibleBounds(handle: OpenCanvasHandle): SemanticBounds | null;
  getTextDocument(handle: OpenCanvasHandle): OpenCanvasTextDocument | null;
  getEditableTextDocument(handle: OpenCanvasHandle): OpenCanvasEditableTextDocument | null;
  getRangeRects(handle: OpenCanvasHandle, start: number, end: number): readonly SemanticBounds[];
  findText(query: string, options?: OpenCanvasFindOptions): OpenCanvasFindResults;
  setFindState(state: OpenCanvasFindState | null, revealActive?: boolean): boolean;
  getFindState(): OpenCanvasFindState | null;
  setFindMatch(match: OpenCanvasFindMatch | null): boolean;
  revealRange(handle: OpenCanvasHandle, start: number, end: number): boolean;
}
