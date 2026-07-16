use fui::event::{self, FocusChangedEventArgs, GestureIntent, PointerType};
use fui::ffi::{
    self, Call, CursorStyle, HandleValue, KeyEventType, KeyModifier, NodeType, Orientation,
    PointerEventType, PositionType, Visibility,
};
use fui::on_loaded;
use fui::prelude::*;
use fui::{
    context_menu, dialog, form, popup, ContextMenuAction, Dialog, MenuItem, PopupPlacement,
    PopupPresenter,
};
use fui::{DragDataObject, DragDropEffects, DropProposal};
use std::cell::Cell;
use std::rc::Rc;

fn created_handles_of_type(calls: &[Call], node_type: NodeType) -> Vec<u64> {
    calls
        .iter()
        .filter_map(|call| match call {
            Call::CreateNode {
                handle,
                node_type: created_node_type,
            } if *created_node_type == node_type as u32 => Some(*handle),
            _ => None,
        })
        .collect()
}

fn handle_with_semantic_role(calls: &[Call], role: SemanticRole) -> u64 {
    calls
        .iter()
        .find_map(|call| match call {
            Call::SetSemanticRole { handle, role_enum } if *role_enum == role as u32 => {
                Some(*handle)
            }
            _ => None,
        })
        .expect("semantic role handle")
}

fn handle_with_semantic_label(calls: &[Call], expected: &str) -> u64 {
    calls
        .iter()
        .find_map(|call| match call {
            Call::SetSemanticLabel { handle, label } if label == expected => Some(*handle),
            _ => None,
        })
        .expect("semantic label handle")
}

fn parent_for_child(calls: &[Call], expected_child: u64) -> u64 {
    calls
        .iter()
        .find_map(|call| match call {
            Call::NodeAddChild { parent, child } if *child == expected_child => Some(*parent),
            _ => None,
        })
        .expect("parent for child")
}

#[test]
fn layout_edge_signatures_match_fui_as_order() {
    ffi::test::reset();
    let root = flex_box();
    root.padding(1.0, 2.0, 3.0, 4.0)
        .margin(5.0, 6.0, 7.0, 8.0)
        .position_type(PositionType::Absolute)
        .position(9.0, 10.0);

    Application::mount(root.clone());
    let calls = ffi::test::take_calls();
    let handle = root.handle().raw();

    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetPadding {
            handle: call_handle,
            left,
            top,
            right,
            bottom,
        } if *call_handle == handle && *left == 1.0 && *top == 2.0 && *right == 3.0 && *bottom == 4.0
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetMargin {
            handle: call_handle,
            left,
            top,
            right,
            bottom,
        } if *call_handle == handle && *left == 5.0 && *top == 6.0 && *right == 7.0 && *bottom == 8.0
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetPosition {
            handle: call_handle,
            left,
            top,
            right,
            bottom,
        } if *call_handle == handle && *left == 9.0 && *top == 10.0 && right.is_nan() && bottom.is_nan()
    )));
}

#[test]
fn unspecified_dimensions_remain_unset_while_explicit_auto_is_preserved() {
    ffi::test::reset();
    let implicit = button("Implicit sizing");
    let explicit = button("Explicit auto sizing");
    explicit.width(0.0, Unit::Auto).height(0.0, Unit::Auto);
    let root = column();
    root.child(&implicit).child(&explicit);

    Application::mount(root);
    let implicit_handle = implicit.handle().raw();
    let explicit_handle = explicit.handle().raw();
    let calls = ffi::test::take_calls();

    assert!(!calls.iter().any(|call| matches!(
        call,
        Call::SetWidth { handle, .. } | Call::SetHeight { handle, .. }
            if *handle == implicit_handle
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetWidth { handle, value, unit_enum }
            if *handle == explicit_handle && *value == 0.0 && *unit_enum == Unit::Auto as u32
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetHeight { handle, value, unit_enum }
            if *handle == explicit_handle && *value == 0.0 && *unit_enum == Unit::Auto as u32
    )));
}

#[test]
fn loaded_callbacks_fire_during_mount_before_initial_commit() {
    ffi::test::reset();
    let label = text("Before");
    let fired = Rc::new(Cell::new(false));

    on_loaded({
        let label = label.clone();
        let fired = fired.clone();
        move |_| {
            fired.set(true);
            label.text("Loaded");
        }
    });

    Application::mount(label.clone());
    assert!(fired.get());
    let calls = ffi::test::take_calls();
    let loaded_text_index = calls
        .iter()
        .position(|call| matches!(call, Call::SetText { text, .. } if text == "Loaded"))
        .expect("loaded text mutation");
    let first_commit_index = calls
        .iter()
        .position(|call| matches!(call, Call::CommitFrame))
        .expect("initial commit");
    assert!(loaded_text_index < first_commit_index);
}

#[test]
fn late_loaded_callback_fires_immediately_after_mount() {
    ffi::test::reset();
    Application::mount(flex_box());
    let fired = Rc::new(Cell::new(false));

    on_loaded({
        let fired = fired.clone();
        move |_| fired.set(true)
    });

    assert!(fired.get());
}

#[test]
fn custom_drawable_mark_dirty_requests_render() {
    ffi::test::reset();
    let drawable = custom_drawable(|_| {});
    Application::mount(drawable.clone());
    ffi::test::take_calls();

    drawable.mark_dirty();
    drawable.mark_dirty();
    let calls = ffi::test::take_calls();
    assert_eq!(
        calls
            .iter()
            .filter(|call| matches!(call, Call::RequestRender))
            .count(),
        1
    );

    Application::flush_renders();
    ffi::test::take_calls();
    drawable.mark_dirty();
    let calls = ffi::test::take_calls();
    assert_eq!(
        calls
            .iter()
            .filter(|call| matches!(call, Call::RequestRender))
            .count(),
        1
    );

    ffi::test::set_visible_bounds(Some((0.0, 0.0, 0.0, 0.0)));
    Application::flush_renders();
    ffi::test::take_calls();
    drawable.mark_dirty();
    let calls = ffi::test::take_calls();
    assert_eq!(
        calls
            .iter()
            .filter(|call| matches!(call, Call::RequestRender))
            .count(),
        0
    );

    ffi::test::set_visible_bounds(Some((0.0, 0.0, 24.0, 24.0)));
    drawable.mark_dirty();
    let calls = ffi::test::take_calls();
    assert_eq!(
        calls
            .iter()
            .filter(|call| matches!(call, Call::RequestRender))
            .count(),
        1
    );
}

fn primary_click(handle: u64, click_count: i32) {
    primary_click_at(handle, 10.0, 10.0, click_count);
}

fn pointer_event(
    event_type: PointerEventType,
    handle: u64,
    scene_x: f32,
    scene_y: f32,
    button: i32,
    buttons: u32,
    click_count: i32,
) {
    event::__fui_on_pointer_event_with_metadata(
        event_type as u32,
        handle,
        scene_x,
        scene_y,
        0,
        1,
        1,
        button,
        buttons,
        0.0,
        0.0,
        0.0,
        click_count,
    );
}

fn primary_click_at(handle: u64, scene_x: f32, scene_y: f32, click_count: i32) {
    pointer_event(
        PointerEventType::Down,
        handle,
        scene_x,
        scene_y,
        0,
        1,
        click_count,
    );
    pointer_event(PointerEventType::Up, handle, scene_x, scene_y, 0, 0, 0);
}

fn key_event(event_type: KeyEventType, key: &str, modifiers: u32) -> bool {
    event::__fui_on_key_event(event_type as u32, key.as_ptr(), key.len() as u32, modifiers)
}

fn cursor_styles(calls: &[Call]) -> Vec<u32> {
    calls
        .iter()
        .filter_map(|call| match call {
            Call::SetCursor { style } => Some(*style),
            _ => None,
        })
        .collect()
}

fn handles_with_bg_color(calls: &[Call], expected_color: u32) -> Vec<u64> {
    calls
        .iter()
        .filter_map(|call| match call {
            Call::SetBgColor { handle, color } if *color == expected_color => Some(*handle),
            Call::SetBoxStyle {
                handle, bg_color, ..
            } if *bg_color == expected_color => Some(*handle),
            _ => None,
        })
        .collect()
}

fn focus<T: Node>(node: &T) {
    event::__fui_on_focus_changed(node.handle().raw(), true);
}

fn pointer_move(handle: u64, scene_x: f32, scene_y: f32) {
    pointer_event(PointerEventType::Move, handle, scene_x, scene_y, 0, 1, 0);
}

#[test]
fn retained_child_construction_does_not_build_native_nodes() {
    ffi::test::reset();
    let root = column();
    let child = text("hello");
    root.child(&child);
    let calls = ffi::test::take_calls();
    assert!(calls.is_empty());
}

#[test]
fn node_bounds_helpers_match_fui_as_zero_fallback_before_build() {
    let root = column();

    assert_eq!(root.get_bounds(), [0.0; 4]);
    assert_eq!(root.absolute_to_local_position(12.0, 34.0), [12.0, 34.0]);
    assert_eq!(root.local_to_absolute_position(12.0, 34.0), [12.0, 34.0]);
}

#[test]
fn length_helpers_emit_matching_layout_units() {
    ffi::test::reset();
    let root = column();
    root.width_len(px(320.0))
        .height_len(pct(50.0))
        .min_width_len(auto())
        .max_height_len(px(640.0));
    Application::mount(root.clone());

    let handle = root.handle().raw();
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetWidth {
            handle: actual,
            value,
            unit_enum
        } if *actual == handle && *value == 320.0 && *unit_enum == Unit::Pixel as u32
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetHeight {
            handle: actual,
            value,
            unit_enum
        } if *actual == handle && *value == 50.0 && *unit_enum == Unit::Percent as u32
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetMinWidth {
            handle: actual,
            value,
            unit_enum
        } if *actual == handle && *value == 0.0 && *unit_enum == Unit::Auto as u32
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetMaxHeight {
            handle: actual,
            value,
            unit_enum
        } if *actual == handle && *value == 640.0 && *unit_enum == Unit::Pixel as u32
    )));
}

#[test]
fn retained_mount_builds_shell_then_user_tree() {
    ffi::test::reset();
    let root = column();
    let child = text("hello");
    root.child(&child);
    Application::mount(root.clone());
    let calls = ffi::test::take_calls();
    let root_handle = root.handle().raw();
    let child_handle = child.handle().raw();
    assert!(matches!(calls[0], Call::Reset));
    assert!(matches!(
        calls
            .iter()
            .find(|call| matches!(call, Call::SetRoot { .. })),
        Some(Call::SetRoot { .. })
    ));
    assert!(calls
        .iter()
        .any(|call| matches!(call, Call::CreateNode { handle, .. } if *handle == root_handle)));
    assert!(calls
        .iter()
        .any(|call| matches!(call, Call::CreateNode { handle, .. } if *handle == child_handle)));
    assert!(calls.iter().any(|call| matches!(call, Call::NodeAddChild { parent, child } if *parent == root_handle && *child == child_handle)));
}

#[test]
fn retained_post_build_add_and_remove_do_not_delete_child() {
    ffi::test::reset();
    let root = column();
    Application::mount(root.clone());
    ffi::test::take_calls();
    let root_handle = root.handle().raw();

    let child = text("late child");
    root.child(&child);
    let calls = ffi::test::take_calls();
    let child_handle = *created_handles_of_type(&calls, NodeType::Text)
        .last()
        .expect("child handle");
    assert!(calls.iter().any(|call| matches!(call, Call::NodeAddChild { parent, child } if *parent == root_handle && *child == child_handle)));
    assert!(calls.iter().any(|call| matches!(call, Call::RequestRender)));

    assert!(root.remove_child(&child));
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(call, Call::NodeRemoveChild { parent, child } if *parent == root_handle && *child == child_handle)));
    assert!(!calls
        .iter()
        .any(|call| matches!(call, Call::DeleteNode { handle } if *handle == child_handle)));
}

#[test]
fn retained_reparent_removes_from_old_parent_before_adding_to_new_parent() {
    ffi::test::reset();
    let root = column();
    let left = column();
    let right = column();
    let child = text("move me");
    root.child(&left).child(&right);
    left.child(&child);
    Application::mount(root);
    ffi::test::take_calls();
    let left_handle = left.handle().raw();
    let right_handle = right.handle().raw();
    let child_handle = child.handle().raw();

    right.child(&child);
    let calls = ffi::test::take_calls();
    let remove_index = calls
        .iter()
        .position(|call| matches!(call, Call::NodeRemoveChild { parent, child } if *parent == left_handle && *child == child_handle))
        .expect("remove from old parent");
    let add_index = calls
        .iter()
        .position(|call| matches!(call, Call::NodeAddChild { parent, child } if *parent == right_handle && *child == child_handle))
        .expect("add to new parent");
    assert!(remove_index < add_index);
}

#[test]
fn retained_mutation_after_build_updates_native_and_requests_render() {
    ffi::test::reset();
    let root = column();
    Application::mount(root.clone());
    ffi::test::take_calls();
    let root_handle = root.handle().raw();

    root.width(240.0, Unit::Pixel).bg_color(0x11223344);
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(call, Call::SetWidth { handle, value, unit_enum } if *handle == root_handle && *value == 240.0 && *unit_enum == Unit::Pixel as u32)));
    assert!(calls.iter().any(|call| matches!(call, Call::SetBgColor { handle, color } if *handle == root_handle && *color == 0x11223344)));
    assert!(calls.iter().any(|call| matches!(call, Call::RequestRender)));
}

