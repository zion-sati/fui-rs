import type { TextChangeLog } from '../../core-types';
import type { EditorDomTarget } from '../local-types';
import {
advanceCodeUnitIndex,
codeUnitIndexToUtf8ByteOffset,
utf8ByteLength,
utf8ByteLengthForCodePoint,
} from './text-encoding';

export type HiddenTextEditor = HTMLInputElement | HTMLTextAreaElement;

export interface ReplacementEdit {
  readonly start: number;
  readonly end: number;
  readonly insertedText: string;
}

export interface HiddenEditorWindow {
  readonly text: string;
  readonly docStart: number;
  readonly docEnd: number;
  readonly textStart: number;
  readonly textEnd: number;
}

export interface PendingLocalReplacementEcho {
  readonly handle: string;
  readonly start: number;
  readonly end: number;
  readonly text: string;
}

export interface PendingLocalSelectionEcho {
  readonly handle: string;
  readonly start: number;
  readonly end: number;
}

export interface PendingTextMutationBatch {
  readonly handle: string;
  readonly docStart: number;
  readonly baseWindowText: string;
  readonly currentWindowText: string;
  readonly caret: number;
  readonly interactionTime: bigint;
  readonly kind: 'typing' | 'paste';
  readonly mutationCount: number;
}

export interface PendingPasteInput {
  readonly handle: string;
  readonly docStart: number;
  readonly selectionStart: number;
  readonly selectionEnd: number;
  readonly text: string;
}

interface TextClampRange {
  readonly start: number;
  readonly end: number;
}

const HIDDEN_EDITOR_WINDOW_OVERSCAN = 2048;
const HIDDEN_EDITOR_WINDOW_MIN_LENGTH = 4096;
const HIDDEN_EDITOR_WINDOW_REUSE_MARGIN = 512;
const TEXT_CHANGE_LOG_PREVIEW_LIMIT = 256;
const TEXTBOX_HARD_CLAMP_MAX_CODEPOINTS = 10000;
const HIDDEN_EDITOR_STYLE_ID = 'effindom-hidden-editor-style';

function ensureHiddenEditorStyle(): void {
  if (document.getElementById(HIDDEN_EDITOR_STYLE_ID) !== null) {
    return;
  }
  const style = document.createElement('style');
  style.id = HIDDEN_EDITOR_STYLE_ID;
  style.textContent = `
[data-effindom-hidden-editor="true"] {
  scrollbar-width: none;
  -ms-overflow-style: none;
}
[data-effindom-hidden-editor="true"]::-webkit-scrollbar {
  width: 0;
  height: 0;
  display: none;
}
`;
  document.head.appendChild(style);
}

function isLineBreakCodeUnit(text: string, index: number): boolean {
  if (index < 0 || index >= text.length) {
    return false;
  }
  const codeUnit = text.charCodeAt(index);
  return codeUnit === 0x0a || codeUnit === 0x0d;
}

function collectTextboxHardLineClampRanges(text: string): readonly TextClampRange[] {
  const ranges: TextClampRange[] = [];
  let index = 0;
  while (index < text.length) {
    let lineCapEnd = index;
    let lineEnd = index;
    let codePointCount = 0;
    while (lineEnd < text.length && !isLineBreakCodeUnit(text, lineEnd)) {
      const next = advanceCodeUnitIndex(text, lineEnd);
      if (codePointCount < TEXTBOX_HARD_CLAMP_MAX_CODEPOINTS) {
        lineCapEnd = next;
      }
      codePointCount += 1;
      lineEnd = next;
    }
    if (codePointCount > TEXTBOX_HARD_CLAMP_MAX_CODEPOINTS) {
      ranges.push({ start: lineCapEnd, end: lineEnd });
    }
    if (lineEnd >= text.length) {
      break;
    }
    if (text.charCodeAt(lineEnd) === 0x0d && lineEnd + 1 < text.length && text.charCodeAt(lineEnd + 1) === 0x0a) {
      index = lineEnd + 2;
    } else {
      index = lineEnd + 1;
    }
  }
  return ranges;
}

