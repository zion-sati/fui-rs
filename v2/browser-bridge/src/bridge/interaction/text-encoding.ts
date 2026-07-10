export function utf8ByteLengthForCodePoint(codePoint: number): number {
  if (codePoint <= 0x7f) {
    return 1;
  }
  if (codePoint <= 0x7ff) {
    return 2;
  }
  if (codePoint <= 0xffff) {
    return 3;
  }
  return 4;
}

export function advanceCodeUnitIndex(text: string, index: number): number {
  const codePoint = text.codePointAt(index) ?? 0;
  return index + (codePoint > 0xffff ? 2 : 1);
}

export function retreatCodeUnitIndex(text: string, index: number): number {
  const clampedIndex = Math.max(0, Math.min(index, text.length));
  if (clampedIndex <= 0) {
    return 0;
  }
  const previousIndex = clampedIndex - 1;
  const previousUnit = text.charCodeAt(previousIndex);
  if (
    previousIndex > 0 &&
    previousUnit >= 0xdc00 &&
    previousUnit <= 0xdfff
  ) {
    const leadUnit = text.charCodeAt(previousIndex - 1);
    if (leadUnit >= 0xd800 && leadUnit <= 0xdbff) {
      return previousIndex - 1;
    }
  }
  return previousIndex;
}

export function codeUnitIndexToUtf8ByteOffset(text: string, index: number): number {
  const clampedIndex = Math.max(0, Math.min(index, text.length));
  let byteOffset = 0;
  for (let current = 0; current < clampedIndex; current = advanceCodeUnitIndex(text, current)) {
    const codePoint = text.codePointAt(current) ?? 0;
    byteOffset += utf8ByteLengthForCodePoint(codePoint);
  }
  return byteOffset;
}

export function utf8ByteLength(text: string): number {
  return codeUnitIndexToUtf8ByteOffset(text, text.length);
}
