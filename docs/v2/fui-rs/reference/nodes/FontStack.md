# FontStack

Ordered font fallback resource. Create one from a `FontFace` or load its first
face directly, then append fallbacks with `fallback_face(...)`,
`fallback_stack(...)`, or `fallback_loaded(...)`.

`is_loaded()` covers every required face. `on_loaded(...)` runs after all faces
needed by the stack are ready. Put deterministic app-supplied coverage before
packaged or system fallback.

See [Custom fonts](../../CUSTOM_FONTS.md).