function mapClampedTextIndex(index: number, ranges: readonly TextClampRange[]): number {
  const clampedIndex = Math.max(0, index);
  let removedBefore = 0;
  for (const range of ranges) {
    if (clampedIndex <= range.start) {
      break;
    }
    if (clampedIndex < range.end) {
      return range.start - removedBefore;
    }
    removedBefore += range.end - range.start;
  }
  return clampedIndex - removedBefore;
}

function clampTextboxHardLines(text: string): {
  readonly text: string;
  readonly changed: boolean;
  mapIndex(index: number): number;
} {
  const ranges = collectTextboxHardLineClampRanges(text);
  if (ranges.length === 0) {
    return {
      text,
      changed: false,
      mapIndex: (index: number) => Math.max(0, Math.min(index, text.length)),
    };
  }
  let result = '';
  let cursor = 0;
  for (const range of ranges) {
    result += text.slice(cursor, range.start);
    cursor = range.end;
  }
  result += text.slice(cursor);
  return {
    text: result,
    changed: true,
    mapIndex: (index: number) => mapClampedTextIndex(index, ranges),
  };
}

export function utf8ByteOffsetToCodeUnitIndex(text: string, byteOffset: number, textByteLength?: number): number {
  const byteLimit = textByteLength ?? utf8ByteLength(text);
  const target = Math.max(0, Math.min(byteOffset, byteLimit));
  let currentByteOffset = 0;
  let currentIndex = 0;
  while (currentIndex < text.length) {
    const codePoint = text.codePointAt(currentIndex) ?? 0;
    const nextByteOffset = currentByteOffset + utf8ByteLengthForCodePoint(codePoint);
    if (nextByteOffset > target) {
      break;
    }
    currentByteOffset = nextByteOffset;
    currentIndex = advanceCodeUnitIndex(text, currentIndex);
    if (currentByteOffset === target) {
      break;
    }
  }
  return currentIndex;
}

export function computeReplacementEdit(previousText: string, nextText: string): ReplacementEdit | null {
  if (previousText === nextText) {
    return null;
  }
  const sharedPrefixLimit = Math.min(previousText.length, nextText.length);
  let prefix = 0;
  while (prefix < sharedPrefixLimit && previousText.charCodeAt(prefix) === nextText.charCodeAt(prefix)) {
    prefix += 1;
  }

  let suffix = 0;
  while (
    suffix < (previousText.length - prefix) &&
    suffix < (nextText.length - prefix) &&
    previousText.charCodeAt(previousText.length - suffix - 1) === nextText.charCodeAt(nextText.length - suffix - 1)
  ) {
    suffix += 1;
  }

  return {
    start: prefix,
    end: previousText.length - suffix,
    insertedText: nextText.slice(prefix, nextText.length - suffix),
  };
}

export function applyUtf8ByteReplacementEdit(text: string, start: number, end: number, insertedText: string): string {
  const clampedStart = utf8ByteOffsetToCodeUnitIndex(text, Math.max(0, start));
  const clampedEnd = utf8ByteOffsetToCodeUnitIndex(text, Math.max(clampedStart, end));
  return `${text.slice(0, clampedStart)}${insertedText}${text.slice(clampedEnd)}`;
}

export function mapPendingBatchCurrentIndexToBaseIndex(batch: PendingTextMutationBatch, index: number): number {
  const clampedIndex = Math.max(0, Math.min(index, batch.currentWindowText.length));
  const replacement = computeReplacementEdit(batch.baseWindowText, batch.currentWindowText);
  if (replacement === null) {
    return Math.max(0, Math.min(clampedIndex, batch.baseWindowText.length));
  }
  const insertedStart = replacement.start;
  const insertedEnd = replacement.start + replacement.insertedText.length;
  if (clampedIndex <= insertedStart) {
    return clampedIndex;
  }
  if (clampedIndex <= insertedEnd) {
    return replacement.start;
  }
  const delta = replacement.insertedText.length - (replacement.end - replacement.start);
  return Math.max(0, Math.min(clampedIndex - delta, batch.baseWindowText.length));
}

