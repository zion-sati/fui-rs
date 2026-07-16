# ContextMenu

Retained context menu surface.

## Constructor

- `context_menu(items)`, `ContextMenu::new()`

## Key APIs

- `items`, `show`, `hide`, `on_visibility_changed`,
  `appearance(ContextMenuAppearance)`, `clear_appearance`. Use `MenuItem` for
  actions/separators.

## Notes

- This is a retained control. Clone values are cheap handles to the same control.
- Store the control in a page/controller field when callbacks need to mutate it later.
- Use `use fui::prelude::*;` in app code.

## See also

- [Per-type reference index](../README.md)
- [Controls and nodes](../../CONTROLS_AND_NODES.md)
- [Events and callbacks](../../EVENTS_AND_CALLBACKS.md)
