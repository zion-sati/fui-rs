#[cfg(feature = "web-route")]
mod generated;

use fui::prelude::*;
#[cfg(feature = "web-route")]
use fui_rs_demo_shared::generated::host_services::{
    demo_shell_accent_color_hex, demo_shell_clock_tick_seconds, demo_shell_is_dark_mode,
    demo_shell_wall_clock_since_epoch_ms,
};
use fui_rs_demo_shared::{clear_demo_shared_state, demo_card, demo_page_root};
#[cfg(feature = "web-route")]
use fui_rs_demo_universal::DemoLinks;
use fui_rs_demo_universal::{DemoEnvironment, DemoPageId, UniversalDemoPage};
use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;
#[cfg(not(feature = "web-route"))]
use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[cfg(feature = "web-route")]
thread_local! {
    static DEMO_HOST_TICK: Cell<i32> = const { Cell::new(0) };
    static DEMO_HOST_DARK_MODE: Cell<bool> = const { Cell::new(false) };
    static DEMO_HOST_EVENT_SUBSCRIPTIONS: RefCell<Vec<HostEventSubscription>> = const { RefCell::new(Vec::new()) };
}

thread_local! {
    static DEMO_SUBSCRIPTIONS: RefCell<Vec<Subscription>> = const { RefCell::new(Vec::new()) };
}

#[cfg(not(feature = "web-route"))]
thread_local! {
    static DEMO_NATIVE_STARTED_AT: Instant = Instant::now();
    static DEMO_NATIVE_HOST_TIMER: RefCell<Option<TimerHandle>> = const { RefCell::new(None) };
}

const SIDEBAR_LIST_TOTAL_ITEMS: i32 = 10_000;
const SIDEBAR_LIST_ITEM_HEIGHT: f32 = 20.0;

struct SidebarListRow {
    label: TextNode,
}

fn update_virtual_list_metrics(
    list: &VirtualList<SidebarListRow>,
    offset_label: &TextNode,
    first_visible_label: &TextNode,
    rendered_rows_label: &TextNode,
) {
    offset_label.text(format!(
        "List offset {} px",
        list.scroll_state().offset_y() as i32
    ));
    first_visible_label.text(format!("First visible item {}", list.first_visible_index()));
    rendered_rows_label.text(format!("Rendered rows {}", list.rendered_item_count()));
}

struct DemoHostSnapshot {
    tick: i32,
    accent: String,
    dark_mode: bool,
    wall_clock_ms: f64,
}

fn host_clock_text(tick: i32, wall_clock_ms: f64) -> (String, String) {
    (
        format!("Tick: {tick}"),
        format!("Wall clock: {wall_clock_ms:.0} ms"),
    )
}

#[cfg(feature = "web-route")]
fn host_snapshot(_environment: &DemoEnvironment) -> DemoHostSnapshot {
    DemoHostSnapshot {
        tick: demo_shell_clock_tick_seconds(),
        accent: demo_shell_accent_color_hex(),
        dark_mode: demo_shell_is_dark_mode(),
        wall_clock_ms: demo_shell_wall_clock_since_epoch_ms(),
    }
}

#[cfg(not(feature = "web-route"))]
fn host_snapshot(environment: &DemoEnvironment) -> DemoHostSnapshot {
    #[cfg(not(feature = "web-route"))]
    let _ = environment;
    DemoHostSnapshot {
        tick: 0,
        accent: "active native accent".to_string(),
        dark_mode: is_dark_mode(),
        wall_clock_ms: 0.0,
    }
}

#[cfg(not(feature = "web-route"))]
fn live_native_host_snapshot(environment: &DemoEnvironment) -> DemoHostSnapshot {
    let _ = environment;
    let wall_clock_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs_f64() * 1_000.0)
        .unwrap_or_default();
    DemoHostSnapshot {
        tick: DEMO_NATIVE_STARTED_AT.with(|started| started.elapsed().as_secs() as i32),
        accent: "active native accent".to_string(),
        dark_mode: is_dark_mode(),
        wall_clock_ms,
    }
}

