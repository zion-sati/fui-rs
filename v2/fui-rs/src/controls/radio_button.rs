use super::internal::pressable_labeled_control::{
    PressableLabeledControlState, WeakPressableLabeledControl,
};
use super::internal::radio_indicator_presenter::{
    create_default_radio_indicator_presenter, RadioIndicatorPresenter, RadioIndicatorTemplate,
    RadioIndicatorVisualState,
};
use super::*;
use crate::bindings::ui;
use crate::controls::radio_group::RadioGroupEventTarget;
use crate::ffi::KeyEventType;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

fn create_indicator_presenter(
    template: Option<Rc<dyn RadioIndicatorTemplate>>,
    sizing: Option<LabeledControlSizing>,
) -> Rc<dyn RadioIndicatorPresenter> {
    if let Some(template) = template {
        return template.create(sizing);
    }
    if let Some(template_set) = get_control_templates() {
        if let Some(template) = template_set.radio_indicator {
            return template.create(sizing);
        }
    }
    create_default_radio_indicator_presenter(sizing)
}

#[derive(Clone)]
pub struct RadioButton {
    base: PressableLabeledControl,
    root: FlexBox,
    indicator_presenter: Rc<RefCell<Rc<dyn RadioIndicatorPresenter>>>,
    template_override: Rc<RefCell<Option<Rc<dyn RadioIndicatorTemplate>>>>,
    sizing_value: Rc<Cell<Option<LabeledControlSizing>>>,
    colors_value: Rc<Cell<Option<LabeledControlColors>>>,
    value_text: String,
    checked: Rc<Cell<bool>>,
    changed: Rc<RefCell<Option<RadioButtonChangedCallback>>>,
    owner_group: Rc<RefCell<Option<WeakRadioGroupEventTarget>>>,
    weak_root: Rc<WeakNodeRef>,
}

impl RadioButton {
    pub fn new(value: impl Into<String>) -> Self {
        let value = value.into();
        Self::with_label(value.clone(), value)
    }

    pub fn with_label(value: impl Into<String>, label: impl Into<String>) -> Self {
        let value = value.into();
        let label = label.into();
        let indicator_presenter = create_indicator_presenter(None, None);
        let base =
            PressableLabeledControl::new(SemanticRole::Radio, label, indicator_presenter.root());
        let root = base.root();
        let weak_root = Rc::new(root.node_ref().downgrade());
        let control = Self {
            base,
            root,
            indicator_presenter: Rc::new(RefCell::new(indicator_presenter)),
            template_override: Rc::new(RefCell::new(None)),
            sizing_value: Rc::new(Cell::new(None)),
            colors_value: Rc::new(Cell::new(None)),
            value_text: value,
            checked: Rc::new(Cell::new(false)),
            changed: Rc::new(RefCell::new(None)),
            owner_group: Rc::new(RefCell::new(None)),
            weak_root,
        };
        control.root.semantic_checked(SemanticCheckedState::False);
        let activation = RadioEventTarget::from_radio(&control);
        control.base.set_activation(move |state| {
            activation.activate(state);
        });
        let state_sync = RadioEventTarget::from_radio(&control);
        control.base.set_state_changed(move |state| {
            state_sync.sync_visual(state, false, false);
        });
        let key_target = RadioEventTarget::from_radio(&control);
        control
            .base
            .set_key_handler(move |event| key_target.handle_key_event(event));
        control.sync_visual(control.base.snapshot_state(), false, false);
        control
    }

    pub fn value(&self) -> &str {
        &self.value_text
    }

    pub fn checked(&self, checked: bool) -> &Self {
        self.check(checked);
        self
    }

    pub fn check(&self, checked: bool) -> &Self {
        self.update_checked(checked, true, false);
        self
    }

    pub fn is_checked(&self) -> bool {
        self.checked.get()
    }

    pub fn sizing(&self, sizing: LabeledControlSizing) -> &Self {
        self.set_sizing(Some(sizing))
    }

    pub fn clear_sizing(&self) -> &Self {
        self.set_sizing(None)
    }

    fn set_sizing(&self, sizing: Option<LabeledControlSizing>) -> &Self {
        self.sizing_value.set(sizing);
        self.base.set_label_font_size_override(
            sizing
                .filter(|sizing| sizing.has_label_font_size())
                .map(|sizing| sizing.label_font_size_px())
                .unwrap_or(0.0),
        );
        self.replace_indicator_presenter(create_indicator_presenter(
            self.template_override.borrow().clone(),
            self.sizing_value.get(),
        ));
        self.sync_visual(self.base.snapshot_state(), false, false);
        self
    }

