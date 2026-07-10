import type {
OpenCanvasFindMatch,
OpenCanvasFindOptions,
OpenCanvasFindResults,
OpenCanvasResolvedFindOptions,
OpenCanvasTextDocument,
} from '../core-types';

const combiningMarkPattern = /\p{Mark}+/gu;
const wordLikeCharPattern = /[\p{Letter}\p{Number}\p{Mark}_]/u;
const textEncoder = new TextEncoder();

interface SearchToken {
  readonly start: number;
  readonly end: number;
}

interface SearchText {
  readonly text: string;
  readonly tokens: readonly SearchToken[];
}

export const DEFAULT_OPEN_CANVAS_FIND_OPTIONS: OpenCanvasResolvedFindOptions = {
  highlightAll: false,
  matchCase: false,
  matchDiacritics: false,
  wholeWords: false,
};

export function normalizeOpenCanvasFindOptions(
  options?: OpenCanvasFindOptions,
): OpenCanvasResolvedFindOptions {
  return {
    highlightAll: options?.highlightAll ?? DEFAULT_OPEN_CANVAS_FIND_OPTIONS.highlightAll,
    matchCase: options?.matchCase ?? DEFAULT_OPEN_CANVAS_FIND_OPTIONS.matchCase,
    matchDiacritics: options?.matchDiacritics ?? DEFAULT_OPEN_CANVAS_FIND_OPTIONS.matchDiacritics,
    wholeWords: options?.wholeWords ?? DEFAULT_OPEN_CANVAS_FIND_OPTIONS.wholeWords,
  };
}

function utf8ByteOffsetFromCodeUnitIndex(text: string, codeUnitIndex: number): number {
  const clampedIndex = Math.max(0, Math.min(codeUnitIndex, text.length));
  return textEncoder.encode(text.slice(0, clampedIndex)).byteLength;
}

function codePointBefore(text: string, boundary: number): string | null {
  if (boundary <= 0) {
    return null;
  }
  let start = boundary - 1;
  const low = text.charCodeAt(start);
  if (
    start > 0 &&
    low >= 0xDC00 &&
    low <= 0xDFFF
  ) {
    const high = text.charCodeAt(start - 1);
    if (high >= 0xD800 && high <= 0xDBFF) {
      start -= 1;
    }
  }
  return text.slice(start, boundary);
}

function codePointAt(text: string, boundary: number): string | null {
  const codePoint = text.codePointAt(boundary);
  return codePoint === undefined ? null : String.fromCodePoint(codePoint);
}

function isWordLikeChar(value: string | null): boolean {
  return value !== null && wordLikeCharPattern.test(value);
}

function isWholeWordMatch(text: string, start: number, end: number): boolean {
  return !isWordLikeChar(codePointBefore(text, start)) &&
    !isWordLikeChar(codePointAt(text, end));
}

function buildSearchText(
  source: string,
  options: OpenCanvasResolvedFindOptions,
): SearchText {
  const tokens: SearchToken[] = [];
  let output = '';
  let sourceIndex = 0;
  for (const segment of source) {
    const start = sourceIndex;
    sourceIndex += segment.length;
    const end = sourceIndex;
    let searchable = options.matchDiacritics
      ? segment
      : segment.normalize('NFD').replace(combiningMarkPattern, '');
    searchable = options.matchCase ? searchable : searchable.toLocaleLowerCase();
    output += searchable;
    tokens.push(...Array.from(searchable, () => ({ start, end })));
  }
  return { text: output, tokens };
}

export function findTextInOpenCanvasDocuments(
  documents: readonly OpenCanvasTextDocument[],
  query: string,
  options?: OpenCanvasFindOptions,
): OpenCanvasFindResults {
  const resolvedOptions = normalizeOpenCanvasFindOptions(options);
  if (query.length === 0) {
    return {
      query,
      options: resolvedOptions,
      matches: [],
    };
  }

  const normalizedQuery = buildSearchText(query, resolvedOptions).text;
  if (normalizedQuery.length === 0) {
    return {
      query,
      options: resolvedOptions,
      matches: [],
    };
  }

  const matches: OpenCanvasFindMatch[] = [];
  for (const document of documents) {
    const searchableDocument = buildSearchText(document.text, resolvedOptions);
    let searchStart = 0;
    while (searchStart <= searchableDocument.text.length - normalizedQuery.length) {
      const matchStart = searchableDocument.text.indexOf(normalizedQuery, searchStart);
      if (matchStart < 0) {
        break;
      }
      const matchEnd = matchStart + normalizedQuery.length;
      const startToken = searchableDocument.tokens[matchStart];
      const endToken = searchableDocument.tokens[matchEnd - 1];
      if (startToken === undefined || endToken === undefined) {
        break;
      }
      const sourceStart = startToken.start;
      const sourceEnd = endToken.end;
      if (!resolvedOptions.wholeWords || isWholeWordMatch(document.text, sourceStart, sourceEnd)) {
        matches.push({
          handle: document.handle,
          start: utf8ByteOffsetFromCodeUnitIndex(document.text, sourceStart),
          end: utf8ByteOffsetFromCodeUnitIndex(document.text, sourceEnd),
        });
      }
      searchStart = matchStart + Math.max(normalizedQuery.length, 1);
    }
  }

  return {
    query,
    options: resolvedOptions,
    matches,
  };
}
