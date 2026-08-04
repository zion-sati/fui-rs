pub mod design_system;
mod environment;
mod page;

pub use environment::{
    DemoCapabilities, DemoCapability, DemoEnvironment, DemoLinks, DemoTerminology,
};
pub use page::{DemoPageFactory, DemoPageId, DemoPageMetadata, UniversalDemoPage};

#[cfg(test)]
mod tests {
    use super::*;
    use fui::platform::PlatformFamily;
    use fui::prelude::*;

    fn links() -> DemoLinks {
        DemoLinks::new(
            "https://github.com/zion-sati/fui-rs/tree/main/demo",
            "https://docs.rs/fui-rs/latest/fui",
        )
    }

    #[test]
    fn browser_and_native_fixtures_expose_only_intentional_capabilities() {
        let browser = DemoEnvironment::browser(PlatformFamily::Apple, links());
        assert!(browser.supports(DemoCapability::BrowserRouting));
        assert!(browser.supports(DemoCapability::PersistedNavigation));
        assert!(!browser.supports(DemoCapability::NativeContextMenus));
        assert_eq!(browser.terminology().primary_modifier, "Command");
        assert_eq!(browser.terminology().application_kind, "web application");

        let native = DemoEnvironment::native(PlatformFamily::Windows, links());
        assert!(!native.supports(DemoCapability::BrowserRouting));
        assert!(!native.supports(DemoCapability::PersistedNavigation));
        assert!(native.supports(DemoCapability::NativeContextMenus));
        assert_eq!(native.terminology().primary_modifier, "Ctrl");
        assert_eq!(native.terminology().application_kind, "desktop application");
    }

    #[test]
    fn every_page_has_stable_shell_metadata_and_resolved_links() {
        let environment = DemoEnvironment::browser(PlatformFamily::Linux, links());
        for id in DemoPageId::ALL {
            let metadata = id.metadata();
            assert!(!metadata.title.is_empty());
            assert!(!metadata.tab_label.is_empty());
            assert!(!metadata.semantic_summary.is_empty());
            let page = UniversalDemoPage::new(
                metadata,
                retained_view(&design_system::page_surface(metadata.semantic_summary)),
            );
            assert!(page.source_url(&environment).contains(metadata.source_path));
            assert!(page
                .documentation_url(&environment)
                .contains(metadata.documentation_path));
        }
    }
}