    pub fn template(&self, template: Rc<dyn RadioIndicatorTemplate>) -> &Self {
        self.set_template(Some(template))
    }

    pub fn clear_template(&self) -> &Self {
        self.set_template(None)
    }

    fn set_template(&self, template: Option<Rc<dyn RadioIndicatorTemplate>>) -> &Self {
        self.template_override.replace(template.clone());
        self.replace_indicator_presenter(create_indicator_presenter(
            template,
            self.sizing_value.get(),
        ));
        self.sync_visual(self.base.snapshot_state(), false, false);
        self
    }

    pub fn colors(&self, colors: LabeledControlColors) -> &Self {
        self.set_colors(Some(colors))
    }

    pub fn clear_colors(&self) -> &Self {
        self.set_colors(None)
    }

    fn set_colors(&self, colors: Option<LabeledControlColors>) -> &Self {
        self.colors_value.set(colors);
        self.base.colors(colors);
        self.sync_visual(self.base.snapshot_state(), false, false);
        self
    }

    pub fn enabled(&self, enabled: bool) -> &Self {
        self.base.enabled(enabled);
        self
    }

    pub fn on_changed(&self, handler: impl Fn(RadioButtonChangedEventArgs) + 'static) -> &Self {
        *self.changed.borrow_mut() = Some(Rc::new(handler));
        self
    }

    pub fn focus_now(&self) {
        ui::request_focus(self.root.retained_node_ref().handle().raw());
    }

    pub(crate) fn bind_group(&self, group: WeakRadioGroupEventTarget) {
        *self.owner_group.borrow_mut() = Some(group);
    }

    pub(crate) fn is_enabled(&self) -> bool {
        self.root.retained_node_ref().is_enabled_for_routing()
    }

    pub(crate) fn update_checked(&self, checked: bool, emit: bool, announce: bool) {
        apply_radio_checked(
            &self.indicator_presenter,
            &self.weak_root,
            &self.checked,
            checked,
            emit,
            announce,
            &self.changed,
            self.colors_value.get(),
            self.base.snapshot_state(),
        );
    }

    fn replace_indicator_presenter(&self, next_presenter: Rc<dyn RadioIndicatorPresenter>) {
        let previous = self.indicator_presenter.borrow().clone();
        if Rc::ptr_eq(&previous, &next_presenter) {
            return;
        }
        self.base.replace_indicator_root(next_presenter.root());
        self.indicator_presenter.replace(next_presenter);
    }

    fn sync_visual(&self, base_state: PressableLabeledControlState, emit: bool, announce: bool) {
        apply_radio_checked(
            &self.indicator_presenter,
            &self.weak_root,
            &self.checked,
            self.checked.get(),
            emit,
            announce,
            &self.changed,
            self.colors_value.get(),
            base_state,
        );
    }
}

impl Clickable for RadioButton {
    fn on_click(&self, handler: impl Fn(ClickEventArgs) + 'static) -> &Self {
        self.base.on_click(handler);
        self
    }
}

impl LabeledControlTextStyle for RadioButton {
    fn set_label_font_family(&self, family: crate::FontFamily) {
        self.base.font_family(family);
    }

    fn set_label_font_size(&self, size: f32) {
        self.base.font_size(size);
    }

    fn set_label_text_color(&self, color: u32) {
        self.base.text_color(color);
    }
}

impl Node for RadioButton {
    fn retained_node_ref(&self) -> NodeRef {
        self.root.retained_node_ref()
    }

    fn build_self(&self) {
        self.sync_visual(self.base.snapshot_state(), false, false);
        self.root.build_self();
    }
}

impl HasFlexBoxRoot for RadioButton {
    fn flex_box_root(&self) -> &FlexBox {
        &self.root
    }
}

impl ThemeBindable for RadioButton {
    fn theme_binding_node(&self) -> NodeRef {
        self.root.retained_node_ref()
    }

    fn weak_theme_target(&self) -> Box<dyn Fn() -> Option<Self>> {
        let target = RadioEventTarget::from_radio(self);
        Box::new(move || target.upgrade())
    }
}

