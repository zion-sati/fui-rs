use crate::bindings::ui;
use crate::ffi::{PositionType, Unit};
use crate::node::{flex_box, portal, FlexBox, Node, NodeHandle};
use crate::theme::current_theme;
use std::cell::RefCell;

const STANDARD_FOCUS_RING_WIDTH: f32 = 2.0;
const STANDARD_FOCUS_RING_OUTSET: f32 = 2.0;
const TRANSPARENT: u32 = 0x00000000;

#[derive(Clone, Copy)]
struct FocusAdornerStyle {
    top_left_radius: f32,
    top_right_radius: f32,
    bottom_right_radius: f32,
    bottom_left_radius: f32,
}

#[derive(Clone, Copy)]
struct FocusAdornerRect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

struct FocusAdornerState {
    host_root: Option<FlexBox>,
    ring_node: Option<FlexBox>,
    active_owner: Option<NodeHandle>,
    active_style: Option<FocusAdornerStyle>,
    attached: bool,
    last_host_x: f32,
    last_host_y: f32,
    last_host_width: f32,
    last_host_height: f32,
    last_ring_x: f32,
    last_ring_y: f32,
    last_ring_width: f32,
    last_ring_height: f32,
    last_color: u32,
    last_top_left_radius: f32,
    last_top_right_radius: f32,
    last_bottom_right_radius: f32,
    last_bottom_left_radius: f32,
}

impl FocusAdornerState {
    fn new() -> Self {
        Self {
            host_root: None,
            ring_node: None,
            active_owner: None,
            active_style: None,
            attached: false,
            last_host_x: f32::NAN,
            last_host_y: f32::NAN,
            last_host_width: f32::NAN,
            last_host_height: f32::NAN,
            last_ring_x: f32::NAN,
            last_ring_y: f32::NAN,
            last_ring_width: f32::NAN,
            last_ring_height: f32::NAN,
            last_color: 0,
            last_top_left_radius: f32::NAN,
            last_top_right_radius: f32::NAN,
            last_bottom_right_radius: f32::NAN,
            last_bottom_left_radius: f32::NAN,
        }
    }

    fn reset_cached_geometry(&mut self) {
        self.last_host_x = f32::NAN;
        self.last_host_y = f32::NAN;
        self.last_host_width = f32::NAN;
        self.last_host_height = f32::NAN;
        self.last_ring_x = f32::NAN;
        self.last_ring_y = f32::NAN;
        self.last_ring_width = f32::NAN;
        self.last_ring_height = f32::NAN;
        self.last_color = 0;
        self.last_top_left_radius = f32::NAN;
        self.last_top_right_radius = f32::NAN;
        self.last_bottom_right_radius = f32::NAN;
        self.last_bottom_left_radius = f32::NAN;
    }
}

thread_local! {
    static FOCUS_ADORNER_STATE: RefCell<FocusAdornerState> =
        RefCell::new(FocusAdornerState::new());
}

pub(crate) fn create_default_host() -> FlexBox {
    FOCUS_ADORNER_STATE.with(|slot| {
        let mut state = slot.borrow_mut();
        if let Some(existing_host) = state.host_root.as_ref() {
            return existing_host.clone();
        }
        let ring_node = flex_box();
        ring_node
            .position_type(PositionType::Absolute)
            .bg_color(TRANSPARENT)
            .border(STANDARD_FOCUS_RING_WIDTH, TRANSPARENT);
        let host_root = portal();
        host_root
            .position_type(PositionType::Absolute)
            .position(0.0, 0.0)
            .width(0.0, Unit::Pixel)
            .height(0.0, Unit::Pixel)
            .clip_to_bounds(false);
        state.host_root = Some(host_root.clone());
        state.ring_node = Some(ring_node);
        host_root
    })
}

pub(crate) fn clear() {
    hide();
    FOCUS_ADORNER_STATE.with(|slot| {
        let mut state = slot.borrow_mut();
        state.active_owner = None;
        state.active_style = None;
    });
}

pub(crate) fn show_standard<T: Node>(owner: &T, corner_radius: f32) {
    show_standard_corners(
        owner,
        corner_radius,
        corner_radius,
        corner_radius,
        corner_radius,
    );
}

pub(crate) fn show_standard_corners<T: Node>(owner: &T, tl: f32, tr: f32, br: f32, bl: f32) {
    FOCUS_ADORNER_STATE.with(|slot| {
        let mut state = slot.borrow_mut();
        state.active_owner = Some(owner.handle());
        state.active_style = Some(FocusAdornerStyle {
            top_left_radius: tl + STANDARD_FOCUS_RING_OUTSET,
            top_right_radius: tr + STANDARD_FOCUS_RING_OUTSET,
            bottom_right_radius: br + STANDARD_FOCUS_RING_OUTSET,
            bottom_left_radius: bl + STANDARD_FOCUS_RING_OUTSET,
        });
    });
    sync();
}

pub(crate) fn hide_owner<T: Node>(owner: &T) {
    hide_owner_handle(owner.handle());
}

pub(crate) fn handle_owner_destroyed(owner: NodeHandle) {
    hide_owner_handle(owner);
}

pub(crate) fn refresh_after_commit() -> bool {
    sync()
}