export function buildClampedTextboxEdit(
  fullPreviousText: string,
  absoluteStart: number,
  absoluteEnd: number,
  insertedText: string,
  absoluteCaret: number,
): {
  readonly fullNextText: string;
  readonly replacement: ReplacementEdit | null;
  readonly caretByte: number;
  readonly clampChanged: boolean;
} {
  const replaceStart = utf8ByteOffsetToCodeUnitIndex(fullPreviousText, absoluteStart);
  const replaceEnd = utf8ByteOffsetToCodeUnitIndex(fullPreviousText, absoluteEnd);
  const intendedFullNextText =
    `${fullPreviousText.slice(0, replaceStart)}${insertedText}${fullPreviousText.slice(replaceEnd)}`;
  const intendedCaret = utf8ByteOffsetToCodeUnitIndex(intendedFullNextText, absoluteCaret);
  const clamped = clampTextboxHardLines(intendedFullNextText);
  const finalCaret = clamped.mapIndex(intendedCaret);
  return {
    fullNextText: clamped.text,
    replacement: computeReplacementEdit(fullPreviousText, clamped.text),
    caretByte: codeUnitIndexToUtf8ByteOffset(clamped.text, finalCaret),
    clampChanged: clamped.changed,
  };
}

export function summarizeTextChange(handle: string, text: string): TextChangeLog {
  if (text.length <= TEXT_CHANGE_LOG_PREVIEW_LIMIT) {
    return { handle, text };
  }
  return {
    handle,
    text: `${text.slice(0, TEXT_CHANGE_LOG_PREVIEW_LIMIT)}…`,
    textLength: text.length,
    truncated: true,
  };
}

export function buildHiddenEditorWindow(
  text: string,
  start: number,
  end: number,
  textByteLength: number,
  previousWindow?: HiddenEditorWindow,
): HiddenEditorWindow {
  const normalizedStart = Math.max(0, Math.min(start, text.length));
  const normalizedEnd = Math.max(normalizedStart, Math.min(end, text.length));
  const selectionSpan = normalizedEnd - normalizedStart;
  const targetLength = Math.max(
    HIDDEN_EDITOR_WINDOW_MIN_LENGTH,
    selectionSpan + (HIDDEN_EDITOR_WINDOW_OVERSCAN * 2),
  );
  if (text.length <= targetLength) {
    return {
      text,
      docStart: 0,
      docEnd: textByteLength,
      textStart: 0,
      textEnd: text.length,
    };
  }

  if (previousWindow !== undefined) {
    const previousLength = previousWindow.textEnd - previousWindow.textStart;
    const clampedPreviousStart = Math.max(0, Math.min(previousWindow.textStart, text.length));
    const clampedPreviousEnd = Math.max(clampedPreviousStart, Math.min(previousWindow.textEnd, text.length));
    if (previousLength > 0 &&
      clampedPreviousEnd > clampedPreviousStart &&
      clampedPreviousEnd - clampedPreviousStart >= targetLength) {
      const reuseMargin = Math.min(
        HIDDEN_EDITOR_WINDOW_REUSE_MARGIN,
        Math.max(0, Math.floor((clampedPreviousEnd - clampedPreviousStart) / 4)),
      );
      const reusableStart = clampedPreviousStart + reuseMargin;
      const reusableEnd = clampedPreviousEnd - reuseMargin;
      if (normalizedStart >= reusableStart && normalizedEnd <= reusableEnd) {
        return {
          text: text.slice(clampedPreviousStart, clampedPreviousEnd),
          docStart: previousWindow.docStart,
          docEnd: previousWindow.docEnd,
          textStart: clampedPreviousStart,
          textEnd: clampedPreviousEnd,
        };
      }
    }
  }

  let docStart = Math.max(0, normalizedStart - HIDDEN_EDITOR_WINDOW_OVERSCAN);
  let docEnd = Math.min(text.length, normalizedEnd + HIDDEN_EDITOR_WINDOW_OVERSCAN);
  const currentLength = docEnd - docStart;
  if (currentLength < targetLength) {
    let remaining = targetLength - currentLength;
    const extendBefore = Math.min(docStart, Math.floor(remaining / 2));
    docStart -= extendBefore;
    remaining -= extendBefore;
    const extendAfter = Math.min(text.length - docEnd, remaining);
    docEnd += extendAfter;
    remaining -= extendAfter;
    if (remaining > 0) {
      const extraBefore = Math.min(docStart, remaining);
      docStart -= extraBefore;
      remaining -= extraBefore;
      if (remaining > 0) {
        docEnd = Math.min(text.length, docEnd + remaining);
      }
    }
  }

  return {
    text: text.slice(docStart, docEnd),
    docStart: codeUnitIndexToUtf8ByteOffset(text, docStart),
    docEnd: codeUnitIndexToUtf8ByteOffset(text, docEnd),
    textStart: docStart,
    textEnd: docEnd,
  };
}