#[derive(Clone)]
struct RadioEventTarget {
    base: WeakPressableLabeledControl,
    presenter: Rc<RefCell<Rc<dyn RadioIndicatorPresenter>>>,
    template_override: Rc<RefCell<Option<Rc<dyn RadioIndicatorTemplate>>>>,
    sizing_value: Rc<Cell<Option<LabeledControlSizing>>>,
    value_text: String,
    checked: Rc<Cell<bool>>,
    colors_value: Rc<Cell<Option<LabeledControlColors>>>,
    changed: Rc<RefCell<Option<RadioButtonChangedCallback>>>,
    owner_group: Rc<RefCell<Option<WeakRadioGroupEventTarget>>>,
    weak_root: Rc<WeakNodeRef>,
}

impl RadioEventTarget {
    fn from_radio(radio: &RadioButton) -> Self {
        Self {
            base: radio.base.downgrade(),
            presenter: radio.indicator_presenter.clone(),
            template_override: radio.template_override.clone(),
            sizing_value: radio.sizing_value.clone(),
            value_text: radio.value_text.clone(),
            checked: radio.checked.clone(),
            colors_value: radio.colors_value.clone(),
            changed: radio.changed.clone(),
            owner_group: radio.owner_group.clone(),
            weak_root: radio.weak_root.clone(),
        }
    }

    fn upgrade(&self) -> Option<RadioButton> {
        let base = self.base.upgrade()?;
        Some(RadioButton {
            root: base.root(),
            base,
            indicator_presenter: self.presenter.clone(),
            template_override: self.template_override.clone(),
            sizing_value: self.sizing_value.clone(),
            colors_value: self.colors_value.clone(),
            value_text: self.value_text.clone(),
            checked: self.checked.clone(),
            changed: self.changed.clone(),
            owner_group: self.owner_group.clone(),
            weak_root: self.weak_root.clone(),
        })
    }

    fn activate(&self, base_state: PressableLabeledControlState) {
        if let Some(group) = self.owner_group() {
            if let Some(handle) = upgraded_handle(&self.weak_root) {
                group.select_radio_handle(handle.raw(), true);
                return;
            }
        }
        apply_radio_checked(
            &self.presenter,
            &self.weak_root,
            &self.checked,
            true,
            true,
            true,
            &self.changed,
            self.colors_value.get(),
            base_state,
        );
    }

    fn handle_key_event(&self, event: &mut KeyEventArgs) -> bool {
        if event.event_type != KeyEventType::Down || event.modifiers != 0 {
            return false;
        }
        let Some(group) = self.owner_group() else {
            return false;
        };
        let Some(handle) = upgraded_handle(&self.weak_root) else {
            return false;
        };
        match event.key.as_str() {
            "ArrowLeft" | "ArrowUp" => {
                group.move_selection_from_handle(handle.raw(), -1);
                true
            }
            "ArrowRight" | "ArrowDown" => {
                group.move_selection_from_handle(handle.raw(), 1);
                true
            }
            "Home" => {
                group.select_first_enabled(true);
                true
            }
            "End" => {
                group.select_last_enabled(true);
                true
            }
            _ => false,
        }
    }

    fn sync_visual(&self, base_state: PressableLabeledControlState, emit: bool, announce: bool) {
        apply_radio_checked(
            &self.presenter,
            &self.weak_root,
            &self.checked,
            self.checked.get(),
            emit,
            announce,
            &self.changed,
            self.colors_value.get(),
            base_state,
        );
    }

    fn owner_group(&self) -> Option<Rc<RadioGroupEventTarget>> {
        self.owner_group
            .borrow()
            .as_ref()
            .and_then(WeakRadioGroupEventTarget::upgrade)
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_radio_checked(
    presenter: &Rc<RefCell<Rc<dyn RadioIndicatorPresenter>>>,
    weak_root: &Rc<WeakNodeRef>,
    checked_cell: &Rc<Cell<bool>>,
    checked: bool,
    emit: bool,
    announce: bool,
    changed: &Rc<RefCell<Option<RadioButtonChangedCallback>>>,
    colors: Option<LabeledControlColors>,
    base_state: PressableLabeledControlState,
) {
    let previous = checked_cell.get();
    checked_cell.set(checked);
    update_semantic_checked(
        weak_root,
        if checked {
            SemanticCheckedState::True
        } else {
            SemanticCheckedState::False
        },
        announce && previous != checked,
    );
    presenter.borrow().apply(
        current_theme(),
        RadioIndicatorVisualState::new(
            checked,
            base_state.hovered,
            base_state.pressed,
            base_state.focused,
            base_state.enabled,
        ),
        colors,
    );
    if emit && previous != checked {
        if let Some(callback) = changed.borrow().clone() {
            callback(RadioButtonChangedEventArgs { checked });
        }
    }
}
