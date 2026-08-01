# CustomDrawable

`CustomDrawable` is a retained visual whose callback records immediate drawing
commands. The same callback runs through shared Skia-backed drawing on web and
native macOS, Windows, and Linux hosts.

## Construction

- `custom_drawable(|context| ...)`
- `CustomDrawable::new(|context| ...)`

The callback redraws current retained state and must not reconstruct the UI.
The runtime saves canvas state, clips to the drawable's current rectangular or
rounded bounds, invokes the callback, restores state, and flushes its batch.

## Invalidating

- `mark_dirty()` schedules a frame when retained drawing state changes.
- `invalidator()` returns a weak `DrawableInvalidator`; use its `mark_dirty()`
  from timers, font/image readiness callbacks, and model subscriptions without
  keeping the drawable or page alive.

## Drawing surface

`DrawContext` supports save/restore, translate/scale/rotate, rectangular and
rounded clips, rectangles, circles, lines, rounded rectangles, `Path`, retained
and dynamic text layouts, texture images with sampling, and SVG resources.
Keep referenced resources alive outside the callback.

`CustomDrawable` implements normal retained visual capabilities, including
layout, margin/padding, box styling, theming, pointer events, semantics,
focusability, and `focus_now()`.

See [Custom drawing and bitmaps](../../CUSTOM_DRAWING_AND_BITMAPS.md).

## See also

- [Bitmap](./Bitmap.md)
- [Controls and nodes](../../CONTROLS_AND_NODES.md)
- [API reference](../../API_REFERENCE.md)