#[cfg(not(feature = "web-route"))]
fn schedule_native_host_refresh(
    environment: DemoEnvironment,
    tick_label: TextNode,
    dark_mode_label: TextNode,
    wall_clock_label: TextNode,
) {
    let timer = set_timeout(250, move || {
        let host = live_native_host_snapshot(&environment);
        let (tick_text, wall_clock_text) = host_clock_text(host.tick, host.wall_clock_ms);
        tick_label.text(tick_text);
        dark_mode_label.text(format!(
            "Dark mode: {}",
            if host.dark_mode { "true" } else { "false" }
        ));
        wall_clock_label.text(wall_clock_text);
        schedule_native_host_refresh(
            environment.clone(),
            tick_label.clone(),
            dark_mode_label.clone(),
            wall_clock_label.clone(),
        );
    });
    DEMO_NATIVE_HOST_TIMER.with(|slot| *slot.borrow_mut() = Some(timer));
}

fn build_dashboard_page(environment: &DemoEnvironment) -> ScrollBox {
    let host = host_snapshot(environment);
    let (tick_text, wall_clock_text) = host_clock_text(host.tick, host.wall_clock_ms);
    let root = ui! {
    demo_page_root("FUI-RS demo dashboard").height_len(auto())
    };

    let tick_label = ui! {
        text("").text(tick_text)
    };
    let accent_label = ui! {
        text("").text(format!("Accent: {}", host.accent))
    };
    let dark_mode_label = ui! {
        text("").text(format!(
            "Dark mode: {}",
            if host.dark_mode {
                "true"
            } else {
                "false"
            }
        ))
    };
    let wall_clock_label = ui! {
        text(wall_clock_text)
    };
    #[cfg(not(feature = "web-route"))]
    schedule_native_host_refresh(
        environment.clone(),
        tick_label.clone(),
        dark_mode_label.clone(),
        wall_clock_label.clone(),
    );
    #[cfg(feature = "web-route")]
    DEMO_HOST_EVENT_SUBSCRIPTIONS.with(|slot| {
        let mut subscriptions = slot.borrow_mut();
        subscriptions.push(generated::host_events::on_demo_shell_clock_tick_changed({
            let tick_label = tick_label.clone();
            let wall_clock_label = wall_clock_label.clone();
            move |tick| {
                DEMO_HOST_TICK.with(|slot| slot.set(tick));
                let (tick_text, wall_clock_text) =
                    host_clock_text(tick, demo_shell_wall_clock_since_epoch_ms());
                tick_label.text(tick_text);
                wall_clock_label.text(wall_clock_text);
            }
        }));
        subscriptions.push(generated::host_events::on_demo_shell_accent_color_changed(
            {
                let accent_label = accent_label.clone();
                move |_accent| {
                    accent_label.text(format!("Accent: {}", demo_shell_accent_color_hex()));
                }
            },
        ));
        subscriptions.push(generated::host_events::on_demo_shell_dark_mode_changed({
            let dark_mode_label = dark_mode_label.clone();
            move |is_dark| {
                DEMO_HOST_DARK_MODE.with(|slot| slot.set(is_dark));
                dark_mode_label.text(format!(
                    "Dark mode: {}",
                    if is_dark { "true" } else { "false" }
                ));
            }
        }));
    });

    let virtual_list_card = ui! {
        demo_card(
            "Virtual list",
            "FUI-RS dashboard now uses the same pooled-row virtualization model as FUI-AS: fixed-height rows, recycled SelectionArea containers, and ScrollState-driven windowing.",
            0xFDE68AFF,
        ).margin(0.0, 0.0, 0.0, 16.0)
    };
    let sidebar_list = virtual_list(SIDEBAR_LIST_TOTAL_ITEMS, SIDEBAR_LIST_ITEM_HEIGHT)
        .item_template(|container| {
            let label = text("");
            container.child(&label);
            SidebarListRow { label }
        });
    sidebar_list
        .on_bind_item(|row, index| {
            let text_value = format!("Item {}", index);
            row.label.text(&text_value);
            row.label.semantic_label(&text_value);
        })
        .node_id("demo-dashboard:sidebar-list")
        .persist_scroll(true)
        .width(100.0, Unit::Percent)
        .height(180.0, Unit::Pixel);
    let list_offset_label = text("");
    let first_visible_label = text("");
    let rendered_rows_label = text("");
    update_virtual_list_metrics(
        &sidebar_list,
        &list_offset_label,
        &first_visible_label,
        &rendered_rows_label,
    );
    DEMO_SUBSCRIPTIONS.with(|slot| {
        let mut guards = slot.borrow_mut();
        guards.push(sidebar_list.scroll_state().subscribe_offset_y({
            let sidebar_list = sidebar_list.clone();
            let list_offset_label = list_offset_label.clone();
            let first_visible_label = first_visible_label.clone();
            let rendered_rows_label = rendered_rows_label.clone();
            move || {
                update_virtual_list_metrics(
                    &sidebar_list,
                    &list_offset_label,
                    &first_visible_label,
                    &rendered_rows_label,
                );
            }
        }));
        guards.push(sidebar_list.scroll_state().subscribe_viewport_height({
            let sidebar_list = sidebar_list.clone();
            let list_offset_label = list_offset_label.clone();
            let first_visible_label = first_visible_label.clone();
            let rendered_rows_label = rendered_rows_label.clone();
            move || {
                update_virtual_list_metrics(
                    &sidebar_list,
                    &list_offset_label,
                    &first_visible_label,
                    &rendered_rows_label,
                );
            }
        }));
        guards.push(sidebar_list.scroll_state().subscribe_content_height({
            let sidebar_list = sidebar_list.clone();
            let list_offset_label = list_offset_label.clone();
            let first_visible_label = first_visible_label.clone();
            let rendered_rows_label = rendered_rows_label.clone();
            move || {
                update_virtual_list_metrics(
                    &sidebar_list,
                    &list_offset_label,
                    &first_visible_label,
                    &rendered_rows_label,
                );
            }
        }));
    });
    virtual_list_card.children(children![
        sidebar_list,
        list_offset_label,
        first_visible_label,
        rendered_rows_label,
    ]);

    let slider_card = ui! {
        demo_card(
            "Slider orientations",
            "FUI-RS sliders support horizontal and vertical orientation. The vertical slider uses the same retained control and keyboard/value semantics as the horizontal one.",
            0xE0F2FEFF,
        ).margin(0.0, 0.0, 0.0, 16.0)
    };
    let horizontal_slider = ui! {
        slider()
            .value(35.0)
            .length(180.0)
            .semantic_label("Dashboard horizontal slider")
    };
    let vertical_slider = ui! {
        slider()
            .value(65.0)
            .length(120.0)
            .orientation(Orientation::Vertical)
            .semantic_label("Dashboard vertical slider")
    };
    let slider_status = text("Horizontal: 35 | Vertical: 65");
    let horizontal_slider_value = Rc::new(Cell::new(35.0_f32));
    let vertical_slider_value = Rc::new(Cell::new(65.0_f32));
    {
        let status = slider_status.clone();
        let horizontal_value = horizontal_slider_value.clone();
        let vertical_value = vertical_slider_value.clone();
        horizontal_slider.on_changed(move |event| {
            horizontal_value.set(event.value);
            status.text(format!(
                "Horizontal: {} | Vertical: {}",
                event.value as i32,
                vertical_value.get() as i32
            ));
        });
    }
    {
        let status = slider_status.clone();
        let horizontal_value = horizontal_slider_value.clone();
        let vertical_value = vertical_slider_value.clone();
        vertical_slider.on_changed(move |event| {
            vertical_value.set(event.value);
            status.text(format!(
                "Horizontal: {} | Vertical: {}",
                horizontal_value.get() as i32,
                event.value as i32
            ));
        });
    }
    slider_card.children(children![
        ui! {
            row().align_items(AlignItems::Center) {
                row() {
                    horizontal_slider,
                },
                row().margin(18.0, 0.0, 0.0, 0.0) {
                    vertical_slider,
                },
            }
        },
        slider_status,
    ]);

    let activation_card = ui! {
        demo_card(
            "Semantic activation and raw pointer gestures",
            "The button's semantic action includes keyboard activation. Raw pointer click always fires for pointer activation, followed by an exact double/triple callback when applicable. The switch reports state changes separately from semantic clicks.",
            0xDCFCE7FF,
        ).margin(0.0, 0.0, 0.0, 16.0)
    };
    let activation_status = text("Button semantic: 0 | raw: none");
    let semantic_count = Rc::new(Cell::new(0_u32));
    let activation_button = button("Activate with pointer or keyboard").configure(|button| {
        button
            .on_click({
                let semantic_count = semantic_count.clone();
                let activation_status = activation_status.clone();
                move |_| {
                    semantic_count.set(semantic_count.get() + 1);
                    activation_status.text(format!(
                        "Button semantic: {} | raw: unchanged by keyboard",
                        semantic_count.get()
                    ));
                }
            })
            .on_pointer_click({
                let semantic_count = semantic_count.clone();
                let activation_status = activation_status.clone();
                move |event| {
                    activation_status.text(format!(
                        "Button semantic: {} | raw click count {}",
                        semantic_count.get(),
                        event.click_count
                    ));
                }
            })
            .on_pointer_double_click({
                let activation_status = activation_status.clone();
                move |_| {
                    activation_status.text("Exact raw double-click (after ordinary raw click)");
                }
            })
            .on_pointer_triple_click({
                let activation_status = activation_status.clone();
                move |_| {
                    activation_status.text("Exact raw triple-click (after ordinary raw click)");
                }
            });
    });
    let switch_status = text("Switch changed: off | semantic clicks: 0");
    let switch_value = Rc::new(Cell::new(false));
    let switch_clicks = Rc::new(Cell::new(0_u32));
    let activation_switch = switch("Separate changed from click").configure(|switch| {
        switch
            .on_changed({
                let switch_value = switch_value.clone();
                let switch_clicks = switch_clicks.clone();
                let switch_status = switch_status.clone();
                move |event| {
                    switch_value.set(event.checked);
                    switch_status.text(format!(
                        "Switch changed: {} | semantic clicks: {}",
                        if event.checked { "on" } else { "off" },
                        switch_clicks.get()
                    ));
                }
            })
            .on_click({
                let switch_value = switch_value.clone();
                let switch_clicks = switch_clicks.clone();
                let switch_status = switch_status.clone();
                move |_| {
                    switch_clicks.set(switch_clicks.get() + 1);
                    switch_status.text(format!(
                        "Switch changed: {} | semantic clicks: {}",
                        if switch_value.get() { "on" } else { "off" },
                        switch_clicks.get()
                    ));
                }
            });
    });
    activation_card.children(children![
        activation_button,
        activation_status,
        activation_switch,
        switch_status,
    ]);
    root.children(children![
        ui! {
                demo_card(
                    "Stage 4 routed demo scaffold",
                    "This route is mounted through the shared browser routed harness. Use the retained NavLinks above to swap to the workbench route and back.",
                    0xD7EAFEFF,
                ).margin(0.0, 16.0, 0.0, 0.0)
        },
        ui! {
                demo_card("Current route", "/v2/fui-rs/demo/index.html", 0xDCFCE7FF)
                    .margin(0.0, 0.0, 0.0, 16.0)
        },
        ui! {
                demo_card(
                    "Generated host services",
                    "This route reads browser-side host services through Rust generated bindings and listens to generated host events.",
                    0xE0F2FEFF,
                ).margin(0.0, 0.0, 0.0, 16.0) {
                    tick_label,
                    accent_label,
                    dark_mode_label,
                    wall_clock_label,
                }
        },
        virtual_list_card,
        slider_card,
        activation_card,
        demo_card(
            "Next phase",
            "The routed demo now uses canvas-owned navigation, matching the FUI-AS demo shape.",
            0xFDE68AFF,
        ),
    ]);
    let page_scroll = ui! {
        scroll_box()
            .fill_size()
            .scroll_enabled_x(false)
            .scroll_enabled_y(true)
            .node_id("demo-dashboard-page-scroll") {
                root,
            }
    };
    page_scroll
}

