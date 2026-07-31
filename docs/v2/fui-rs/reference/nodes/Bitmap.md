# Bitmap

Retained pixel buffer/GPU texture resource.

The pixel buffer is four-byte-per-pixel **premultiplied RGBA**, not
straight-alpha RGBA. Premultiply each color channel by alpha before storing it.

## Constructor

`Bitmap::new(width, height)` requires non-zero dimensions.

## Updating pixels

Borrow `pixels()` mutably, update the required bytes, release the borrow, add
dirty rectangles when partial upload is useful, and call `commit()`. With no
dirty rectangles, commit uploads the complete bitmap.

`canvas()` provides an offscreen `DrawContext`. Commit after drawing to flush
and upload its result.

## Text readiness

Text and fonts can become ready asynchronously. Use `on_text_ready(...)` before
rasterizing text into the bitmap. Ordinary retained `Text` and `RichText` nodes
track readiness automatically; this callback is for application-owned bitmap
rasterization.

See [Custom drawing and bitmaps](../../CUSTOM_DRAWING_AND_BITMAPS.md).

- `Bitmap` constructors

## Key APIs

- pixel access and retained bitmap text rendering support.

## Notes

- This is retained SDK state or a retained runtime resource.
- Prefer public constructors/helpers from `fui::prelude::*`.
- Avoid raw runtime handles in app code; use public node/resource APIs.

## See also

- [Per-type reference index](../README.md)
- [Controls and nodes](../../CONTROLS_AND_NODES.md)
- [API reference](../../API_REFERENCE.md)
