# Text

Selectable retained text node.

## Constructor

- `text(content)`, `Text::new(content)`

## Key APIs

- `text`, `font_family`, `font_size`, `font_weight`, `font_style`, `text_color`, alignment, selection styling.

## Notes

- This is retained SDK state or a retained runtime resource.
- Prefer public constructors/helpers from `fui::prelude::*`.
- Avoid raw runtime handles in app code; use public node/resource APIs.

## See also

- [Per-type reference index](../README.md)
- [Controls and nodes](../../CONTROLS_AND_NODES.md)
- [API reference](../../API_REFERENCE.md)
