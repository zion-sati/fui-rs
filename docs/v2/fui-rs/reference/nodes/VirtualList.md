# VirtualList

Pooled retained list surface.

## Constructor

- `virtual_list(total_items, item_height)`, `VirtualList::new(...)`

## Key APIs

- `on_bind_item`, total items, item height, scroll box/state access, recycled row rendering.

## Notes

- This is retained SDK state or a retained runtime resource.
- Prefer public constructors/helpers from `fui::prelude::*`.
- Avoid raw runtime handles in app code; use public node/resource APIs.

## See also

- [Per-type reference index](../README.md)
- [Controls and nodes](../../CONTROLS_AND_NODES.md)
- [API reference](../../API_REFERENCE.md)
