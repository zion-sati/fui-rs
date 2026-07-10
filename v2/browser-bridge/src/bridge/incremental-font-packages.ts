import {
BUILT_IN_FONT_BODY,
BUILT_IN_FONT_BODY_BOLD_ITALIC,
BUILT_IN_FONT_BODY_ITALIC,
BUILT_IN_FONT_HEADING,
BUILT_IN_FONT_MONO,
BUILT_IN_FONT_MONO_BOLD,
} from './font-catalog';

export const UI_MISSING_FONT_COVERAGE_UNKNOWN = 0;
export const UI_MISSING_FONT_COVERAGE_ARABIC = 1;
export const UI_MISSING_FONT_COVERAGE_THAI = 2;
export const UI_MISSING_FONT_COVERAGE_CJK = 3;
export const UI_MISSING_FONT_COVERAGE_SUPPLEMENTAL = 4;

export interface ResolvedIncrementalFontShardRequest {
  readonly packageId: string;
  readonly coverageKind: number;
  readonly familyKey: string;
  readonly googleFamily: string;
  readonly text: string;
}

const AUTO_EXTENDABLE_PRIMARY_FONT_IDS = new Set<number>([
  BUILT_IN_FONT_BODY,
  BUILT_IN_FONT_HEADING,
  BUILT_IN_FONT_BODY_ITALIC,
  BUILT_IN_FONT_BODY_BOLD_ITALIC,
  BUILT_IN_FONT_MONO,
  BUILT_IN_FONT_MONO_BOLD,
]);

interface SupplementalFamilyDefinition {
  readonly familyKey: string;
  readonly googleFamily: string;
  readonly ranges: readonly [number, number][];
}

const SUPPLEMENTAL_FAMILIES: readonly SupplementalFamilyDefinition[] = [
  { familyKey: 'hebrew', googleFamily: 'Noto Sans Hebrew', ranges: [[0x0590, 0x05FF]] },
  { familyKey: 'armenian', googleFamily: 'Noto Sans Armenian', ranges: [[0x0530, 0x058F]] },
  { familyKey: 'georgian', googleFamily: 'Noto Sans Georgian', ranges: [[0x10A0, 0x10FF], [0x2D00, 0x2D2F]] },
  { familyKey: 'devanagari', googleFamily: 'Noto Sans Devanagari', ranges: [[0x0900, 0x097F]] },
  { familyKey: 'bengali', googleFamily: 'Noto Sans Bengali', ranges: [[0x0980, 0x09FF]] },
  { familyKey: 'gurmukhi', googleFamily: 'Noto Sans Gurmukhi', ranges: [[0x0A00, 0x0A7F]] },
  { familyKey: 'gujarati', googleFamily: 'Noto Sans Gujarati', ranges: [[0x0A80, 0x0AFF]] },
  { familyKey: 'oriya', googleFamily: 'Noto Sans Oriya', ranges: [[0x0B00, 0x0B7F]] },
  { familyKey: 'tamil', googleFamily: 'Noto Sans Tamil', ranges: [[0x0B80, 0x0BFF]] },
  { familyKey: 'telugu', googleFamily: 'Noto Sans Telugu', ranges: [[0x0C00, 0x0C7F]] },
  { familyKey: 'kannada', googleFamily: 'Noto Sans Kannada', ranges: [[0x0C80, 0x0CFF]] },
  { familyKey: 'malayalam', googleFamily: 'Noto Sans Malayalam', ranges: [[0x0D00, 0x0D7F]] },
  { familyKey: 'sinhala', googleFamily: 'Noto Sans Sinhala', ranges: [[0x0D80, 0x0DFF]] },
  { familyKey: 'lao', googleFamily: 'Noto Sans Lao', ranges: [[0x0E80, 0x0EFF]] },
  { familyKey: 'myanmar', googleFamily: 'Noto Sans Myanmar', ranges: [[0x1000, 0x109F]] },
  { familyKey: 'khmer', googleFamily: 'Noto Sans Khmer', ranges: [[0x1780, 0x17FF]] },
  { familyKey: 'ethiopic', googleFamily: 'Noto Sans Ethiopic', ranges: [[0x1200, 0x137F], [0x1380, 0x139F], [0x2D80, 0x2DDF]] },
] as const;

function inRanges(codePoint: number, ranges: readonly [number, number][]): boolean {
  return ranges.some(([start, end]) => codePoint >= start && codePoint <= end);
}

function uniqueCodePoints(text: string): readonly string[] {
  const seen = new Set<string>();
  const ordered: string[] = [];
  for (const character of text) {
    if (seen.has(character)) {
      continue;
    }
    seen.add(character);
    ordered.push(character);
  }
  return ordered;
}

function isCjkPunctuation(codePoint: number): boolean {
  return (codePoint >= 0x3000 && codePoint <= 0x303F)
    || (codePoint >= 0xFF01 && codePoint <= 0xFF0F)
    || (codePoint >= 0xFF1A && codePoint <= 0xFF20)
    || (codePoint >= 0xFF3B && codePoint <= 0xFF40)
    || (codePoint >= 0xFF5B && codePoint <= 0xFF65);
}

