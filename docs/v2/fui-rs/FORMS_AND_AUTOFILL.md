# FUI-RS Forms, Password Managers, and Browser Autofill

Canvas UI does not give password managers or browser autofill a native DOM form
to inspect by default. FUI-RS provides explicit form metadata so the browser
host can project compatible hidden DOM fields when you opt in.

## Recommended login pattern

Use all of the following:

1. Wrap related credentials in a `Form`.
2. Give each field a stable `node_id(...)`.
3. Set `host_autofill(...)` explicitly.
4. Mark password fields with `password(true)`.

```rust
use fui::prelude::*;

let username = text_input();
username
    .node_id("username")
    .semantic_label("Username")
    .placeholder("Username or email")
    .host_autofill("username")
    .fill_width();

let password = text_input();
password
    .node_id("current-password")
    .semantic_label("Password")
    .placeholder("Password")
    .password(true)
    .host_autofill("current-password")
    .fill_width();

let login_form = ui! {
    form() {
        column().gap(12.0).fill_width() {
            username.clone(),
            password.clone(),
            button("Sign in"),
        }
    }
};
```

## Why `Form` matters

`Form` groups related editable fields. Browser autofill and password managers
make better decisions when username/password fields are part of the same form
projection rather than unrelated canvas text boxes.

`Form` also owns default/cancel action behavior for Enter/Escape flows.

## Why `node_id(...)` matters

`node_id(...)` is the stable retained identity for the field. The browser host
uses it as the projected DOM `name` and `id` for host integrations.

Use stable, meaningful IDs such as:

- `username`
- `current-password`
- `email`
- `shipping-postal-code`

## Autofill hints

`host_autofill(hint)` accepts standard browser autocomplete token strings.
Use `clear_host_autofill()` to remove a previously configured hint.
Common values:

- `username`
- `current-password`
- `new-password`
- `email`
- `one-time-code`
- `tel`
- `name`
- `given-name`
- `family-name`
- `street-address`
- `address-line1`
- `address-line2`
- `postal-code`
- `country`

## Address, name, and phone autofill

Use the same explicit pattern: fields inside a `Form`, stable `node_id(...)`, and
standard `host_autofill(...)` tokens.

## Accessibility separation

Projected autofill DOM fields are not the accessibility source of truth. The
retained semantic tree remains the a11y layer; projected fields exist for host
browser integrations only.

## Should every text input be projected?

No. Project only fields that need host autofill/password-manager behavior.
Blanket projection creates more DOM churn, duplicate host heuristics, and more
focus/IME synchronization work.

## See also

- [Text input reference](./TEXT_INPUT_REFERENCE.md)
- [Accessibility and semantics](./ACCESSIBILITY_AND_SEMANTICS.md)
- [Controls and nodes](./CONTROLS_AND_NODES.md)
