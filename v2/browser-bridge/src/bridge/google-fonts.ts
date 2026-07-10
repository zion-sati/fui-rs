export interface GoogleFontShardBytes {
  readonly url: string;
  readonly bytes: Uint8Array;
}

const GOOGLE_FONTS_STYLESHEET_ORIGIN = 'https://fonts.googleapis.com';
const GOOGLE_FONTS_BINARY_ORIGIN = 'https://fonts.gstatic.com';

function ensureLinkTag(rel: string, href: string, crossOrigin?: '' | 'anonymous'): void {
  const existing = document.head.querySelector(`link[rel="${rel}"][href="${href}"]`);
  if (existing instanceof HTMLLinkElement) {
    return;
  }
  const link = document.createElement('link');
  link.rel = rel;
  link.href = href;
  if (crossOrigin !== undefined) {
    link.crossOrigin = crossOrigin;
  }
  document.head.appendChild(link);
}

export function ensureGoogleFontShardPreconnect(): void {
  ensureLinkTag('preconnect', GOOGLE_FONTS_STYLESHEET_ORIGIN);
  ensureLinkTag('preconnect', GOOGLE_FONTS_BINARY_ORIGIN, 'anonymous');
}

export function buildGoogleFontStylesheetUrl(googleFamily: string, text: string): string {
  const params = new URLSearchParams();
  params.set('family', `${googleFamily}:wght@400`);
  params.set('text', text);
  params.set('display', 'swap');
  return `${GOOGLE_FONTS_STYLESHEET_ORIGIN}/css2?${params.toString()}`;
}

