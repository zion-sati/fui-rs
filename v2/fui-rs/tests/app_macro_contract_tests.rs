use fui::prelude::*;

#[derive(Clone)]
struct RetainedPage {
    root: FlexBox,
    _owned_label: Text,
}

fui_component!(RetainedPage => root);

fn build_page() -> RetainedPage {
    let label = text("Retained state");
    let root = column();
    root.child(&label);
    RetainedPage {
        root,
        _owned_label: label,
    }
}

fui_app!(RetainedPage, build_page);

#[test]
fn fui_app_accepts_a_retained_component_that_owns_state() {
    let page = build_page();
    let _owned_handle = page._owned_label.handle();
}
