use fui::bridge_callbacks;
use fui::ffi::{self, Call};
use fui::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[test]
fn flex_box_clips_to_bounds_by_default_and_portal_opts_out() {
    ffi::test::reset();

    let root = column();
    let overlay = portal();
    root.child(&overlay);
    Application::mount(root.clone());

    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetClipToBounds { handle, clip }
        if *handle == root.handle().raw() && *clip
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetClipToBounds { handle, clip }
        if *handle == overlay.handle().raw() && !*clip
    )));
}

#[test]
fn scroll_view_scroll_bridge_updates_bound_scroll_state() {
    ffi::test::reset();

    let state = ScrollState::new();
    let view = scroll_view();
    view.bind_scroll_state(state.clone());
    Application::mount(view.clone());

    bridge_callbacks::__fui_on_scroll(view.handle().raw(), 12.0, 34.0, 640.0, 960.0, 200.0, 120.0);

    assert_eq!(state.offset_x(), 12.0);
    assert_eq!(state.offset_y(), 34.0);
    assert_eq!(state.content_width(), 640.0);
    assert_eq!(state.content_height(), 960.0);
    assert_eq!(state.viewport_width(), 200.0);
    assert_eq!(state.viewport_height(), 120.0);
}

#[test]
fn scroll_box_mounts_scroll_chrome_with_proxy_targets() {
    ffi::test::reset();

    let scroll = scroll_box();
    scroll
        .width(320.0, Unit::Pixel)
        .height(220.0, Unit::Pixel)
        .scrollbar_gutter(4.0);
    scroll.child(&text("hello"));
    Application::mount(scroll.clone());

    let _initial_calls = ffi::test::take_calls();

    bridge_callbacks::__fui_on_scroll(
        scroll.viewport().handle().raw(),
        0.0,
        0.0,
        640.0,
        960.0,
        200.0,
        120.0,
    );

    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetScrollProxyTarget { scroll_handle, .. }
        if *scroll_handle == scroll.viewport().handle().raw()
    )));
}

#[test]
fn selection_area_sets_bridge_flag_and_receives_cross_selection_text() {
    ffi::test::reset();

    let area = selection_area();
    area.child(&text("Selectable"));
    Application::mount(area.clone());

    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSelectionArea { handle, is_area }
        if *handle == area.handle().raw() && *is_area
    )));

    let text = "picked";
    unsafe {
        fui::event::__fui_on_cross_selection_changed(
            area.handle().raw(),
            text.as_ptr(),
            text.len() as u32,
        );
    }
    assert_eq!(area.selected_text(), "picked");
}

#[test]
fn virtual_list_mounts_selection_barrier_and_visible_window() {
    ffi::test::reset();

    let rendered = Rc::new(RefCell::new(Vec::<i32>::new()));
    let labels: Rc<RefCell<HashMap<usize, TextNode>>> = Rc::new(RefCell::new(HashMap::new()));
    let list = virtual_list(10_000, 20.0);
    let captured_rendered = rendered.clone();
    let captured_labels = labels.clone();
    list.on_bind_item(move |container, index| {
        captured_rendered.borrow_mut().push(index);
        let key = std::ptr::from_ref(container) as usize;
        let existing = { captured_labels.borrow().get(&key).cloned() };
        let label = if let Some(existing) = existing {
            existing
        } else {
            let label = text("");
            container.child(&label);
            captured_labels.borrow_mut().insert(key, label.clone());
            label
        };
        let text_value = format!("Item {}", index);
        label.text(&text_value);
        label.semantic_label(&text_value);
    });
    list.node_id("demo-dashboard:sidebar-list")
        .persist_scroll(true)
        .width(180.0, Unit::Pixel)
        .height(100.0, Unit::Pixel);

    Application::mount(list.clone());

    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSelectionAreaBarrier { handle, is_barrier }
        if *handle == list.handle().raw() && *is_barrier
    )));
    assert_eq!(list.first_visible_index(), 0);
    assert_eq!(list.rendered_item_count(), 6);
    assert!(rendered.borrow().len() >= 6);
}

#[test]
fn virtual_list_restores_persisted_scroll_through_inner_scroll_box() {
    ffi::test::reset();
    fui::persisted::store_scroll_offset("demo-dashboard:sidebar-list", 0.0, 80.0);
    let _ = ffi::test::take_calls();

    let labels: Rc<RefCell<HashMap<usize, TextNode>>> = Rc::new(RefCell::new(HashMap::new()));
    let list = virtual_list(10_000, 20.0);
    let captured_labels = labels.clone();
    list.on_bind_item(move |container, index| {
        let key = std::ptr::from_ref(container) as usize;
        let existing = { captured_labels.borrow().get(&key).cloned() };
        let label = if let Some(existing) = existing {
            existing
        } else {
            let label = text("");
            container.child(&label);
            captured_labels.borrow_mut().insert(key, label.clone());
            label
        };
        label.text(format!("Item {}", index));
    });
    list.node_id("demo-dashboard:sidebar-list")
        .persist_scroll(true)
        .width(180.0, Unit::Pixel)
        .height(100.0, Unit::Pixel);

    Application::mount(list.clone());

    assert_eq!(list.scroll_state().offset_y(), 80.0);
    assert_eq!(list.first_visible_index(), 4);
}

#[test]
fn scroll_box_common_style_applies_to_both_axes_and_allows_axis_overrides() {
    ffi::test::reset();
    let content = flex_box();
    content.width(400.0, Unit::Pixel).height(400.0, Unit::Pixel);
    let scroll = scroll_box();
    scroll
        .width(100.0, Unit::Pixel)
        .height(100.0, Unit::Pixel)
        .vertical_scrollbar_visibility(ScrollBarVisibility::Always)
        .horizontal_scrollbar_visibility(ScrollBarVisibility::Always)
        .scrollbar_style(
            ScrollBarStyle::new()
                .track_width(12.0)
                .thumb_width(7.0)
                .thumb_min_height(24.0)
                .track_corner_radius(5.0)
                .thumb_corner_radius(3.0)
                .track_color(0x112233FF)
                .thumb_color(0x445566FF),
        )
        .child(&content);
    scroll.vertical_scrollbar().thumb_width(9.0);

    Application::mount(scroll);
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetBoxStyle { bg_color, .. } if *bg_color == 0x112233FF
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetBoxStyle { bg_color, .. } if *bg_color == 0x445566FF
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetWidth { value, .. } if (*value - 9.0).abs() < f32::EPSILON
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetHeight { value, .. } if (*value - 7.0).abs() < f32::EPSILON
    )));
}
