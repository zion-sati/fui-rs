# Custom drawing and bitmaps

FUI-RS provides the same retained custom-drawing API in browser/WebAssembly and
native macOS, Windows, and Linux applications. Application code records
host-neutral `DrawContext` commands; EffinDOM executes them through its shared
Skia renderer. Native window/input integration uses SDL3 and the native host
selects Metal, D3D, Vulkan, or software recovery without changing FUI-RS code.

Choose `CustomDrawable` for resolution-independent output reproducible from
retained state. Choose `Bitmap` when the application owns RGBA pixels, needs an
offscreen canvas, or needs to rasterize a retained subtree.

## Immediate drawing

```rust
use fui::prelude::*;

let mut marker = Path::new();
marker
    .move_to(8.0, 42.0)
    .line_to(28.0, 12.0)
    .line_to(48.0, 42.0)
    .close();

let canvas = custom_drawable(move |context| {
    context.draw_round_rect(
        0.0,
        0.0,
        320.0,
        120.0,
        12.0,
        12.0,
        Paint::fill(rgb(0x0b, 0x16, 0x2a)),
    );
    context.save();
    context.translate(24.0, 18.0);
    context.clip_rect(0.0, 0.0, 80.0, 64.0);
    context.draw_path(&marker, Paint::stroke(rgb(0x38, 0xbd, 0xf8), 3.0));
    context.restore();
});

canvas
    .height(120.0, Unit::Pixel)
    .corner_radius(12.0)
    .semantic_label("Waveform preview");
```

The callback redraws current retained state; it must not rebuild the UI tree.
The runtime clips it to the drawable bounds and flushes its command batch after
the callback. Retain paths, text layouts, image/SVG resources, and paint state
outside the callback.

After state changes, call `CustomDrawable::mark_dirty()`. Prefer a
`DrawableInvalidator` in timers, readiness callbacks, and model subscriptions;
it weakly references the drawable and will safely do nothing after teardown.

For drawing gestures, capture the pointer on pointer-down and release both the
capture and local pressed state on pointer-up or pointer-cancel. Moving outside
the drawable must not leave a latent drawing gesture.

## Paths and transforms

`Path` supports `move_to`, `line_to`, quadratic/cubic curves, `close`,
`add_rect`, and `add_circle`. It releases the engine path when its final Rust
handle drops. A `DrawContext` retains every path referenced by the current
batch until `flush()`, so temporary clones cannot disappear during replay.

Balance every `save()` with `restore()`. Transforms and clips affect subsequent
commands in order. `CustomDrawable` applies its own bounds clip before invoking
the callback; additional clips are local to application drawing.

## Text, images, and SVGs

Use `TextLayout` for formatted text whose layout can be retained. Use
`DynamicTextLayout` for short frequently changing values with a known character
set, including numeric counters and chart labels. Register their readiness
callbacks and invalidate the drawable when fonts become ready.

`draw_text_node`, `draw_text_layout`, `draw_dynamic_text_layout`, `draw_image`,
`draw_image_sampling`, and `draw_svg` all use retained resource IDs. Keep those
resources alive for at least as long as the drawable can reference them. Use
ordinary retained `Text` or `RichText` when content should participate in
layout, selection, semantics, and automatic font refresh.

## Direct pixel bitmaps

`Bitmap` stores four-byte-per-pixel **premultiplied RGBA**. Given straight
channels `r`, `g`, `b`, and `a`, store `(channel * a + 127) / 255` for each
color channel and store `a` unchanged.

```rust
use fui::prelude::*;

let bitmap = Bitmap::new(96, 40);
{
    let mut pixels = bitmap.pixels();
    for pixel in pixels.chunks_exact_mut(4) {
        pixel.copy_from_slice(&[14, 116, 190, 255]);
    }
} // release the RefMut before calling any other Bitmap method

bitmap.clear_dirty_rects().commit(); // no dirty rects means a full upload
```

For a partial update, mutate the corresponding bytes, release the mutable
borrow, then call `clear_dirty_rects().dirty_rect(x, y, w, h).commit()`.
Rectangles are clipped to the bitmap, zero/outside rectangles are ignored, and
at most 16 rectangles are retained per commit. `commit()` consumes the pending
dirty list. A first dirty commit creates a transparent full texture before
applying the subregion.

Never call `width()`, `height()`, `commit()`, or another bitmap method while a
`pixels()` borrow is alive; `Bitmap` uses checked runtime borrowing.

## Offscreen drawing and readback

`bitmap.canvas()` returns a persistent offscreen `DrawContext`. Draw commands
are flushed and the offscreen surface is read back into `pixels()` by the next
`commit()`, after which the bitmap texture can be sampled with
`context.draw_image(bitmap.texture_id(), ...)`.

Treat a bitmap as either direct-pixel-owned or offscreen-canvas-owned. Once
`canvas()` has been used, every later commit refreshes the pixel buffer from the
offscreen surface; direct pixel writes made afterward would be overwritten.
Dirty rectangles still restrict texture upload, but offscreen readback itself
is complete.

## Retained-node rasterization

`Bitmap::render(&node, x, y, scale)` renders a built, laid-out retained subtree
into the bitmap's RGBA buffer. Call it after the first layout (for example from
`on_loaded` followed by deferred work or from an explicit user action), then
call `commit()`. `scale` maps logical coordinates to physical bitmap pixels and
should normally match the intended device-pixel ratio. It returns `true` when
the prepared node was available for rasterization; retain retryable state when
calling it during custom drawing because preparation may commit on the next frame.

Use `on_text_ready(...)` before bitmap text rasterization. For a prepared
`TextLayout`, `render_text_layout(...)` is the convenience equivalent.

## Timer-driven drawing

FUI-RS timers are one-shot and run their callback on the application UI queue
on both web and native hosts. Store the `TimerHandle` when cancellation is
required. Recurring animation rearms a new timeout only while its app-owned
running state remains true, mutates retained state, and then invalidates the
drawable. Cancel the outstanding handle when pausing or disposing owner state.

## Resource lifecycle

- `Bitmap` clones share one texture/offscreen resource. The final clone
  releases both; `dispose()` is available for intentional early teardown.
- `Path` clones share one native path and release it on final drop.
- `DrawableInvalidator` is weak and does not keep a page alive.
- A disposed application must not retain timers or callbacks that mutate its
  former UI state.

## Troubleshooting

| Symptom | Check |
| --- | --- |
| Blank `CustomDrawable` | Give it non-zero retained bounds, keep resources alive, and ensure state changes call `mark_dirty()` or an invalidator. |
| Bitmap remains transparent/stale | Release the `pixels()` borrow and call `commit()` before drawing its texture. |
| Partial update is wrong | Update bytes using the full bitmap stride, clear stale dirty rectangles, then add the exact clipped region before commit. |
| Pixel writes vanish after commit | Do not mix direct writes with a bitmap that has entered offscreen-canvas mode. |
| Retained raster is empty | Wait until the source node is built and laid out; use its current bounds and commit after `render()`. |
| Text is missing | Wait for `TextLayout`/font readiness and invalidate after the callback. |
| Animation never advances | Rearm the one-shot timer and invalidate after each state mutation; keep cancellation state app-owned. |

Complete examples are available in the demo's
[`immediate-drawing widgets`](https://github.com/zion-sati/fui-rs-demo/tree/main/crates/routes/immediate-drawing/src/widgets),
the [`bitmap sample`](https://github.com/zion-sati/fui-rs-demo/blob/main/crates/routes/home/src/bitmap.rs),
and the internal native FUI-RS showcase in
`v2/fui-rs/native-demo/src/lib.rs`.
