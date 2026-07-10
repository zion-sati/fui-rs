# FUI-RS Control Customization and Templating (v2)

FUI-RS controls expose two customization levels:

1. Style token APIs for common visual changes.
2. Template/presenter APIs for full retained visual replacement.

Prefer style tokens first. Use templates when the control's retained visual tree
needs to change.

## Style token examples

```rust
let primary = button("Save");
primary.colors(ButtonColors {
    background: 0x2563EBFF,
    hover_background: 0x1D4ED8FF,
    pressed_background: 0x1E40AFFF,
    text: 0xFFFFFFFF,
    border: 0x1D4ED8FF,
});
```

Labeled controls share sizing and color token concepts:

- `LabeledControlColors`
- `LabeledControlSizing`
- `SliderColors`
- `SliderSizing`
- `DropdownColors`
- `DropdownSizing`
- `TextInputColors`

## Templates

Templates create presenter instances for controls. Presenters own the retained
visual subtree for that template.

Public template families:

- `ButtonTemplate` / `ButtonPresenter`
- `CheckboxIndicatorTemplate` / `CheckboxIndicatorPresenter`
- `RadioIndicatorTemplate` / `RadioIndicatorPresenter`
- `SwitchIndicatorTemplate` / `SwitchIndicatorPresenter`
- `SliderTemplate` / `SliderPresenter`
- `DropdownFieldTemplate` / `DropdownFieldPresenter`
- `DropdownChevronTemplate` / `DropdownChevronPresenter`
- `DropdownOptionRowTemplate` / `DropdownOptionRowPresenter`
- `TextInputTemplate` / `TextInputPresenter`

Default template constants and presenter helpers are exported for composition:

- `DEFAULT_BUTTON_TEMPLATE`
- `DEFAULT_CHECKBOX_INDICATOR_TEMPLATE`
- `DEFAULT_RADIO_INDICATOR_TEMPLATE`
- `DEFAULT_SWITCH_INDICATOR_TEMPLATE`
- `DEFAULT_SLIDER_TEMPLATE`
- `DEFAULT_DROPDOWN_FIELD_TEMPLATE`
- `DEFAULT_DROPDOWN_CHEVRON_TEMPLATE`
- `DEFAULT_DROPDOWN_OPTION_ROW_TEMPLATE`
- `DEFAULT_TEXT_INPUT_TEMPLATE`
- `create_default_*_presenter(...)`

## Global control template set

Use `ControlTemplateSet` to replace defaults app-wide:

```rust
let templates = ControlTemplateSet::new()
    .button(DEFAULT_BUTTON_TEMPLATE.clone())
    .text_input(DEFAULT_TEXT_INPUT_TEMPLATE.clone());

use_control_templates(templates);
```

Use `clear_control_templates()` to return to defaults.

## Presenter boundaries

A presenter owns visual composition, not interaction semantics. Built-in controls
own enabled state, focus state, keyboard behavior, events, and semantic state.
Custom presenters should render the visual state they are given and avoid taking
over control logic unless the template contract explicitly asks for it.

## Retained lifecycle guidance

Templates and presenters are retained objects. Store any subscription guards or
owned child nodes in the presenter/control state. Do not create `Rc` self-cycles:
subscription closures should capture only the retained fields they need or use
weak references where necessary.

## See also

- [Theming and style matrix](./THEMING_STYLE_MATRIX.md)
- [Controls and nodes](./CONTROLS_AND_NODES.md)
- [API reference](./API_REFERENCE.md)
