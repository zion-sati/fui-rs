import type { BridgeLoaderInfo, BridgeLogs } from '../core-types';
import type { HiddenTextEditor } from './interaction/editor-model';

export type WasmArchitecture = 'auto' | 'wasm32' | 'wasm32-simd' | 'wasm64' | 'wasm64-simd';
export type RequestedRendererBackend = 'auto' | 'webgpu' | 'webgl2' | 'cpu';

export interface BundleAssetDescriptor {
  readonly js: string;
  readonly js_integrity?: string | null;
  readonly wasm: string;
  readonly wasm_integrity?: string | null;
}

export interface SharedAssetDescriptor {
  readonly url: string;
  readonly integrity?: string | null;
}

export interface RuntimeManifest {
  readonly version: string;
  readonly manifest_hash?: string | null;
  readonly architectures: Partial<Record<Exclude<WasmArchitecture, 'auto'>, {
    readonly core: BundleAssetDescriptor;
    readonly ui: BundleAssetDescriptor;
  }>>;
  readonly assets?: {
    readonly icu?: SharedAssetDescriptor;
  };
}

export interface ArchitectureSelection {
  readonly requestedArchitecture: WasmArchitecture;
  readonly selectedArchitecture: Exclude<WasmArchitecture, 'auto'>;
  readonly availableArchitectures: readonly string[];
  readonly memory64Supported: boolean;
  readonly simdSupported: boolean;
  readonly selectionReason: string;
  readonly manifestEntry: {
    readonly core: BundleAssetDescriptor;
    readonly ui: BundleAssetDescriptor;
  };
}

export interface PreparedWasmAsset {
  readonly url: string;
  readonly integrity: string | null;
  readonly bytesPromise: Promise<ArrayBuffer>;
  readonly modulePromise: Promise<WebAssembly.Module>;
}

export interface PreparedBinaryAsset {
  readonly url: string;
  readonly integrity: string | null;
  readonly bytesPromise: Promise<Uint8Array>;
}

export interface PreparedRuntimeAssets {
  readonly manifest: RuntimeManifest;
  readonly selection: ArchitectureSelection;
  readonly loaderInfo: BridgeLoaderInfo;
  readonly coreBundle: BundleAssetDescriptor;
  readonly uiBundle: BundleAssetDescriptor;
  readonly coreWasm: PreparedWasmAsset;
  readonly uiWasm: PreparedWasmAsset;
  readonly icu: PreparedBinaryAsset;
}

export interface BridgeInteractionState {
  readonly logs: BridgeLogs;
  readonly textByHandle: Record<string, string>;
  readonly selectionsByHandle: Record<string, { start: number; end: number }>;
  flushPendingTextMutationsToRuntime(): void;
  hasPendingTextMutations(): boolean;
  materializePendingTextMutations(): boolean;
  getActiveTextEditable(): boolean;
  getActiveTextHandle(): bigint | null;
  getActiveTextMultiline(): boolean;
  getCapturedPointerHandle(): bigint | null;
  getLastPointerClientPosition(): { x: number | null; y: number | null };
  getLastPointerPosition(): { x: number; y: number };
  getLastPointerModifiers(): number;
  getLastInteractivePointerHandle(): bigint | null;
  isActiveTextInputFocused(): boolean;
  isPointerInsideCanvas(): boolean;
  applyActiveTextDeletion(forward: boolean): boolean;
  replaceActiveTextSelectionWithText(text: string): boolean;
  syncActiveTextSelectionFromDom(): void;
  beginTouchTextFocusDeferral(handle: bigint): void;
  cancelTouchTextFocusDeferral(): void;
  commitTouchTextFocusDeferral(handle: bigint): void;
  refocusActiveTextInput(): void;
  resetAppSession(): void;
  reconcileLiveHandles(handles: readonly string[]): void;
  syncActiveTextInputViewport(): void;
  registerSemanticTextEditor(handle: string, editor: HiddenTextEditor | null): void;
  consumePendingSemanticAnnouncements(): readonly string[];
  getFocusedHandle(): string | null;
  setCapturedPointerHandle(handle: bigint | null): void;
  setLastPointerClientPosition(x: number, y: number): void;
  setLastPointerModifiers(modifiers: number): void;
  setLastPointerPosition(x: number, y: number): void;
  setLastInteractivePointerHandle(handle: bigint | null): void;
  setPointerInsideCanvas(flag: boolean): void;
}

export interface EditorDomTarget {
  readonly singleLineEditor: HTMLInputElement;
  readonly multiLineEditor: HTMLTextAreaElement;
  getEditor(handle: string | null, multiline: boolean): HiddenTextEditor;
  hasSemanticTextEditor(handle: string | null): boolean;
  focus(handle: string | null, multiline: boolean, options?: FocusOptions): void;
  detach(): void;
  clearAll(): void;
  attachListeners(attach: (editor: HiddenTextEditor) => void): void;
  registerSemanticTextEditor(handle: string, editor: HiddenTextEditor | null): void;
}

export interface SoftwarePresenter {
  canvas: HTMLCanvasElement;
  ctx: CanvasRenderingContext2D;
  imageData: ImageData | null;
  width: number;
  height: number;
}
