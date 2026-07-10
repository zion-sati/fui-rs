use crate::bindings::ui;
use crate::event;
use crate::node::{NodeHandle, NodeRef};
use std::cell::RefCell;

const BOUNDS_TOLERANCE: f32 = 0.5;

thread_local! {
    static SCROLL_VIEWS: RefCell<Vec<NodeHandle>> = const { RefCell::new(Vec::new()) };
    static SELECTED_SCROLL_VIEW: RefCell<Option<NodeHandle>> = const { RefCell::new(None) };
    static SELECTED_BRANCH_ROOT: RefCell<Option<NodeHandle>> = const { RefCell::new(None) };
}

fn get_bounds(handle: NodeHandle) -> Option<[f32; 4]> {
    ui::get_bounds(handle.raw())
}

fn is_visible(bounds: [f32; 4]) -> bool {
    let viewport_width = ui::get_viewport_width();
    let viewport_height = ui::get_viewport_height();
    bounds[0] + bounds[2] > 0.0
        && bounds[1] + bounds[3] > 0.0
        && bounds[0] < viewport_width
        && bounds[1] < viewport_height
}

fn get_usable_visible_bounds(handle: NodeHandle) -> Option<[f32; 4]> {
    let bounds = get_bounds(handle)?;
    if bounds[2] <= BOUNDS_TOLERANCE || bounds[3] <= BOUNDS_TOLERANCE || !is_visible(bounds) {
        return None;
    }
    Some(bounds)
}

fn contains_point(bounds: [f32; 4], x: f32, y: f32) -> bool {
    x >= bounds[0] && x <= bounds[0] + bounds[2] && y >= bounds[1] && y <= bounds[1] + bounds[3]
}

fn area(bounds: [f32; 4]) -> f32 {
    bounds[2] * bounds[3]
}

fn distance_squared(bounds: [f32; 4], x: f32, y: f32) -> f32 {
    let delta_x = if x < bounds[0] {
        bounds[0] - x
    } else {
        let right = bounds[0] + bounds[2];
        if x > right {
            x - right
        } else {
            0.0
        }
    };
    let delta_y = if y < bounds[1] {
        bounds[1] - y
    } else {
        let bottom = bounds[1] + bounds[3];
        if y > bottom {
            y - bottom
        } else {
            0.0
        }
    };
    (delta_x * delta_x) + (delta_y * delta_y)
}

fn is_better_point_candidate(
    candidate_bounds: [f32; 4],
    candidate_contains_point: bool,
    best_bounds: [f32; 4],
    best_contains_point: bool,
    point_x: f32,
    point_y: f32,
) -> bool {
    if candidate_contains_point != best_contains_point {
        return candidate_contains_point;
    }
    if candidate_contains_point {
        let candidate_area = area(candidate_bounds);
        let best_area = area(best_bounds);
        if candidate_area + BOUNDS_TOLERANCE < best_area {
            return true;
        }
        if best_area + BOUNDS_TOLERANCE < candidate_area {
            return false;
        }
    } else {
        let candidate_distance = distance_squared(candidate_bounds, point_x, point_y);
        let best_distance = distance_squared(best_bounds, point_x, point_y);
        if candidate_distance + BOUNDS_TOLERANCE < best_distance {
            return true;
        }
        if best_distance + BOUNDS_TOLERANCE < candidate_distance {
            return false;
        }
    }
    if candidate_bounds[1] + BOUNDS_TOLERANCE < best_bounds[1] {
        return true;
    }
    if best_bounds[1] + BOUNDS_TOLERANCE < candidate_bounds[1] {
        return false;
    }
    candidate_bounds[0] < best_bounds[0]
}

fn is_better_default_candidate(candidate_bounds: [f32; 4], best_bounds: [f32; 4]) -> bool {
    if candidate_bounds[1] + BOUNDS_TOLERANCE < best_bounds[1] {
        return true;
    }
    if best_bounds[1] + BOUNDS_TOLERANCE < candidate_bounds[1] {
        return false;
    }
    candidate_bounds[0] < best_bounds[0]
}

fn is_descendant_of(node: NodeRef, ancestor: NodeRef) -> bool {
    let mut current = Some(node);
    while let Some(node) = current {
        if node.handle() == ancestor.handle() {
            return true;
        }
        current = node.parent();
    }
    false
}