#[test]
fn retained_remove_unregisters_event_route_without_deleting_child() {
    ffi::test::reset();
    let root = column();
    let child = flex_box();
    child.on_pan_gesture(|event| event.handled = true);
    root.child(&child);
    Application::mount(root.clone());
    ffi::test::take_calls();
    let child_handle = child.handle().raw();
    assert_eq!(
        event::__fui_resolve_gesture_owner(child_handle),
        child_handle
    );

    assert!(root.remove_child(&child));
    let calls = ffi::test::take_calls();
    assert!(calls
        .iter()
        .any(|call| matches!(call, Call::NodeRemoveChild { child, .. } if *child == child_handle)));
    assert!(!calls
        .iter()
        .any(|call| matches!(call, Call::DeleteNode { handle } if *handle == child_handle)));
    assert_eq!(
        event::__fui_resolve_gesture_owner(child_handle),
        child_handle
    );

    child.dispose();
    assert_eq!(
        event::__fui_resolve_gesture_owner(child_handle),
        HandleValue::Invalid as u64
    );
}

#[test]
fn composed_controls_inherit_node_gesture_and_long_press_surface() {
    ffi::test::reset();
    let pan_hits = Rc::new(Cell::new(0));
    let long_press_hits = Rc::new(Cell::new(0));
    let pan_hits_clone = pan_hits.clone();
    let long_press_hits_clone = long_press_hits.clone();
    let input = text_input();
    input
        .node_id("gesture-text-input")
        .on_pan_gesture(move |event| {
            pan_hits_clone.set(pan_hits_clone.get() + 1);
            event.handled = true;
        })
        .long_press_options(650, 18.0)
        .on_long_press(move |event| {
            long_press_hits_clone.set(long_press_hits_clone.get() + 1);
            event.handled = true;
        });

    Application::mount(input.clone());
    let input_handle = input.handle().raw();
    assert_eq!(
        event::__fui_resolve_gesture_owner(input_handle),
        input_handle
    );
    assert_eq!(
        event::__fui_resolve_long_press_owner(input_handle),
        input_handle
    );
    assert_eq!(
        event::__fui_get_long_press_minimum_duration_ms(input_handle),
        650
    );
    assert_eq!(
        event::__fui_get_long_press_movement_tolerance(input_handle),
        18.0
    );

    assert!(event::__fui_on_gesture_event(
        input_handle,
        1,
        1,
        12.0,
        14.0,
        4.0,
        2.0,
        1.0,
        1
    ));
    assert!(event::__fui_on_long_press_event(
        input_handle,
        12.0,
        14.0,
        1,
        1,
        0,
        660
    ));
    assert_eq!(pan_hits.get(), 1);
    assert_eq!(long_press_hits.get(), 1);
}

#[test]
fn retained_dispose_deletes_tree_recursively() {
    ffi::test::reset();
    let root = column();
    let child = column();
    let grandchild = text("grandchild");
    child.child(&grandchild);
    root.child(&child);
    Application::mount(root.clone());
    ffi::test::take_calls();
    let handles = vec![
        root.handle().raw(),
        child.handle().raw(),
        grandchild.handle().raw(),
    ];

    root.dispose();
    let calls = ffi::test::take_calls();
    for handle in handles {
        assert!(calls.iter().any(
            |call| matches!(call, Call::DeleteNode { handle: deleted } if *deleted == handle)
        ));
    }
}

#[test]
fn popup_presenter_show_attaches_overlay_and_pushes_semantic_scope() {
    ffi::test::reset();
    let root = portal();
    let surface = column();
    let presenter = PopupPresenter::new(root.clone(), surface.clone());
    Application::mount(root.clone());
    ffi::test::take_calls();
    let root_handle = root.handle().raw();

    presenter.show_at_point(24.0, 32.0, 120.0, 80.0);
    let calls = ffi::test::take_calls();
    let new_handles = created_handles_of_type(&calls, NodeType::FlexBox);
    assert_eq!(new_handles.len(), 2);
    let overlay_handle = new_handles[0];
    let surface_handle = new_handles[1];
    assert!(presenter.is_open());
    assert_eq!(presenter.surface_x(), 24.0);
    assert_eq!(presenter.surface_y(), 32.0);
    assert!(calls.iter().any(|call| matches!(call, Call::NodeAddChild { parent, child } if *parent == root_handle && *child == overlay_handle)));
    assert!(calls.iter().any(|call| matches!(call, Call::NodeAddChild { parent, child } if *parent == overlay_handle && *child == surface_handle)));
    assert!(calls.iter().any(|call| matches!(call, Call::PushSemanticScope { handle, token } if *handle == surface_handle && *token == 1)));
}

#[test]
fn popup_presenter_hide_detaches_without_deleting_and_reshow_reuses_handles() {
    ffi::test::reset();
    let root = portal();
    let surface = column();
    let presenter = PopupPresenter::new(root.clone(), surface);
    Application::mount(root.clone());
    ffi::test::take_calls();
    let root_handle = root.handle().raw();

    presenter.show_at_point(12.0, 16.0, 120.0, 80.0);
    let calls = ffi::test::take_calls();
    let handles = created_handles_of_type(&calls, NodeType::FlexBox);
    let overlay_handle = handles[0];
    let surface_handle = handles[1];

    presenter.hide();
    let calls = ffi::test::take_calls();
    assert!(!presenter.is_open());
    assert!(calls.iter().any(|call| matches!(call, Call::NodeRemoveChild { parent, child } if *parent == root_handle && *child == overlay_handle)));
    assert!(calls
        .iter()
        .any(|call| matches!(call, Call::RemoveSemanticScope { token } if *token == 1)));
    assert!(!calls.iter().any(|call| matches!(call, Call::DeleteNode { handle } if *handle == overlay_handle || *handle == surface_handle)));

    presenter.show_at_point(20.0, 24.0, 120.0, 80.0);
    let calls = ffi::test::take_calls();
    assert!(presenter.is_open());
    assert!(created_handles_of_type(&calls, NodeType::FlexBox).is_empty());
    assert!(calls.iter().any(|call| matches!(call, Call::NodeAddChild { parent, child } if *parent == root_handle && *child == overlay_handle)));
    assert!(calls.iter().any(|call| matches!(call, Call::PushSemanticScope { handle, token } if *handle == surface_handle && *token == 2)));
}

#[test]
fn popup_presenter_clamps_point_and_anchor_placement() {
    ffi::test::reset();
    ffi::test::set_viewport(320.0, 220.0);
    let root = portal();
    let surface = column();
    let presenter = PopupPresenter::new(root.clone(), surface);
    presenter.edge_padding(10.0).anchor_gap(6.0);
    Application::mount(root.clone());
    ffi::test::take_calls();

    presenter.show_at_point(400.0, 400.0, 100.0, 50.0);
    assert_eq!(presenter.surface_x(), 210.0);
    assert_eq!(presenter.surface_y(), 160.0);
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(call, Call::SetPosition { left, top, .. } if *left == 210.0 && *top == 160.0)));

    presenter.show_anchored_with_placement(
        40.0,
        170.0,
        80.0,
        30.0,
        100.0,
        80.0,
        PopupPlacement::Auto,
    );
    assert_eq!(presenter.surface_x(), 40.0);
    assert_eq!(presenter.surface_y(), 84.0);

    presenter.show_anchored_with_placement(
        40.0,
        170.0,
        80.0,
        30.0,
        100.0,
        80.0,
        PopupPlacement::Overlap,
    );
    assert_eq!(presenter.surface_y(), 130.0);
}

#[test]
fn drag_drop_session_updates_target_and_completes_drop() {
    ffi::test::reset();
    let source = flex_box();
    let target = flex_box();
    let dropped = Rc::new(Cell::new(false));
    let completed_effect = Rc::new(Cell::new(DragDropEffects::None as u32));

    source
        .interactive(true)
        .drag_allowed_effects(DragDropEffects::Move)
        .drag_data(|| {
            Some(
                DragDataObject::new()
                    .set_format("application/x-test", "row-a")
                    .set_text("Row A"),
            )
        })
        .on_drag_completed({
            let completed_effect = completed_effect.clone();
            move |event| completed_effect.set(event.effect as u32)
        });

    target
        .interactive(true)
        .allow_drop(true)
        .on_drag_enter(|_args| DropProposal::new(DragDropEffects::Move, true))
        .on_drag_over(|_args| DropProposal::new(DragDropEffects::Move, true))
        .on_drop({
            let dropped = dropped.clone();
            move |args| {
                if args
                    .session
                    .data
                    .get_format("application/x-test")
                    .as_deref()
                    == Some("row-a")
                {
                    dropped.set(true);
                }
            }
        });

    let root = column();
    root.child(&source).child(&target);
    Application::mount(root);
    ffi::test::take_calls();

    pointer_event(
        PointerEventType::Down,
        source.handle().raw(),
        10.0,
        10.0,
        0,
        1,
        1,
    );
    pointer_move(source.handle().raw(), 20.0, 20.0);
    pointer_move(target.handle().raw(), 30.0, 30.0);
    pointer_event(
        PointerEventType::Up,
        target.handle().raw(),
        30.0,
        30.0,
        0,
        0,
        0,
    );

    assert!(dropped.get());
    assert_eq!(completed_effect.get(), DragDropEffects::Move as u32);
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetPointerCapture { .. } | Call::ReleasePointerCapture
    )));
    assert!(cursor_styles(&calls).contains(&(CursorStyle::Move as u32)));
}

#[test]
fn touch_drag_waits_for_long_press_then_drops_on_release() {
    ffi::test::reset();
    let source = flex_box();
    let target = flex_box();
    let dropped = Rc::new(Cell::new(false));

    source
        .interactive(true)
        .drag_allowed_effects(DragDropEffects::Move)
        .drag_data(|| Some(DragDataObject::new().set_text("Touch payload")));
    target
        .interactive(true)
        .allow_drop(true)
        .on_drag_enter(|_args| DropProposal::new(DragDropEffects::Move, true))
        .on_drag_over(|_args| DropProposal::new(DragDropEffects::Move, true))
        .on_drop({
            let dropped = dropped.clone();
            move |_args| dropped.set(true)
        });

    let root = column();
    root.child(&source).child(&target);
    Application::mount(root);
    ffi::test::take_calls();

    assert_eq!(
        event::__fui_resolve_long_press_owner(source.handle().raw()),
        source.handle().raw()
    );
    event::__fui_on_pointer_event_with_metadata(
        PointerEventType::Down as u32,
        source.handle().raw(),
        10.0,
        10.0,
        0,
        7,
        PointerType::Touch as u32,
        0,
        1,
        0.0,
        0.0,
        0.0,
        0,
    );
    event::__fui_on_pointer_event_with_metadata(
        PointerEventType::Move as u32,
        source.handle().raw(),
        20.0,
        20.0,
        0,
        7,
        PointerType::Touch as u32,
        0,
        1,
        0.0,
        0.0,
        0.0,
        0,
    );
    assert!(!event::__fui_on_pointer_event_with_metadata(
        PointerEventType::Move as u32,
        target.handle().raw(),
        30.0,
        30.0,
        0,
        7,
        PointerType::Touch as u32,
        0,
        1,
        0.0,
        0.0,
        0.0,
        0,
    ));
    assert!(event::__fui_on_long_press_event(
        source.handle().raw(),
        20.0,
        20.0,
        7,
        PointerType::Touch as u32,
        0,
        500,
    ));
    event::__fui_on_pointer_event_with_metadata(
        PointerEventType::Move as u32,
        target.handle().raw(),
        30.0,
        30.0,
        0,
        7,
        PointerType::Touch as u32,
        0,
        1,
        0.0,
        0.0,
        0.0,
        0,
    );
    event::__fui_on_pointer_event_with_metadata(
        PointerEventType::Up as u32,
        target.handle().raw(),
        30.0,
        30.0,
        0,
        7,
        PointerType::Touch as u32,
        0,
        0,
        0.0,
        0.0,
        0.0,
        0,
    );

    assert!(dropped.get());
}

#[test]
fn popup_presenter_syncs_overlay_bounds_from_root_bounds_and_viewport() {
    ffi::test::reset();
    ffi::test::set_viewport(640.0, 480.0);
    let root = portal();
    let surface = column();
    let presenter = PopupPresenter::new(root.clone(), surface);
    Application::mount(root.clone());
    ffi::test::take_calls();

    presenter.show_at_point(20.0, 20.0, 100.0, 80.0);
    let calls = ffi::test::take_calls();
    let overlay_handle = created_handles_of_type(&calls, NodeType::FlexBox)[0];
    assert!(calls
        .iter()
        .any(|call| matches!(call, Call::GetBounds { .. })));
    assert!(calls.iter().any(|call| matches!(call, Call::SetPosition { handle, left, top, .. } if *handle == overlay_handle && *left == -0.0 && *top == -0.0)));
    assert!(calls.iter().any(|call| matches!(call, Call::SetWidth { handle, value, unit_enum } if *handle == overlay_handle && *value == 640.0 && *unit_enum == Unit::Pixel as u32)));
    assert!(calls.iter().any(|call| matches!(call, Call::SetHeight { handle, value, unit_enum } if *handle == overlay_handle && *value == 480.0 && *unit_enum == Unit::Pixel as u32)));
}