pub fn build_universal_page(environment: &DemoEnvironment) -> UniversalDemoPage {
    let root = build_dashboard_page(environment);
    UniversalDemoPage::new(
        DemoPageId::Dashboard.metadata(),
        retained_view(&root).on_dispose(|| {
            #[cfg(not(feature = "web-route"))]
            DEMO_NATIVE_HOST_TIMER.with(|slot| {
                if let Some(timer) = slot.borrow_mut().take() {
                    cancel_timeout(timer);
                }
            });
            DEMO_SUBSCRIPTIONS.with(|slot| slot.borrow_mut().clear());
            clear_demo_shared_state();
        }),
    )
}

#[cfg(feature = "web-route")]
fn web_environment() -> DemoEnvironment {
    DemoEnvironment::browser(
        fui::platform::platform_family(),
        DemoLinks::new(
            "https://github.com/zion-sati/fui-rs",
            "https://docs.rs/fui-rs/latest/fui",
        ),
    )
}

#[cfg(feature = "web-route")]
fn build_web_page() -> fui_rs_demo_shared::RoutedDemoShell<UniversalDemoPage> {
    Application::caption(DemoPageId::Dashboard.metadata().title);
    let page = build_universal_page(&web_environment());
    let root = page.view().root();
    fui_rs_demo_shared::routed_demo_shell(page, root, DemoPageId::Dashboard)
}

