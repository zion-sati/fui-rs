# FUI-RS Accessibility and Semantics (v2)

FUI-RS exports a retained semantic tree alongside visual canvas rendering. Use
normal controls first; they provide default roles, labels, and state.

## Semantics are not behavior

Assigning `SemanticRole::Button` does not add focus, keyboard activation, an
action handler, disabled behavior, or press state. A custom interactive control
must implement the complete behavioral contract represented by its role.

Before assigning an interactive semantic role, verify:

- The control is focusable when enabled and participates in the intended tab
  order.
- The platform-standard keyboard operation performs the same high-level action
  as pointer or touch input.
- Labels, values, checked/selected/expanded state, orientation, and disabled
  state stay synchronized with retained state.
- The action remains available without pointer input.
- Focus, activation, and state behavior have platform-independent tests.

If an affordance is intentionally pointer-only, use truthful non-interactive
semantics rather than presenting it as a partially implemented button or form
control.

Browser semantic projection, macOS accessibility, Windows accessibility, and
Linux AT-SPI are separate adapters over the same retained model and require
separate acceptance evidence. The routed browser demo demonstrates browser
projection only.

## Default semantic behavior

| Surface | Default role | Default label behavior | Auto semantic state |
|---|---|---|---|
| `Button` | Button | constructor label | n/a |
| `Checkbox` | Checkbox | constructor label | checked false/true/mixed |
| `Switch` | Switch | constructor label | checked false/true |
| `RadioButton` | Radio | constructor label | checked false/true |
| `RadioGroup` | RadioGroup | group container | child radios carry checked state |
| `ProgressBar` | inherited container semantics | generated value/range label unless overridden | value range and orientation |
| `Slider` | Slider | generated value/range label unless overridden | value range and orientation |
| `Dropdown` | ComboBox | selected option label | expanded/collapsed and options |
| `ComboBox` | ComboBox/Textbox hybrid | current text or selected item | expanded/collapsed and options |
| `TextInput` | Textbox | placeholder, explicit label, or default label | focus, edit, selection state |
| `TextArea` | Textbox | placeholder, explicit label, or default label | focus, edit, selection state |
| `NavLink` | Link | constructor label or explicit label | link URL |
| `Dialog` | Dialog | title/body text | modal semantic scope while open |
| `Form` | Form | none by default | grouped form fields |
| `Text` | Static text when applicable | content text | n/a |
| `Image` / `Svg` | none unless set | use `alt_text(...)` | n/a |

## When to override

Use explicit semantic APIs when visible text is ambiguous:

```rust
image("/logo.png").alt_text("Contoso logo");
button("?").semantic_label("Open help");
```

Prefer built-in controls over manually assigning roles to generic boxes. Use
`semantic_role(...)` for custom controls only when a generic retained node is
intentionally acting as a specific semantic surface.

## Disabled and visibility semantics

Enabled/disabled state is mirrored into semantic disabled state for built-in
interactive controls. `Visibility::Hidden` and `Visibility::Collapsed` remove
nodes from paint/hit/focus and semantic export.

## Text selection and find-on-page

Selectable `Text` and editor controls contribute text to the semantic/find layer.
Text inside `AntiSelectionArea` blocks ancestor selection collection.

The normal desktop find shortcut opens EffinDOM's retained find experience.
Mobile browser find and explicitly invoked browser-native find search the
projected semantic text. Browser-native matching works, but its DOM-owned
highlight can use a slightly different font rendering from the canvas text.

## Autofill projection is not the semantic tree

`Form` + `TextInput::host_autofill(...)` can project hidden DOM fields for
browser/password-manager compatibility. Those projected fields are host
integration plumbing. Accessibility should rely on the retained semantic tree.

## See also

- [Forms and autofill](./FORMS_AND_AUTOFILL.md)
- [Text input reference](./TEXT_INPUT_REFERENCE.md)
- [Events and callbacks](./EVENTS_AND_CALLBACKS.md)
