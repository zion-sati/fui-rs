# FUI-RS Keyboard Policy (v2)

This page documents key behavior you can rely on when building FUI-RS apps.

## Routing model

1. Focused controls receive key events first.
2. Active modal and overlay surfaces can intercept keys while open.
3. Top-most active overlay wins global key ownership.
4. Keyboard focus adorners are shown for keyboard-driven focus, not pointer focus.
5. Browser shortcuts pass through unless a focused control or user handler marks the key handled.

## Control key contracts

| Surface | Key(s) | Behavior |
|---|---|---|
| `Button` | Enter, Space | release-based activation |
| `Checkbox` / `Switch` / `RadioButton` | Space | down arms pressed visual, up commits |
| `Form` | Enter, Escape | default/cancel action |
| `Dialog` | Enter, Escape | delegates to active form/default/cancel actions |
| `Dropdown` closed | Enter, Space, ArrowDown, ArrowUp | opens popup |
| `Dropdown` open | Escape, Enter, Home, End, ArrowUp, ArrowDown | close, commit, jump, or move highlight |
| `ComboBox` | text input keys, arrows, Enter, Escape | text edit, filter, highlight, commit/cancel |
| `ContextMenu` open | Escape | closes menu |
| `NavLink` | Enter and primary shortcut variants | route/link activation |
| `Slider` | Home, End, Arrow keys | value updates by orientation |
| `TextInput` / `TextArea` | editing/navigation shortcuts | editor-owned behavior |

## Text editors

Text editors consume editing/navigation keys they own. Arrow keys move caret or
selection without scrolling ancestor scroll views. Browser shortcuts pass through
when not handled by the editor or app.

`TextArea` can be configured to accept literal tab characters:

```rust
text_area().accepts_tab(true);
```

When `accepts_tab(false)`, Tab participates in focus traversal.

## Browser shortcut policy

FUI-RS does not blanket-block browser shortcuts. If a focused control or app
handler handles a key, the browser default is suppressed. If not handled, the
browser receives it.

## Platform helpers

Use platform helpers instead of hard-coding Ctrl/Cmd behavior:

- `platform::primary_shortcut_modifier()`
- `platform::has_primary_shortcut_modifier(modifiers)`
- `platform::is_undo_shortcut(key, modifiers)`
- `platform::is_redo_shortcut(key, modifiers)`
- `platform::format_primary_shortcut_label(key)`

## See also

- [Events and callbacks](./EVENTS_AND_CALLBACKS.md)
- [Text input reference](./TEXT_INPUT_REFERENCE.md)
- [Overlays and portals](./OVERLAYS_AND_PORTALS.md)