#[test]
fn popup_presenter_dispose_hides_and_deletes_overlay_tree() {
    ffi::test::reset();
    let root = portal();
    let surface = column();
    let presenter = PopupPresenter::new(root.clone(), surface);
    Application::mount(root.clone());
    ffi::test::take_calls();

    presenter.show_at_point(10.0, 10.0, 100.0, 60.0);
    let calls = ffi::test::take_calls();
    let handles = created_handles_of_type(&calls, NodeType::FlexBox);
    let overlay_handle = handles[0];
    let surface_handle = handles[1];

    presenter.dispose();
    let calls = ffi::test::take_calls();
    assert!(!presenter.is_open());
    assert!(calls
        .iter()
        .any(|call| matches!(call, Call::RemoveSemanticScope { token } if *token == 1)));
    assert!(calls
        .iter()
        .any(|call| matches!(call, Call::DeleteNode { handle } if *handle == overlay_handle)));
    assert!(calls
        .iter()
        .any(|call| matches!(call, Call::DeleteNode { handle } if *handle == surface_handle)));
}

#[test]
fn popup_control_show_hide_and_reshow_reuses_overlay_handles() {
    ffi::test::reset();
    let root = portal();
    let popup = popup();
    root.child(&popup);
    Application::mount(root.clone());
    ffi::test::take_calls();

    popup.show_at_point(18.0, 24.0, 160.0, 96.0);
    let calls = ffi::test::take_calls();
    let surface_handle = calls
        .iter()
        .find_map(|call| match call {
            Call::PushSemanticScope { handle, .. } => Some(*handle),
            _ => None,
        })
        .expect("surface semantic scope");
    let overlay_handle = parent_for_child(&calls, surface_handle);
    let overlay_parent_handle = parent_for_child(&calls, overlay_handle);
    assert!(popup.is_open());
    assert!(calls
        .iter()
        .any(|call| matches!(call, Call::NodeAddChild { child, .. } if *child == overlay_handle)));
    assert!(calls.iter().any(|call| matches!(call, Call::PushSemanticScope { handle, token } if *handle == surface_handle && *token == 1)));

    popup.hide();
    let calls = ffi::test::take_calls();
    assert!(!popup.is_open());
    assert!(calls.iter().any(|call| matches!(call, Call::NodeRemoveChild { parent, child } if *parent == overlay_parent_handle && *child == overlay_handle)));
    assert!(calls
        .iter()
        .any(|call| matches!(call, Call::RemoveSemanticScope { token } if *token == 1)));
    assert!(!calls.iter().any(
        |call| matches!(call, Call::DeleteNode { handle } if *handle == overlay_handle || *handle == surface_handle)
    ));

    popup.show_at_point(20.0, 28.0, 160.0, 96.0);
    let calls = ffi::test::take_calls();
    assert!(popup.is_open());
    assert!(created_handles_of_type(&calls, NodeType::FlexBox).is_empty());
    assert!(calls.iter().any(|call| matches!(call, Call::PushSemanticScope { handle, token } if *handle == surface_handle && *token == 2)));
}

#[test]
fn popup_control_backdrop_click_hides_only_when_enabled() {
    ffi::test::reset();
    let root = portal();
    let popup = popup();
    root.child(&popup);
    Application::mount(root);
    ffi::test::take_calls();

    popup.dismiss_on_backdrop_click(false);
    popup.show_at_point(24.0, 32.0, 140.0, 80.0);
    let calls = ffi::test::take_calls();
    let overlay_handle = created_handles_of_type(&calls, NodeType::FlexBox)[0];
    primary_click_at(overlay_handle, 500.0, 500.0, 1);
    assert!(popup.is_open());
    ffi::test::take_calls();

    popup.dismiss_on_backdrop_click(true);
    primary_click(overlay_handle, 1);
    assert!(!popup.is_open());
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(
        |call| matches!(call, Call::NodeRemoveChild { child, .. } if *child == overlay_handle)
    ));
}

#[test]
fn popup_control_children_attach_to_surface_not_overlay_root() {
    ffi::test::reset();
    let root = portal();
    let popup = popup();
    let content = text("popup child");
    popup.child(&content);
    root.child(&popup);
    Application::mount(root);
    ffi::test::take_calls();

    popup.show_at_point(16.0, 22.0, 140.0, 80.0);
    let calls = ffi::test::take_calls();
    let flex_handles = created_handles_of_type(&calls, NodeType::FlexBox);
    let overlay_handle = flex_handles[0];
    let surface_handle = flex_handles[1];
    let text_handle = created_handles_of_type(&calls, NodeType::Text)[0];
    assert!(calls.iter().any(|call| matches!(call, Call::NodeAddChild { parent, child } if *parent == overlay_handle && *child == surface_handle)));
    assert!(calls.iter().any(|call| matches!(call, Call::NodeAddChild { parent, child } if *parent == surface_handle && *child == text_handle)));
}

#[test]
fn popup_control_show_methods_set_surface_size_and_delegate_placement() {
    ffi::test::reset();
    ffi::test::set_viewport(320.0, 220.0);
    let root = portal();
    let popup = popup();
    popup
        .edge_padding(10.0)
        .anchor_gap(6.0)
        .placement(PopupPlacement::Auto);
    root.child(&popup);
    Application::mount(root);
    ffi::test::take_calls();

    popup.show_at_point(400.0, 400.0, 100.0, 50.0);
    let calls = ffi::test::take_calls();
    let surface_handle = created_handles_of_type(&calls, NodeType::FlexBox)[1];
    assert!(calls.iter().any(|call| matches!(call, Call::SetWidth { handle, value, unit_enum } if *handle == surface_handle && *value == 100.0 && *unit_enum == Unit::Pixel as u32)));
    assert!(calls.iter().any(|call| matches!(call, Call::SetHeight { handle, value, unit_enum } if *handle == surface_handle && *value == 50.0 && *unit_enum == Unit::Pixel as u32)));
    assert!(calls.iter().any(|call| matches!(call, Call::SetPosition { handle, left, top, .. } if *handle == surface_handle && *left == 210.0 && *top == 160.0)));

    popup.show_anchored(40.0, 170.0, 80.0, 30.0, 100.0, 80.0);
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(call, Call::SetPosition { handle, left, top, .. } if *handle == surface_handle && *left == 40.0 && *top == 84.0)));
}

#[test]
fn dialog_show_attaches_overlay_pushes_scope_and_shown_fires_after_commit() {
    ffi::test::reset();
    let root = portal();
    let dialog = dialog("Rust dialog control", "Body");
    let shown_count = Rc::new(Cell::new(0));
    let shown_count_for_callback = shown_count.clone();
    dialog.on_shown(move |_event| {
        shown_count_for_callback.set(shown_count_for_callback.get() + 1);
    });
    root.child(&dialog);
    Application::mount(root.clone());
    ffi::test::take_calls();

    dialog.show();
    assert!(dialog.is_open());
    assert_eq!(shown_count.get(), 0);
    let calls = ffi::test::take_calls();
    let overlay_handle = calls
        .iter()
        .find_map(|call| match call {
            Call::PushSemanticScope { handle, .. } => Some(*handle),
            _ => None,
        })
        .expect("overlay scope handle");
    let card_handle = handle_with_semantic_role(&calls, SemanticRole::Dialog);
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::NodeAddChild { child, .. } if *child == overlay_handle
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::PushSemanticScope { handle, token } if *handle == overlay_handle && *token == 1
    )));
    assert_eq!(
        card_handle,
        handle_with_semantic_role(&calls, SemanticRole::Dialog)
    );

    Application::flush_renders();
    assert_eq!(shown_count.get(), 1);
}

#[test]
fn dialog_click_inside_card_does_not_dismiss_and_backdrop_click_cancels() {
    ffi::test::reset();
    let root = portal();
    let dialog = dialog("Rust dialog control", "Body");
    let cancel_count = Rc::new(Cell::new(0));
    let cancel_count_for_callback = cancel_count.clone();
    dialog.on_cancel(move || {
        cancel_count_for_callback.set(cancel_count_for_callback.get() + 1);
    });
    root.child(&dialog);
    Application::mount(root);
    ffi::test::take_calls();

    dialog.show();
    let calls = ffi::test::take_calls();
    let overlay_handle = calls
        .iter()
        .find_map(|call| match call {
            Call::PushSemanticScope { handle, .. } => Some(*handle),
            _ => None,
        })
        .expect("overlay scope handle");
    let card_handle = handle_with_semantic_role(&calls, SemanticRole::Dialog);
    let card_bounds = fui::bindings::ui::get_bounds(card_handle).expect("card bounds");

    primary_click(card_handle, 1);
    assert!(dialog.is_open());
    assert_eq!(cancel_count.get(), 0);
    ffi::test::take_calls();

    primary_click_at(
        overlay_handle,
        card_bounds[0] + card_bounds[2] + 24.0,
        card_bounds[1] + card_bounds[3] + 24.0,
        1,
    );
    assert!(!dialog.is_open());
    assert_eq!(cancel_count.get(), 1);
    let calls = ffi::test::take_calls();
    assert!(calls
        .iter()
        .any(|call| matches!(call, Call::RemoveSemanticScope { token } if *token == 1)));
}

#[test]
fn dialog_accept_cancel_and_active_routes_hide_current_dialog() {
    ffi::test::reset();
    let root = portal();
    let dialog = dialog("Rust dialog control", "Body");
    let accepts = Rc::new(Cell::new(0));
    let accepts_for_callback = accepts.clone();
    dialog.on_accept(move || {
        accepts_for_callback.set(accepts_for_callback.get() + 1);
    });
    let cancels = Rc::new(Cell::new(0));
    let cancels_for_callback = cancels.clone();
    dialog.on_cancel(move || {
        cancels_for_callback.set(cancels_for_callback.get() + 1);
    });
    root.child(&dialog);
    Application::mount(root);
    ffi::test::take_calls();

    dialog.show();
    ffi::test::take_calls();
    Dialog::accept_active_dialog();
    assert_eq!(accepts.get(), 1);
    assert_eq!(cancels.get(), 0);
    assert!(!dialog.is_open());
    let calls = ffi::test::take_calls();
    assert!(calls
        .iter()
        .any(|call| matches!(call, Call::RemoveSemanticScope { token } if *token == 1)));

    dialog.show();
    ffi::test::take_calls();
    Dialog::cancel_active_dialog();
    assert_eq!(accepts.get(), 1);
    assert_eq!(cancels.get(), 1);
    assert!(!dialog.is_open());
}

#[test]
fn form_enter_and_escape_route_to_default_and_cancel_buttons() {
    ffi::test::reset();
    let root = column();
    let submit = button("Submit");
    let cancel = button("Cancel");
    let submit_clicks = Rc::new(Cell::new(0));
    let submit_clicks_for_cb = submit_clicks.clone();
    submit.on_click(move |_event| {
        submit_clicks_for_cb.set(submit_clicks_for_cb.get() + 1);
    });
    let cancel_clicks = Rc::new(Cell::new(0));
    let cancel_clicks_for_cb = cancel_clicks.clone();
    cancel.on_click(move |_event| {
        cancel_clicks_for_cb.set(cancel_clicks_for_cb.get() + 1);
    });
    let form = form();
    form.default_btn(&submit)
        .cancel_btn(&cancel)
        .child(&submit)
        .child(&cancel);
    root.child(&form);
    Application::mount(root);
    ffi::test::take_calls();

    form.activate();
    assert!(key_event(KeyEventType::Down, "Enter", 0));
    assert_eq!(submit_clicks.get(), 0);
    assert!(key_event(KeyEventType::Up, "Enter", 0));
    assert_eq!(submit_clicks.get(), 1);

    assert!(key_event(KeyEventType::Down, "Escape", 0));
    assert_eq!(cancel_clicks.get(), 0);
    assert!(key_event(KeyEventType::Up, "Escape", 0));
    assert_eq!(cancel_clicks.get(), 1);
}

#[test]
fn form_defers_enter_to_focused_button_and_deactivate_removes_filter() {
    ffi::test::reset();
    let root = column();
    let submit = button("Submit");
    let other = button("Other");
    let submit_clicks = Rc::new(Cell::new(0));
    let submit_clicks_for_cb = submit_clicks.clone();
    submit.on_click(move |_event| {
        submit_clicks_for_cb.set(submit_clicks_for_cb.get() + 1);
    });
    let other_clicks = Rc::new(Cell::new(0));
    let other_clicks_for_cb = other_clicks.clone();
    other.on_click(move |_event| {
        other_clicks_for_cb.set(other_clicks_for_cb.get() + 1);
    });
    let form = form();
    form.default_btn(&submit).child(&submit).child(&other);
    root.child(&form);
    Application::mount(root);
    ffi::test::take_calls();

    form.activate();
    focus(&other);
    assert!(key_event(KeyEventType::Down, "Enter", 0));
    assert!(key_event(KeyEventType::Up, "Enter", 0));
    assert_eq!(other_clicks.get(), 1);
    assert_eq!(submit_clicks.get(), 0);

    form.deactivate();
    focus(&submit);
    assert!(key_event(KeyEventType::Down, "Enter", 0));
    assert!(key_event(KeyEventType::Up, "Enter", 0));
    assert_eq!(submit_clicks.get(), 1);
}

