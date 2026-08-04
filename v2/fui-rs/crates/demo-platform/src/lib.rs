use fui::prelude::*;
use fui_rs_demo_shared::{clear_demo_shared_state, demo_card, demo_page_root};
use fui_rs_demo_universal::{DemoCapability, DemoEnvironment, DemoPageId, UniversalDemoPage};
use std::cell::RefCell;
use std::rc::Rc;

struct PlatformPage {
    root: ScrollBox,
    _animation: Rc<RefCell<Option<Animation>>>,
}

fn set_status(node: &TextNode, value: impl Into<String>) {
    let value = value.into();
    node.text(&value).semantic_label(value);
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

fn build_page(environment: &DemoEnvironment) -> PlatformPage {
    let host = environment.host();
    let page = demo_page_root("FUI-RS platform capabilities");
    let host_card = demo_card(
        "Current host",
        "One retained page implementation runs in browser and native shells. Host differences are explicit capabilities, not separate page ports.",
        0xE0F2FEFF,
    );
    host_card.children(children![
        text(format!("Environment: {:?}", host.environment)),
        text(format!("Platform family: {:?}", host.platform_family)),
        text(format!("Application kind: {}", environment.terminology().application_kind)),
        text(format!("Primary modifier: {}", environment.terminology().primary_modifier)),
    ]);

    let capability_card = demo_card(
        "Portable capability contract",
        "The environment describes genuine host differences while controls continue to use public FUI APIs directly.",
        0xDCFCE7FF,
    );
    capability_card.children(children![
        text(format!("Page zoom: {}", yes_no(environment.supports(DemoCapability::PageZoom)))),
        text(format!("Host workers: {}", yes_no(environment.supports(DemoCapability::HostWorkers)))),
        text(format!("External file drop: {}", yes_no(environment.supports(DemoCapability::ExternalFileDrop)))),
        text(format!("File dialogs: {}", yes_no(environment.supports(DemoCapability::FileDialogs)))),
        text(format!("Native context menus: {}", yes_no(environment.supports(DemoCapability::NativeContextMenus)))),
        text(format!("Browser routing: {}", yes_no(environment.supports(DemoCapability::BrowserRouting)))),
    ]);

    let pointer_status = text("Pointer: move, press, touch, or use a pen inside the pad");
    pointer_status.semantic_label("Platform pointer diagnostics idle");
    let gesture_status = text("Gesture: pan or pinch inside the pad");
    gesture_status.semantic_label("Platform gesture diagnostics idle");
    let gesture_pad = ui! {
        column()
            .node_id("demo-platform:gesture-pad")
            .fill_width()
            .height(150.0, Unit::Pixel)
            .padding(16.0, 16.0, 16.0, 16.0)
            .corner_radius(14.0)
            .semantic_label("Touch, pen, pan, and pinch diagnostics")
            .bind_theme(|node, theme| {
                node.bg_color(theme.colors.surface).border(1.0, theme.colors.border);
            }) {
                text("Touch and pinch pad").font_size(17.0).font_weight(FontWeight::Bold),
                text("Use pointer, touch, pen, trackpad pan, or pinch. The same FUI events update these diagnostics on every host."),
            }
    };
    gesture_pad
        .on_pointer_down({
            let status = pointer_status.clone();
            move |event| set_status(&status, format!(
                "Pointer down: {:?} id {} at {:.1}, {:.1} pressure {:.2}",
                event.pointer_type, event.pointer_id, event.x, event.y, event.pressure
            ))
        })
        .on_pointer_move({
            let status = pointer_status.clone();
            move |event| set_status(&status, format!(
                "Pointer move: {:?} at {:.1}, {:.1} pressure {:.2}",
                event.pointer_type, event.x, event.y, event.pressure
            ))
        })
        .on_pan_gesture({
            let status = gesture_status.clone();
            move |event| {
                set_status(&status, format!("Pan {:?}: delta {:.1}, {:.1}", event.phase, event.delta_x, event.delta_y));
                event.handled = true;
            }
        })
        .on_pinch_gesture({
            let status = gesture_status.clone();
            move |event| {
                set_status(&status, format!("Pinch {:?}: scale {:.3}", event.phase, event.scale));
                event.handled = true;
            }
        });
    let input_card = demo_card(
        "Input diagnostics",
        "Pointer metadata and retained gesture ownership are shared by browser and native hosts.",
        0xFDE68AFF,
    );
    input_card.children(children![pointer_status, gesture_status, gesture_pad]);

    let zoom_status = text("Page zoom follows the application setting");
    zoom_status.semantic_label("Page zoom follows the application setting");
    let enable_zoom = button("Enable page zoom");
    enable_zoom.on_click({
        let status = zoom_status.clone();
        move |_| {
            Application::page_zoom(PageZoomMode::Enabled);
            set_status(&status, "Page zoom enabled");
        }
    });
    let disable_zoom = button("Disable page zoom");
    disable_zoom.on_click({
        let status = zoom_status.clone();
        move |_| {
            Application::page_zoom(PageZoomMode::Disabled);
            set_status(&status, "Page zoom disabled");
        }
    });
    let zoom_card = demo_card(
        "Application page zoom",
        "Pinch-to-zoom policy is configured through FUI Application on browser and native hosts.",
        0xEDE9FEFF,
    );
    zoom_card.children(children![
        ui! { row().height_len(auto()).children(children![enable_zoom, disable_zoom]) },
        zoom_status,
    ]);

    let frame_status = text("Frame animation: idle");
    frame_status.semantic_label("Frame animation idle");
    let frame_indicator = ui! {
        flex_box().height(12.0, Unit::Pixel).width(0.0, Unit::Percent).corner_radius(6.0)
            .bind_theme(|node, theme| { node.bg_color(theme.colors.accent); })
    };
    let animation = Rc::new(RefCell::new(None));
    let run_animation = button("Run frame animation");
    run_animation.on_click({
        let status = frame_status.clone();
        let indicator = frame_indicator.clone();
        let slot = animation.clone();
        move |_| {
            set_status(&status, "Frame animation: 0%");
            let update_status = status.clone();
            let update_indicator = indicator.clone();
            slot.replace(Some(animate_float(0.0, 1.0, AnimationTiming::new(1000.0), move |value| {
                update_indicator.width(value * 100.0, Unit::Percent);
                set_status(&update_status, if value >= 1.0 {
                    "Frame animation: complete".to_string()
                } else {
                    format!("Frame animation: {:.0}%", value * 100.0)
                });
            })));
        }
    });
    let frame_card = demo_card(
        "Demand-driven frames",
        "App-owned animation, transitions, programmatic scrolling, and drawing invalidation share one frame lifecycle.",
        0xFCE7F3FF,
    );
    frame_card.children(children![run_animation, frame_indicator, frame_status]);

    page.children(children![host_card, capability_card, input_card, zoom_card, frame_card]);
    let root = ui! { scroll_box().fill_size().persist_scroll(false).child(&page) };
    PlatformPage { root, _animation: animation }
}

pub fn build_universal_page(environment: &DemoEnvironment) -> UniversalDemoPage {
    let page = build_page(environment);
    let root = page.root.clone();
    UniversalDemoPage::new(
        DemoPageId::Platform.metadata(),
        retained_view(&root).keep_alive(page).on_dispose(clear_demo_shared_state),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use fui::platform::PlatformFamily;
    use fui_rs_demo_universal::DemoLinks;

    fn links() -> DemoLinks {
        DemoLinks::new("https://example.test/source", "https://example.test/docs")
    }

    #[test]
    fn one_platform_factory_builds_for_browser_and_native_environments() {
        let browser = build_universal_page(&DemoEnvironment::browser(PlatformFamily::Apple, links()));
        let native = build_universal_page(&DemoEnvironment::native(PlatformFamily::Windows, links()));
        assert_eq!(browser.metadata().id, DemoPageId::Platform);
        assert_eq!(native.metadata().id, DemoPageId::Platform);
        assert!(!browser.view().is_disposed());
        assert!(!native.view().is_disposed());
    }
}