fn default_ordering_anchor(scroll_view: NodeRef) -> NodeRef {
    let mut anchor = scroll_view.clone();
    let mut cursor = scroll_view.parent();
    while let Some(current) = cursor {
        if current.is_scroll_view_for_routing() {
            break;
        }
        anchor = current.clone();
        cursor = current.parent();
    }
    anchor
}

fn append_unique_scroll_view(target: &mut Vec<NodeHandle>, handle: NodeHandle) {
    if target.contains(&handle) || get_usable_visible_bounds(handle).is_none() {
        return;
    }
    SCROLL_VIEWS.with(|scroll_views| {
        if scroll_views.borrow().contains(&handle) {
            target.push(handle);
        }
    });
}

fn append_default_ordered_candidate(target: &mut Vec<NodeHandle>, candidate: NodeHandle) {
    let candidate_bounds = match get_usable_visible_bounds(candidate) {
        Some(bounds) => bounds,
        None => return,
    };
    let Some(candidate_node) = event::resolve_node(candidate) else {
        return;
    };
    let candidate_anchor_bounds =
        match get_usable_visible_bounds(default_ordering_anchor(candidate_node).handle()) {
            Some(bounds) => bounds,
            None => return,
        };
    let mut insert_index = target.len();
    while insert_index > 0 {
        let current = target[insert_index - 1];
        let Some(current_node) = event::resolve_node(current) else {
            insert_index -= 1;
            continue;
        };
        let Some(current_anchor_bounds) =
            get_usable_visible_bounds(default_ordering_anchor(current_node).handle())
        else {
            insert_index -= 1;
            continue;
        };
        let Some(current_candidate_bounds) = get_usable_visible_bounds(current) else {
            insert_index -= 1;
            continue;
        };
        if is_better_default_candidate(candidate_anchor_bounds, current_anchor_bounds) {
            insert_index -= 1;
            continue;
        }
        if candidate_anchor_bounds == current_anchor_bounds
            && is_better_default_candidate(candidate_bounds, current_candidate_bounds)
        {
            insert_index -= 1;
            continue;
        }
        break;
    }
    target.push(candidate);
    for cursor in (insert_index + 1..target.len()).rev() {
        target[cursor] = target[cursor - 1];
    }
    target[insert_index] = candidate;
}

fn select_default_candidates_within(root: Option<NodeRef>) -> Vec<NodeHandle> {
    let mut ordered = Vec::new();
    let Some(root) = root else {
        return ordered;
    };
    SCROLL_VIEWS.with(|scroll_views| {
        for handle in scroll_views.borrow().iter().copied() {
            let Some(candidate) = event::resolve_node(handle) else {
                continue;
            };
            if !is_descendant_of(candidate, root.clone()) {
                continue;
            }
            append_default_ordered_candidate(&mut ordered, handle);
        }
    });
    ordered
}

fn find_nearest_descendant_scroll_branch(node: Option<NodeRef>) -> Option<NodeHandle> {
    let mut current = node;
    while let Some(node) = current {
        if node.is_scroll_view_for_routing() {
            return None;
        }
        if !select_default_candidates_within(Some(node.clone())).is_empty() {
            return Some(node.handle());
        }
        current = node.parent();
    }
    None
}

fn select_scroll_view_by_point(point_x: f32, point_y: f32) -> Option<NodeHandle> {
    let mut best_view = None;
    let mut best_bounds = None;
    let mut best_contains_point = false;
    SCROLL_VIEWS.with(|scroll_views| {
        for handle in scroll_views.borrow().iter().copied() {
            let Some(candidate_bounds) = get_usable_visible_bounds(handle) else {
                continue;
            };
            let candidate_contains_point = contains_point(candidate_bounds, point_x, point_y);
            if best_bounds.is_none()
                || is_better_point_candidate(
                    candidate_bounds,
                    candidate_contains_point,
                    best_bounds.unwrap_or([0.0; 4]),
                    best_contains_point,
                    point_x,
                    point_y,
                )
            {
                best_view = Some(handle);
                best_bounds = Some(candidate_bounds);
                best_contains_point = candidate_contains_point;
            }
        }
    });
    best_view
}

fn resolve_ancestor_scroll_view(node: Option<NodeRef>) -> Option<NodeHandle> {
    let mut current = node;
    while let Some(node) = current {
        if node.is_scroll_view_for_routing() {
            return Some(node.handle());
        }
        current = node.parent();
    }
    None
}

