# Bitmap

`Bitmap` is an app-owned premultiplied-RGBA pixel buffer, native/WebAssembly
texture, and optional offscreen drawing surface. Clones share one resource.

## Construction and identity

- `Bitmap::new(width, height)` requires non-zero dimensions and checks byte-size overflow.
- `width()` / `height()` return physical pixel dimensions.
- `texture_id()` identifies the retained texture for `DrawContext::draw_image`.
- `pixel_ptr()` exposes the current buffer address for low-level interop; normal application code should use `pixels()`.

## Direct updates

1. Borrow `pixels()` mutably and write premultiplied RGBA bytes.
2. Drop the mutable borrow before calling another bitmap method.
3. For a full upload, call `clear_dirty_rects().commit()`.
4. For partial uploads, call `clear_dirty_rects()`, add up to 16
   `dirty_rect(x, y, width, height)` regions, then `commit()`.

`dirty_rect` clips to bitmap bounds and ignores empty/outside regions.
`has_dirty_rects()` reports whether the next commit is partial. A successful
commit consumes pending dirty rectangles and returns `texture_id()`.

## Offscreen mode

`canvas()` returns a retained offscreen `DrawContext`. Once used, every
`commit()` flushes it and reads the complete offscreen surface into the pixel
buffer before uploading. Do not mix later direct pixel writes into the same
bitmap because readback will replace them.

## Retained rasterization and text

- `render(node, x, y, scale)` rasterizes a built, laid-out retained node into
  the pixel buffer; call `commit()` afterward.
- `render_text_layout(layout, x, y, scale)` rasterizes a ready `TextLayout` and returns whether the prepared node was available to render.
- `on_text_ready(node, callback)` waits for required fonts and initial app load,
  prepares the node, and invokes the callback once.
- `prepare_text(node)` explicitly builds/prepares a text node when integrating
  a custom readiness flow.

## Lifecycle

The final `Bitmap` clone calls `dispose()`, releasing the native/WebAssembly
texture and offscreen surface. `dispose()` is idempotent and available for
early release; pixel/canvas/commit operations after disposal are invalid.

All operations above are implemented on web and native macOS, Windows, and
Linux hosts. See [Custom drawing and bitmaps](../../CUSTOM_DRAWING_AND_BITMAPS.md).

## See also

- [CustomDrawable](./CustomDrawable.md)
- [Controls and nodes](../../CONTROLS_AND_NODES.md)
- [API reference](../../API_REFERENCE.md)
