# FUI-RS Overlays and Portals (v2)

Overlay controls attach detached retained surfaces above the normal layout tree.
They are used for dropdowns, context menus, dialogs, tooltips, and custom popup
surfaces.

## Behavior matrix

| Surface | Open trigger | Placement | Close triggers | Key ownership |
|---|---|---|---|---|
| `Dropdown` | click or activation keys | anchored to trigger | selection, overlay click, Escape | yes while open |
| `ComboBox` | click, typing, activation keys | anchored to editor | commit, blur, Escape | yes while open |
| `ContextMenu` | `show(...)` or context gesture | point placement, clamped | action invoke, outside click, Escape | yes while open |
| `Dialog` | `show()` | centered modal | `hide()`, accept/cancel, backdrop click | yes while open |
| `Popup` | app-controlled | app-controlled | app-controlled | app-controlled |
| `ToolTip` | hover/focus timing | near target/pointer | pointer/focus exit | no app key ownership |

## Context menus

Create menus with `ContextMenu::new()` or `context_menu(items)`:

```rust
let menu = context_menu(vec![
    MenuItem::command("Rename", || logger::log("Menu", "Rename")),
    MenuItem::command("Delete", || logger::log("Menu", "Delete")),
    MenuItem::separator(),
    MenuItem::new("Copy", ContextMenuAction::CopyCurrentSelection),
    MenuItem::new("Select all", ContextMenuAction::SelectAllText),
]);

menu.show(120.0, 80.0);
```

`MenuItem::command(label, callback)` is an application command; the other items
run a built-in `ContextMenuAction`, and `on_invoke(...)` adds a callback that runs
before it. Use `show(x, y)` for absolute scene coordinates and
`show_relative_to(&control, x, y)` for coordinates relative to a control.

## Dialogs

```rust
let sign_in = dialog("Sign in", "Enter your credentials");
sign_in
    .on_accept(|| logger::log("Dialog", "accepted"))
    .on_cancel(|| logger::log("Dialog", "cancelled"));

button("Open dialog").on_click({
    let sign_in = sign_in.clone();
    move |_| sign_in.show()
});
```

Dialogs are modal while open. Enter/Escape route through the active dialog/form
behavior.

## Dropdowns and comboboxes

Dropdowns and comboboxes own their popup keyboard behavior while open. They
collapse on commit, Escape, blur, and outside interaction.

## Lifecycle guarantees

- Repeated open/close calls are safe.
- Closing an overlay removes active keyboard interception.
- Disposed overlays detach their retained visual surfaces.
- Overlay placement is clamped to viewport bounds.

## See also

- [Keyboard policy](./KEYBOARD_POLICY.md)
- [Events and callbacks](./EVENTS_AND_CALLBACKS.md)
- [Theming and style matrix](./THEMING_STYLE_MATRIX.md)
