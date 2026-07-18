mod generated;
mod widgets;

use fui::prelude::*;
use fui_rs_demo_shared::{clear_demo_shared_state, demo_card, demo_page_root};
use widgets::DrawingGallery;

#[derive(Clone)]
struct ImmediateDrawingPage {
    root: ScrollBox,
    _gallery: DrawingGallery,
}

fn build_page() -> ImmediateDrawingPage {
    use_system_theme();
    let theme = current_theme();
    let page = demo_page_root("FUI-RS Immediate Drawing");
    page.height_len(auto());
    let gallery = DrawingGallery::new();
    let card = demo_card(
        "Live drawing surfaces",
        "Watch the charts update, pull the dancing yarn, and drag across the paint surface.",
        theme.colors.surface,
    );
    card.child(&gallery);
    page.child(&card);
    let root = ui! {
        scroll_box()
            .fill_size()
            .persist_scroll(false) {
                page,
            }
    };
    ImmediateDrawingPage {
        root,
        _gallery: gallery,
    }
}

fn dispose_page(_: &ImmediateDrawingPage) {
    clear_demo_shared_state();
}

fui_managed_app!(
    ImmediateDrawingPage,
    build_page,
    |page: &ImmediateDrawingPage| page.root.clone(),
    dispose: dispose_page
);