#[test]
fn form_enter_defers_to_any_focused_enabled_button() {
    ffi::test::reset();
    let root = column();
    let submit = button("Submit");
    let outside = button("Outside");
    let submit_clicks = Rc::new(Cell::new(0));
    let submit_clicks_for_cb = submit_clicks.clone();
    submit.on_click(move |_event| {
        submit_clicks_for_cb.set(submit_clicks_for_cb.get() + 1);
    });
    let outside_clicks = Rc::new(Cell::new(0));
    let outside_clicks_for_cb = outside_clicks.clone();
    outside.on_click(move |_event| {
        outside_clicks_for_cb.set(outside_clicks_for_cb.get() + 1);
    });
    let form = form();
    form.default_btn(&submit).child(&submit);
    root.child(&outside).child(&form);
    Application::mount(root);
    ffi::test::take_calls();

    form.activate();
    focus(&outside);
    assert!(key_event(KeyEventType::Down, "Enter", 0));
    assert!(key_event(KeyEventType::Up, "Enter", 0));
    assert_eq!(submit_clicks.get(), 0);
    assert_eq!(outside_clicks.get(), 1);
}

#[test]
fn form_clones_share_activation_state() {
    ffi::test::reset();
    let root = column();
    let submit = button("Submit");
    let submit_clicks = Rc::new(Cell::new(0));
    let submit_clicks_for_cb = submit_clicks.clone();
    submit.on_click(move |_event| {
        submit_clicks_for_cb.set(submit_clicks_for_cb.get() + 1);
    });
    let form = form();
    form.default_btn(&submit).child(&submit);
    let form_clone = form.clone();
    root.child(&form);
    Application::mount(root);
    ffi::test::take_calls();

    form.activate();
    form_clone.deactivate();
    assert!(!key_event(KeyEventType::Down, "Enter", 0));
    assert!(!key_event(KeyEventType::Up, "Enter", 0));
    assert_eq!(submit_clicks.get(), 0);

    form_clone.activate();
    assert!(key_event(KeyEventType::Down, "Enter", 0));
    assert!(key_event(KeyEventType::Up, "Enter", 0));
    assert_eq!(submit_clicks.get(), 1);
}

#[test]
fn form_drop_removes_active_key_filter() {
    ffi::test::reset();
    let submit = button("Submit");
    let submit_clicks = Rc::new(Cell::new(0));
    let submit_clicks_for_cb = submit_clicks.clone();
    submit.on_click(move |_event| {
        submit_clicks_for_cb.set(submit_clicks_for_cb.get() + 1);
    });

    {
        let form = form();
        form.default_btn(&submit).activate();
        assert!(key_event(KeyEventType::Down, "Enter", 0));
    }

    assert!(!key_event(KeyEventType::Up, "Enter", 0));
    assert_eq!(submit_clicks.get(), 0);
}

#[test]
fn form_host_autofill_text_inputs_emit_metadata_and_semantic_form() {
    ffi::test::reset();
    let login_form = form();
    let username = text_input();
    username
        .node_id("login-username")
        .placeholder("Username")
        .host_autofill("username");
    let password = text_input();
    password
        .node_id("login-password")
        .placeholder("Password")
        .password(true)
        .host_autofill("current-password");
    login_form.child(&username).child(&password);
    Application::mount(login_form);

    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticRole { role_enum, .. } if *role_enum == SemanticRole::Form as u32
    )));
    let username_handle = calls
        .iter()
        .find_map(|call| match call {
            Call::SetNodeId { handle, node_id } if node_id == "login-username" => Some(*handle),
            _ => None,
        })
        .expect("expected username editor node id");
    let password_handle = calls
        .iter()
        .find_map(|call| match call {
            Call::SetNodeId { handle, node_id } if node_id == "login-password" => Some(*handle),
            _ => None,
        })
        .expect("expected password editor node id");
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::RegisterTextInputMetadata { handle, is_password, hint }
            if *handle == username_handle && !*is_password && hint == "username"
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::RegisterTextInputMetadata { handle, is_password, hint }
            if *handle == password_handle && *is_password && hint == "current-password"
    )));
}

#[test]
fn router_unregisters_old_nodes_when_app_replaced() {
    ffi::test::reset();
    let root = column();
    let child = text("hello");
    child.key(1);
    root.on_pan_gesture(|event| event.handled = true)
        .child(&child);
    Application::mount(root.clone());
    let calls = ffi::test::take_calls();
    let old_child = *created_handles_of_type(&calls, NodeType::Text)
        .last()
        .expect("old child");

    let replacement = text("replacement");
    Application::mount(replacement);
    let invalid = event::__fui_resolve_gesture_owner(old_child);
    assert_eq!(invalid, HandleValue::Invalid as u64);
}

#[test]
fn pointer_click_bubbles_to_parent() {
    ffi::test::reset();
    let clicked = Rc::new(Cell::new(0));
    let clicked_clone = clicked.clone();
    let root = column();
    let child = text("hit");
    child.key(1);
    root.on_click(move |_event| clicked_clone.set(clicked_clone.get() + 1))
        .child(&child);
    Application::mount(root);
    ffi::test::take_calls();
    let child_handle = child.handle().raw();
    event::__fui_on_pointer_event_with_metadata(
        1,
        child_handle,
        10.0,
        10.0,
        0,
        1,
        1,
        0,
        1,
        0.0,
        0.0,
        0.0,
        1,
    );
    event::__fui_on_pointer_event_with_metadata(
        2,
        child_handle,
        10.0,
        10.0,
        0,
        1,
        1,
        0,
        0,
        0.0,
        0.0,
        0.0,
        1,
    );
    assert_eq!(clicked.get(), 1);
}

#[test]
fn pointer_click_uses_pending_down_click_count_when_up_has_zero_count() {
    ffi::test::reset();
    let clicked = Rc::new(Cell::new(0));
    let clicked_clone = clicked.clone();
    let root = flex_box();
    root.on_click(move |event| clicked_clone.set(event.click_count));
    Application::mount(root.clone());
    ffi::test::take_calls();
    let handle = root.handle().raw();

    event::__fui_on_pointer_event_with_metadata(
        1, handle, 10.0, 10.0, 0, 1, 1, 0, 1, 0.0, 0.0, 0.0, 1,
    );
    event::__fui_on_pointer_event_with_metadata(
        2, handle, 10.0, 10.0, 0, 1, 1, 0, 0, 0.0, 0.0, 0.0, 0,
    );

    assert_eq!(clicked.get(), 1);
}

#[test]
fn handled_wheel_event_stops_bubbling() {
    ffi::test::reset();
    let parent_hits = Rc::new(Cell::new(0));
    let parent_hits_clone = parent_hits.clone();
    let child_hits = Rc::new(Cell::new(0));
    let child_hits_clone = child_hits.clone();
    let root = column();
    let child = flex_box();
    child
        .key(1)
        .on_wheel(move |event| {
            child_hits_clone.set(child_hits_clone.get() + 1);
            event.handled = true;
        })
        .width(100.0, Unit::Pixel)
        .height(40.0, Unit::Pixel);
    root.on_wheel(move |_event| parent_hits_clone.set(parent_hits_clone.get() + 1))
        .child(&child);
    Application::mount(root);
    ffi::test::take_calls();
    let child_handle = child.handle().raw();
    assert!(event::__fui_on_wheel_event(
        child_handle,
        5.0,
        5.0,
        0.0,
        1.0,
        0,
        0
    ));
    assert_eq!(child_hits.get(), 1);
    assert_eq!(parent_hits.get(), 0);
}

#[test]
fn wheel_gesture_and_long_press_use_local_coordinates_and_visibility_enabled_guards() {
    ffi::test::reset();
    let wheel_local = Rc::new(Cell::new((0.0_f32, 0.0_f32)));
    let gesture_local = Rc::new(Cell::new((0.0_f32, 0.0_f32)));
    let long_press_local = Rc::new(Cell::new((0.0_f32, 0.0_f32)));
    let wheel_local_clone = wheel_local.clone();
    let gesture_local_clone = gesture_local.clone();
    let long_press_local_clone = long_press_local.clone();

    let root = column();
    let child = flex_box();
    child
        .width(120.0, Unit::Pixel)
        .height(60.0, Unit::Pixel)
        .on_wheel(move |event| {
            wheel_local_clone.set((event.x, event.y));
            event.handled = true;
        })
        .on_pan_gesture(move |event| {
            gesture_local_clone.set((event.x, event.y));
            event.handled = true;
        })
        .on_long_press(move |event| {
            long_press_local_clone.set((event.x, event.y));
            event.handled = true;
        });
    root.child(&child);
    Application::mount(root);
    let child_handle = child.handle().raw();
    let bounds = fui::bindings::ui::get_bounds(child_handle).expect("child bounds");
    let scene_x = bounds[0] + 9.0;
    let scene_y = bounds[1] + 13.0;

    assert!(event::__fui_on_wheel_event(
        child_handle,
        scene_x,
        scene_y,
        0.0,
        1.0,
        0,
        0
    ));
    assert!(event::__fui_on_gesture_event(
        child_handle,
        1,
        1,
        scene_x,
        scene_y,
        4.0,
        2.0,
        1.0,
        1
    ));
    assert!(event::__fui_on_long_press_event(
        child_handle,
        scene_x,
        scene_y,
        1,
        1,
        0,
        600
    ));

    assert_eq!(wheel_local.get(), (9.0, 13.0));
    assert_eq!(gesture_local.get(), (9.0, 13.0));
    assert_eq!(long_press_local.get(), (9.0, 13.0));

    let disabled_hits = Rc::new(Cell::new(0));
    let disabled_hits_clone = disabled_hits.clone();
    let disabled = flex_box();
    disabled
        .enabled(false)
        .width(40.0, Unit::Pixel)
        .height(20.0, Unit::Pixel)
        .on_wheel(move |_event| disabled_hits_clone.set(disabled_hits_clone.get() + 1));
    Application::mount(disabled.clone());
    let disabled_bounds =
        fui::bindings::ui::get_bounds(disabled.handle().raw()).expect("disabled bounds");
    assert!(!event::__fui_on_wheel_event(
        disabled.handle().raw(),
        disabled_bounds[0] + 4.0,
        disabled_bounds[1] + 4.0,
        0.0,
        1.0,
        0,
        0
    ));
    assert_eq!(disabled_hits.get(), 0);
}

#[test]
fn focused_key_event_routes_to_focused_node() {
    ffi::test::reset();
    let key_hits = Rc::new(Cell::new(0));
    let key_hits_clone = key_hits.clone();
    let focus_state = Rc::new(Cell::new(false));
    let focus_state_clone = focus_state.clone();
    let root = flex_box();
    root.key(7)
        .focusable(true, 0)
        .on_focus_changed(move |event: FocusChangedEventArgs| focus_state_clone.set(event.focused))
        .on_key_down(move |event| {
            if event.key == "Enter" {
                key_hits_clone.set(key_hits_clone.get() + 1);
                event.handled = true;
            }
        });
    Application::mount(root.clone());
    ffi::test::take_calls();
    let handle = root.handle().raw();
    event::__fui_on_focus_changed(handle, true);
    assert!(focus_state.get());
    assert!(event::__fui_on_key_event(1, b"Enter".as_ptr(), 5, 0));
    assert_eq!(key_hits.get(), 1);
}

#[test]
fn pointer_capture_calls_host_capture_api() {
    ffi::test::reset();
    let root = flex_box();
    root.on_pointer_down(move |event| event.capture_pointer());
    Application::mount(root.clone());
    ffi::test::take_calls();
    let handle = root.handle().raw();
    event::__fui_on_pointer_event_with_metadata(
        1, handle, 10.0, 10.0, 0, 1, 1, 0, 1, 0.0, 0.0, 0.0, 1,
    );
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(
        |call| matches!(call, Call::SetPointerCapture { handle: captured } if *captured == handle)
    ));
}

#[test]
fn gesture_event_bubbles_and_resolves_owner() {
    ffi::test::reset();
    let pan_hits = Rc::new(Cell::new(0));
    let pan_hits_clone = pan_hits.clone();
    let root = column();
    let child = text("child");
    child.key(1);
    root.on_pan_gesture(move |event| {
        pan_hits_clone.set(pan_hits_clone.get() + 1);
        event.handled = true;
    })
    .child(&child);
    Application::mount(root.clone());
    ffi::test::take_calls();
    let root_handle = root.handle().raw();
    let child_handle = child.handle().raw();
    assert_eq!(
        event::__fui_resolve_gesture_owner(child_handle),
        root_handle
    );
    assert_eq!(
        event::__fui_get_gesture_intent(root_handle),
        GestureIntent::Pan as u32
    );
    assert!(event::__fui_on_gesture_event(
        root_handle,
        1,
        1,
        24.0,
        24.0,
        8.0,
        2.0,
        1.0,
        1
    ));
    assert_eq!(pan_hits.get(), 1);
}