#[cfg(test)]
mod tests {
    use super::host_clock_text;

    #[test]
    fn host_clock_text_reflects_each_refresh() {
        assert_eq!(
            host_clock_text(7, 1_725_000_000_123.6),
            (
                "Tick: 7".to_string(),
                "Wall clock: 1725000000124 ms".to_string(),
            )
        );
        assert_eq!(
            host_clock_text(8, 1_725_000_001_001.0),
            (
                "Tick: 8".to_string(),
                "Wall clock: 1725000001001 ms".to_string(),
            )
        );
    }
}

#[cfg(feature = "web-route")]
fn mount_dashboard_page(_: &fui_rs_demo_shared::RoutedDemoShell<UniversalDemoPage>) {
    DEMO_HOST_TICK.with(|slot| slot.set(demo_shell_clock_tick_seconds()));
    DEMO_HOST_DARK_MODE.with(|slot| slot.set(demo_shell_is_dark_mode()));
}

#[cfg(feature = "web-route")]
fn dispose_dashboard_page(page: &fui_rs_demo_shared::RoutedDemoShell<UniversalDemoPage>) {
    page.content().view().dispose();
    DEMO_HOST_EVENT_SUBSCRIPTIONS.with(|slot| slot.borrow_mut().clear());
}

#[cfg(feature = "web-route")]
fui_managed_app!(
    fui_rs_demo_shared::RoutedDemoShell<UniversalDemoPage>,
    build_web_page,
    |page: &fui_rs_demo_shared::RoutedDemoShell<UniversalDemoPage>| page.root.clone(),
    mount: mount_dashboard_page,
    dispose: dispose_dashboard_page
);

#[cfg(feature = "web-route")]
#[no_mangle]
pub extern "C" fn __getDemoHostTick() -> i32 {
    DEMO_HOST_TICK.with(Cell::get)
}

#[cfg(feature = "web-route")]
#[no_mangle]
pub extern "C" fn __getDemoHostDarkMode() -> bool {
    DEMO_HOST_DARK_MODE.with(Cell::get)
}
