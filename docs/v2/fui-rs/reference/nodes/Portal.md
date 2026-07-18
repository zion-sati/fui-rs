# Portal

Overlay-host configuration for a retained `FlexBox`.

## Constructor

- `portal()` returns a `FlexBox` configured as a non-clipping portal.

## Key APIs

- inherited `FlexBox` layout, child composition, styling, and event APIs.
- detached overlay composition surface used by overlay controls and custom surfaces.

## Notes

- This is retained SDK state or a retained runtime resource.
- Prefer public constructors/helpers from `fui::prelude::*`.
- Avoid raw runtime handles in app code; use public node/resource APIs.

## See also

- [Per-type reference index](../README.md)
- [Controls and nodes](../../CONTROLS_AND_NODES.md)
- [API reference](../../API_REFERENCE.md)