#[test]
fn disabled_nodes_do_not_resolve_as_gesture_or_long_press_owners() {
    ffi::test::reset();
    let root = column();
    let child = text("child");
    child.key(1);
    root.enabled(false)
        .on_pan_gesture(|event| event.handled = true)
        .on_long_press(|event| event.handled = true)
        .child(&child);
    Application::mount(root);
    let child_handle = child.handle().raw();
    assert_eq!(
        event::__fui_resolve_gesture_owner(child_handle),
        HandleValue::Invalid as u64
    );
    assert_eq!(
        event::__fui_resolve_long_press_owner(child_handle),
        HandleValue::Invalid as u64
    );
}

#[test]
fn keyboard_scroll_fallback_scrolls_focused_scroll_view_ancestor() {
    ffi::test::reset();
    let root = scroll_view();
    let content = text("Scrollable");
    content.focusable(true, 0);
    root.width(120.0, Unit::Pixel)
        .height(80.0, Unit::Pixel)
        .scroll_content_size(120.0, 400.0)
        .child(&content);
    Application::mount(root.clone());
    let viewport_handle = root.handle().raw();
    focus(&content);
    assert!(key_event(KeyEventType::Down, "PageDown", 0));
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetScrollOffset { handle, offset_x, offset_y }
            if *handle == viewport_handle && *offset_x == 0.0 && *offset_y > 0.0
    )));
}

#[test]
fn keyboard_scroll_pointer_up_selects_scroll_fallback_candidate_without_focus() {
    ffi::test::reset();
    let outer = column();
    let top = scroll_view();
    let bottom = scroll_view();
    let top_text = text("Top");
    let bottom_text = text("Bottom");
    top.width(120.0, Unit::Pixel)
        .height(80.0, Unit::Pixel)
        .scroll_content_size(120.0, 400.0)
        .child(&top_text);
    bottom
        .width(120.0, Unit::Pixel)
        .height(80.0, Unit::Pixel)
        .scroll_content_size(120.0, 400.0)
        .child(&bottom_text);
    outer.child(&top).child(&bottom);
    Application::mount(outer);

    let bottom_bounds =
        fui::bindings::ui::get_bounds(bottom.handle().raw()).expect("bottom bounds");
    pointer_event(
        PointerEventType::Up,
        bottom_text.handle().raw(),
        bottom_bounds[0] + 10.0,
        bottom_bounds[1] + 10.0,
        0,
        0,
        0,
    );

    assert!(key_event(KeyEventType::Down, "ArrowDown", 0));
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetScrollOffset { handle, offset_y, .. }
            if *handle == bottom.handle().raw() && *offset_y > 0.0
    )));
}

#[test]
fn button_click_key_and_multi_click_handlers_fire() {
    ffi::test::reset();
    let click_count = Rc::new(Cell::new(0));
    let double_count = Rc::new(Cell::new(0));
    let triple_count = Rc::new(Cell::new(0));
    let click_count_clone = click_count.clone();
    let double_count_clone = double_count.clone();
    let triple_count_clone = triple_count.clone();
    let button = button("Action");
    button
        .on_click(move |_event| click_count_clone.set(click_count_clone.get() + 1))
        .on_double_click(move |_event| double_count_clone.set(double_count_clone.get() + 1))
        .on_triple_click(move |_event| triple_count_clone.set(triple_count_clone.get() + 1));
    Application::mount(button);
    let calls = ffi::test::take_calls();
    let handle = handle_with_semantic_role(&calls, SemanticRole::Button);

    primary_click(handle, 1);
    primary_click(handle, 2);
    primary_click(handle, 3);
    primary_click(handle, 4);
    event::__fui_on_focus_changed(handle, true);
    assert!(event::__fui_on_key_event(1, b"Enter".as_ptr(), 5, 0));
    assert!(event::__fui_on_key_event(2, b"Enter".as_ptr(), 5, 0));

    assert_eq!(click_count.get(), 5);
    assert_eq!(double_count.get(), 1);
    assert_eq!(triple_count.get(), 1);
}

#[test]
fn button_text_updates_semantic_label_and_visible_text() {
    ffi::test::reset();
    let button = button("Before");
    Application::mount(button.clone());
    let _ = ffi::test::take_calls();

    button.text("After");
    let calls = ffi::test::take_calls();

    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetText { text, .. } if text == "After"
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticLabel { label, .. } if label == "After"
    )));
}

#[test]
fn checkbox_cycles_state_updates_semantics_and_announces() {
    ffi::test::reset();
    let last_state = Rc::new(Cell::new(CheckState::False));
    let last_state_clone = last_state.clone();
    let checkbox = checkbox("Agree");
    checkbox
        .tri_state(true)
        .on_changed(move |event| last_state_clone.set(event.state));
    Application::mount(checkbox);
    let calls = ffi::test::take_calls();
    let handle = handle_with_semantic_role(&calls, SemanticRole::Checkbox);

    primary_click(handle, 1);
    assert_eq!(last_state.get(), CheckState::True);
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(call, Call::SetSemanticChecked { handle: checked_handle, checked_state_enum } if *checked_handle == handle && *checked_state_enum == SemanticCheckedState::True as u32)));
    assert!(calls.iter().any(|call| matches!(call, Call::RequestSemanticAnnouncement { handle: announced } if *announced == handle)));

    primary_click(handle, 1);
    assert_eq!(last_state.get(), CheckState::Mixed);
    primary_click(handle, 1);
    assert_eq!(last_state.get(), CheckState::False);
}

#[test]
fn radio_button_and_switch_activate_from_pointer() {
    ffi::test::reset();
    let radio_state = Rc::new(Cell::new(false));
    let switch_state = Rc::new(Cell::new(false));
    let radio_state_clone = radio_state.clone();
    let switch_state_clone = switch_state.clone();
    let root = column();
    let radio = radio_button("Choice");
    let switch = switch("Enabled");
    radio.on_changed(move |event| radio_state_clone.set(event.checked));
    switch.on_changed(move |event| switch_state_clone.set(event.checked));
    root.child(&radio).child(&switch);
    Application::mount(root);
    let calls = ffi::test::take_calls();
    let radio_handle = handle_with_semantic_role(&calls, SemanticRole::Radio);
    let switch_handle = handle_with_semantic_role(&calls, SemanticRole::Switch);

    primary_click(radio_handle, 1);
    primary_click(switch_handle, 1);

    assert!(radio_state.get());
    assert!(switch_state.get());
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(call, Call::SetSemanticChecked { handle, checked_state_enum } if *handle == radio_handle && *checked_state_enum == SemanticCheckedState::True as u32)));
    assert!(calls.iter().any(|call| matches!(call, Call::SetSemanticChecked { handle, checked_state_enum } if *handle == switch_handle && *checked_state_enum == SemanticCheckedState::True as u32)));
}

#[test]
fn progress_bar_sets_fill_geometry_and_value_range() {
    ffi::test::reset();
    let progress = progress_bar();
    progress
        .min(0.0)
        .max(200.0)
        .value(50.0)
        .length(300.0)
        .thickness(12.0);
    Application::mount(progress);
    let calls = ffi::test::take_calls();

    assert!(calls.iter().any(|call| matches!(call, Call::SetSemanticValueRange { has_value_range, value_now, value_min, value_max, .. } if *has_value_range && *value_now == 50.0 && *value_min == 0.0 && *value_max == 200.0)));
    assert!(calls.iter().any(|call| matches!(call, Call::SetSemanticLabel { label, .. } if label == "Progress bar, value 50, range 0 to 200")));
    assert!(calls.iter().any(|call| matches!(call, Call::SetWidth { value, unit_enum, .. } if *value == 75.0 && *unit_enum == Unit::Pixel as u32)));
    assert!(calls.iter().any(|call| matches!(call, Call::SetHeight { value, unit_enum, .. } if *value == 12.0 && *unit_enum == Unit::Pixel as u32)));
}

#[test]
fn slider_keyboard_changes_value_and_semantic_range() {
    ffi::test::reset();
    let last_value = Rc::new(Cell::new(25.0));
    let last_value_clone = last_value.clone();
    let slider = slider();
    slider
        .min(0.0)
        .max(100.0)
        .step(5.0)
        .value(25.0)
        .on_changed(move |event| last_value_clone.set(event.value));
    Application::mount(slider);
    let calls = ffi::test::take_calls();
    let handle = handle_with_semantic_role(&calls, SemanticRole::Slider);
    event::__fui_on_focus_changed(handle, true);

    assert!(event::__fui_on_key_event(1, b"ArrowRight".as_ptr(), 10, 0));
    assert_eq!(last_value.get(), 30.0);
    let calls = ffi::test::take_calls();
    assert_eq!(
        calls
            .iter()
            .filter(|call| matches!(call, Call::RequestSemanticAnnouncement { handle: announce_handle } if *announce_handle == handle))
            .count(),
        1
    );
    event::__fui_on_pointer_event_with_metadata(
        PointerEventType::Enter as u32,
        handle,
        10.0,
        10.0,
        0,
        1,
        PointerType::Mouse as u32,
        0,
        0,
        0.0,
        0.0,
        0.0,
        0,
    );
    let calls = ffi::test::take_calls();
    assert!(!calls
        .iter()
        .any(|call| matches!(call, Call::RequestSemanticAnnouncement { .. })));
    assert!(event::__fui_on_key_event(1, b"Home".as_ptr(), 4, 0));
    assert_eq!(last_value.get(), 0.0);
    assert!(event::__fui_on_key_event(1, b"End".as_ptr(), 3, 0));
    assert_eq!(last_value.get(), 100.0);
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(call, Call::SetSemanticValueRange { handle: range_handle, value_now, value_min, value_max, .. } if *range_handle == handle && *value_now == 100.0 && *value_min == 0.0 && *value_max == 100.0)));
}

#[test]
fn slider_pointer_drag_horizontal_updates_value_and_pointer_capture() {
    ffi::test::reset();
    let last_value = Rc::new(Cell::new(0.0));
    let last_value_clone = last_value.clone();
    let slider = slider();
    slider
        .min(0.0)
        .max(100.0)
        .step(5.0)
        .length(180.0)
        .value(0.0)
        .on_changed(move |event| last_value_clone.set(event.value));
    Application::mount(slider);
    let calls = ffi::test::take_calls();
    let handle = handle_with_semantic_role(&calls, SemanticRole::Slider);

    event::__fui_on_pointer_event_with_metadata(
        PointerEventType::Down as u32,
        handle,
        170.0,
        15.0,
        0,
        1,
        PointerType::Mouse as u32,
        0,
        1,
        0.0,
        0.0,
        0.0,
        1,
    );
    event::__fui_on_pointer_event_with_metadata(
        PointerEventType::Up as u32,
        handle,
        170.0,
        15.0,
        0,
        1,
        PointerType::Mouse as u32,
        0,
        0,
        0.0,
        0.0,
        0.0,
        0,
    );

    assert_eq!(last_value.get(), 95.0);
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(
        |call| matches!(call, Call::SetPointerCapture { handle: capture } if *capture == handle)
    ));
    assert!(calls
        .iter()
        .any(|call| matches!(call, Call::ReleasePointerCapture)));
}

#[test]
fn slider_pointer_drag_vertical_updates_value() {
    ffi::test::reset();
    let last_value = Rc::new(Cell::new(0.0));
    let last_value_clone = last_value.clone();
    let slider = slider();
    slider
        .orientation(Orientation::Vertical)
        .min(0.0)
        .max(100.0)
        .step(5.0)
        .length(180.0)
        .value(0.0)
        .on_changed(move |event| last_value_clone.set(event.value));
    Application::mount(slider);
    let calls = ffi::test::take_calls();
    let handle = handle_with_semantic_role(&calls, SemanticRole::Slider);

    event::__fui_on_pointer_event_with_metadata(
        PointerEventType::Down as u32,
        handle,
        15.0,
        15.0,
        0,
        1,
        PointerType::Mouse as u32,
        0,
        1,
        0.0,
        0.0,
        0.0,
        1,
    );

    assert_eq!(last_value.get(), 100.0);
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticOrientation { handle: orientation_handle, orientation_enum }
            if *orientation_handle == handle && *orientation_enum == Orientation::Vertical as u32
    )));
}

#[test]
fn disabled_slider_ignores_pointer_and_keyboard() {
    ffi::test::reset();
    let last_value = Rc::new(Cell::new(25.0));
    let last_value_clone = last_value.clone();
    let slider = slider();
    slider
        .min(0.0)
        .max(100.0)
        .step(5.0)
        .value(25.0)
        .enabled(false)
        .on_changed(move |event| last_value_clone.set(event.value));
    Application::mount(slider);
    let calls = ffi::test::take_calls();
    let handle = handle_with_semantic_role(&calls, SemanticRole::Slider);
    event::__fui_on_focus_changed(handle, true);

    event::__fui_on_pointer_event_with_metadata(
        PointerEventType::Down as u32,
        handle,
        170.0,
        15.0,
        0,
        1,
        PointerType::Mouse as u32,
        0,
        1,
        0.0,
        0.0,
        0.0,
        1,
    );
    assert!(!event::__fui_on_key_event(1, b"ArrowRight".as_ptr(), 10, 0));

    assert_eq!(last_value.get(), 25.0);
    let calls = ffi::test::take_calls();
    assert!(!calls
        .iter()
        .any(|call| matches!(call, Call::SetPointerCapture { .. })));
}

