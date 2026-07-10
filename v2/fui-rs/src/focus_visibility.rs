use crate::ffi::{KeyEventType, KeyModifier, PointerEventType};
use crate::signal::{Callback, Signal, SubscriptionGuard};
use std::cell::RefCell;
use std::rc::Rc;

struct FocusVisibilityState {
    signal: Signal<bool>,
}

thread_local! {
    static FOCUS_VISIBILITY_STATE: RefCell<FocusVisibilityState> = RefCell::new(FocusVisibilityState {
        signal: Signal::new(true),
    });
}

fn current() -> bool {
    FOCUS_VISIBILITY_STATE.with(|slot| slot.borrow().signal.get())
}

fn set(next: bool) {
    let callbacks = FOCUS_VISIBILITY_STATE.with(|slot| slot.borrow_mut().signal.set(next));
    if let Some(callbacks) = callbacks {
        for callback in callbacks {
            callback();
        }
    }
}

fn is_modifier_key(key: &str) -> bool {
    matches!(key, "Shift" | "Control" | "Alt" | "Meta")
}

fn has_non_shift_modifier(modifiers: u32) -> bool {
    (modifiers
        & ((KeyModifier::Ctrl as u32) | (KeyModifier::Alt as u32) | (KeyModifier::Meta as u32)))
        != 0
}

fn is_caret_navigation_key(key: &str) -> bool {
    matches!(
        key,
        "ArrowLeft"
            | "ArrowRight"
            | "ArrowUp"
            | "ArrowDown"
            | "Home"
            | "End"
            | "PageUp"
            | "PageDown"
    )
}

pub(crate) fn keyboard_focus_visible() -> bool {
    current()
}

pub(crate) fn subscribe(handler: impl Fn(bool) + 'static) -> SubscriptionGuard {
    handler(current());
    FOCUS_VISIBILITY_STATE.with(|slot| {
        let callback: Callback = Rc::new(move || handler(current()));
        slot.borrow_mut().signal.subscribe(callback)
    })
}

pub(crate) fn show_keyboard_focus_for_pointer_event(event_type: PointerEventType) {
    if event_type == PointerEventType::Down {
        set(false);
    }
}

pub fn show_keyboard_focus_for_key_event(event_type: KeyEventType, key: &str, modifiers: u32) {
    if event_type != KeyEventType::Down {
        return;
    }
    if is_modifier_key(key) {
        return;
    }
    if has_non_shift_modifier(modifiers) {
        return;
    }
    if is_caret_navigation_key(key) {
        return;
    }
    if key == "Escape" {
        set(false);
        return;
    }
    set(true);
}

pub(crate) fn reset_keyboard_focus_visibility() {
    set(true);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pointer_down_hides_keyboard_focus_visibility() {
        reset_keyboard_focus_visibility();
        show_keyboard_focus_for_pointer_event(PointerEventType::Down);
        assert!(!keyboard_focus_visible());
    }

    #[test]
    fn regular_key_shows_keyboard_focus_visibility() {
        reset_keyboard_focus_visibility();
        show_keyboard_focus_for_pointer_event(PointerEventType::Down);
        show_keyboard_focus_for_key_event(KeyEventType::Down, "Tab", 0);
        assert!(keyboard_focus_visible());
    }

    #[test]
    fn modifier_key_does_not_show_keyboard_focus_visibility() {
        reset_keyboard_focus_visibility();
        show_keyboard_focus_for_pointer_event(PointerEventType::Down);
        show_keyboard_focus_for_key_event(KeyEventType::Down, "Shift", 0);
        assert!(!keyboard_focus_visible());
    }
}
