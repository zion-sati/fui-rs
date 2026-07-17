use super::internal::checkbox_indicator_presenter::{
    create_default_checkbox_indicator_presenter, CheckboxIndicatorPresenter,
    CheckboxIndicatorTemplate, CheckboxIndicatorVisualState,
};
use super::internal::pressable_labeled_control::{
    PressableLabeledControlState, WeakPressableLabeledControl,
};
use super::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

fn create_indicator_presenter(
    template: Option<Rc<dyn CheckboxIndicatorTemplate>>,
    sizing: Option<LabeledControlSizing>,
) -> Rc<dyn CheckboxIndicatorPresenter> {
    if let Some(template) = template {
        return template.create(sizing);
    }
    if let Some(template_set) = get_control_templates() {
        if let Some(template) = template_set.checkbox_indicator {
            return template.create(sizing);
        }
    }
    create_default_checkbox_indicator_presenter(sizing)
}

#[derive(Clone)]
pub struct Checkbox {
    base: PressableLabeledControl,
    root: FlexBox,
    indicator_presenter: Rc<RefCell<Rc<dyn CheckboxIndicatorPresenter>>>,
    template_override: Rc<RefCell<Option<Rc<dyn CheckboxIndicatorTemplate>>>>,
    sizing_value: Rc<Cell<Option<LabeledControlSizing>>>,
    state: Rc<Cell<CheckState>>,
    tri_state: Rc<Cell<bool>>,
    colors_value: Rc<Cell<Option<LabeledControlColors>>>,
    changed: Rc<RefCell<Option<CheckboxChangedCallback>>>,
    weak_root: Rc<WeakNodeRef>,
}

impl Checkbox {
    pub fn new(label: impl Into<String>) -> Self {
        let label = label.into();
        let indicator_presenter = create_indicator_presenter(None, None);
        let base =
            PressableLabeledControl::new(SemanticRole::Checkbox, label, indicator_presenter.root());
        let root = base.root();
        let weak_root = Rc::new(root.node_ref().downgrade());
        let control = Self {
            base,
            root,
            indicator_presenter: Rc::new(RefCell::new(indicator_presenter)),
            template_override: Rc::new(RefCell::new(None)),
            sizing_value: Rc::new(Cell::new(None)),
            state: Rc::new(Cell::new(CheckState::False)),
            tri_state: Rc::new(Cell::new(false)),
            colors_value: Rc::new(Cell::new(None)),
            changed: Rc::new(RefCell::new(None)),
            weak_root,
        };
        control.root.semantic_checked(SemanticCheckedState::False);
        let activation = CheckboxEventTarget::from_checkbox(&control);
        control.base.set_activation(move |state| {
            activation.activate(state);
        });
        let state_sync = CheckboxEventTarget::from_checkbox(&control);
        control.base.set_state_changed(move |state| {
            state_sync.sync_visual(state, false, false);
        });
        let snapshot = control.base.state_snapshotter();
        let state = control.state.clone();
        let tri_state = control.tri_state.clone();
        let weak_root = control.weak_root.clone();
        let presenter = control.indicator_presenter.clone();
        let colors_value = control.colors_value.clone();
        let changed = control.changed.clone();
        control.persist_state(crate::persisted::persisted_value_adapter(
            "checkbox-checked-state",
            crate::persisted::PersistedInt32Codec,
            1,
            {
                let state = state.clone();
                move || {
                    Some(match state.get() {
                        CheckState::False => 0,
                        CheckState::True => 1,
                        CheckState::Mixed => 2,
                    })
                }
            },
            move |value| {
                let next = match value {
                    1 => CheckState::True,
                    2 => CheckState::Mixed,
                    _ => CheckState::False,
                };
                let effective = if !tri_state.get() && next == CheckState::Mixed {
                    CheckState::False
                } else {
                    next
                };
                apply_checkbox_state(
                    &presenter,
                    &weak_root,
                    &state,
                    effective,
                    true,
                    false,
                    &changed,
                    colors_value.get(),
                    snapshot(),
                );
            },
        ));
        control.sync_visual(control.base.snapshot_state(), false, false);
        control
    }