#[test]
fn disabling_slider_during_drag_releases_pointer_capture() {
    ffi::test::reset();
    let slider = slider();
    slider.length(180.0);
    Application::mount(slider.clone());
    let calls = ffi::test::take_calls();
    let handle = handle_with_semantic_role(&calls, SemanticRole::Slider);

    event::__fui_on_pointer_event_with_metadata(
        PointerEventType::Down as u32,
        handle,
        120.0,
        15.0,
        0,
        1,
        PointerType::Mouse as u32,
        0,
        1,
        0.0,
        0.0,
        0.0,
        1,
    );
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(
        |call| matches!(call, Call::SetPointerCapture { handle: capture } if *capture == handle)
    ));

    slider.enabled(false);
    let calls = ffi::test::take_calls();
    assert!(calls
        .iter()
        .any(|call| matches!(call, Call::ReleasePointerCapture)));
}

#[test]
fn long_press_event_bubbles_and_uses_custom_config() {
    ffi::test::reset();
    let long_press_hits = Rc::new(Cell::new(0));
    let long_press_hits_clone = long_press_hits.clone();
    let root = column();
    let child = text("child");
    child.key(1);
    root.long_press_options(700, 24.0)
        .on_long_press(move |event| {
            long_press_hits_clone.set(long_press_hits_clone.get() + 1);
            event.handled = true;
        })
        .child(&child);
    Application::mount(root.clone());
    ffi::test::take_calls();
    let root_handle = root.handle().raw();
    let child_handle = child.handle().raw();
    assert_eq!(
        event::__fui_resolve_long_press_owner(child_handle),
        root_handle
    );
    assert_eq!(
        event::__fui_get_long_press_minimum_duration_ms(root_handle),
        700
    );
    assert_eq!(
        event::__fui_get_long_press_movement_tolerance(root_handle),
        24.0
    );
    assert!(event::__fui_on_long_press_event(
        root_handle,
        16.0,
        18.0,
        7,
        2,
        0,
        740
    ));
    assert_eq!(long_press_hits.get(), 1);
}

#[test]
fn context_menu_show_attaches_overlay_and_hide_detaches_without_delete() {
    ffi::test::reset();
    let root = portal();
    let menu = context_menu(vec![MenuItem::new("Primary", ContextMenuAction::OpenLink)]);
    root.child(&menu);
    Application::mount(root.clone());
    ffi::test::take_calls();

    menu.show(24.0, 32.0);
    let calls = ffi::test::take_calls();
    let panel_handle = calls
        .iter()
        .find_map(|call| match call {
            Call::PushSemanticScope { handle, .. } => Some(*handle),
            _ => None,
        })
        .expect("menu semantic scope");
    let overlay_handle = parent_for_child(&calls, panel_handle);
    let overlay_parent_handle = parent_for_child(&calls, overlay_handle);
    assert!(menu.is_open());
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::NodeAddChild { child, .. } if *child == overlay_handle
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::PushSemanticScope { handle, token } if *handle == panel_handle && *token == 1
    )));

    menu.hide();
    let calls = ffi::test::take_calls();
    assert!(!menu.is_open());
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::NodeRemoveChild { parent, child } if *parent == overlay_parent_handle && *child == overlay_handle
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::RemoveSemanticScope { token } if *token == 1
    )));
    assert!(!calls.iter().any(|call| matches!(
        call,
        Call::DeleteNode { handle } if *handle == overlay_handle || *handle == panel_handle
    )));
}

#[test]
fn built_in_context_menu_matches_fui_as_text_and_background_items() {
    ffi::test::reset();
    let target = text("Selectable text");
    Application::mount(target.clone());
    ffi::test::take_calls();

    fui::bridge_callbacks::__fui_on_context_menu(target.handle().raw(), 12.0, 18.0);
    let calls = ffi::test::take_calls();
    let copy_handle = handle_with_semantic_label(&calls, "Copy");
    assert!(calls
        .iter()
        .any(|call| matches!(call, Call::SetSemanticLabel { label, .. } if label == "Copy")));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticDisabled {
            handle,
            has_disabled: true,
            disabled: true,
        } if *handle == copy_handle
    )));
    assert!(calls
        .iter()
        .any(|call| matches!(call, Call::SetSemanticLabel { label, .. } if label == "Select All")));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticLabel { label, .. } if label == "Reload Page"
    )));
    assert!(
        !calls
            .iter()
            .any(|call| matches!(call, Call::SetTextVerticalAlign { .. })),
        "FUI-AS ContextMenu entries leave TextCore vertical alignment at its default; forcing center clips tight menu text."
    );

    fui::bridge_callbacks::__fui_hide_active_context_menu();
    ffi::test::take_calls();
    fui::bridge_callbacks::__fui_on_context_menu(HandleValue::Invalid as u64, 20.0, 24.0);
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticLabel { label, .. } if label == "Reload Page"
    )));
}

#[test]
fn long_press_owner_resolves_fui_as_builtin_targets() {
    ffi::test::reset();
    let root = column();
    let selectable = text("Selectable text");
    let link = nav_link("https://example.test");
    let image_node = image(1);
    let svg_node = svg(1);
    root.child(&selectable)
        .child(&link)
        .child(&image_node)
        .child(&svg_node);
    Application::mount(root.clone());

    assert_eq!(
        event::__fui_resolve_long_press_owner(selectable.handle().raw()),
        selectable.handle().raw()
    );
    assert_eq!(
        event::__fui_resolve_long_press_owner(link.handle().raw()),
        link.handle().raw()
    );
    assert_eq!(
        event::__fui_resolve_long_press_owner(image_node.handle().raw()),
        image_node.handle().raw()
    );
    assert_eq!(
        event::__fui_resolve_long_press_owner(svg_node.handle().raw()),
        svg_node.handle().raw()
    );
    let blank = column();
    Application::mount(blank.clone());
    assert_eq!(
        event::__fui_resolve_long_press_owner(blank.handle().raw()),
        HandleValue::Invalid as u64
    );
}

#[test]
fn long_press_on_selectable_text_selects_word_and_suppresses_context_menu() {
    ffi::test::reset();
    let target = text("Selectable text");
    Application::mount(target.clone());
    ffi::test::take_calls();

    assert!(event::__fui_on_long_press_event(
        target.handle().raw(),
        12.0,
        18.0,
        1,
        2,
        0,
        600
    ));
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SelectWordAt {
            handle,
            x,
            y,
        } if *handle == target.handle().raw() && *x == 12.0 && *y == 18.0
    )));
    assert!(!calls
        .iter()
        .any(|call| matches!(call, Call::SetSemanticLabel { label, .. } if label == "Copy")));
    assert!(!calls
        .iter()
        .any(|call| matches!(call, Call::SetSemanticLabel { label, .. } if label == "Select All")));
}

#[test]
fn mobile_selection_toolbar_and_teardrops_use_fui_as_geometry() {
    ffi::test::reset();
    ffi::test::set_coarse_pointer(true);
    ffi::test::set_viewport(640.0, 480.0);
    ffi::test::set_text_range_rects(&[(100.0, 80.0, 60.0, 20.0)]);
    let target = text("Selectable text");
    Application::mount(target.clone());
    ffi::test::take_calls();

    event::__fui_on_selection_changed(target.handle().raw(), 0, 10);
    let calls = ffi::test::take_calls();

    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetWidth {
            value,
            unit_enum,
            ..
        } if *value == 201.0 && *unit_enum == Unit::Pixel as u32
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetFont {
            font_id,
            size,
            ..
        } if *font_id == 1 && *size == 13.0
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetPosition {
            left,
            top,
            right,
            bottom,
            ..
        } if *left == 8.0 && *top == 34.0 && right.is_nan() && bottom.is_nan()
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetPosition {
            left,
            top,
            right,
            bottom,
            ..
        } if *left == 28.0 && *top == 75.0 && right.is_nan() && bottom.is_nan()
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetPosition {
            left,
            top,
            right,
            bottom,
            ..
        } if *left == 142.0 && *top == 75.0 && right.is_nan() && bottom.is_nan()
    )));
}

#[test]
fn mobile_selection_editable_toolbar_uses_fui_as_overflow_items() {
    ffi::test::reset();
    ffi::test::set_coarse_pointer(true);
    ffi::test::set_viewport(640.0, 480.0);
    ffi::test::set_text_range_rects(&[(100.0, 80.0, 60.0, 20.0)]);
    let target = text_input();
    target.text("Editable selection");
    Application::mount(target.clone());
    let mount_calls = ffi::test::take_calls();
    let editor_handle = handle_with_semantic_role(&mount_calls, SemanticRole::Textbox);

    event::__fui_on_selection_changed(editor_handle, 0, 8);
    let calls = ffi::test::take_calls();

    assert!(calls
        .iter()
        .any(|call| matches!(call, Call::SetSemanticLabel { label, .. } if label == "Cut")));
    assert!(calls
        .iter()
        .any(|call| matches!(call, Call::SetSemanticLabel { label, .. } if label == "Copy")));
    assert!(calls
        .iter()
        .any(|call| matches!(call, Call::SetSemanticLabel { label, .. } if label == "Paste")));
    assert!(calls
        .iter()
        .any(|call| matches!(call, Call::SetSemanticLabel { label, .. } if label == "Select all")));
    assert!(calls
        .iter()
        .any(|call| matches!(call, Call::SetSemanticLabel { label, .. } if label == "More")));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetWidth {
            value,
            unit_enum,
            ..
        } if *value == 271.0 && *unit_enum == Unit::Pixel as u32
    )));
}

#[test]
fn mobile_selection_toolbar_can_reappear_after_outside_dismiss() {
    ffi::test::reset();
    ffi::test::set_coarse_pointer(true);
    ffi::test::set_viewport(640.0, 480.0);
    ffi::test::set_text_range_rects(&[(100.0, 80.0, 60.0, 20.0)]);
    let target = text("Selectable text");
    Application::mount(target.clone());
    ffi::test::take_calls();

    event::__fui_on_selection_changed(target.handle().raw(), 0, 10);
    let calls = ffi::test::take_calls();
    let toolbar_handle = calls
        .iter()
        .find_map(|call| match call {
            Call::SetWidth {
                handle,
                value,
                unit_enum,
            } if *value == 201.0 && *unit_enum == Unit::Pixel as u32 => Some(*handle),
            _ => None,
        })
        .expect("horizontal toolbar panel");
    ffi::test::take_calls();

    pointer_event(
        PointerEventType::Down,
        HandleValue::Invalid as u64,
        320.0,
        240.0,
        0,
        1,
        1,
    );
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetVisibility {
            handle,
            visibility_enum,
        } if *handle == toolbar_handle && *visibility_enum == Visibility::Collapsed as u32
    )));

    event::__fui_on_selection_changed(target.handle().raw(), 0, 10);
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetVisibility {
            visibility_enum,
            ..
        } if *visibility_enum == Visibility::Normal as u32
    )));
}

#[test]
fn mobile_selection_teardrop_touch_down_starts_endpoint_drag() {
    ffi::test::reset();
    ffi::test::set_coarse_pointer(true);
    ffi::test::set_text_range_rects(&[(100.0, 80.0, 60.0, 20.0)]);
    let target = text("Selectable text");
    Application::mount(target.clone());
    ffi::test::take_calls();

    event::__fui_on_selection_changed(target.handle().raw(), 0, 10);
    let calls = ffi::test::take_calls();
    let start_handle = calls
        .iter()
        .find_map(|call| match call {
            Call::SetPosition {
                handle,
                left,
                top,
                right,
                bottom,
            } if *left == 28.0 && *top == 75.0 && right.is_nan() && bottom.is_nan() => {
                Some(*handle)
            }
            _ => None,
        })
        .expect("start selection handle");
    ffi::test::take_calls();

    event::__fui_on_pointer_event_with_metadata(
        PointerEventType::Down as u32,
        start_handle,
        100.0,
        120.0,
        0,
        7,
        PointerType::Touch as u32,
        0,
        1,
        1.0,
        1.0,
        1.0,
        1,
    );
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::BeginSelectionEndpointDrag { handle, endpoint }
            if *handle == target.handle().raw() && *endpoint == 0
    )));
    assert!(calls
        .iter()
        .any(|call| matches!(call, Call::SetPointerCapture { handle } if *handle == start_handle)));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetInteractive {
            handle,
            interactive: false,
        } if *handle == start_handle
    )));
}

#[test]
fn mobile_selection_toolbar_does_not_dismiss_when_touching_selected_text() {
    ffi::test::reset();
    ffi::test::set_coarse_pointer(true);
    ffi::test::set_viewport(640.0, 480.0);
    ffi::test::set_text_range_rects(&[(100.0, 80.0, 60.0, 20.0)]);
    let target = text("Selectable text");
    Application::mount(target.clone());
    ffi::test::take_calls();

    event::__fui_on_selection_changed(target.handle().raw(), 0, 10);
    let calls = ffi::test::take_calls();
    let toolbar_handle = calls
        .iter()
        .find_map(|call| match call {
            Call::SetWidth {
                handle,
                value,
                unit_enum,
            } if *value == 201.0 && *unit_enum == Unit::Pixel as u32 => Some(*handle),
            _ => None,
        })
        .expect("horizontal toolbar panel");
    ffi::test::set_is_point_in_selection(true);
    ffi::test::take_calls();

    pointer_event(
        PointerEventType::Down,
        target.handle().raw(),
        320.0,
        240.0,
        0,
        1,
        1,
    );
    let calls = ffi::test::take_calls();
    assert!(!calls.iter().any(|call| matches!(
        call,
        Call::SetVisibility {
            handle,
            visibility_enum,
        } if *handle == toolbar_handle && *visibility_enum == Visibility::Collapsed as u32
    )));
}