export function parseGoogleFontBinaryUrl(stylesheetText: string): string {
  const match = /src:\s*url\(([^)]+)\)\s*format\((['"]?)(woff2|woff|truetype|opentype)\2\)/.exec(stylesheetText);
  if (match?.[1] === undefined) {
    throw new Error('Google Fonts stylesheet did not expose a usable shard URL.');
  }
  return match[1].trim();
}

function readU16(bytes: Uint8Array, offset: number): number {
  if (offset + 2 > bytes.length) {
    throw new Error('Invalid WOFF font: truncated uint16.');
  }
  const first = bytes[offset];
  const second = bytes[offset + 1];
  if (first === undefined || second === undefined) {
    throw new Error('Invalid WOFF font: truncated uint16.');
  }
  return (first << 8) | second;
}

function readU32(bytes: Uint8Array, offset: number): number {
  if (offset + 4 > bytes.length) {
    throw new Error('Invalid WOFF font: truncated uint32.');
  }
  const first = bytes[offset];
  const second = bytes[offset + 1];
  const third = bytes[offset + 2];
  const fourth = bytes[offset + 3];
  if (first === undefined || second === undefined || third === undefined || fourth === undefined) {
    throw new Error('Invalid WOFF font: truncated uint32.');
  }
  return (
    (first * 0x1000000)
    + ((second << 16) >>> 0)
    + ((third << 8) >>> 0)
    + fourth
  ) >>> 0;
}

function writeU16(bytes: Uint8Array, offset: number, value: number): void {
  bytes[offset] = (value >>> 8) & 0xFF;
  bytes[offset + 1] = value & 0xFF;
}

function writeU32(bytes: Uint8Array, offset: number, value: number): void {
  bytes[offset] = (value >>> 24) & 0xFF;
  bytes[offset + 1] = (value >>> 16) & 0xFF;
  bytes[offset + 2] = (value >>> 8) & 0xFF;
  bytes[offset + 3] = value & 0xFF;
}

function align4(value: number): number {
  return (value + 3) & ~3;
}

async function inflateWoffTable(bytes: Uint8Array): Promise<Uint8Array> {
  if (typeof DecompressionStream === 'undefined') {
    throw new Error('Browser does not expose DecompressionStream for WOFF table inflation.');
  }
  const input = new ArrayBuffer(bytes.byteLength);
  new Uint8Array(input).set(bytes);
  const stream = new Blob([input]).stream().pipeThrough(new DecompressionStream('deflate'));
  return new Uint8Array(await new Response(stream).arrayBuffer());
}

async function decodeWoffToSfnt(bytes: Uint8Array): Promise<Uint8Array> {
  if (
    bytes.length < 44
    || bytes[0] !== 0x77
    || bytes[1] !== 0x4F
    || bytes[2] !== 0x46
    || bytes[3] !== 0x46
  ) {
    return bytes;
  }

  const flavor = readU32(bytes, 4);
  const numTables = readU16(bytes, 12);
  const totalSfntSize = readU32(bytes, 16);
  if (numTables <= 0 || totalSfntSize < 12 + numTables * 16) {
    throw new Error('Invalid WOFF font: invalid table directory.');
  }

  const records: {
    tag: number;
    offset: number;
    compLength: number;
    origLength: number;
    checksum: number;
    outputOffset: number;
  }[] = [];
  let outputOffset = 12 + numTables * 16;
  for (let index = 0; index < numTables; index += 1) {
    const recordOffset = 44 + index * 20;
    const tag = readU32(bytes, recordOffset);
    const offset = readU32(bytes, recordOffset + 4);
    const compLength = readU32(bytes, recordOffset + 8);
    const origLength = readU32(bytes, recordOffset + 12);
    const checksum = readU32(bytes, recordOffset + 16);
    if (
      compLength <= 0
      || origLength <= 0
      || compLength > origLength
      || offset + compLength > bytes.length
    ) {
      throw new Error('Invalid WOFF font: invalid table record.');
    }
    records.push({ tag, offset, compLength, origLength, checksum, outputOffset });
    outputOffset = align4(outputOffset + origLength);
  }

  const sfnt = new Uint8Array(Math.max(totalSfntSize, outputOffset));
  writeU32(sfnt, 0, flavor);
  writeU16(sfnt, 4, numTables);
  let maxPowerOfTwo = 1;
  let entrySelector = 0;
  while (maxPowerOfTwo * 2 <= numTables) {
    maxPowerOfTwo *= 2;
    entrySelector += 1;
  }
  const searchRange = maxPowerOfTwo * 16;
  writeU16(sfnt, 6, searchRange);
  writeU16(sfnt, 8, entrySelector);
  writeU16(sfnt, 10, numTables * 16 - searchRange);

  let index = 0;
  for (const record of records) {
    const sfntRecordOffset = 12 + index * 16;
    writeU32(sfnt, sfntRecordOffset, record.tag);
    writeU32(sfnt, sfntRecordOffset + 4, record.checksum);
    writeU32(sfnt, sfntRecordOffset + 8, record.outputOffset);
    writeU32(sfnt, sfntRecordOffset + 12, record.origLength);

    const compressed = bytes.subarray(record.offset, record.offset + record.compLength);
    const tableBytes = record.compLength === record.origLength
      ? compressed
      : await inflateWoffTable(compressed);
    if (tableBytes.byteLength !== record.origLength) {
      throw new Error('Invalid WOFF font: inflated table length mismatch.');
    }
    sfnt.set(tableBytes, record.outputOffset);
    index += 1;
  }

  return sfnt;
}

export async function fetchGoogleFontShardBytes(
  googleFamily: string,
  text: string,
): Promise<GoogleFontShardBytes> {
  ensureGoogleFontShardPreconnect();
  const stylesheetUrl = buildGoogleFontStylesheetUrl(googleFamily, text);
  const stylesheetResponse = await fetch(stylesheetUrl, { mode: 'cors' });
  if (!stylesheetResponse.ok) {
    throw new Error(`Failed to fetch Google Fonts stylesheet ${stylesheetUrl}: ${String(stylesheetResponse.status)}`);
  }
  const binaryUrl = parseGoogleFontBinaryUrl(await stylesheetResponse.text());
  const binaryResponse = await fetch(binaryUrl);
  if (!binaryResponse.ok) {
    throw new Error(`Failed to fetch font ${binaryUrl}: ${String(binaryResponse.status)}`);
  }
  const bytes = new Uint8Array(await binaryResponse.arrayBuffer());
  return {
    url: binaryUrl,
    bytes: await decodeWoffToSfnt(bytes),
  };
}
