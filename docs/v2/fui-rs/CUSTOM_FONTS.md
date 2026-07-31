# Custom fonts

FUI-RS models app typography with `FontFace`, `FontStack`, and `FontFamily`.
Applications should not allocate or split internal runtime font IDs.

## Registering app fonts

1. Stage each font file in the logical application asset set used by every
   target.
2. Load physical faces with `FontFace::load(...)`.
3. Build an ordered `FontStack` for script and emoji fallback.
4. Build a `FontFamily` when regular, bold, italic, or bold-italic requests
   should resolve to different stacks.
5. Apply the family or stack through public text and theme APIs.

The routed demo contains complete family, direct-stack, mixed-script, and emoji
examples in
[`custom_font_section.rs`](https://github.com/zion-sati/fui-rs-demo/blob/main/crates/routes/text-fonts/src/custom_font_section.rs).

## Readiness

Font loading is asynchronous. Ordinary retained `Text` and `RichText` nodes
track their required resources and refresh when fonts become ready. Do not add
an application loading overlay or manual invalidation only to refresh those
nodes.

Custom drawing and bitmap rasterization own their output lifecycle. A
`TextLayout` readiness callback should invalidate its `CustomDrawable`; bitmap
text should use `Bitmap::on_text_ready(...)` and commit after rasterization.

## Browser and native assets

Browser hosts resolve logical font assets from published URLs with the required
MIME, CORS, and cache behavior. Native hosts resolve the same logical assets
from packaged application resources and can use system fallback where the
application did not supply a deterministic face. Do not hard-code source-tree
paths or require native applications to fetch Google Fonts.

## Fallback coverage

One Latin face does not imply emoji, CJK, Arabic, Thai, or symbol coverage.
Place deterministic app-supplied faces first, then ordered packaged or system
fallbacks. Use an actual monospaced face when claiming monospaced rendering.

See [`FontFace`](./reference/README.md), [`FontStack`](./reference/README.md),
[`FontFamily`](./reference/README.md), and
[Custom drawing and bitmaps](./CUSTOM_DRAWING_AND_BITMAPS.md).