fn hide_owner_handle(owner: NodeHandle) {
    let should_hide = FOCUS_ADORNER_STATE.with(|slot| slot.borrow().active_owner == Some(owner));
    if !should_hide {
        return;
    }
    FOCUS_ADORNER_STATE.with(|slot| {
        let mut state = slot.borrow_mut();
        state.active_owner = None;
        state.active_style = None;
    });
    hide();
}

fn sync() -> bool {
    let owner = FOCUS_ADORNER_STATE.with(|slot| slot.borrow().active_owner);
    let style = FOCUS_ADORNER_STATE.with(|slot| slot.borrow().active_style);
    let (Some(owner), Some(style)) = (owner, style) else {
        return hide();
    };
    let (Some(host_root), Some(ring_node)) = FOCUS_ADORNER_STATE.with(|slot| {
        let state = slot.borrow();
        (state.host_root.clone(), state.ring_node.clone())
    }) else {
        return false;
    };
    if owner == NodeHandle::INVALID || host_root.handle() == NodeHandle::INVALID {
        return hide();
    }
    let Some(ring_rect) = resolve_ring_rect(owner) else {
        return hide();
    };
    let Some(visible_rect) = resolve_visible_rect(ring_rect) else {
        return hide();
    };
    let color = current_theme().colors.focus_ring;
    let mut changed = false;
    FOCUS_ADORNER_STATE.with(|slot| {
        let mut state = slot.borrow_mut();
        if !state.attached {
            host_root.child(&ring_node);
            state.attached = true;
            changed = true;
        }
        let relative_ring_x = ring_rect.x - visible_rect.x;
        let relative_ring_y = ring_rect.y - visible_rect.y;
        if visible_rect.x != state.last_host_x
            || visible_rect.y != state.last_host_y
            || visible_rect.width != state.last_host_width
            || visible_rect.height != state.last_host_height
            || relative_ring_x != state.last_ring_x
            || relative_ring_y != state.last_ring_y
            || ring_rect.width != state.last_ring_width
            || ring_rect.height != state.last_ring_height
        {
            host_root.position(visible_rect.x, visible_rect.y);
            host_root.width(visible_rect.width, Unit::Pixel);
            host_root.height(visible_rect.height, Unit::Pixel);
            ring_node.position(relative_ring_x, relative_ring_y);
            ring_node.width(ring_rect.width, Unit::Pixel);
            ring_node.height(ring_rect.height, Unit::Pixel);
            state.last_host_x = visible_rect.x;
            state.last_host_y = visible_rect.y;
            state.last_host_width = visible_rect.width;
            state.last_host_height = visible_rect.height;
            state.last_ring_x = relative_ring_x;
            state.last_ring_y = relative_ring_y;
            state.last_ring_width = ring_rect.width;
            state.last_ring_height = ring_rect.height;
            changed = true;
        }
        if color != state.last_color
            || style.top_left_radius != state.last_top_left_radius
            || style.top_right_radius != state.last_top_right_radius
            || style.bottom_right_radius != state.last_bottom_right_radius
            || style.bottom_left_radius != state.last_bottom_left_radius
        {
            ring_node.corners(
                style.top_left_radius,
                style.top_right_radius,
                style.bottom_right_radius,
                style.bottom_left_radius,
            );
            ring_node.border(STANDARD_FOCUS_RING_WIDTH, color);
            state.last_color = color;
            state.last_top_left_radius = style.top_left_radius;
            state.last_top_right_radius = style.top_right_radius;
            state.last_bottom_right_radius = style.bottom_right_radius;
            state.last_bottom_left_radius = style.bottom_left_radius;
            changed = true;
        }
    });
    changed
}

fn hide() -> bool {
    FOCUS_ADORNER_STATE.with(|slot| {
        let mut state = slot.borrow_mut();
        let (Some(host_root), Some(ring_node)) = (state.host_root.clone(), state.ring_node.clone())
        else {
            state.attached = false;
            state.reset_cached_geometry();
            return false;
        };
        if !state.attached {
            state.attached = false;
            state.reset_cached_geometry();
            return false;
        }
        host_root.remove_child(&ring_node);
        host_root.position(0.0, 0.0);
        host_root.width(0.0, Unit::Pixel);
        host_root.height(0.0, Unit::Pixel);
        state.attached = false;
        state.reset_cached_geometry();
        true
    })
}

fn resolve_ring_rect(owner: NodeHandle) -> Option<FocusAdornerRect> {
    let bounds = ui::get_bounds(owner.raw())?;
    Some(FocusAdornerRect {
        x: bounds[0] - STANDARD_FOCUS_RING_OUTSET,
        y: bounds[1] - STANDARD_FOCUS_RING_OUTSET,
        width: bounds[2] + (STANDARD_FOCUS_RING_OUTSET * 2.0),
        height: bounds[3] + (STANDARD_FOCUS_RING_OUTSET * 2.0),
    })
}

fn resolve_visible_rect(ring_rect: FocusAdornerRect) -> Option<FocusAdornerRect> {
    let min_x = ring_rect.x.max(0.0);
    let min_y = ring_rect.y.max(0.0);
    let max_x = (ring_rect.x + ring_rect.width).min(ui::get_viewport_width());
    let max_y = (ring_rect.y + ring_rect.height).min(ui::get_viewport_height());
    let width = max_x - min_x;
    let height = max_y - min_y;
    if width <= 0.0 || height <= 0.0 {
        return None;
    }
    Some(FocusAdornerRect {
        x: min_x,
        y: min_y,
        width,
        height,
    })
}