#[test]
fn mobile_selection_clears_teardrops_when_dragged_endpoints_meet() {
    ffi::test::reset();
    ffi::test::set_coarse_pointer(true);
    ffi::test::set_text_range_rects(&[(100.0, 80.0, 60.0, 20.0)]);
    let target = text("Selectable text");
    Application::mount(target.clone());
    ffi::test::take_calls();

    event::__fui_on_selection_changed(target.handle().raw(), 0, 10);
    let calls = ffi::test::take_calls();
    let start_handle = calls
        .iter()
        .find_map(|call| match call {
            Call::SetPosition {
                handle,
                left,
                top,
                right,
                bottom,
            } if *left == 28.0 && *top == 75.0 && right.is_nan() && bottom.is_nan() => {
                Some(*handle)
            }
            _ => None,
        })
        .expect("start selection handle");
    ffi::test::take_calls();

    event::__fui_on_pointer_event_with_metadata(
        PointerEventType::Down as u32,
        start_handle,
        100.0,
        120.0,
        0,
        7,
        PointerType::Touch as u32,
        0,
        1,
        1.0,
        1.0,
        1.0,
        1,
    );
    ffi::test::take_calls();

    event::__fui_on_selection_changed(target.handle().raw(), 6, 6);
    let calls = ffi::test::take_calls();
    assert!(!calls
        .iter()
        .any(|call| matches!(call, Call::ReleasePointerCapture)));
    let move_handled = event::__fui_on_pointer_event_with_metadata(
        PointerEventType::Move as u32,
        HandleValue::Invalid as u64,
        108.0,
        100.0,
        0,
        7,
        PointerType::Touch as u32,
        0,
        1,
        1.0,
        1.0,
        1.0,
        1,
    );
    assert!(move_handled);
    ffi::test::take_calls();

    event::__fui_on_pointer_event_with_metadata(
        PointerEventType::Up as u32,
        HandleValue::Invalid as u64,
        108.0,
        100.0,
        0,
        7,
        PointerType::Touch as u32,
        0,
        0,
        1.0,
        1.0,
        1.0,
        1,
    );
    let calls = ffi::test::take_calls();
    assert!(calls
        .iter()
        .any(|call| matches!(call, Call::ReleasePointerCapture)));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetInteractive {
            handle,
            interactive: true,
        } if *handle == start_handle
    )));

    let move_after_up_handled = event::__fui_on_pointer_event_with_metadata(
        PointerEventType::Move as u32,
        HandleValue::Invalid as u64,
        108.0,
        100.0,
        0,
        7,
        PointerType::Touch as u32,
        0,
        1,
        1.0,
        1.0,
        1.0,
        1,
    );
    assert!(!move_after_up_handled);
}

#[test]
fn mobile_selection_drag_uses_captured_visual_side_when_selection_is_reversed() {
    ffi::test::reset();
    ffi::test::set_coarse_pointer(true);
    ffi::test::set_text_range_rects(&[(100.0, 80.0, 60.0, 20.0)]);
    let target = text("Selectable text");
    Application::mount(target.clone());
    ffi::test::take_calls();

    event::__fui_on_selection_changed(target.handle().raw(), 10, 0);
    let calls = ffi::test::take_calls();
    let start_handle = calls
        .iter()
        .find_map(|call| match call {
            Call::SetPosition {
                handle,
                left,
                top,
                right,
                bottom,
            } if *left == 28.0 && *top == 75.0 && right.is_nan() && bottom.is_nan() => {
                Some(*handle)
            }
            _ => None,
        })
        .expect("start selection handle");
    let end_handle = calls
        .iter()
        .find_map(|call| match call {
            Call::SetPosition {
                handle,
                left,
                top,
                right,
                bottom,
            } if *left == 142.0 && *top == 75.0 && right.is_nan() && bottom.is_nan() => {
                Some(*handle)
            }
            _ => None,
        })
        .expect("end selection handle");
    ffi::test::take_calls();

    event::__fui_on_pointer_event_with_metadata(
        PointerEventType::Down as u32,
        start_handle,
        100.0,
        100.0,
        0,
        7,
        PointerType::Touch as u32,
        0,
        1,
        1.0,
        1.0,
        1.0,
        1,
    );
    ffi::test::take_calls();

    event::__fui_on_selection_changed(target.handle().raw(), 10, 0);
    let calls = ffi::test::take_calls();
    assert!(
        calls.iter().any(|call| matches!(
            call,
            Call::SetPosition {
                handle,
                left,
                top,
                right,
                bottom,
            } if *handle == start_handle
                && *left == 28.0
                && *top == 75.0
                && right.is_nan()
                && bottom.is_nan()
        )),
        "expected crossed start handle at x=172; calls: {calls:?}"
    );
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetPosition {
            handle,
            left,
            top,
            right,
            bottom,
        } if *handle == end_handle
            && *left == 142.0
            && *top == 75.0
            && right.is_nan()
            && bottom.is_nan()
    )));
}

#[test]
fn mobile_selection_drag_start_keeps_single_text_handle_positions_before_crossover() {
    ffi::test::reset();
    ffi::test::set_coarse_pointer(true);
    ffi::test::set_text_range_rects(&[(100.0, 80.0, 60.0, 20.0)]);
    let target = text("Selectable text");
    Application::mount(target.clone());
    ffi::test::take_calls();

    event::__fui_on_selection_changed(target.handle().raw(), 0, 10);
    let calls = ffi::test::take_calls();
    let start_handle = calls
        .iter()
        .find_map(|call| match call {
            Call::SetPosition {
                handle,
                left,
                top,
                right,
                bottom,
            } if *left == 28.0 && *top == 75.0 && right.is_nan() && bottom.is_nan() => {
                Some(*handle)
            }
            _ => None,
        })
        .expect("start selection handle");
    let end_handle = calls
        .iter()
        .find_map(|call| match call {
            Call::SetPosition {
                handle,
                left,
                top,
                right,
                bottom,
            } if *left == 142.0 && *top == 75.0 && right.is_nan() && bottom.is_nan() => {
                Some(*handle)
            }
            _ => None,
        })
        .expect("end selection handle");
    ffi::test::take_calls();

    event::__fui_on_pointer_event_with_metadata(
        PointerEventType::Down as u32,
        start_handle,
        100.0,
        120.0,
        0,
        7,
        PointerType::Touch as u32,
        0,
        1,
        1.0,
        1.0,
        1.0,
        1,
    );
    ffi::test::take_calls();

    event::__fui_on_selection_changed(target.handle().raw(), 2, 10);
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetPosition {
            handle,
            left,
            top,
            right,
            bottom,
        } if *handle == start_handle
            && *left == 28.0
            && *top == 75.0
            && right.is_nan()
            && bottom.is_nan()
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetPosition {
            handle,
            left,
            top,
            right,
            bottom,
        } if *handle == end_handle
            && *left == 142.0
            && *top == 75.0
            && right.is_nan()
            && bottom.is_nan()
    )));
}

#[test]
fn mobile_selection_toolbar_reappears_after_teardrop_drag_on_single_text() {
    ffi::test::reset();
    ffi::test::set_coarse_pointer(true);
    ffi::test::set_text_range_rects(&[(100.0, 80.0, 60.0, 20.0)]);
    let target = text("Selectable text");
    Application::mount(target.clone());
    ffi::test::take_calls();

    event::__fui_on_selection_changed(target.handle().raw(), 0, 10);
    let calls = ffi::test::take_calls();
    let toolbar_handle = calls
        .iter()
        .find_map(|call| match call {
            Call::SetWidth {
                handle,
                value,
                unit_enum,
            } if *value == 201.0 && *unit_enum == Unit::Pixel as u32 => Some(*handle),
            _ => None,
        })
        .expect("horizontal toolbar panel");
    let start_handle = calls
        .iter()
        .find_map(|call| match call {
            Call::SetPosition {
                handle,
                left,
                top,
                right,
                bottom,
            } if *left == 28.0 && *top == 75.0 && right.is_nan() && bottom.is_nan() => {
                Some(*handle)
            }
            _ => None,
        })
        .expect("start selection handle");
    ffi::test::take_calls();

    event::__fui_on_pointer_event_with_metadata(
        PointerEventType::Down as u32,
        start_handle,
        100.0,
        120.0,
        0,
        7,
        PointerType::Touch as u32,
        0,
        1,
        1.0,
        1.0,
        1.0,
        1,
    );
    ffi::test::take_calls();

    event::__fui_on_selection_changed(target.handle().raw(), 2, 10);
    ffi::test::take_calls();

    event::__fui_on_pointer_event_with_metadata(
        PointerEventType::Up as u32,
        HandleValue::Invalid as u64,
        108.0,
        100.0,
        0,
        7,
        PointerType::Touch as u32,
        0,
        0,
        1.0,
        1.0,
        1.0,
        1,
    );
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetVisibility {
            handle,
            visibility_enum,
        } if *handle == toolbar_handle && *visibility_enum == Visibility::Normal as u32
    )));
}

#[test]
fn mobile_selection_toolbar_item_activates_on_pointer_up() {
    ffi::test::reset();
    ffi::test::set_coarse_pointer(true);
    ffi::test::set_text_range_rects(&[(100.0, 80.0, 60.0, 20.0)]);
    let target = text("Selectable text");
    Application::mount(target.clone());
    ffi::test::take_calls();

    event::__fui_on_selection_changed(target.handle().raw(), 0, 10);
    let calls = ffi::test::take_calls();
    let copy_handle = handle_with_semantic_label(&calls, "Copy");
    ffi::test::take_calls();

    event::__fui_on_pointer_event_with_metadata(
        PointerEventType::Up as u32,
        copy_handle,
        20.0,
        40.0,
        0,
        8,
        PointerType::Touch as u32,
        0,
        0,
        1.0,
        1.0,
        1.0,
        1,
    );
    let calls = ffi::test::take_calls();
    assert!(calls
        .iter()
        .any(|call| matches!(call, Call::ClearCurrentSelection)));
}

#[test]
fn node_context_menu_handler_and_disable_follow_fui_as_ancestor_routing() {
    ffi::test::reset();
    let invoked = Rc::new(Cell::new(0));
    let invoked_clone = invoked.clone();
    let root = column();
    let child = text("child");
    root.on_context_menu(move |event| {
        assert_eq!(event.x, 32.0);
        assert_eq!(event.y, 48.0);
        assert_ne!(event.target.raw(), HandleValue::Invalid as u64);
        invoked_clone.set(invoked_clone.get() + 1);
    })
    .child(&child);
    Application::mount(root.clone());
    ffi::test::take_calls();

    fui::bridge_callbacks::__fui_on_context_menu(child.handle().raw(), 32.0, 48.0);
    assert_eq!(invoked.get(), 1);

    root.disable_context_menu(true);
    assert!(!fui::bridge_callbacks::__fui_can_show_context_menu(
        child.handle().raw()
    ));
    fui::bridge_callbacks::__fui_on_context_menu(child.handle().raw(), 32.0, 48.0);
    assert_eq!(invoked.get(), 1);
}

#[test]
fn context_menu_disabled_item_and_separator_do_not_invoke() {
    ffi::test::reset();
    let root = portal();
    let count = Rc::new(Cell::new(0));
    let count_for_enabled = count.clone();
    let menu = context_menu(vec![
        MenuItem::new("Enabled", ContextMenuAction::OpenLink).on_invoke(move || {
            count_for_enabled.set(count_for_enabled.get() + 1);
        }),
        MenuItem::new("Disabled", ContextMenuAction::OpenLink)
            .disabled(true)
            .on_invoke({
                let count = count.clone();
                move || count.set(count.get() + 10)
            }),
    ]);
    root.child(&menu);
    Application::mount(root);
    ffi::test::take_calls();

    menu.show(24.0, 32.0);
    let calls = ffi::test::take_calls();
    let disabled_handle = handle_with_semantic_label(&calls, "Disabled");

    let disabled_bounds = fui::bindings::ui::get_bounds(disabled_handle).expect("disabled bounds");
    pointer_event(
        PointerEventType::Enter,
        disabled_handle,
        disabled_bounds[0] + 4.0,
        disabled_bounds[1] + 4.0,
        0,
        0,
        0,
    );
    pointer_event(
        PointerEventType::Down,
        disabled_handle,
        disabled_bounds[0] + 4.0,
        disabled_bounds[1] + 4.0,
        0,
        1,
        1,
    );
    pointer_event(
        PointerEventType::Up,
        disabled_handle,
        disabled_bounds[0] + 4.0,
        disabled_bounds[1] + 4.0,
        0,
        0,
        0,
    );
    assert_eq!(count.get(), 0);

    ffi::test::reset();
    let root = portal();
    let separator_count = Rc::new(Cell::new(0));
    let separator_count_for_enabled = separator_count.clone();
    let menu = context_menu(vec![
        MenuItem::new("Enabled", ContextMenuAction::OpenLink).on_invoke({
            move || separator_count_for_enabled.set(separator_count_for_enabled.get() + 1)
        }),
        MenuItem::separator(),
        MenuItem::new("Other", ContextMenuAction::OpenLink),
    ]);
    root.child(&menu);
    Application::mount(root);
    ffi::test::take_calls();

    menu.show(24.0, 32.0);
    let calls = ffi::test::take_calls();
    let menu_handle = menu.handle().raw();
    let overlay_handle = calls
        .iter()
        .find_map(|call| match call {
            Call::NodeAddChild { parent, child } if *parent == menu_handle => Some(*child),
            _ => None,
        })
        .expect("context menu overlay handle");
    let enabled_handle = handle_with_semantic_label(&calls, "Enabled");
    let other_handle = handle_with_semantic_label(&calls, "Other");
    let button_role_count = calls
        .iter()
        .filter(|call| {
            matches!(
                call,
                Call::SetSemanticRole { role_enum, .. } if *role_enum == SemanticRole::Button as u32
            )
        })
        .count();
    assert_eq!(button_role_count, 2);

    let enabled_bounds = fui::bindings::ui::get_bounds(enabled_handle).expect("enabled bounds");
    let other_bounds = fui::bindings::ui::get_bounds(other_handle).expect("other bounds");
    let enabled_bottom = enabled_bounds[1] + enabled_bounds[3];
    if other_bounds[1] > enabled_bottom {
        let separator_y = (enabled_bottom + other_bounds[1]) / 2.0;
        primary_click_at(overlay_handle, enabled_bounds[0] + 4.0, separator_y, 1);
    }
    assert_eq!(separator_count.get(), 0);
}

