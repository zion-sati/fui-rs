# FUI-RS Accessibility and Semantics (v2)

FUI-RS exports a retained semantic tree alongside visual canvas rendering. Use
normal controls first; they provide default roles, labels, and state.

## Default semantic behavior

| Surface | Default role | Default label behavior | Auto semantic state |
|---|---|---|---|
| `Button` | Button | constructor label | n/a |
| `Checkbox` | Checkbox | constructor label | checked false/true/mixed |
| `Switch` | Switch | constructor label | checked false/true |
| `RadioButton` | Radio | constructor label | checked false/true |
| `RadioGroup` | RadioGroup | group container | child radios carry checked state |
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

## Autofill projection is not the semantic tree

`Form` + `TextInput::host_autofill(...)` can project hidden DOM fields for
browser/password-manager compatibility. Those projected fields are host
integration plumbing. Accessibility should rely on the retained semantic tree.

## See also

- [Forms and autofill](./FORMS_AND_AUTOFILL.md)
- [Text input reference](./TEXT_INPUT_REFERENCE.md)
- [Events and callbacks](./EVENTS_AND_CALLBACKS.md)
