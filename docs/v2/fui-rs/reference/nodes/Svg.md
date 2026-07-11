# Svg

Retained SVG node.

## Constructor

- `svg(source)`, `Svg::new(...)`

## Key APIs

- source URL/asset, tint, sampling, alt text, inherited box styling.

## Notes

- This is retained SDK state or a retained runtime resource.
- Prefer public constructors/helpers from `fui::prelude::*`.
- Avoid raw runtime handles in app code; use public node/resource APIs.

## See also

- [Per-type reference index](../README.md)
- [Controls and nodes](../../CONTROLS_AND_NODES.md)
- [API reference](../../API_REFERENCE.md)
