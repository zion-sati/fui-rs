# RichText

Attributed inline rich text resource.

## Constructor

- `RichText::new(...)`, `span(text)`

## Key APIs

- span-level color/background/bold/italic/font styling used by retained and immediate text surfaces.

## Notes

- This is retained SDK state or a retained runtime resource.
- Prefer public constructors/helpers from `fui::prelude::*`.
- Avoid raw runtime handles in app code; use public node/resource APIs.

## See also

- [Per-type reference index](../README.md)
- [Controls and nodes](../../CONTROLS_AND_NODES.md)
- [API reference](../../API_REFERENCE.md)
