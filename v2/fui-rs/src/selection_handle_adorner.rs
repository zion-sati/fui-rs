use crate::bindings::ui;
use crate::event::{PointerEventArgs, PointerType};
use crate::ffi::{PointerEventType, PositionType, Unit, Visibility};
use crate::node::{flex_box, portal, FlexBox, Node, NodeHandle};
use std::cell::RefCell;

const HANDLE_COLOR: u32 = 0x0A84FFFF;
const HIT_TARGET_SIZE: f32 = 90.0;
const HIT_TARGET_PADDING: f32 = 25.0;
const START_ANCHOR_WIDTH: f32 = 0.0;
const KNOB_SIZE: f32 = 18.0;
const SHOULDER_SIZE: f32 = 8.0;
const START_STEM_X: f32 = 72.0;
const END_STEM_X: f32 = 18.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SelectionHandleSide {
    Start,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SelectionHandleDragSide {
    None,
    Start,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum SelectionHandleMode {
    Auto,
    Always,
    Disabled,
}

struct State {
    host_root: Option<FlexBox>,
    start_handle: Option<FlexBox>,
    end_handle: Option<FlexBox>,
    attached: bool,
    active_handle: NodeHandle,
    active_start: u32,
    active_end: u32,
    active_uses_range_geometry: bool,
    active_uses_cross_geometry: bool,
    mode_value: SelectionHandleMode,
    last_pointer_type: PointerType,
    dragging_side: SelectionHandleDragSide,
    dragging_captured_visual_side: SelectionHandleDragSide,
    start_anchor: Option<(f32, f32)>,
    end_anchor: Option<(f32, f32)>,
    stationary_anchor: Option<(f32, f32)>,
}

impl State {
    fn new() -> Self {
        Self {
            host_root: None,
            start_handle: None,
            end_handle: None,
            attached: false,
            active_handle: NodeHandle::INVALID,
            active_start: 0,
            active_end: 0,
            active_uses_range_geometry: false,
            active_uses_cross_geometry: false,
            mode_value: SelectionHandleMode::Auto,
            last_pointer_type: PointerType::Unknown,
            dragging_side: SelectionHandleDragSide::None,
            dragging_captured_visual_side: SelectionHandleDragSide::None,
            start_anchor: None,
            end_anchor: None,
            stationary_anchor: None,
        }
    }
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::new());
}

fn handle_stem_x(side: SelectionHandleSide) -> f32 {
    match side {
        SelectionHandleSide::Start => START_STEM_X,
        SelectionHandleSide::End => END_STEM_X,
    }
}

fn handle_knob_x(side: SelectionHandleSide) -> f32 {
    match side {
        SelectionHandleSide::Start => handle_stem_x(side) - KNOB_SIZE + START_ANCHOR_WIDTH,
        SelectionHandleSide::End => handle_stem_x(side),
    }
}

fn handle_shoulder_x(side: SelectionHandleSide) -> f32 {
    let knob_x = handle_knob_x(side);
    match side {
        SelectionHandleSide::Start => knob_x + KNOB_SIZE - SHOULDER_SIZE,
        SelectionHandleSide::End => knob_x,
    }
}

fn make_knob(side: SelectionHandleSide) -> FlexBox {
    let knob = flex_box();
    knob.position_type(PositionType::Absolute)
        .position(handle_knob_x(side), HIT_TARGET_PADDING)
        .width(KNOB_SIZE, Unit::Pixel)
        .height(KNOB_SIZE, Unit::Pixel)
        .corner_radius(KNOB_SIZE * 0.5)
        .bg_color(HANDLE_COLOR)
        .border(1.0, 0x00000022)
        .preserve_selection_on_pointer_down(true);
    knob
}

fn make_shoulder(side: SelectionHandleSide) -> FlexBox {
    let shoulder = flex_box();
    shoulder
        .position_type(PositionType::Absolute)
        .position(handle_shoulder_x(side), HIT_TARGET_PADDING)
        .width(SHOULDER_SIZE, Unit::Pixel)
        .height(SHOULDER_SIZE, Unit::Pixel)
        .bg_color(HANDLE_COLOR)
        .preserve_selection_on_pointer_down(true);
    shoulder
}

fn make_handle(side: SelectionHandleSide) -> FlexBox {
    let knob = make_knob(side);
    let shoulder = make_shoulder(side);
    match side {
        SelectionHandleSide::Start => {
            knob.on_pointer_down(handle_start_pointer_down);
            shoulder.on_pointer_down(handle_start_pointer_down);
        }
        SelectionHandleSide::End => {
            knob.on_pointer_down(handle_end_pointer_down);
            shoulder.on_pointer_down(handle_end_pointer_down);
        }
    }
    knob.on_pointer_move(handle_pointer_move)
        .on_pointer_up(handle_pointer_up)
        .on_pointer_cancel(handle_pointer_cancel);
    shoulder
        .on_pointer_move(handle_pointer_move)
        .on_pointer_up(handle_pointer_up)
        .on_pointer_cancel(handle_pointer_cancel);
    let handle = flex_box();
    handle
        .position_type(PositionType::Absolute)
        .width(HIT_TARGET_SIZE, Unit::Pixel)
        .height(HIT_TARGET_SIZE, Unit::Pixel)
        .bg_color(0x00000000)
        .child(&knob)
        .child(&shoulder)
        .interactive(true)
        .visibility(Visibility::Hidden)
        .preserve_selection_on_pointer_down(true);
    handle
}

pub(crate) fn create_default_host() -> FlexBox {
    STATE.with(|slot| {
        let mut state = slot.borrow_mut();
        if let Some(host_root) = state.host_root.as_ref() {
            return host_root.clone();
        }
        let start_handle = make_handle(SelectionHandleSide::Start);
        let end_handle = make_handle(SelectionHandleSide::End);
        start_handle
            .on_pointer_down(handle_start_pointer_down)
            .on_pointer_move(handle_pointer_move)
            .on_pointer_up(handle_pointer_up)
            .on_pointer_cancel(handle_pointer_cancel);
        end_handle
            .on_pointer_down(handle_end_pointer_down)
            .on_pointer_move(handle_pointer_move)
            .on_pointer_up(handle_pointer_up)
            .on_pointer_cancel(handle_pointer_cancel);
        let host_root = portal();
        host_root
            .position_type(PositionType::Absolute)
            .position(0.0, 0.0)
            .width(100.0, Unit::Percent)
            .height(100.0, Unit::Percent)
            .child(&start_handle)
            .child(&end_handle);
        state.host_root = Some(host_root.clone());
        state.start_handle = Some(start_handle);
        state.end_handle = Some(end_handle);
        host_root
    })
}

pub(crate) fn reset() {
    STATE.with(|slot| {
        let mut state = slot.borrow_mut();
        if let Some(host_root) = state.host_root.take() {
            host_root.dispose();
        }
        *state = State::new();
    });
}

pub(crate) fn clear() {
    let dragging = STATE.with(|slot| slot.borrow().dragging_side);
    if dragging != SelectionHandleDragSide::None {
        end_handle_drag();
    }
    STATE.with(|slot| {
        let mut state = slot.borrow_mut();
        state.active_handle = NodeHandle::INVALID;
        state.active_start = 0;
        state.active_end = 0;
        state.active_uses_range_geometry = false;
        state.active_uses_cross_geometry = false;
        state.dragging_side = SelectionHandleDragSide::None;
        state.dragging_captured_visual_side = SelectionHandleDragSide::None;
        state.stationary_anchor = None;
    });
    hide_handles();
}

pub(crate) fn record_pointer_event(event_type: PointerEventType, pointer_type: PointerType) {
    if event_type == PointerEventType::Down {
        STATE.with(|slot| slot.borrow_mut().last_pointer_type = pointer_type);
    }
}

pub(crate) fn handle_selection_changed(handle: NodeHandle, start: u32, end: u32) {
    let dragging = STATE.with(|slot| slot.borrow().dragging_side);
    if start == end || !should_show() {
        if start == end
            && dragging != SelectionHandleDragSide::None
            && STATE.with(|slot| slot.borrow().active_handle == handle)
        {
            STATE.with(|slot| {
                let mut state = slot.borrow_mut();
                state.active_start = start;
                state.active_end = end;
            });
            create_default_host();
            position_range_handles();
            show_non_dragged_handles();
            return;
        }
        clear();
        return;
    }
    STATE.with(|slot| {
        let mut state = slot.borrow_mut();
        state.active_handle = handle;
        state.active_start = start;
        state.active_end = end;
        state.active_uses_range_geometry = true;
        state.active_uses_cross_geometry = false;
    });
    create_default_host();
    if !position_range_handles() {
        clear();
        return;
    }
    if dragging != SelectionHandleDragSide::None {
        show_non_dragged_handles();
        return;
    }
    show_handles();
}

pub(crate) fn handle_cross_selection_changed(handle: NodeHandle, text: &str) {
    let dragging = STATE.with(|slot| slot.borrow().dragging_side);
    if text.is_empty() || !should_show() {
        if text.is_empty()
            && dragging != SelectionHandleDragSide::None
            && STATE.with(|slot| slot.borrow().active_handle == handle)
        {
            STATE.with(|slot| {
                let mut state = slot.borrow_mut();
                state.active_start = 0;
                state.active_end = 0;
            });
            create_default_host();
            show_non_dragged_handles();
            return;
        }
        clear();
        return;
    }
    STATE.with(|slot| {
        let mut state = slot.borrow_mut();
        state.active_handle = handle;
        state.active_start = 0;
        state.active_end = text.len() as u32;
        state.active_uses_range_geometry = true;
        state.active_uses_cross_geometry = true;
    });
    create_default_host();
    if !position_cross_selection_handles() {
        STATE.with(|slot| {
            let mut state = slot.borrow_mut();
            state.active_uses_range_geometry = false;
            state.active_uses_cross_geometry = false;
        });
        position_placeholder_handles();
    }
    if dragging != SelectionHandleDragSide::None {
        show_non_dragged_handles();
        return;
    }
    show_handles();
}

pub(crate) fn refresh_active_geometry() {
    let (active_handle, uses_range_geometry, uses_cross_geometry) = STATE.with(|slot| {
        let state = slot.borrow();
        (
            state.active_handle,
            state.active_uses_range_geometry,
            state.active_uses_cross_geometry,
        )
    });
    if active_handle == NodeHandle::INVALID || !uses_range_geometry {
        return;
    }
    if uses_cross_geometry {
        if !position_cross_selection_handles() {
            clear();
        }
        return;
    }
    if !position_range_handles() {
        clear();
    }
}

pub(crate) fn is_visible() -> bool {
    STATE.with(|slot| slot.borrow().active_handle != NodeHandle::INVALID)
}

pub(crate) fn route_active_handle_drag_event(event: &mut PointerEventArgs) -> bool {
    let dragging_side = STATE.with(|slot| slot.borrow().dragging_side);
    if dragging_side == SelectionHandleDragSide::None {
        return false;
    }
    match event.event_type {
        PointerEventType::Move => handle_pointer_move(event),
        PointerEventType::Up => handle_pointer_up(event),
        PointerEventType::Cancel => handle_pointer_cancel(event),
        _ => {}
    }
    event.handled
}

fn should_show() -> bool {
    STATE.with(|slot| {
        let state = slot.borrow();
        match state.mode_value {
            SelectionHandleMode::Disabled => false,
            SelectionHandleMode::Always => true,
            SelectionHandleMode::Auto => match state.last_pointer_type {
                PointerType::Touch => true,
                PointerType::Unknown => {
                    crate::generated::framework_host_services::fui_is_coarse_pointer()
                }
                _ => false,
            },
        }
    })
}

fn show_handles() {
    ensure_attached();
    STATE.with(|slot| {
        let state = slot.borrow();
        if let Some(start_handle) = state.start_handle.as_ref() {
            start_handle.visibility(Visibility::Normal);
            set_handle_chrome_visibility(start_handle, Visibility::Normal);
        }
        if let Some(end_handle) = state.end_handle.as_ref() {
            end_handle.visibility(Visibility::Normal);
            set_handle_chrome_visibility(end_handle, Visibility::Normal);
        }
    });
}

fn hide_handles() {
    STATE.with(|slot| {
        let state = slot.borrow();
        if let Some(start_handle) = state.start_handle.as_ref() {
            set_handle_chrome_visibility(start_handle, Visibility::Normal);
            start_handle.visibility(Visibility::Hidden);
        }
        if let Some(end_handle) = state.end_handle.as_ref() {
            set_handle_chrome_visibility(end_handle, Visibility::Normal);
            end_handle.visibility(Visibility::Hidden);
        }
    });
    detach_handles();
}

fn set_handle_chrome_visibility(handle: &FlexBox, visibility: Visibility) {
    for child in handle.node_ref().children() {
        if child.handle() != NodeHandle::INVALID {
            ui::set_visibility(child.handle().raw(), visibility as u32);
        }
    }
}

fn hide_handle(side: SelectionHandleDragSide) {
    if let Some(handle) = handle_for_side(side) {
        handle.visibility(Visibility::Normal);
        set_handle_chrome_visibility(&handle, Visibility::Hidden);
    }
}

fn show_handle(side: SelectionHandleDragSide) {
    if let Some(handle) = handle_for_side(side) {
        handle.visibility(Visibility::Normal);
        set_handle_chrome_visibility(&handle, Visibility::Normal);
    }
}

fn show_non_dragged_handles() {
    ensure_attached();
    let hidden_visual_side = STATE.with(|slot| slot.borrow().dragging_captured_visual_side);
    STATE.with(|slot| {
        let state = slot.borrow();
        if let Some(start_handle) = state.start_handle.as_ref() {
            start_handle.visibility(Visibility::Normal);
            set_handle_chrome_visibility(
                start_handle,
                if hidden_visual_side == SelectionHandleDragSide::Start {
                    Visibility::Hidden
                } else {
                    Visibility::Normal
                },
            );
        }
        if let Some(end_handle) = state.end_handle.as_ref() {
            end_handle.visibility(Visibility::Normal);
            set_handle_chrome_visibility(
                end_handle,
                if hidden_visual_side == SelectionHandleDragSide::End {
                    Visibility::Hidden
                } else {
                    Visibility::Normal
                },
            );
        }
    });
}

fn handle_start_pointer_down(event: &mut PointerEventArgs) {
    event.handled = begin_handle_drag(
        semantic_side_for_visual_side(SelectionHandleDragSide::Start),
        SelectionHandleDragSide::Start,
        event.pointer_type,
    );
}

fn handle_end_pointer_down(event: &mut PointerEventArgs) {
    event.handled = begin_handle_drag(
        semantic_side_for_visual_side(SelectionHandleDragSide::End),
        SelectionHandleDragSide::End,
        event.pointer_type,
    );
}

fn handle_pointer_move(event: &mut PointerEventArgs) {
    if STATE.with(|slot| slot.borrow().dragging_side) == SelectionHandleDragSide::None {
        return;
    }
    event.handled = true;
    refresh_active_geometry();
    show_non_dragged_handles();
}

fn handle_pointer_up(event: &mut PointerEventArgs) {
    if STATE.with(|slot| slot.borrow().dragging_side) == SelectionHandleDragSide::None {
        return;
    }
    event.handled = true;
    end_handle_drag();
}

fn handle_pointer_cancel(event: &mut PointerEventArgs) {
    if STATE.with(|slot| slot.borrow().dragging_side) == SelectionHandleDragSide::None {
        return;
    }
    event.handled = true;
    end_handle_drag();
}

fn begin_handle_drag(
    side: SelectionHandleDragSide,
    visual_side: SelectionHandleDragSide,
    pointer_type: PointerType,
) -> bool {
    if !matches!(pointer_type, PointerType::Touch | PointerType::Pen) {
        return false;
    }
    let active_handle = STATE.with(|slot| slot.borrow().active_handle);
    if active_handle == NodeHandle::INVALID {
        return false;
    }
    if !capture_stationary_anchor(visual_side) {
        return false;
    }
    let Some(handle) = handle_for_side(visual_side) else {
        STATE.with(|slot| slot.borrow_mut().stationary_anchor = None);
        return false;
    };
    STATE.with(|slot| {
        let mut state = slot.borrow_mut();
        state.dragging_side = side;
        state.dragging_captured_visual_side = visual_side;
    });
    if !ui::begin_selection_endpoint_drag(
        active_handle.raw(),
        match side {
            SelectionHandleDragSide::Start => 0,
            SelectionHandleDragSide::End => 1,
            SelectionHandleDragSide::None => 0,
        },
    ) {
        STATE.with(|slot| {
            let mut state = slot.borrow_mut();
            state.dragging_side = SelectionHandleDragSide::None;
            state.dragging_captured_visual_side = SelectionHandleDragSide::None;
            state.stationary_anchor = None;
        });
        return false;
    }
    crate::mobile_text_selection_toolbar::hide_for_handle_drag();
    if handle.handle() != NodeHandle::INVALID {
        crate::event::capture_pointer(handle.handle());
        unsafe { crate::ffi::fui_set_pointer_capture(handle.handle().raw()) };
    }
    set_handle_hit_test_visible(visual_side, false);
    hide_handle(visual_side);
    show_handle(if visual_side == SelectionHandleDragSide::Start {
        SelectionHandleDragSide::End
    } else {
        SelectionHandleDragSide::Start
    });
    true
}

fn semantic_side_for_visual_side(visual_side: SelectionHandleDragSide) -> SelectionHandleDragSide {
    if visual_side == SelectionHandleDragSide::None {
        return SelectionHandleDragSide::None;
    }
    let (active_start, active_end) = STATE.with(|slot| {
        let state = slot.borrow();
        (state.active_start, state.active_end)
    });
    let forward = active_start <= active_end;
    match visual_side {
        SelectionHandleDragSide::Start if forward => SelectionHandleDragSide::Start,
        SelectionHandleDragSide::Start => SelectionHandleDragSide::End,
        SelectionHandleDragSide::End if forward => SelectionHandleDragSide::End,
        SelectionHandleDragSide::End => SelectionHandleDragSide::Start,
        SelectionHandleDragSide::None => SelectionHandleDragSide::None,
    }
}

fn end_handle_drag() {
    let visual_side = STATE.with(|slot| slot.borrow().dragging_captured_visual_side);
    if let Some(handle) = handle_for_side(visual_side) {
        if handle.handle() != NodeHandle::INVALID {
            crate::event::release_pointer(handle.handle());
            unsafe { crate::ffi::fui_release_pointer_capture() };
        }
    }
    set_handle_hit_test_visible(visual_side, true);
    STATE.with(|slot| {
        let mut state = slot.borrow_mut();
        state.dragging_side = SelectionHandleDragSide::None;
        state.dragging_captured_visual_side = SelectionHandleDragSide::None;
        state.stationary_anchor = None;
    });
    if STATE.with(|slot| {
        let state = slot.borrow();
        state.active_start == state.active_end
    }) {
        clear();
        return;
    }
    refresh_active_geometry();
    show_handles();
    crate::mobile_text_selection_toolbar::show_after_handle_drag(is_visible());
}

fn set_handle_hit_test_visible(side: SelectionHandleDragSide, visible: bool) {
    let Some(handle) = handle_for_side(side) else {
        return;
    };
    if handle.handle() == NodeHandle::INVALID {
        return;
    }
    ui::set_interactive(handle.handle().raw(), visible);
}

fn position_placeholder_handles() {
    ensure_attached();
    STATE.with(|slot| {
        let state = slot.borrow();
        if let Some(start_handle) = state.start_handle.as_ref() {
            start_handle.position(0.0, 0.0);
        }
        if let Some(end_handle) = state.end_handle.as_ref() {
            end_handle.position(HIT_TARGET_SIZE + 8.0, 0.0);
        }
    });
}

fn position_range_handles() -> bool {
    ensure_attached();
    let (active_handle, active_start, active_end, dragging_side) = STATE.with(|slot| {
        let state = slot.borrow();
        (
            state.active_handle,
            state.active_start,
            state.active_end,
            state.dragging_side,
        )
    });
    let (Some(start_handle), Some(end_handle)) = STATE.with(|slot| {
        let state = slot.borrow();
        (state.start_handle.clone(), state.end_handle.clone())
    }) else {
        return false;
    };
    if active_handle == NodeHandle::INVALID {
        return false;
    }
    let (range_start, range_end) = if active_end < active_start {
        (active_end, active_start)
    } else {
        (active_start, active_end)
    };
    let rects = ui::get_text_range_rects(active_handle.raw(), range_start, range_end);
    if rects.is_empty() {
        return false;
    }
    let first_rect = rects.first().copied().unwrap();
    let last_rect = rects.last().copied().unwrap();
    if dragging_side != SelectionHandleDragSide::None {
        position_dragging_range_handles(
            &start_handle,
            &end_handle,
            first_rect.x,
            first_rect.y + first_rect.height,
            last_rect.x + last_rect.width,
            last_rect.y + last_rect.height,
        );
        return true;
    }
    position_start_handle(
        &start_handle,
        first_rect.x,
        first_rect.y + first_rect.height,
    );
    position_end_handle(
        &end_handle,
        last_rect.x + last_rect.width,
        last_rect.y + last_rect.height,
    );
    true
}

fn position_dragging_range_handles(
    start_handle: &FlexBox,
    end_handle: &FlexBox,
    lower_x: f32,
    lower_y: f32,
    upper_x: f32,
    upper_y: f32,
) {
    let (active_start, active_end, dragging_side, dragging_captured_visual_side, stationary_anchor) =
        STATE.with(|slot| {
            let state = slot.borrow();
            (
                state.active_start,
                state.active_end,
                state.dragging_side,
                state.dragging_captured_visual_side,
                state.stationary_anchor,
            )
        });
    let forward = active_start <= active_end;
    let dragging_start = dragging_side == SelectionHandleDragSide::Start;
    let moving_is_lower = dragging_start == forward;
    let (moving_x, moving_y) = if moving_is_lower {
        (lower_x, lower_y)
    } else {
        (upper_x, upper_y)
    };
    let (stationary_x, stationary_y) = if moving_is_lower {
        (upper_x, upper_y)
    } else {
        (lower_x, lower_y)
    };
    let ((moving_x, moving_y), (stationary_x, stationary_y)) =
        if let Some((stationary_x, stationary_y)) = stationary_anchor {
            let lower_distance = distance_squared(lower_x, lower_y, stationary_x, stationary_y);
            let upper_distance = distance_squared(upper_x, upper_y, stationary_x, stationary_y);
            let moving = if upper_distance >= lower_distance {
                (upper_x, upper_y)
            } else {
                (lower_x, lower_y)
            };
            (moving, (stationary_x, stationary_y))
        } else {
            ((moving_x, moving_y), (stationary_x, stationary_y))
        };
    match dragging_captured_visual_side {
        SelectionHandleDragSide::Start => {
            position_end_handle(end_handle, stationary_x, stationary_y);
            position_start_handle(start_handle, moving_x, moving_y);
        }
        SelectionHandleDragSide::End => {
            position_start_handle(start_handle, stationary_x, stationary_y);
            position_end_handle(end_handle, moving_x, moving_y);
        }
        SelectionHandleDragSide::None => {}
    }
}

fn distance_squared(left_x: f32, left_y: f32, right_x: f32, right_y: f32) -> f32 {
    let dx = left_x - right_x;
    let dy = left_y - right_y;
    (dx * dx) + (dy * dy)
}

fn position_cross_selection_handles() -> bool {
    ensure_attached();
    let active_handle = STATE.with(|slot| slot.borrow().active_handle);
    let (Some(start_handle), Some(end_handle)) = STATE.with(|slot| {
        let state = slot.borrow();
        (state.start_handle.clone(), state.end_handle.clone())
    }) else {
        return false;
    };
    if active_handle == NodeHandle::INVALID {
        return false;
    }
    let Some(rects) = ui::get_cross_selection_endpoint_rects(active_handle.raw()) else {
        return false;
    };
    position_start_handle(
        &start_handle,
        rects.start.x,
        rects.start.y + rects.start.height,
    );
    position_end_handle(
        &end_handle,
        rects.end.x + rects.end.width,
        rects.end.y + rects.end.height,
    );
    true
}

fn position_start_handle(handle: &FlexBox, x: f32, y: f32) {
    STATE.with(|slot| slot.borrow_mut().start_anchor = Some((x, y)));
    handle.position(x - START_STEM_X, y - HIT_TARGET_PADDING);
}

fn position_end_handle(handle: &FlexBox, x: f32, y: f32) {
    STATE.with(|slot| slot.borrow_mut().end_anchor = Some((x, y)));
    handle.position(x - END_STEM_X, y - HIT_TARGET_PADDING);
}

fn capture_stationary_anchor(visual_side: SelectionHandleDragSide) -> bool {
    STATE.with(|slot| {
        let mut state = slot.borrow_mut();
        state.stationary_anchor = match visual_side {
            SelectionHandleDragSide::Start => state.end_anchor,
            SelectionHandleDragSide::End => state.start_anchor,
            SelectionHandleDragSide::None => None,
        };
        state.stationary_anchor.is_some()
    })
}

fn handle_for_side(side: SelectionHandleDragSide) -> Option<FlexBox> {
    STATE.with(|slot| {
        let state = slot.borrow();
        match side {
            SelectionHandleDragSide::Start => state.start_handle.clone(),
            SelectionHandleDragSide::End => state.end_handle.clone(),
            SelectionHandleDragSide::None => None,
        }
    })
}

fn ensure_attached() {
    STATE.with(|slot| {
        let mut state = slot.borrow_mut();
        if state.attached {
            return;
        }
        let Some(host_root) = state.host_root.as_ref() else {
            return;
        };
        let Some(start_handle) = state.start_handle.as_ref() else {
            return;
        };
        let Some(end_handle) = state.end_handle.as_ref() else {
            return;
        };
        host_root.child(start_handle).child(end_handle);
        state.attached = true;
    });
}

fn detach_handles() {
    STATE.with(|slot| {
        let mut state = slot.borrow_mut();
        if !state.attached {
            return;
        }
        let Some(host_root) = state.host_root.as_ref() else {
            return;
        };
        let Some(start_handle) = state.start_handle.as_ref() else {
            return;
        };
        let Some(end_handle) = state.end_handle.as_ref() else {
            return;
        };
        let _ = host_root.remove_child(start_handle);
        let _ = host_root.remove_child(end_handle);
        state.attached = false;
    });
}
