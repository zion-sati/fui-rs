mod generated;

use fui::prelude::*;
use fui_rs_demo_shared::generated::host_services::{
    demo_shell_accent_color_hex, demo_shell_clock_tick_seconds, demo_shell_is_dark_mode,
    demo_shell_wall_clock_since_epoch_ms,
};
use fui_rs_demo_shared::{clear_demo_shared_state, demo_card, demo_page_root};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use AlignItems;

thread_local! {
    static DEMO_HOST_TICK: Cell<i32> = const { Cell::new(0) };
    static DEMO_HOST_DARK_MODE: Cell<bool> = const { Cell::new(false) };
    static DEMO_SUBSCRIPTIONS: RefCell<Vec<Subscription>> = const { RefCell::new(Vec::new()) };
}

const SIDEBAR_LIST_TOTAL_ITEMS: i32 = 10_000;
const SIDEBAR_LIST_ITEM_HEIGHT: f32 = 20.0;

fn update_virtual_list_metrics(
    list: &VirtualList,
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

fn build_dashboard_page() -> ScrollBox {
    let root = ui! {
    demo_page_root("FUI-RS demo dashboard").height_len(auto())
    };

    let tick_label = ui! {
        text("").text(format!("Tick: {}", demo_shell_clock_tick_seconds()))
    };
    let accent_label = ui! {
        text("").text(format!("Accent: {}", demo_shell_accent_color_hex()))
    };
    let dark_mode_label = ui! {
        text("").text(format!(
            "Dark mode: {}",
            if demo_shell_is_dark_mode() {
                "true"
            } else {
                "false"
            }
        ))
    };
    generated::host_events::on_demo_shell_clock_tick_changed({
        let tick_label = tick_label.clone();
        move |tick| {
            DEMO_HOST_TICK.with(|slot| slot.set(tick));
            tick_label.text(format!("Tick: {}", tick));
        }
    });
    generated::host_events::on_demo_shell_accent_color_changed({
        let accent_label = accent_label.clone();
        move |_accent| {
            accent_label.text(format!("Accent: {}", demo_shell_accent_color_hex()));
        }
    });
    generated::host_events::on_demo_shell_dark_mode_changed({
        let dark_mode_label = dark_mode_label.clone();
        move |is_dark| {
            DEMO_HOST_DARK_MODE.with(|slot| slot.set(is_dark));
            dark_mode_label.text(format!(
                "Dark mode: {}",
                if is_dark { "true" } else { "false" }
            ));
        }
    });

    let virtual_list_card = ui! {
        demo_card(
            "Virtual list",
            "FUI-RS dashboard now uses the same pooled-row virtualization model as FUI-AS: fixed-height rows, recycled SelectionArea containers, and ScrollState-driven windowing.",
            0xFDE68AFF,
        ).margin(0.0, 0.0, 0.0, 16.0)
    };
    let row_labels: Rc<std::cell::RefCell<HashMap<usize, TextNode>>> =
        Rc::new(std::cell::RefCell::new(HashMap::new()));
    let sidebar_list = ui! {
        virtual_list(SIDEBAR_LIST_TOTAL_ITEMS, SIDEBAR_LIST_ITEM_HEIGHT)
            .onBindItem({
                let captured_row_labels = row_labels.clone();
                move |container, index| {
                    let key = std::ptr::from_ref(container) as usize;
                    let existing = { captured_row_labels.borrow().get(&key).cloned() };
                    let label = if let Some(existing) = existing {
                        existing
                    } else {
                        let label = text("");
                        container.child(&label);
                        captured_row_labels.borrow_mut().insert(key, label.clone());
                        label
                    };
                    let text_value = format!("Item {}", index);
                    label.text(&text_value);
                    label.semantic_label(&text_value);
                }
            })
            .node_id("demo-dashboard:sidebar-list")
            .persist_scroll(true)
            .width(100.0, Unit::Percent)
            .height(180.0, Unit::Pixel)
    };
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
                    text("").text(format!(
                        "Wall clock: {} ms",
                        demo_shell_wall_clock_since_epoch_ms()
                    )),
                }
        },
        virtual_list_card,
        slider_card,
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

fn mount_dashboard_page(_: &ScrollBox) {
    DEMO_HOST_TICK.with(|slot| slot.set(demo_shell_clock_tick_seconds()));
    DEMO_HOST_DARK_MODE.with(|slot| slot.set(demo_shell_is_dark_mode()));
}

fn dispose_dashboard_page(_: &ScrollBox) {
    DEMO_SUBSCRIPTIONS.with(|slot| slot.borrow_mut().clear());
    clear_demo_shared_state();
}

fui_managed_app!(
    ScrollBox,
    build_dashboard_page,
    |page: &ScrollBox| page.clone(),
    mount: mount_dashboard_page,
    dispose: dispose_dashboard_page
);

#[no_mangle]
pub extern "C" fn __getDemoHostTick() -> i32 {
    DEMO_HOST_TICK.with(Cell::get)
}

#[no_mangle]
pub extern "C" fn __getDemoHostDarkMode() -> bool {
    DEMO_HOST_DARK_MODE.with(Cell::get)
}
