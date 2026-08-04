#[cfg(feature = "web-route")]
mod generated;
mod widgets;

use fui::prelude::*;
use fui_rs_demo_shared::clear_demo_shared_state;
use fui_rs_demo_shared::{demo_card, demo_page_root};
use fui_rs_demo_universal::{DemoEnvironment, DemoPageId, UniversalDemoPage};
use widgets::DrawingGallery;

#[derive(Clone)]
struct ImmediateDrawingPage {
    root: ScrollBox,
    _gallery: DrawingGallery,
}

fn build_page() -> ImmediateDrawingPage {
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

pub fn build_universal_page(_environment: &DemoEnvironment) -> UniversalDemoPage {
    let page = build_page();
    let root = page.root.clone();
    UniversalDemoPage::new(
        DemoPageId::ImmediateDrawing.metadata(),
        retained_view(&root)
            .keep_alive(page)
            .on_dispose(clear_demo_shared_state),
    )
}

#[cfg(feature = "web-route")]
fn web_environment() -> DemoEnvironment {
    DemoEnvironment::browser(
        fui::platform::platform_family(),
        fui_rs_demo_universal::DemoLinks::new(
            "https://github.com/zion-sati/fui-rs",
            "https://docs.rs/fui-rs/latest/fui",
        ),
    )
}

#[cfg(feature = "web-route")]
fn build_web_page() -> fui_rs_demo_shared::RoutedDemoShell<UniversalDemoPage> {
    Application::caption(DemoPageId::ImmediateDrawing.metadata().title);
    use_system_theme();
    let page = build_universal_page(&web_environment());
    let root = page.view().root();
    fui_rs_demo_shared::routed_demo_shell(
        page,
        root,
        DemoPageId::ImmediateDrawing,
    )
}

#[cfg(feature = "web-route")]
fn dispose_page(page: &fui_rs_demo_shared::RoutedDemoShell<UniversalDemoPage>) {
    page.content().view().dispose();
}

#[cfg(feature = "web-route")]
fui_managed_app!(
    fui_rs_demo_shared::RoutedDemoShell<UniversalDemoPage>,
    build_web_page,
    |page: &fui_rs_demo_shared::RoutedDemoShell<UniversalDemoPage>| page.root.clone(),
    dispose: dispose_page
);

#[cfg(test)]
mod tests {
    use super::*;
    use fui::platform::PlatformFamily;
    use fui_rs_demo_universal::DemoLinks;

    fn links() -> DemoLinks {
        DemoLinks::new("https://example.test/source", "https://example.test/docs")
    }

    #[test]
    fn one_page_factory_builds_for_browser_and_native_environments() {
        let browser =
            build_universal_page(&DemoEnvironment::browser(PlatformFamily::Apple, links()));
        let native =
            build_universal_page(&DemoEnvironment::native(PlatformFamily::Windows, links()));
        assert_eq!(browser.metadata().id, DemoPageId::ImmediateDrawing);
        assert_eq!(native.metadata().id, DemoPageId::ImmediateDrawing);
        assert!(!browser.view().is_disposed());
        assert!(!native.view().is_disposed());
    }
}