export function createHiddenTextEditor(multiline: boolean): HiddenTextEditor {
  ensureHiddenEditorStyle();
  const editor = multiline ? document.createElement('textarea') : document.createElement('input');
  const isSemanticLightDomField = (): boolean =>
    editor.dataset.effindomSemanticLightDomField === 'true';
  let hostAutofillWakeupRevision = 0;
  if (editor instanceof HTMLInputElement) {
    editor.type = 'text';
  } else {
    editor.rows = 1;
    editor.wrap = 'off';
    editor.style.resize = 'none';
    editor.style.whiteSpace = 'pre';
  }
  editor.autocapitalize = 'off';
  editor.autocomplete = 'off';
  editor.autocorrect = false;
  editor.spellcheck = false;
  editor.tabIndex = -1;
  editor.dataset.effindomHiddenEditor = 'true';
  editor.setAttribute('aria-hidden', 'true');
  editor.style.position = 'fixed';
  editor.style.left = '-9999px';
  editor.style.top = '0';
  editor.style.width = '1px';
  editor.style.height = '1px';
  editor.style.opacity = '0';
  editor.style.pointerEvents = 'none';
  editor.style.font = '16px "Noto Sans Symbols 2", "Apple Color Emoji", "Segoe UI Emoji", "Noto Color Emoji", monospace';
  editor.style.overflow = 'hidden';
  editor.style.scrollbarWidth = 'none';

  const scheduleHostAutofillWakeup = (): void => {
    const autocomplete = editor.getAttribute('autocomplete');
    if (autocomplete === null || autocomplete === '' || autocomplete === 'off') {
      return;
    }
    const wakeupRevision = ++hostAutofillWakeupRevision;
    const name = editor.getAttribute('name');
    const id = editor.getAttribute('id');
    window.setTimeout(() => {
      if (document.activeElement !== editor || wakeupRevision !== hostAutofillWakeupRevision) {
        return;
      }
      editor.removeAttribute('autocomplete');
      editor.removeAttribute('name');
      editor.removeAttribute('id');
      window.setTimeout(() => {
        if (document.activeElement !== editor || wakeupRevision !== hostAutofillWakeupRevision) {
          return;
        }
        editor.setAttribute('autocomplete', autocomplete);
        if (name !== null) {
          editor.setAttribute('name', name);
        }
        if (id !== null) {
          editor.setAttribute('id', id);
        }
        editor.dataset.effindomAutofillWakeup = 'true';
        editor.dispatchEvent(new Event('input', { bubbles: true }));
      }, 0);
    }, 0);
  };

  const rearmHostAutofillDomPresence = (): void => {
    const autocomplete = editor.getAttribute('autocomplete');
    if (autocomplete === null || autocomplete === '' || autocomplete === 'off') {
      return;
    }
    const parent = editor.parentNode;
    if (parent === null) {
      return;
    }
    const nextSibling = editor.nextSibling;
    parent.removeChild(editor);
    if (nextSibling === null) {
      parent.appendChild(editor);
      return;
    }
    parent.insertBefore(editor, nextSibling);
  };

  // Ensure assistive tech can see focus: clear aria-hidden before focusing,
  // perform the native focus synchronously (so callers relying on immediate focus
  // observe document.activeElement), and re-hide on blur in a macrotask.
  const nativeFocus: typeof editor.focus = editor.focus.bind(editor);
  editor.focus = (options?: FocusOptions): void => {
    rearmHostAutofillDomPresence();
    try {
      if (!isSemanticLightDomField() && editor.getAttribute('aria-hidden') === 'true') {
        editor.setAttribute('aria-hidden', 'false');
      } else if (isSemanticLightDomField()) {
        editor.removeAttribute('aria-hidden');
      }
    } catch {
      // Defensive: ignore if attribute manipulation fails for any reason.
    }
    try {
      nativeFocus(options);
    } catch {
      // swallow errors from native focus
    }
    scheduleHostAutofillWakeup();
    // If the browser doesn't immediately make the editor the activeElement
    // (some platforms may delay programmatic focus), retry once on the next
    // macrotask so callers waiting with setTimeout(0) will observe the focus.
    if (document.activeElement !== editor) {
      window.setTimeout(() => {
        try {
          nativeFocus(options);
        } catch {
          // ignore
        }
        scheduleHostAutofillWakeup();
      }, 0);
    }
  };

  editor.addEventListener('blur', () => {
    hostAutofillWakeupRevision += 1;
    if (isSemanticLightDomField()) {
      return;
    }
    // Re-hide in a macrotask so assistive tech has seen the focused state first.
    window.setTimeout(() => {
      try {
        if (document.activeElement !== editor) {
          editor.setAttribute('aria-hidden', 'true');
        }
      } catch {
        // ignore
      }
    }, 1);
  });

  document.body.appendChild(editor);
  return editor;
}

