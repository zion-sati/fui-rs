# NavLink

Route/link control integrated with browser navigation.

## Constructor

- `nav_link(href)`, `NavLink::new(href)`

## Key APIs

- `href_to`, `label`, `new_tab`, `on_navigate`, inherited node/text styling APIs.

## Notes

- This is a retained control. Clone values are cheap handles to the same control.
- Store the control in a page/controller field when callbacks need to mutate it later.
- Use `use fui_rs::prelude::*;` in app code.

## See also

- [Per-type reference index](../README.md)
- [Controls and nodes](../../CONTROLS_AND_NODES.md)
- [Events and callbacks](../../EVENTS_AND_CALLBACKS.md)
