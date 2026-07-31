# FontFace

Retained physical font resource loaded from a logical application asset URL.

Use `FontFace::load(url)` for an app-supplied face. `is_loaded()` reports its
current state and `on_loaded(...)` runs immediately when already loaded or once
after asynchronous loading completes.

Ordinary retained `Text` and `RichText` nodes refresh automatically. Use the
callback directly only when application-owned drawing or rasterization must be
invalidated.

See [Custom fonts](../../CUSTOM_FONTS.md).
