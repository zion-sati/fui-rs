# RichText

Attributed inline rich text resource.

## Constructor

- `rich_text![...]`, `RichText::new(...)`, `span(text)`

## Key APIs

- span-level color/background/bold/italic/font styling used by retained and immediate text surfaces.
- literal spans, braced dynamic text, and `span => expression` prebuilt spans through `rich_text!`.

## Notes

- This is retained SDK state or a retained runtime resource.
- Prefer public constructors/helpers from `fui::prelude::*`.
- Prefer `rich_text!` for static or mixed fluent spans; use `RichText::new(...)`
  when the span vector is assembled programmatically.
- Avoid raw runtime handles in app code; use public node/resource APIs.

## See also

- [Per-type reference index](../README.md)
- [Controls and nodes](../../CONTROLS_AND_NODES.md)
- [API reference](../../API_REFERENCE.md)
