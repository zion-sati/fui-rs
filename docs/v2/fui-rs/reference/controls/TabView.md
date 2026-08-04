# TabView

Headless retained content switcher with one attached active panel.

`TabView` deliberately does not create tab headers or selector chrome. Compose
`Button`, `NavLink`, clickable `Text`, custom drawing, or any other suitable
control outside it, then call `select_index(...)` from that selector. This keeps
responsive layout, visuals, keyboard behavior, and selector semantics under
application control.

## Construction

- `tab_view()`, `TabView::new()`
- `TabView::with_items(...)`
- `tab_item(label)`, `TabItem::new(label)`
- `TabItem::with_content(label, factory)`
- `TabItem::content(factory)`
- `TabItem::content_view(view)`
- `tab_items![...]`

The factory returns a `RetainedView`. It runs at most once, on first activation.
Inactive content remains owned but detached; activation/deactivation hooks run
on every switch, retained control and scroll state survive reselection, and
disposal is deterministic when an item is removed or the control is disposed.

## Selection and collection

- `items(...)`, `add_item(...)`
- `remove_item(...)`, `remove_item_at(...)`, `clear_items()`
- `select_index(...)`
- `selected_index()`, `selected_item()`, `item_count()`
- `on_selection_changed(...)`, `bind_selection_changed(...)`

Disabling or removing the selected item chooses the next enabled item, then the
previous enabled item. The control never attaches more than one panel root.
Re-entrant selection requests are serialized.

## Selector composition

```rust
use fui::prelude::*;

let tabs = tab_view();
tabs.items(tab_items![
    tab_item("Overview").content(|| retained_view(&text("Overview content"))),
    tab_item("Settings").content(|| retained_view(&text("Settings content"))),
]);

let overview = button("Overview");
overview.on_click({
    let tabs = tabs.clone();
    move |_| { tabs.select_index(0); }
});

let settings = button("Settings");
settings.on_click({
    let tabs = tabs.clone();
    move |_| { tabs.select_index(1); }
});

let root = ui! {
    column().fill_size() {
        row().flex_wrap(FlexWrap::Wrap) { overview, settings },
        tabs,
    }
};
```

`TabView` projects `TabPanel` semantics for its selected-content host. A visual
selector is a separate application-owned control tree. If it behaves as a tab
strip, give its container `TabList` semantics, each selector `Tab` semantics,
and keep selected and disabled state synchronized with `TabView`.

This separation is intentional: selectors can wrap, scroll, collapse into a
menu, or use entirely custom visuals without fighting built-in tab chrome.

Use `TabView` for in-window navigation where the selected page does not need
its own URL. Use the routed harness for browser history, deep links,
restoration, and independently loaded route WASMs.
