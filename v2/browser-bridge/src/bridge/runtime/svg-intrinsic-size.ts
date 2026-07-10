import type { AssetLoadResult } from '../../core-types';

const svgTextDecoder = new TextDecoder();
const svgTextEncoder = new TextEncoder();
const SVG_OPEN_TAG_PATTERN = /<svg\b([^>]*)>/i;
const SVG_ATTRIBUTE_PATTERN = /([^\s=/>]+)\s*=\s*(?:"([^"]*)"|'([^']*)')/g;

function readRootSvgAttributes(markup: string): Map<string, string> {
  const svgTagMatch = SVG_OPEN_TAG_PATTERN.exec(markup);
  if (svgTagMatch === null) {
    return new Map<string, string>();
  }
  const rawAttributes = svgTagMatch[1] ?? '';
  const attributes = new Map<string, string>();
  SVG_ATTRIBUTE_PATTERN.lastIndex = 0;
  let attributeMatch: RegExpExecArray | null = SVG_ATTRIBUTE_PATTERN.exec(rawAttributes);
  while (attributeMatch !== null) {
    const name = attributeMatch[1];
    const value = attributeMatch[2] ?? attributeMatch[3] ?? '';
    if (name !== undefined) {
      attributes.set(name, value);
    }
    attributeMatch = SVG_ATTRIBUTE_PATTERN.exec(rawAttributes);
  }
  return attributes;
}

function parseSvgLength(value: string | null | undefined): number | null {
  if (value === null || value === undefined) {
    return null;
  }
  const trimmed = value.trim();
  if (trimmed.length === 0 || trimmed.endsWith('%')) {
    return null;
  }
  const match = /^([+-]?(?:\d+\.?\d*|\.\d+))(?:\s*px)?$/i.exec(trimmed);
  if (match === null) {
    return null;
  }
  const parsed = Number.parseFloat(match[1] ?? '');
  return Number.isFinite(parsed) && parsed > 0 ? parsed : null;
}

function parseSvgViewBox(value: string | null | undefined): AssetLoadResult | null {
  if (value === null || value === undefined) {
    return null;
  }
  const parts = value
    .trim()
    .split(/[\s,]+/)
    .map((entry) => Number.parseFloat(entry))
    .filter((entry) => Number.isFinite(entry));
  if (parts.length !== 4) {
    return null;
  }
  const width = parts[2];
  const height = parts[3];
  if (width === undefined || height === undefined || width <= 0 || height <= 0) {
    return null;
  }
  return { width, height };
}

export function parseSvgIntrinsicSizeFromMarkup(markup: string): AssetLoadResult {
  const attributes = readRootSvgAttributes(markup);
  const width = parseSvgLength(attributes.get('width'));
  const height = parseSvgLength(attributes.get('height'));
  if (width !== null && height !== null) {
    return { width, height };
  }

  const viewBox = parseSvgViewBox(attributes.get('viewBox'));
  if (viewBox !== null) {
    if (width !== null) {
      return {
        width,
        height: width * (viewBox.height / viewBox.width),
      };
    }
    if (height !== null) {
      return {
        width: height * (viewBox.width / viewBox.height),
        height,
      };
    }
    return viewBox;
  }

  return {
    width: width ?? 1,
    height: height ?? 1,
  };
}

function stripRootSvgAttribute(tag: string, attributeName: 'width' | 'height'): string {
  const pattern = attributeName === 'width'
    ? /\swidth\s*=\s*(?:"[^"]*"|'[^']*')/i
    : /\sheight\s*=\s*(?:"[^"]*"|'[^']*')/i;
  return tag.replace(pattern, '');
}

export function normalizeSvgMarkupForCore(markup: string): string {
  const rootTagMatch = SVG_OPEN_TAG_PATTERN.exec(markup);
  if (rootTagMatch?.index === undefined) {
    return markup;
  }

  const attributes = readRootSvgAttributes(markup);
  const width = parseSvgLength(attributes.get('width'));
  const height = parseSvgLength(attributes.get('height'));
  if (width !== null && height !== null) {
    return markup;
  }

  const intrinsicSize = parseSvgIntrinsicSizeFromMarkup(markup);
  const originalTag = rootTagMatch[0];
  const selfClosing = originalTag.endsWith('/>');
  const closing = selfClosing ? '/>' : '>';
  const tagBody = stripRootSvgAttribute(stripRootSvgAttribute(
    originalTag.slice(0, originalTag.length - closing.length),
    'width',
  ), 'height');
  const normalizedTag = `${tagBody} width="${String(intrinsicSize.width)}" height="${String(intrinsicSize.height)}"${closing}`;
  return `${markup.slice(0, rootTagMatch.index)}${normalizedTag}${markup.slice(rootTagMatch.index + originalTag.length)}`;
}

export function parseSvgIntrinsicSize(bytes: Uint8Array): AssetLoadResult {
  return parseSvgIntrinsicSizeFromMarkup(svgTextDecoder.decode(bytes));
}

export function normalizeSvgBytesForCore(bytes: Uint8Array): Uint8Array {
  return svgTextEncoder.encode(normalizeSvgMarkupForCore(svgTextDecoder.decode(bytes)));
}
