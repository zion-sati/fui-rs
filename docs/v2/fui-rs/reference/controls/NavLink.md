# NavLink

Route/link control integrated with browser navigation.

## Constructor

- `nav_link(href)`, `NavLink::new(href)`

## Key APIs

- `href_to`, `open_in_new_tab`, `on_navigate`, `bind_interaction_state`, and
  inherited `FlexBox` child/layout/style APIs.

`NavLink` is a navigation behavior and arbitrary-content host. It does not
create a label or own text styling. Add any retained child tree and provide an
explicit semantic label when the child content does not project a sufficient
accessible name:

```rust
let label = text("Documentation").selectable(false);
let link = nav_link("/docs");
link.child(&label).semantic_label("Documentation");

let label_for_state = label.clone();
link.bind_interaction_state(move |state, theme| {
    label_for_state.text_color(if state.hovered || state.pressed {
        theme.colors.accent_hovered
    } else {
        theme.colors.accent
    });
});
```

## Notes

- This is a retained control. Clone values are cheap handles to the same control.
- Content can be text, icons, badges, composed rows, custom drawing, or any
  other retained node tree.
- Store the control in a page/controller field when callbacks need to mutate it later.
- Use `use fui::prelude::*;` in app code.

## See also

- [Per-type reference index](../README.md)
- [Controls and nodes](../../CONTROLS_AND_NODES.md)
- [Events and callbacks](../../EVENTS_AND_CALLBACKS.md)
