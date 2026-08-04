use fui::prelude::*;
use fui_rs_demo_universal::design_system::DemoTabSelector;
use fui_rs_demo_universal::{DemoEnvironment, DemoPageMetadata, UniversalDemoPage};

pub type UniversalPageFactory = fn(&DemoEnvironment) -> UniversalDemoPage;

pub const UNIVERSAL_PAGE_FACTORIES: [(DemoPageMetadata, UniversalPageFactory); 6] = [
    (
        fui_rs_demo_universal::DemoPageId::Dashboard.metadata(),
        fui_rs_demo_home::build_universal_page,
    ),
    (
        fui_rs_demo_universal::DemoPageId::TextAndFonts.metadata(),
        fui_rs_demo_workbench::build_universal_page,
    ),
    (
        fui_rs_demo_universal::DemoPageId::BasicControls.metadata(),
        fui_rs_demo_stage4::build_universal_page,
    ),
    (
        fui_rs_demo_universal::DemoPageId::Advanced.metadata(),
        fui_rs_demo_stage5::build_universal_page,
    ),
    (
        fui_rs_demo_universal::DemoPageId::ImmediateDrawing.metadata(),
        fui_rs_demo_immediate_drawing::build_universal_page,
    ),
    (
        fui_rs_demo_universal::DemoPageId::Platform.metadata(),
        fui_rs_demo_platform::build_universal_page,
    ),
];

pub fn build_universal_pages(environment: &DemoEnvironment) -> Vec<UniversalDemoPage> {
    UNIVERSAL_PAGE_FACTORIES
        .iter()
        .map(|(_, factory)| factory(environment))
        .collect()
}

#[derive(Clone)]
pub struct TabbedDemoShell {
    root: FlexBox,
    tabs: TabView,
    _selector: DemoTabSelector,
}

impl TabbedDemoShell {
    pub fn new(environment: DemoEnvironment) -> Self {
        let tabs = TabView::new();
        tabs.fill_size().semantic_label("FUI-RS universal demo pages");
        for (metadata, factory) in UNIVERSAL_PAGE_FACTORIES {
            let page_environment = environment.clone();
            let item = TabItem::with_content(metadata.tab_label, move || {
                factory(&page_environment).view()
            });
            item.description(metadata.semantic_summary);
            tabs.add_item(item);
        }
        let selector = DemoTabSelector::new(&tabs);
        tabs.on_selection_changed({
            let selector = selector.sync_handle();
            move |event| {
                selector.sync_selected(event.selected_index);
                if let Some((metadata, _)) =
                    UNIVERSAL_PAGE_FACTORIES.get(event.selected_index as usize)
                {
                    Application::caption(metadata.title);
                }
            }
        });
        let root = ui! {
            column().fill_size().bind_theme(|node, theme| {
                node.bg_color(theme.colors.background);
            }) {
                selector.clone(),
                tabs,
            }
        };
        Self {
            root,
            tabs,
            _selector: selector,
        }
    }

    pub fn root(&self) -> FlexBox {
        self.root.clone()
    }

    pub fn tabs(&self) -> TabView {
        self.tabs.clone()
    }
}

pub type NativeDemoShell = TabbedDemoShell;
pub type WebTabbedDemoShell = TabbedDemoShell;

#[cfg(test)]
mod tests {
    use super::*;
    use fui::platform::PlatformFamily;
    use fui::{ffi, ffi::Call};
    use fui_rs_demo_universal::{DemoLinks, DemoPageId};

    fn links() -> DemoLinks {
        DemoLinks::new("https://example.test/source", "https://example.test/docs")
    }

    #[test]
    fn one_binary_links_and_builds_every_universal_page_factory() {
        let native_environment = DemoEnvironment::native(
            PlatformFamily::Apple,
            links(),
        );
        let browser_environment = DemoEnvironment::browser(PlatformFamily::Linux, links());
        let native_pages = build_universal_pages(&native_environment);
        let browser_pages = build_universal_pages(&browser_environment);
        let ids = native_pages
            .iter()
            .map(|page| page.metadata().id)
            .collect::<Vec<_>>();

        assert_eq!(
            ids,
            vec![
                DemoPageId::Dashboard,
                DemoPageId::TextAndFonts,
                DemoPageId::BasicControls,
                DemoPageId::Advanced,
                DemoPageId::ImmediateDrawing,
                DemoPageId::Platform,
            ]
        );
        assert_eq!(
            ids,
            browser_pages
                .iter()
                .map(|page| page.metadata().id)
                .collect::<Vec<_>>()
        );
        assert!(native_pages.iter().all(|page| !page.view().is_disposed()));
        assert!(browser_pages.iter().all(|page| !page.view().is_disposed()));
    }

    #[test]
    fn native_and_web_tabbed_shells_share_lazy_factories_and_update_caption() {
        ffi::test::reset();
        let native = NativeDemoShell::new(DemoEnvironment::native(
            PlatformFamily::Apple,
            links(),
        ));
        let native_tabs = native.tabs();
        assert_eq!(native_tabs.item_count(), UNIVERSAL_PAGE_FACTORIES.len());
        assert!(native_tabs.item_at(0).unwrap().is_materialized());
        assert!(!native_tabs.item_at(1).unwrap().is_materialized());
        ffi::test::take_calls();

        native_tabs.select_index(1);
        assert!(native_tabs.item_at(0).unwrap().is_materialized());
        assert!(native_tabs.item_at(1).unwrap().is_materialized());
        let calls = ffi::test::take_calls();
        assert_eq!(
            calls
                .iter()
                .filter(|call| matches!(call, Call::SetApplicationCaption { .. }))
                .count(),
            1
        );
        assert!(calls.iter().any(|call| matches!(
            call,
            Call::SetApplicationCaption { caption }
                if caption == DemoPageId::TextAndFonts.metadata().title
        )));
        native_tabs.select_index(0);
        native_tabs.select_index(1);
        assert!(native_tabs.item_at(0).unwrap().is_materialized());
        assert!(native_tabs.item_at(1).unwrap().is_materialized());

        let web = WebTabbedDemoShell::new(DemoEnvironment::browser(
            PlatformFamily::Linux,
            links(),
        ));
        assert_eq!(web.tabs().item_count(), UNIVERSAL_PAGE_FACTORIES.len());
    }
}