function buildShardRequest(
  packageId: string,
  coverageKind: number,
  familyKey: string,
  googleFamily: string,
  characters: readonly string[],
): ResolvedIncrementalFontShardRequest | null {
  if (characters.length === 0) {
    return null;
  }
  const text = characters.join('');
  return {
    packageId,
    coverageKind,
    familyKey,
    googleFamily,
    text,
  };
}

function resolveCjkRequests(sampleText: string): readonly ResolvedIncrementalFontShardRequest[] {
  const codePoints = uniqueCodePoints(sampleText);
  const kana: string[] = [];
  const hangul: string[] = [];
  const han: string[] = [];
  const punctuation: string[] = [];
  for (const character of codePoints) {
    const codePoint = character.codePointAt(0);
    if (codePoint === undefined) {
      continue;
    }
    if (isCjkPunctuation(codePoint)) {
      punctuation.push(character);
      continue;
    }
    if ((codePoint >= 0x3040 && codePoint <= 0x30FF) || (codePoint >= 0x31F0 && codePoint <= 0x31FF)) {
      kana.push(character);
      continue;
    }
    if (codePoint >= 0xAC00 && codePoint <= 0xD7AF) {
      hangul.push(character);
      continue;
    }
    if (
      (codePoint >= 0x3400 && codePoint <= 0x4DBF)
      || (codePoint >= 0x4E00 && codePoint <= 0x9FFF)
      || (codePoint >= 0xF900 && codePoint <= 0xFAFF)
    ) {
      han.push(character);
    }
  }

  const requests: ResolvedIncrementalFontShardRequest[] = [];
  const pushShardRequest = (
    packageKey: string,
    coverageKind: number,
    shardKey: string,
    googleFamily: string,
    text: readonly string[],
  ): void => {
    const request = buildShardRequest(packageKey, coverageKind, shardKey, googleFamily, text);
    if (request !== null) {
      requests.push(request);
    }
  };
  if (kana.length > 0) {
    pushShardRequest('cjk-sans', UI_MISSING_FONT_COVERAGE_CJK, 'cjk-jp', 'Noto Sans JP', [...kana, ...han, ...punctuation]);
  }
  if (hangul.length > 0) {
    pushShardRequest('cjk-sans', UI_MISSING_FONT_COVERAGE_CJK, 'cjk-kr', 'Noto Sans KR', [...hangul, ...(kana.length === 0 ? han : []), ...punctuation]);
  }
  if (han.length > 0 && kana.length === 0 && hangul.length === 0) {
    pushShardRequest('cjk-sans', UI_MISSING_FONT_COVERAGE_CJK, 'cjk-sc', 'Noto Sans SC', [...han, ...punctuation]);
  }
  if (requests.length === 0 && punctuation.length > 0) {
    pushShardRequest('cjk-sans', UI_MISSING_FONT_COVERAGE_CJK, 'cjk-sc', 'Noto Sans SC', punctuation);
  }
  return requests;
}

function resolveSupplementalRequests(sampleText: string): readonly ResolvedIncrementalFontShardRequest[] {
  const codePoints = uniqueCodePoints(sampleText);
  const textByFamily = new Map<string, string[]>();
  for (const character of codePoints) {
    const codePoint = character.codePointAt(0);
    if (codePoint === undefined) {
      continue;
    }
    const family = SUPPLEMENTAL_FAMILIES.find((entry) => inRanges(codePoint, entry.ranges));
    if (family === undefined) {
      continue;
    }
    const current = textByFamily.get(family.familyKey) ?? [];
    current.push(character);
    textByFamily.set(family.familyKey, current);
  }
  return SUPPLEMENTAL_FAMILIES
    .map((family) => buildShardRequest(
      'supplemental-sans',
      UI_MISSING_FONT_COVERAGE_SUPPLEMENTAL,
      family.familyKey,
      family.googleFamily,
      textByFamily.get(family.familyKey) ?? [],
    ))
    .filter((request): request is ResolvedIncrementalFontShardRequest => request !== null);
}

export function resolveIncrementalFontPackageRequests(
  primaryFontId: number,
  coverageKind: number,
  sampleText: string,
): readonly ResolvedIncrementalFontShardRequest[] {
  if (!AUTO_EXTENDABLE_PRIMARY_FONT_IDS.has(primaryFontId) || sampleText.length === 0) {
    return [];
  }
  const uniqueText = uniqueCodePoints(sampleText).join('');
  if (uniqueText.length === 0) {
    return [];
  }
  if (coverageKind === UI_MISSING_FONT_COVERAGE_ARABIC) {
    const request = buildShardRequest('arabic-sans', coverageKind, 'arabic-core', 'Noto Naskh Arabic', Array.from(uniqueText));
    return request === null ? [] : [request];
  }
  if (coverageKind === UI_MISSING_FONT_COVERAGE_THAI) {
    const request = buildShardRequest('thai-sans', coverageKind, 'thai-core', 'Noto Sans Thai', Array.from(uniqueText));
    return request === null ? [] : [request];
  }
  if (coverageKind === UI_MISSING_FONT_COVERAGE_CJK) {
    return resolveCjkRequests(uniqueText);
  }
  if (coverageKind === UI_MISSING_FONT_COVERAGE_SUPPLEMENTAL) {
    return resolveSupplementalRequests(uniqueText);
  }
  return [];
}
