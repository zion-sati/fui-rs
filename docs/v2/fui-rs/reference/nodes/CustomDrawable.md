# CustomDrawable

Retained immediate-mode drawing surface. Its callback redraws current retained
state; it does not rebuild the UI tree.

Keep paths, images, SVGs, text layouts, and other expensive resources outside
the draw callback. Call `mark_dirty()` only when visible state changes, and use
`DrawableInvalidator` from long-lived callbacks to avoid strongly retaining the
drawable.

Pointer-driven drawing must capture the pointer while active and release state
on pointer-up and pointer-cancel, including when the pointer leaves the bounds.

See [Custom drawing and bitmaps](../../CUSTOM_DRAWING_AND_BITMAPS.md).

Retained custom drawing surface.

## Constructor

- `custom_drawable(|ctx| ...)`, `CustomDrawable::new(...)`

## Key APIs

- `DrawContext` drawing commands, `mark_dirty`, inherited box styling.

## Notes

- This is retained SDK state or a retained runtime resource.
- Prefer public constructors/helpers from `fui::prelude::*`.
- Avoid raw runtime handles in app code; use public node/resource APIs.

## See also

- [Per-type reference index](../README.md)
- [Controls and nodes](../../CONTROLS_AND_NODES.md)
- [API reference](../../API_REFERENCE.md)
