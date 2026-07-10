# Dropdown

Non-editable selection popup.

## Constructor

- `dropdown()`, `Dropdown::new()`

## Key APIs

- `items`, `select_index`, `selected_index`, `on_changed`, `max_visible_items`, `popup_width`, templates, colors, sizing.

## Notes

- This is a retained control. Clone values are cheap handles to the same control.
- Store the control in a page/controller field when callbacks need to mutate it later.
- Use `use fui_rs::prelude::*;` in app code.

## See also

- [Per-type reference index](../README.md)
- [Controls and nodes](../../CONTROLS_AND_NODES.md)
- [Events and callbacks](../../EVENTS_AND_CALLBACKS.md)
