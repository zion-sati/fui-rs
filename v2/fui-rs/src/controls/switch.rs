use super::internal::pressable_labeled_control::{
    PressableLabeledControlState, WeakPressableLabeledControl,
};
use super::internal::switch_indicator_presenter::{
    create_default_switch_indicator_presenter, SwitchIndicatorPresenter, SwitchIndicatorTemplate,
    SwitchIndicatorVisualState,
};
use super::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

fn create_indicator_presenter(
    template: Option<Rc<dyn SwitchIndicatorTemplate>>,
    sizing: Option<LabeledControlSizing>,
) -> Rc<dyn SwitchIndicatorPresenter> {
    if let Some(template) = template {
        return template.create(sizing);
    }
    if let Some(template_set) = get_control_templates() {
        if let Some(template) = template_set.switch_indicator {
            return template.create(sizing);
        }
    }
    create_default_switch_indicator_presenter(sizing)
}

#[derive(Clone)]
pub struct Switch {
    base: PressableLabeledControl,
    root: FlexBox,
    indicator_presenter: Rc<RefCell<Rc<dyn SwitchIndicatorPresenter>>>,
    template_override: Rc<RefCell<Option<Rc<dyn SwitchIndicatorTemplate>>>>,
    sizing_value: Rc<Cell<Option<LabeledControlSizing>>>,
    colors_value: Rc<Cell<Option<LabeledControlColors>>>,
    checked: Rc<Cell<bool>>,
    changed: Rc<RefCell<Option<SwitchChangedCallback>>>,
    weak_root: Rc<WeakNodeRef>,
}

