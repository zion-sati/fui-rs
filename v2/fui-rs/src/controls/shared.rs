use super::*;

pub(crate) const PROGRESS_LENGTH: f32 = 220.0;
pub(crate) const PROGRESS_THICKNESS: f32 = 14.0;
pub(crate) const SLIDER_FOCUS_BORDER_WIDTH: f32 = 2.0;
pub(crate) const SLIDER_PADDING: f32 = 2.0;
pub(crate) const SLIDER_CONTENT_INSET: f32 = 1.0;
pub(crate) const SLIDER_OUTER_INSET: f32 =
    SLIDER_FOCUS_BORDER_WIDTH + SLIDER_PADDING + SLIDER_CONTENT_INSET;
pub(crate) const SLIDER_CHILD_INSET: f32 = SLIDER_PADDING + SLIDER_CONTENT_INSET;

pub(crate) type ClickCallback = Rc<dyn Fn(ClickEventArgs)>;
pub(crate) type CheckboxChangedCallback = Rc<dyn Fn(CheckboxChangedEventArgs)>;
pub(crate) type RadioButtonChangedCallback = Rc<dyn Fn(RadioButtonChangedEventArgs)>;
pub(crate) type RadioGroupChangedCallback = Rc<dyn Fn(RadioGroupChangedEventArgs)>;
pub(crate) type SwitchChangedCallback = Rc<dyn Fn(SwitchChangedEventArgs)>;
pub(crate) type SliderChangedCallback = Rc<dyn Fn(SliderChangedEventArgs)>;

pub(crate) fn is_activation_key(event: &KeyEventArgs) -> bool {
    event.key == "Enter" || event.key == " " || event.key == "Space" || event.key == "Spacebar"
}

pub(crate) fn fire_click_callback(click: &Rc<RefCell<Option<ClickCallback>>>) {
    if let Some(callback) = click.borrow().clone() {
        callback(ClickEventArgs);
    }
}

pub(crate) fn normalize_slider_value(value: f32, min: f32, max: f32, step: f32) -> f32 {
    let clamped = value.clamp(min.min(max), min.max(max));
    if step <= 0.0 {
        return clamped;
    }
    let snapped = ((clamped - min) / step).round() * step;
    (min + snapped).clamp(min.min(max), min.max(max))
}

pub(crate) fn upgraded_handle(weak_root: &Rc<WeakNodeRef>) -> Option<crate::node::NodeHandle> {
    weak_root.upgrade().map(|node| node.handle())
}

pub(crate) fn update_semantic_checked(
    weak_root: &Rc<WeakNodeRef>,
    state: SemanticCheckedState,
    announce: bool,
) {
    if let Some(root) = weak_root.upgrade() {
        root.set_semantic_checked(state, announce);
    }
}
