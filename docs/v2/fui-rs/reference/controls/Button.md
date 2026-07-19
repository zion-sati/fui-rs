# Button

Theme-aware action control.

## Constructor

- `button(label)`, `Button::new(label)`

## Key APIs

- `on_click`, `template`, `colors`,
  `bind_theme`, inherited node/layout APIs.

`on_click(...)` is high-level Button activation and includes supported pointer
and keyboard input. Use inherited `on_pointer_click(...)` only when raw routed
pointer data is required; it is not a replacement for control activation.
Exact raw pointer gestures use inherited `on_pointer_double_click(...)` and
`on_pointer_triple_click(...)`. Semantic `ClickEventArgs` intentionally carries
no click count.

## Theme-aware control styling

Use `bind_theme(...)` when theme changes must call Button-specific APIs. The
callback receives the retained `Button`, rather than only its underlying
`FlexBox`, and the subscription is owned by the control.

```rust
use fui::prelude::*;

let save = button("Save");
save.bind_theme(|button, theme| {
    button.colors(
        ButtonColors::new()
            .background(theme.colors.accent)
            .text_primary(theme.colors.text_on_accent),
    );
});
```

The binding uses a weak retained target internally, so it does not introduce an
`Rc` ownership cycle. Use the free `bind_theme(...)`/`subscribe(...)` APIs only
when the subscription lifetime is not naturally owned by a retained node.

## Notes

- This is a retained control. Clone values are cheap handles to the same control.
- Store the control in a page/controller field when callbacks need to mutate it later.
- Use `use fui::prelude::*;` in app code.

## See also

- [Per-type reference index](../README.md)
- [Controls and nodes](../../CONTROLS_AND_NODES.md)
- [Events and callbacks](../../EVENTS_AND_CALLBACKS.md)