#[test]
fn context_menu_hover_pressed_pointer_up_invokes_once_and_opening_suppression_blocks_first_up() {
    ffi::test::reset();
    let root = portal();
    let count = Rc::new(Cell::new(0));
    let count_for_click = count.clone();
    let menu = context_menu(vec![MenuItem::new("Primary", ContextMenuAction::OpenLink)
        .on_invoke(move || {
            count_for_click.set(count_for_click.get() + 1);
        })]);
    root.child(&menu);
    Application::mount(root);
    ffi::test::take_calls();

    menu.show_from_context_pointer(24.0, 32.0);
    let calls = ffi::test::take_calls();
    let item_handle = handle_with_semantic_label(&calls, "Primary");
    let bounds = fui::bindings::ui::get_bounds(item_handle).expect("item bounds");

    pointer_event(
        PointerEventType::Enter,
        item_handle,
        bounds[0] + 4.0,
        bounds[1] + 4.0,
        0,
        0,
        0,
    );
    pointer_event(
        PointerEventType::Up,
        item_handle,
        bounds[0] + 4.0,
        bounds[1] + 4.0,
        2,
        0,
        0,
    );
    assert_eq!(count.get(), 0);
    assert!(menu.is_open());

    pointer_event(
        PointerEventType::Down,
        item_handle,
        bounds[0] + 4.0,
        bounds[1] + 4.0,
        0,
        1,
        1,
    );
    pointer_event(
        PointerEventType::Up,
        item_handle,
        bounds[0] + 4.0,
        bounds[1] + 4.0,
        0,
        0,
        0,
    );
    assert_eq!(count.get(), 1);
    assert!(!menu.is_open());
}

#[test]
fn context_menu_stale_opening_suppression_does_not_block_later_primary_item_click() {
    ffi::test::reset();
    let root = portal();
    let count = Rc::new(Cell::new(0));
    let count_for_click = count.clone();
    let menu = context_menu(vec![MenuItem::new("Primary", ContextMenuAction::OpenLink)
        .on_invoke(move || {
            count_for_click.set(count_for_click.get() + 1);
        })]);
    root.child(&menu);
    Application::mount(root);
    ffi::test::take_calls();

    menu.show_from_context_pointer(24.0, 32.0);
    let calls = ffi::test::take_calls();
    let item_handle = handle_with_semantic_label(&calls, "Primary");
    let bounds = fui::bindings::ui::get_bounds(item_handle).expect("item bounds");

    pointer_event(
        PointerEventType::Enter,
        item_handle,
        bounds[0] + 4.0,
        bounds[1] + 4.0,
        0,
        0,
        0,
    );
    pointer_event(
        PointerEventType::Down,
        item_handle,
        bounds[0] + 4.0,
        bounds[1] + 4.0,
        0,
        1,
        1,
    );
    pointer_event(
        PointerEventType::Up,
        item_handle,
        bounds[0] + 4.0,
        bounds[1] + 4.0,
        0,
        0,
        0,
    );
    assert_eq!(count.get(), 1);
    assert!(!menu.is_open());
}

#[test]
fn context_menu_pointer_up_without_hover_or_press_does_not_invoke_first_item_and_escape_hides() {
    ffi::test::reset();
    let root = portal();
    let count = Rc::new(Cell::new(0));
    let count_for_click = count.clone();
    let menu = context_menu(vec![
        MenuItem::new("Primary", ContextMenuAction::OpenLink).on_invoke(move || {
            count_for_click.set(count_for_click.get() + 1);
        }),
        MenuItem::new("Secondary", ContextMenuAction::OpenLink),
    ]);
    root.child(&menu);
    Application::mount(root);
    ffi::test::take_calls();

    menu.show(24.0, 32.0);
    let calls = ffi::test::take_calls();
    let item_handle = handle_with_semantic_label(&calls, "Primary");

    pointer_event(PointerEventType::Up, item_handle, 30.0, 40.0, 0, 0, 0);
    assert_eq!(count.get(), 0);
    assert!(menu.is_open());

    assert!(key_event(KeyEventType::Down, "Escape", 0));
    assert!(!menu.is_open());
}

#[test]
fn nav_link_shows_preview_and_navigates_via_pointer_and_keyboard() {
    ffi::test::reset();
    ffi::test::set_platform_family(1);
    let link = nav_link("https://example.com/docs");
    link.text("Example docs");
    Application::mount(link.clone());
    let mount_calls = ffi::test::take_calls();
    let label_handle = *created_handles_of_type(&mount_calls, NodeType::Text)
        .last()
        .expect("nav link label handle");

    let handle = link.handle().raw();
    pointer_event(PointerEventType::Enter, handle, 24.0, 36.0, 0, 0, 0);
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(
        |call| matches!(call, Call::ShowUrlPreview { url } if url == "https://example.com/docs")
    ));
    assert_eq!(cursor_styles(&calls), vec![CursorStyle::Pointer as u32]);

    pointer_event(PointerEventType::Leave, handle, 24.0, 36.0, 0, 0, 0);
    let calls = ffi::test::take_calls();
    assert!(calls
        .iter()
        .any(|call| matches!(call, Call::HideUrlPreview)));
    assert_eq!(cursor_styles(&calls), vec![CursorStyle::Default as u32]);

    pointer_event(PointerEventType::Enter, label_handle, 24.0, 36.0, 0, 0, 0);
    let calls = ffi::test::take_calls();
    assert_eq!(cursor_styles(&calls), vec![CursorStyle::Pointer as u32]);
    pointer_event(PointerEventType::Leave, label_handle, 24.0, 36.0, 0, 0, 0);
    ffi::test::take_calls();

    primary_click(handle, 1);
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::NavigateTo {
            target,
            open_in_new_tab: false,
        } if target == "https://example.com/docs"
    )));

    focus(&link);
    ffi::test::take_calls();
    assert!(key_event(KeyEventType::Down, "Enter", 0));
    assert!(key_event(KeyEventType::Up, "Enter", 0));
    let calls = ffi::test::take_calls();
    let navigate_count = calls
        .iter()
        .filter(|call| {
            matches!(
                call,
                Call::NavigateTo {
                    target,
                    open_in_new_tab: false,
                } if target == "https://example.com/docs"
            )
        })
        .count();
    assert_eq!(navigate_count, 1);

    assert!(key_event(
        KeyEventType::Down,
        "Enter",
        KeyModifier::Meta as u32
    ));
    assert!(key_event(
        KeyEventType::Up,
        "Enter",
        KeyModifier::Meta as u32
    ));
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::NavigateTo {
            target,
            open_in_new_tab: true,
        } if target == "https://example.com/docs"
    )));
}

#[test]
fn scrollbar_track_click_updates_scroll_offset_on_fine_pointer() {
    ffi::test::reset();
    let state = ScrollState::new();
    state.set_viewport_height(100.0);
    state.set_content_height(400.0);
    let scrollbar = ScrollBar::new(state.clone(), Orientation::Vertical);
    let root = scrollbar.render();
    Application::mount(root.clone());
    ffi::test::take_calls();

    pointer_event(
        PointerEventType::Down,
        root.handle().raw(),
        0.0,
        75.0,
        0,
        1,
        1,
    );

    assert_eq!(state.offset_y(), 250.0);
}

#[test]
fn scrollbar_thumb_drag_uses_pointer_capture_and_updates_scroll_offset() {
    ffi::test::reset();
    ffi::test::set_coarse_pointer(false);
    let state = ScrollState::new();
    state.set_viewport_height(100.0);
    state.set_content_height(400.0);
    let scrollbar = ScrollBar::new(state.clone(), Orientation::Vertical);
    Application::mount(scrollbar.render());
    let theme = fui::theme::current_theme();
    let mount_calls = ffi::test::take_calls();
    let thumb_handle = *handles_with_bg_color(&mount_calls, theme.colors.scrollbar_thumb)
        .first()
        .expect("scrollbar thumb handle");

    pointer_event(PointerEventType::Enter, thumb_handle, 4.0, 10.0, 0, 0, 0);
    let calls = ffi::test::take_calls();
    assert_eq!(cursor_styles(&calls), vec![CursorStyle::Grab as u32]);

    pointer_event(PointerEventType::Down, thumb_handle, 4.0, 10.0, 0, 1, 1);
    let calls = ffi::test::take_calls();
    assert!(calls
        .iter()
        .any(|call| matches!(call, Call::SetPointerCapture { handle } if *handle == thumb_handle)));
    assert_eq!(cursor_styles(&calls), vec![CursorStyle::Grabbing as u32]);

    pointer_event(
        PointerEventType::Move,
        HandleValue::Invalid as u64,
        4.0,
        35.0,
        0,
        1,
        0,
    );
    assert_eq!(state.offset_y(), 100.0);

    pointer_event(
        PointerEventType::Up,
        HandleValue::Invalid as u64,
        4.0,
        35.0,
        0,
        0,
        0,
    );
    let calls = ffi::test::take_calls();
    assert!(calls
        .iter()
        .any(|call| matches!(call, Call::ReleasePointerCapture)));
    assert_eq!(cursor_styles(&calls), vec![CursorStyle::Grab as u32]);
}

#[test]
fn scrollbar_is_inert_on_coarse_pointer_like_fui_as() {
    ffi::test::reset();
    ffi::test::set_coarse_pointer(true);
    let state = ScrollState::new();
    state.set_viewport_height(100.0);
    state.set_content_height(400.0);
    let scrollbar = ScrollBar::new(state.clone(), Orientation::Vertical);
    let root = scrollbar.render();
    Application::mount(root.clone());
    ffi::test::take_calls();

    pointer_event(
        PointerEventType::Down,
        root.handle().raw(),
        0.0,
        75.0,
        0,
        1,
        1,
    );

    assert_eq!(state.offset_y(), 0.0);
}

#[test]
fn hover_cursor_tracks_hovered_node_and_resets_on_leave() {
    ffi::test::reset();
    let root = flex_box();
    root.cursor(CursorStyle::Pointer);
    Application::mount(root.clone());
    ffi::test::take_calls();

    let handle = root.handle().raw();
    pointer_event(PointerEventType::Enter, handle, 24.0, 36.0, 0, 0, 0);
    let calls = ffi::test::take_calls();
    assert_eq!(cursor_styles(&calls), vec![CursorStyle::Pointer as u32]);

    pointer_event(PointerEventType::Leave, handle, 24.0, 36.0, 0, 0, 0);
    let calls = ffi::test::take_calls();
    assert_eq!(cursor_styles(&calls), vec![CursorStyle::Default as u32]);
}

#[test]
fn captured_pointer_cursor_overrides_hover_until_release() {
    ffi::test::reset();
    let hover = flex_box();
    hover.cursor(CursorStyle::Pointer);
    let captured = flex_box();
    captured.cursor(CursorStyle::Move);
    captured.on_pointer_down(|event| {
        event.capture_pointer();
    });

    let root = column();
    root.child(&hover);
    root.child(&captured);
    Application::mount(root);
    ffi::test::take_calls();

    let hover_handle = hover.handle().raw();
    let captured_handle = captured.handle().raw();

    pointer_event(PointerEventType::Enter, hover_handle, 24.0, 24.0, 0, 0, 0);
    let calls = ffi::test::take_calls();
    assert_eq!(cursor_styles(&calls), vec![CursorStyle::Pointer as u32]);

    pointer_event(PointerEventType::Down, captured_handle, 24.0, 64.0, 0, 1, 1);
    let calls = ffi::test::take_calls();
    assert_eq!(cursor_styles(&calls), vec![CursorStyle::Move as u32]);

    pointer_event(PointerEventType::Up, hover_handle, 24.0, 24.0, 0, 0, 0);
    let calls = ffi::test::take_calls();
    assert_eq!(cursor_styles(&calls), vec![CursorStyle::Pointer as u32]);
}