    pub fn checked(&self, checked: bool) -> &Self {
        self.check(checked)
    }

    pub fn check(&self, checked: bool) -> &Self {
        self.set_state(
            if checked {
                CheckState::True
            } else {
                CheckState::False
            },
            true,
            false,
        );
        self
    }

    pub fn mixed(&self, mixed: bool) -> &Self {
        if !self.tri_state.get() {
            return self;
        }
        self.set_state(
            if mixed {
                CheckState::Mixed
            } else {
                CheckState::False
            },
            true,
            false,
        );
        self
    }

    pub fn check_state(&self, state: CheckState) -> &Self {
        self.set_state(state, true, false);
        self
    }

    pub fn tri_state(&self, enabled: bool) -> &Self {
        self.tri_state.set(enabled);
        if !enabled && self.state.get() == CheckState::Mixed {
            self.set_state(CheckState::False, true, false);
        }
        self
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

    pub fn template(&self, template: Rc<dyn CheckboxIndicatorTemplate>) -> &Self {
        self.set_template(Some(template))
    }

    pub fn clear_template(&self) -> &Self {
        self.set_template(None)
    }

    fn set_template(&self, template: Option<Rc<dyn CheckboxIndicatorTemplate>>) -> &Self {
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

    pub fn on_changed(&self, handler: impl Fn(CheckboxChangedEventArgs) + 'static) -> &Self {
        *self.changed.borrow_mut() = Some(Rc::new(handler));
        self
    }

    pub fn checked_state(&self) -> CheckState {
        self.state.get()
    }

    pub fn state(&self) -> CheckState {
        self.checked_state()
    }

    pub fn is_checked(&self) -> bool {
        self.state.get().is_checked()
    }

    pub fn enabled(&self, enabled: bool) -> &Self {
        self.base.enabled(enabled);
        self
    }

    #[cfg(test)]
    pub(crate) fn test_parts(&self) -> (FlexBox, FlexBox, FlexBox, FlexBox) {
        self.base.test_parts()
    }

    fn replace_indicator_presenter(&self, next_presenter: Rc<dyn CheckboxIndicatorPresenter>) {
        let previous = self.indicator_presenter.borrow().clone();
        if Rc::ptr_eq(&previous, &next_presenter) {
            return;
        }
        self.base.replace_indicator_root(next_presenter.root());
        self.indicator_presenter.replace(next_presenter);
    }

    fn set_state(&self, state: CheckState, emit: bool, announce: bool) {
        let effective = if !self.tri_state.get() && state == CheckState::Mixed {
            CheckState::False
        } else {
            state
        };
        apply_checkbox_state(
            &self.indicator_presenter,
            &self.weak_root,
            &self.state,
            effective,
            emit,
            announce,
            &self.changed,
            self.colors_value.get(),
            self.base.snapshot_state(),
        );
    }

    fn sync_visual(&self, base_state: PressableLabeledControlState, emit: bool, announce: bool) {
        apply_checkbox_state(
            &self.indicator_presenter,
            &self.weak_root,
            &self.state,
            self.state.get(),
            emit,
            announce,
            &self.changed,
            self.colors_value.get(),
            base_state,
        );
    }
}

impl LabeledControlTextStyle for Checkbox {
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

impl Node for Checkbox {
    fn retained_node_ref(&self) -> NodeRef {
        self.root.retained_node_ref()
    }

    fn build_self(&self) {
        self.sync_visual(self.base.snapshot_state(), false, false);
        self.root.build_self();
    }
}

impl HasFlexBoxRoot for Checkbox {
    fn flex_box_root(&self) -> &FlexBox {
        &self.root
    }
}

impl ThemeBindable for Checkbox {
    fn theme_binding_node(&self) -> NodeRef {
        self.root.retained_node_ref()
    }

    fn weak_theme_target(&self) -> Box<dyn Fn() -> Option<Self>> {
        let target = CheckboxEventTarget::from_checkbox(self);
        Box::new(move || target.upgrade())
    }
}

#[derive(Clone)]
struct CheckboxEventTarget {
    base: WeakPressableLabeledControl,
    presenter: Rc<RefCell<Rc<dyn CheckboxIndicatorPresenter>>>,
    template_override: Rc<RefCell<Option<Rc<dyn CheckboxIndicatorTemplate>>>>,
    sizing_value: Rc<Cell<Option<LabeledControlSizing>>>,
    state: Rc<Cell<CheckState>>,
    tri_state: Rc<Cell<bool>>,
    colors_value: Rc<Cell<Option<LabeledControlColors>>>,
    changed: Rc<RefCell<Option<CheckboxChangedCallback>>>,
    weak_root: Rc<WeakNodeRef>,
}

impl CheckboxEventTarget {
    fn from_checkbox(checkbox: &Checkbox) -> Self {
        Self {
            base: checkbox.base.downgrade(),
            presenter: checkbox.indicator_presenter.clone(),
            template_override: checkbox.template_override.clone(),
            sizing_value: checkbox.sizing_value.clone(),
            state: checkbox.state.clone(),
            tri_state: checkbox.tri_state.clone(),
            colors_value: checkbox.colors_value.clone(),
            changed: checkbox.changed.clone(),
            weak_root: checkbox.weak_root.clone(),
        }
    }

    fn upgrade(&self) -> Option<Checkbox> {
        let base = self.base.upgrade()?;
        Some(Checkbox {
            root: base.root(),
            base,
            indicator_presenter: self.presenter.clone(),
            template_override: self.template_override.clone(),
            sizing_value: self.sizing_value.clone(),
            state: self.state.clone(),
            tri_state: self.tri_state.clone(),
            colors_value: self.colors_value.clone(),
            changed: self.changed.clone(),
            weak_root: self.weak_root.clone(),
        })
    }

    fn activate(&self, base_state: PressableLabeledControlState) {
        let next = match (self.state.get(), self.tri_state.get()) {
            (CheckState::False, _) => CheckState::True,
            (CheckState::True, true) => CheckState::Mixed,
            (CheckState::Mixed, _) | (CheckState::True, false) => CheckState::False,
        };
        apply_checkbox_state(
            &self.presenter,
            &self.weak_root,
            &self.state,
            next,
            true,
            true,
            &self.changed,
            self.colors_value.get(),
            base_state,
        );
    }

    fn sync_visual(&self, base_state: PressableLabeledControlState, emit: bool, announce: bool) {
        apply_checkbox_state(
            &self.presenter,
            &self.weak_root,
            &self.state,
            self.state.get(),
            emit,
            announce,
            &self.changed,
            self.colors_value.get(),
            base_state,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_checkbox_state(
    presenter: &Rc<RefCell<Rc<dyn CheckboxIndicatorPresenter>>>,
    weak_root: &Rc<WeakNodeRef>,
    state_cell: &Rc<Cell<CheckState>>,
    state: CheckState,
    emit: bool,
    announce: bool,
    changed: &Rc<RefCell<Option<CheckboxChangedCallback>>>,
    colors: Option<LabeledControlColors>,
    base_state: PressableLabeledControlState,
) {
    let previous = state_cell.get();
    state_cell.set(state);
    update_semantic_checked(weak_root, state.semantic(), announce && previous != state);
    presenter.borrow().apply(
        current_theme(),
        CheckboxIndicatorVisualState::new(
            state.semantic(),
            base_state.hovered,
            base_state.pressed,
            base_state.focused,
            base_state.enabled,
        ),
        colors,
    );
    if emit && previous != state {
        if let Some(callback) = changed.borrow().clone() {
            callback(CheckboxChangedEventArgs {
                state,
                checked: state.is_checked(),
            });
        }
    }
}