impl Switch {
    pub fn new(label: impl Into<String>) -> Self {
        let label = label.into();
        let indicator_presenter = create_indicator_presenter(None, None);
        let base =
            PressableLabeledControl::new(SemanticRole::Switch, label, indicator_presenter.root());
        let root = base.root();
        let weak_root = Rc::new(root.node_ref().downgrade());
        let control = Self {
            base,
            root,
            indicator_presenter: Rc::new(RefCell::new(indicator_presenter)),
            template_override: Rc::new(RefCell::new(None)),
            sizing_value: Rc::new(Cell::new(None)),
            colors_value: Rc::new(Cell::new(None)),
            checked: Rc::new(Cell::new(false)),
            changed: Rc::new(RefCell::new(None)),
            weak_root,
        };
        control.root.semantic_checked(SemanticCheckedState::False);
        let activation = SwitchEventTarget::from_switch(&control);
        control.base.set_activation(move |state| {
            activation.activate(state);
        });
        let state_sync = SwitchEventTarget::from_switch(&control);
        control.base.set_state_changed(move |state| {
            state_sync.sync_visual(state, false, false);
        });
        let snapshot = control.base.state_snapshotter();
        let presenter = control.indicator_presenter.clone();
        let weak_root = control.weak_root.clone();
        let checked = control.checked.clone();
        let changed = control.changed.clone();
        let colors_value = control.colors_value.clone();
        control.persist_state(crate::persisted::persisted_value_adapter(
            "switch-checked",
            crate::persisted::PersistedBoolCodec,
            1,
            {
                let checked = checked.clone();
                move || Some(checked.get())
            },
            move |next| {
                apply_switch_checked(
                    &presenter,
                    &weak_root,
                    &checked,
                    next,
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
        self.check(checked);
        self
    }

    pub fn check(&self, checked: bool) -> &Self {
        apply_switch_checked(
            &self.indicator_presenter,
            &self.weak_root,
            &self.checked,
            checked,
            true,
            false,
            &self.changed,
            self.colors_value.get(),
            self.base.snapshot_state(),
        );
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

    pub fn template(&self, template: Rc<dyn SwitchIndicatorTemplate>) -> &Self {
        self.set_template(Some(template))
    }

    pub fn clear_template(&self) -> &Self {
        self.set_template(None)
    }

    fn set_template(&self, template: Option<Rc<dyn SwitchIndicatorTemplate>>) -> &Self {
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

    pub fn is_checked(&self) -> bool {
        self.checked.get()
    }

    pub fn enabled(&self, enabled: bool) -> &Self {
        self.base.enabled(enabled);
        self
    }

    pub fn on_changed(&self, handler: impl Fn(SwitchChangedEventArgs) + 'static) -> &Self {
        *self.changed.borrow_mut() = Some(Rc::new(handler));
        self
    }

    fn replace_indicator_presenter(&self, next_presenter: Rc<dyn SwitchIndicatorPresenter>) {
        let previous = self.indicator_presenter.borrow().clone();
        if Rc::ptr_eq(&previous, &next_presenter) {
            return;
        }
        self.base.replace_indicator_root(next_presenter.root());
        self.indicator_presenter.replace(next_presenter);
    }

    fn sync_visual(&self, base_state: PressableLabeledControlState, emit: bool, announce: bool) {
        apply_switch_checked(
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

impl LabeledControlTextStyle for Switch {
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

impl Node for Switch {
    fn retained_node_ref(&self) -> NodeRef {
        self.root.retained_node_ref()
    }

    fn build_self(&self) {
        self.sync_visual(self.base.snapshot_state(), false, false);
        self.root.build_self();
    }
}

impl HasFlexBoxRoot for Switch {
    fn flex_box_root(&self) -> &FlexBox {
        &self.root
    }
}

impl ThemeBindable for Switch {
    fn theme_binding_node(&self) -> NodeRef {
        self.root.retained_node_ref()
    }

    fn weak_theme_target(&self) -> Box<dyn Fn() -> Option<Self>> {
        let target = SwitchEventTarget::from_switch(self);
        Box::new(move || target.upgrade())
    }
}

#[derive(Clone)]
struct SwitchEventTarget {
    base: WeakPressableLabeledControl,
    presenter: Rc<RefCell<Rc<dyn SwitchIndicatorPresenter>>>,
    template_override: Rc<RefCell<Option<Rc<dyn SwitchIndicatorTemplate>>>>,
    sizing_value: Rc<Cell<Option<LabeledControlSizing>>>,
    checked: Rc<Cell<bool>>,
    colors_value: Rc<Cell<Option<LabeledControlColors>>>,
    changed: Rc<RefCell<Option<SwitchChangedCallback>>>,
    weak_root: Rc<WeakNodeRef>,
}

impl SwitchEventTarget {
    fn from_switch(switch: &Switch) -> Self {
        Self {
            base: switch.base.downgrade(),
            presenter: switch.indicator_presenter.clone(),
            template_override: switch.template_override.clone(),
            sizing_value: switch.sizing_value.clone(),
            checked: switch.checked.clone(),
            colors_value: switch.colors_value.clone(),
            changed: switch.changed.clone(),
            weak_root: switch.weak_root.clone(),
        }
    }

    fn upgrade(&self) -> Option<Switch> {
        let base = self.base.upgrade()?;
        Some(Switch {
            root: base.root(),
            base,
            indicator_presenter: self.presenter.clone(),
            template_override: self.template_override.clone(),
            sizing_value: self.sizing_value.clone(),
            colors_value: self.colors_value.clone(),
            checked: self.checked.clone(),
            changed: self.changed.clone(),
            weak_root: self.weak_root.clone(),
        })
    }

    fn activate(&self, base_state: PressableLabeledControlState) {
        apply_switch_checked(
            &self.presenter,
            &self.weak_root,
            &self.checked,
            !self.checked.get(),
            true,
            true,
            &self.changed,
            self.colors_value.get(),
            base_state,
        );
    }

    fn sync_visual(&self, base_state: PressableLabeledControlState, emit: bool, announce: bool) {
        apply_switch_checked(
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
}

fn apply_switch_checked(
    presenter: &Rc<RefCell<Rc<dyn SwitchIndicatorPresenter>>>,
    weak_root: &Rc<WeakNodeRef>,
    checked_cell: &Rc<Cell<bool>>,
    checked: bool,
    emit: bool,
    announce: bool,
    changed: &Rc<RefCell<Option<SwitchChangedCallback>>>,
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
        SwitchIndicatorVisualState::new(
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
            callback(SwitchChangedEventArgs { checked });
        }
    }
}