export function createSingleHiddenEditorTarget(): EditorDomTarget {
  const singleLineEditor = createHiddenTextEditor(false) as HTMLInputElement;
  const multiLineEditor = createHiddenTextEditor(true) as HTMLTextAreaElement;
  const semanticEditorsByHandle = new Map<string, HiddenTextEditor>();
  const attachedEditors = new WeakSet<HiddenTextEditor>();
  let attachListener: ((editor: HiddenTextEditor) => void) | null = null;
  const attachIfNeeded = (editor: HiddenTextEditor): void => {
    if (attachListener === null || attachedEditors.has(editor)) {
      return;
    }
    attachListener(editor);
    attachedEditors.add(editor);
  };
  const resolveEditor = (handle: string | null, multiline: boolean): HiddenTextEditor => {
    if (handle !== null) {
      const semanticEditor = semanticEditorsByHandle.get(handle);
      if (semanticEditor !== undefined) {
        return semanticEditor;
      }
    }
    return multiline ? multiLineEditor : singleLineEditor;
  };
  return {
    singleLineEditor,
    multiLineEditor,
    getEditor: (handle: string | null, multiline: boolean): HiddenTextEditor => resolveEditor(handle, multiline),
    hasSemanticTextEditor: (handle: string | null): boolean => handle !== null && semanticEditorsByHandle.has(handle),
    focus: (handle: string | null, multiline: boolean, options?: FocusOptions): void => {
      resolveEditor(handle, multiline).focus(options);
    },
    detach: (): void => {
      singleLineEditor.blur();
      multiLineEditor.blur();
      for (const editor of semanticEditorsByHandle.values()) {
        editor.blur();
      }
    },
    clearAll: (): void => {
      clearHiddenTextEditor(singleLineEditor);
      clearHiddenTextEditor(multiLineEditor);
      for (const editor of semanticEditorsByHandle.values()) {
        clearHiddenTextEditor(editor);
      }
    },
    attachListeners: (attach: (editor: HiddenTextEditor) => void): void => {
      attachListener = attach;
      attachIfNeeded(singleLineEditor);
      attachIfNeeded(multiLineEditor);
      for (const editor of semanticEditorsByHandle.values()) {
        attachIfNeeded(editor);
      }
    },
    registerSemanticTextEditor: (handle: string, editor: HiddenTextEditor | null): void => {
      if (editor === null) {
        semanticEditorsByHandle.delete(handle);
        return;
      }
      semanticEditorsByHandle.set(handle, editor);
      attachIfNeeded(editor);
    },
  };
}

export function clearHiddenTextEditor(editor: HiddenTextEditor): void {
  if (editor.value.length !== 0) {
    editor.value = '';
  }
  editor.setSelectionRange(0, 0, 'none');
}
