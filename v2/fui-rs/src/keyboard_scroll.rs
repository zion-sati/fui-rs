use crate::event;

const KEYBOARD_SCROLL_LINE_STEP: f32 = 40.0;
const KEYBOARD_SCROLL_PAGE_OVERLAP: f32 = 40.0;
const KEYBOARD_SCROLL_TOLERANCE: f32 = 0.5;

fn is_keyboard_scroll_key(key: &str) -> bool {
    matches!(
        key,
        "ArrowLeft"
            | "ArrowRight"
            | "ArrowUp"
            | "ArrowDown"
            | "PageUp"
            | "PageDown"
            | "Home"
            | "End"
    )
}

fn is_horizontal_keyboard_scroll_key(key: &str) -> bool {
    matches!(key, "ArrowLeft" | "ArrowRight")
}

fn is_vertical_keyboard_scroll_key(key: &str) -> bool {
    matches!(
        key,
        "ArrowUp" | "ArrowDown" | "PageUp" | "PageDown" | "Home" | "End"
    )
}

fn page_step(viewport_height: f32) -> f32 {
    if viewport_height <= 0.0 {
        return 0.0;
    }
    if viewport_height > KEYBOARD_SCROLL_PAGE_OVERLAP {
        return viewport_height - KEYBOARD_SCROLL_PAGE_OVERLAP;
    }
    viewport_height * 0.875
}

fn clamp(value: f32, min_value: f32, max_value: f32) -> f32 {
    if value < min_value {
        min_value
    } else if value > max_value {
        max_value
    } else {
        value
    }
}

fn try_scroll_viewport(handle: crate::node::NodeHandle, key: &str) -> bool {
    let Some(node) = event::resolve_node(handle) else {
        return false;
    };
    let Some(state) = node.scroll_routing_state() else {
        return false;
    };
    if is_horizontal_keyboard_scroll_key(key) {
        if !state.enabled_x {
            return false;
        }
        let max_offset_x = (state.content_width - state.viewport_width).max(0.0);
        if max_offset_x <= KEYBOARD_SCROLL_TOLERANCE {
            return false;
        }
        let current_offset_x = clamp(state.offset_x, 0.0, max_offset_x);
        let mut next_offset_x = match key {
            "ArrowLeft" => current_offset_x - KEYBOARD_SCROLL_LINE_STEP,
            "ArrowRight" => current_offset_x + KEYBOARD_SCROLL_LINE_STEP,
            _ => return false,
        };
        next_offset_x = clamp(next_offset_x, 0.0, max_offset_x);
        if (next_offset_x - current_offset_x).abs() <= KEYBOARD_SCROLL_TOLERANCE {
            return false;
        }
        crate::bindings::ui::set_scroll_offset(handle.raw(), next_offset_x, state.offset_y);
        node.set_scroll_routing_offsets(next_offset_x, state.offset_y);
        return true;
    }

    if !is_vertical_keyboard_scroll_key(key) || !state.enabled_y {
        return false;
    }
    let max_offset_y = (state.content_height - state.viewport_height).max(0.0);
    if max_offset_y <= KEYBOARD_SCROLL_TOLERANCE {
        return false;
    }
    let current_offset_y = clamp(state.offset_y, 0.0, max_offset_y);
    let mut next_offset_y = match key {
        "ArrowUp" => current_offset_y - KEYBOARD_SCROLL_LINE_STEP,
        "ArrowDown" => current_offset_y + KEYBOARD_SCROLL_LINE_STEP,
        "PageUp" => current_offset_y - page_step(state.viewport_height),
        "PageDown" => current_offset_y + page_step(state.viewport_height),
        "Home" => 0.0,
        "End" => max_offset_y,
        _ => return false,
    };
    next_offset_y = clamp(next_offset_y, 0.0, max_offset_y);
    if (next_offset_y - current_offset_y).abs() <= KEYBOARD_SCROLL_TOLERANCE {
        return false;
    }
    crate::bindings::ui::set_scroll_offset(handle.raw(), state.offset_x, next_offset_y);
    node.set_scroll_routing_offsets(state.offset_x, next_offset_y);
    true
}

pub(crate) fn handle_keyboard_scroll_fallback(key: &str, modifiers: u32) -> bool {
    if modifiers != 0 || !is_keyboard_scroll_key(key) {
        return false;
    }

    if let Some(focused_node) = event::focused_node() {
        let mut current = Some(focused_node);
        while let Some(node) = current {
            if node.is_scroll_view_for_routing() && try_scroll_viewport(node.handle(), key) {
                return true;
            }
            current = node.parent();
        }
    }

    if let Some(selected_candidate) =
        crate::keyboard_scroll_tracker::get_keyboard_scroll_selected_candidate()
    {
        if try_scroll_viewport(selected_candidate, key) {
            return true;
        }
    }

    for candidate in crate::keyboard_scroll_tracker::get_keyboard_scroll_fallback_candidates() {
        if try_scroll_viewport(candidate, key) {
            return true;
        }
    }
    false
}
