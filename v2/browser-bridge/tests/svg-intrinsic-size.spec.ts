import { expect, test } from '@playwright/test';

import { normalizeSvgMarkupForCore, parseSvgIntrinsicSizeFromMarkup } from '../src/bridge/runtime/svg-intrinsic-size';

test('percentage root SVG sizes fall back to viewBox intrinsic dimensions', () => {
  const size = parseSvgIntrinsicSizeFromMarkup(
    "<svg xmlns='http://www.w3.org/2000/svg' width='100%' height='100%' viewBox='0 0 1000 620' preserveAspectRatio='xMinYMin meet'></svg>",
  );

  expect(size).toEqual({
    width: 1000,
    height: 620,
  });
});

test('single absolute SVG axis derives the missing dimension from the viewBox ratio', () => {
  const size = parseSvgIntrinsicSizeFromMarkup(
    "<svg xmlns='http://www.w3.org/2000/svg' width='240' viewBox='0 0 1000 620'></svg>",
  );

  expect(size.width).toBe(240);
  expect(size.height).toBeCloseTo(148.8, 4);
});

test('bridge normalization materializes intrinsic root dimensions for percentage-sized SVGs', () => {
  const normalized = normalizeSvgMarkupForCore(
    "<svg xmlns='http://www.w3.org/2000/svg' width='100%' height='100%' viewBox='0 0 1000 620'></svg>",
  );

  expect(normalized).toContain("width=\"1000\"");
  expect(normalized).toContain("height=\"620\"");
  expect(normalized).not.toContain("width='100%'");
  expect(normalized).not.toContain("height='100%'");
});