fn append_ancestor_scroll_view_fallbacks(view: Option<NodeHandle>, target: &mut Vec<NodeHandle>) {
    let Some(view) = view else {
        return;
    };
    let Some(view_node) = event::resolve_node(view) else {
        return;
    };
    let mut current = view_node.parent();
    while let Some(node) = current {
        if node.is_scroll_view_for_routing() {
            append_unique_scroll_view(target, node.handle());
        }
        current = node.parent();
    }
}

fn select_default_candidates() -> Vec<NodeHandle> {
    let mut ordered = Vec::new();
    SCROLL_VIEWS.with(|scroll_views| {
        for handle in scroll_views.borrow().iter().copied() {
            append_default_ordered_candidate(&mut ordered, handle);
        }
    });
    ordered
}

pub(crate) fn register_keyboard_scroll_node(node: &NodeRef) {
    if !node.is_scroll_view_for_routing() {
        return;
    }
    SCROLL_VIEWS.with(|scroll_views| {
        let mut scroll_views = scroll_views.borrow_mut();
        if !scroll_views.contains(&node.handle()) {
            scroll_views.push(node.handle());
        }
    });
}

pub(crate) fn unregister_keyboard_scroll_node(handle: NodeHandle) {
    SCROLL_VIEWS.with(|scroll_views| {
        scroll_views
            .borrow_mut()
            .retain(|candidate| *candidate != handle);
    });
    SELECTED_SCROLL_VIEW.with(|selected| {
        if *selected.borrow() == Some(handle) {
            selected.replace(None);
        }
    });
}

pub(crate) fn track_keyboard_scroll_pointer_up(target_node: Option<NodeRef>, x: f32, y: f32) {
    let selected_branch_root = target_node.clone().and_then(|node| {
        if node.is_scroll_view_for_routing() {
            Some(node.handle())
        } else {
            find_nearest_descendant_scroll_branch(Some(node))
        }
    });
    SELECTED_BRANCH_ROOT.with(|branch| branch.replace(selected_branch_root));

    if let Some(ancestor_scroll_view) = resolve_ancestor_scroll_view(target_node.clone()) {
        SELECTED_BRANCH_ROOT.with(|branch| {
            if branch.borrow().is_none() {
                branch.replace(Some(ancestor_scroll_view));
            }
        });
        SELECTED_SCROLL_VIEW.with(|selected| selected.replace(Some(ancestor_scroll_view)));
        return;
    }

    if let Some(branch_root) = SELECTED_BRANCH_ROOT.with(|branch| *branch.borrow()) {
        let branch_candidates = select_default_candidates_within(event::resolve_node(branch_root));
        if let Some(candidate) = branch_candidates.first().copied() {
            SELECTED_SCROLL_VIEW.with(|selected| selected.replace(Some(candidate)));
            return;
        }
    }

    let selected = select_scroll_view_by_point(x, y);
    SELECTED_SCROLL_VIEW.with(|slot| slot.replace(selected));
    if selected.is_none() {
        SELECTED_BRANCH_ROOT.with(|branch| branch.replace(None));
    }
}

pub(crate) fn get_keyboard_scroll_selected_candidate() -> Option<NodeHandle> {
    SELECTED_SCROLL_VIEW.with(|selected| *selected.borrow())
}

pub(crate) fn get_keyboard_scroll_fallback_candidates() -> Vec<NodeHandle> {
    let mut ordered = Vec::new();
    append_ancestor_scroll_view_fallbacks(get_keyboard_scroll_selected_candidate(), &mut ordered);
    if let Some(branch_root) = SELECTED_BRANCH_ROOT.with(|branch| *branch.borrow()) {
        for candidate in select_default_candidates_within(event::resolve_node(branch_root)) {
            if Some(candidate) != get_keyboard_scroll_selected_candidate() {
                append_unique_scroll_view(&mut ordered, candidate);
            }
        }
    }
    for candidate in select_default_candidates() {
        if Some(candidate) != get_keyboard_scroll_selected_candidate() {
            append_unique_scroll_view(&mut ordered, candidate);
        }
    }
    ordered
}

pub(crate) fn reset_keyboard_scroll_tracking() {
    SCROLL_VIEWS.with(|scroll_views| scroll_views.borrow_mut().clear());
    SELECTED_SCROLL_VIEW.with(|selected| selected.replace(None));
    SELECTED_BRANCH_ROOT.with(|branch| branch.replace(None));
}
