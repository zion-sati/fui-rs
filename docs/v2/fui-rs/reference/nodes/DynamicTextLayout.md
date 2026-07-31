# DynamicTextLayout

Retained custom-drawing text layout optimized for short text that changes
frequently within a known character set, such as counters, clocks, chart
labels, and numeric values. Create it outside the draw callback and update its
value rather than rebuilding a full `TextLayout` every frame.

It follows the same asynchronous font-readiness and drawable invalidation
rules as `TextLayout`.

See [Custom drawing and bitmaps](../../CUSTOM_DRAWING_AND_BITMAPS.md).

Immediate-mode short-label text resource.

## Constructor

- `DynamicTextLayout::fixed_charset(...)`

## Key APIs

- `set_text`, overflow behavior, drawing via `DrawContext`.

## Notes

- This is retained SDK state or a retained runtime resource.
- Prefer public constructors/helpers from `fui::prelude::*`.
- Avoid raw runtime handles in app code; use public node/resource APIs.

## See also

- [Per-type reference index](../README.md)
- [Controls and nodes](../../CONTROLS_AND_NODES.md)
- [API reference](../../API_REFERENCE.md)
