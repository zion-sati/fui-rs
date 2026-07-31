# Custom drawing and bitmaps

FUI-RS supports retained immediate drawing through `CustomDrawable` and direct
pixel ownership through `Bitmap`. Choose the path that matches the source data.

## Immediate drawing

Use `custom_drawable(...)` for resolution-independent output reproducible from
retained state. `DrawContext` supports transforms, clipping, shapes, paths,
text layouts, images, and SVGs.

Retain expensive paths, text layouts, images, SVGs, and paint state outside the
draw callback. Mutate retained state and call `mark_dirty()` only when output
changes. Prefer a `DrawableInvalidator` in callbacks so readiness or animation
state does not strongly retain the drawable.

For drawing gestures, capture the pointer on pointer-down and release both the
capture and local pressed state on pointer-up or pointer-cancel. Moving outside
the drawable must not leave a latent drawing gesture.

## Text in custom drawing

Use `TextLayout` for formatted text whose layout can be retained. Use
`DynamicTextLayout` for short frequently changing values with a known character
set, including numeric counters and chart labels. Both can wait for fonts;
invalidate the drawable from their readiness callback.

Use retained `Text` or `RichText` instead when content should participate in
ordinary retained layout, selection, and automatic font refresh.

## Bitmap drawing

`Bitmap` stores premultiplied RGBA pixels. Given straight channels `r`, `g`,
`b`, and `a`, store `(channel * a + 127) / 255` for each color channel and store
`a` unchanged.

Mutate `pixels()`, release the mutable borrow, optionally record dirty
rectangles, and call `commit()`. An empty dirty-region list uploads the complete
bitmap. `canvas()` provides an offscreen `DrawContext`; commit after drawing.

Use `on_text_ready(...)` before rasterizing text into a bitmap. The bitmap owns
its texture and offscreen canvas until its last handle drops; use explicit
disposal only for intentional early release.

Complete examples are available in the demo's
[`immediate-drawing widgets`](https://github.com/zion-sati/fui-rs-demo/tree/main/crates/routes/immediate-drawing/src/widgets)
and [`bitmap sample`](https://github.com/zion-sati/fui-rs-demo/blob/main/crates/routes/home/src/bitmap.rs).
