# TextInput / TextArea Reference (v2 FUI-RS)

Public imports:

```rust
use fui_rs::prelude::*;
```

FUI-RS editable text APIs expose user-facing caret/selection positions as
Unicode scalar-value character positions. Internally the runtime boundary uses
UTF-8 byte offsets; the SDK converts at the boundary.

## `TextInput`

Single-line retained text editor.

### Constructor

```rust
let input = text_input();
let input = TextInput::new();
```

### Common methods

- `text(value)`
- `placeholder(value)`
- `max_chars(limit)` (`limit < 0` means unlimited)
- `read_only(flag)`
- `password(flag)`
- `host_autofill(hint)`
- `clear_host_autofill()`
- `font_family(family)`
- `font_size(size)`
- `line_height(px)`
- `semantic_label(label)`
- `focus_now()`
- `colors(TextInputColors { ... })`
- `template(...)`
- inherited node/layout methods such as `fill_width()`, `width(...)`, `enabled(...)`, `node_id(...)`

### Callbacks

- `on_changed(|TextChangedEventArgs| ...)`
- `on_text_changed(...)`
- `on_selection_changed(|SelectionChangedEventArgs| ...)`
- `on_focus_changed(|FocusChangedEventArgs| ...)`
- inherited pointer/key/focus callbacks where exposed

Example:

```rust
let value = text("Value: ");
let input = text_input();
input
    .placeholder("Type here")
    .fill_width()
    .on_changed({
        let value = value.clone();
        move |event| value.text(format!("Value: {}", event.text))
    });
```

## `TextArea`

Multiline retained text editor.

### Constructor

```rust
let area = text_area();
let area = TextArea::new();
```

### Multiline-specific methods

- `wrapping(flag)`
- `vertical_scrollbar_visibility(mode)`
- `horizontal_scrollbar_visibility(mode)`
- `accepts_tab(flag)`

`ScrollBarVisibility` controls scrollbar chrome for multiline editors. When
wrapping is enabled, horizontal scrolling is not useful and horizontal scrollbar
behavior follows the wrapping mode.

### Tab behavior

```rust
text_area().accepts_tab(true);
```

When `accepts_tab(true)`, Tab inserts a literal tab character and does not move
focus. When false, Tab participates in normal focus traversal.

## Password behavior

Password text input obscures text and disables copying sensitive content. Word
selection gestures select the obscured password field as a unit rather than
exposing hidden word boundaries.

## Read-only behavior

Read-only editors can still focus, move caret, and select/copy text unless also
disabled. They do not accept text mutation.

## Disabled behavior

Disabled editors reject focus and interactive edits, apply disabled visuals, and
report disabled semantic state.

## Autofill behavior

For browser/password-manager autofill, put fields inside `Form`, set stable
`node_id(...)`, and use `host_autofill(...)`.

```rust
text_input()
    .node_id("email")
    .host_autofill("email")
    .placeholder("Email");
```

## Caret and selection behavior

- Programmatic selection APIs use character positions, not UTF-8 bytes.
- Mouse/touch click moves caret to the closest text position.
- Touch dragging selection handles updates the selected range.
- Cross-over handle movement normalizes selection on pointer/touch release.
- Arrow keys move caret/selection immediately and are consumed by the editor.

## See also

- [Forms and autofill](./FORMS_AND_AUTOFILL.md)
- [Keyboard policy](./KEYBOARD_POLICY.md)
- [Accessibility and semantics](./ACCESSIBILITY_AND_SEMANTICS.md)
