# TextLayout

Retained formatted text resource for custom drawing. A layout may wait for
fonts asynchronously. Check readiness or register `on_ready(...)`, retain the
returned readiness state as required by the API, and invalidate the owning
`CustomDrawable` before drawing it.

Use ordinary retained `Text` or `RichText` when the text should participate in
normal retained layout and selection.

See [Custom fonts](../../CUSTOM_FONTS.md) and
[Custom drawing and bitmaps](../../CUSTOM_DRAWING_AND_BITMAPS.md).

Immediate-mode formatted text resource.

## Constructor

- `TextLayout::text(...)`, `TextLayout::rich(...)`

## Key APIs

- ready callbacks, text metrics, drawing via `DrawContext`.

## Notes

- This is retained SDK state or a retained runtime resource.
- Prefer public constructors/helpers from `fui::prelude::*`.
- Avoid raw runtime handles in app code; use public node/resource APIs.

## See also

- [Per-type reference index](../README.md)
- [Controls and nodes](../../CONTROLS_AND_NODES.md)
- [API reference](../../API_REFERENCE.md)
