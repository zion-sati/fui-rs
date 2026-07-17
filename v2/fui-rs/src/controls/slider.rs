use super::internal::slider_presenter::{
    create_default_slider_presenter, SliderPresenter, SliderTemplate, SliderVisualState,
};
use super::*;
use crate::logger;
use crate::node::WeakFlexBox;
use crate::{focus_adorner, focus_visibility};

const DEFAULT_SLIDER_LENGTH: f32 = 180.0;

fn clamp(value: f32, min: f32, max: f32) -> f32 {
    if value < min {
        return min;
    }
    if value > max {
        return max;
    }
    value
}

fn create_slider_presenter(
    template: Option<Rc<dyn SliderTemplate>>,
    sizing: Option<SliderSizing>,
) -> Rc<dyn SliderPresenter> {
    if let Some(template) = template {
        return template.create(sizing);
    }
    if let Some(template_set) = get_control_templates() {
        if let Some(template) = template_set.slider {
            return template.create(sizing);
        }
    }
    create_default_slider_presenter(sizing)
}

#[derive(Clone)]
pub struct Slider {
    root: FlexBox,
    slider_presenter: Rc<RefCell<Rc<dyn SliderPresenter>>>,
    template_override: Rc<RefCell<Option<Rc<dyn SliderTemplate>>>>,
    sizing_value: Rc<Cell<Option<SliderSizing>>>,
    colors_value: Rc<Cell<Option<SliderColors>>>,
    min: Rc<Cell<f32>>,
    max: Rc<Cell<f32>>,
    step: Rc<Cell<f32>>,
    value: Rc<Cell<f32>>,
    length: Rc<Cell<f32>>,
    orientation: Rc<Cell<Orientation>>,
    changed: Rc<RefCell<Option<SliderChangedCallback>>>,
    hovered_state: Rc<Cell<bool>>,
    dragging_state: Rc<Cell<bool>>,
    focused_state: Rc<Cell<bool>>,
    weak_root: Rc<WeakNodeRef>,
    weak_flex_root: WeakFlexBox,
}

impl Default for Slider {
    fn default() -> Self {
        Self::new()
    }
}

impl Slider {
    pub fn new() -> Self {
        let root = flex_box();
        let slider_presenter = create_slider_presenter(None, None);
        let presenter_root = slider_presenter.root();
        presenter_root.position_type(PositionType::Absolute);

        root.interactive(true)
            .focusable(true, 0)
            .semantic_role(SemanticRole::Slider)
            .reflect_semantic_disabled_from_enabled()
            .semantic_orientation(Orientation::Horizontal)
            .semantic_value_range(0.0, 0.0, 100.0)
            .default_semantic_label("Slider")
            .cursor(CursorStyle::Pointer)
            .child(&presenter_root);

        let control = Self {
            weak_root: Rc::new(root.node_ref().downgrade()),
            weak_flex_root: root.downgrade(),
            root,
            slider_presenter: Rc::new(RefCell::new(slider_presenter)),
            template_override: Rc::new(RefCell::new(None)),
            sizing_value: Rc::new(Cell::new(None)),
            colors_value: Rc::new(Cell::new(None)),
            min: Rc::new(Cell::new(0.0)),
            max: Rc::new(Cell::new(100.0)),
            step: Rc::new(Cell::new(1.0)),
            value: Rc::new(Cell::new(0.0)),
            length: Rc::new(Cell::new(DEFAULT_SLIDER_LENGTH)),
            orientation: Rc::new(Cell::new(Orientation::Horizontal)),
            changed: Rc::new(RefCell::new(None)),
            hovered_state: Rc::new(Cell::new(false)),
            dragging_state: Rc::new(Cell::new(false)),
            focused_state: Rc::new(Cell::new(false)),
        };
        control.install_visual_subscriptions();
        control.bind_events();
        let target = SliderEventTarget::from_slider(&control);
        control.persist_state(crate::persisted::persisted_value_adapter(
            "slider-value",
            crate::persisted::PersistedFloat32Codec,
            1,
            {
                let value = control.value.clone();
                move || Some(value.get())
            },
            move |value| {
                target.apply_persisted_value(value);
            },
        ));
        control.set_value_inner(0.0, false, false);
        control.handle_theme_changed();
        control
    }

    fn bind_events(&self) {
        let target = SliderEventTarget::from_slider(self);
        self.root.on_pointer_enter(move |_event| {
            target.handle_pointer_enter();
        });

        let target = SliderEventTarget::from_slider(self);
        self.root.on_pointer_leave(move |_event| {
            target.handle_pointer_leave();
        });

        let target = SliderEventTarget::from_slider(self);
        self.root.on_pointer_down(move |event| {
            target.handle_pointer_down(event);
        });

        let target = SliderEventTarget::from_slider(self);
        self.root.on_pointer_move(move |event| {
            target.handle_pointer_move(event);
        });

        let target = SliderEventTarget::from_slider(self);
        self.root.on_pointer_up(move |event| {
            target.handle_pointer_up(event);
        });

        let target = SliderEventTarget::from_slider(self);
        self.root.on_key_down(move |event| {
            target.sync_focus_chrome();
            target.key_value(event);
        });

        let target = SliderEventTarget::from_slider(self);
        self.root.on_focus_changed(move |event| {
            if target.focused_state.get() != event.focused {
                target.focused_state.set(event.focused);
                target.handle_theme_changed();
            }
        });
    }

    pub fn min(&self, value: f32) -> &Self {
        self.min.set(value);
        if self.max.get() < value {
            self.max.set(value);
        }
        self.set_value_inner(self.value.get(), true, false);
        self
    }

    pub fn max(&self, value: f32) -> &Self {
        self.max.set(value);
        if self.min.get() > value {
            self.min.set(value);
        }
        self.set_value_inner(self.value.get(), true, false);
        self
    }

    pub fn step(&self, value: f32) -> &Self {
        if value <= 0.0 {
            logger::warn(
                "Layout",
                &format!("Slider.step() received {value}; clamping to 1.0."),
            );
        }
        self.step.set(if value > 0.0 { value } else { 1.0 });
        self.set_value_inner(self.value.get(), true, false);
        self
    }

    pub fn value(&self, value: f32) -> &Self {
        self.set_value_inner(value, true, false);
        self
    }

    pub fn length(&self, value: f32) -> &Self {
        let thumb_size = self.slider_presenter.borrow().metrics().thumb_size;
        if value <= thumb_size {
            logger::warn(
                "Layout",
                &format!(
                    "Slider.length() received {value}; clamping to a value above the thumb size."
                ),
            );
        }
        self.length.set(if value > thumb_size {
            value
        } else {
            thumb_size + 1.0
        });
        self.sync_presentation();
        self
    }

    pub fn orientation(&self, orientation: Orientation) -> &Self {
        let orientation = if orientation == Orientation::Vertical {
            Orientation::Vertical
        } else {
            Orientation::Horizontal
        };
        self.orientation.set(orientation);
        self.root.semantic_orientation(orientation);
        self.sync_semantic_label();
        self.sync_presentation();
        self
    }

    pub fn sizing(&self, sizing: SliderSizing) -> &Self {
        self.set_sizing(Some(sizing))
    }

    pub fn clear_sizing(&self) -> &Self {
        self.set_sizing(None)
    }

    fn set_sizing(&self, sizing: Option<SliderSizing>) -> &Self {
        self.sizing_value.set(sizing);
        self.replace_presenter(create_slider_presenter(
            self.template_override.borrow().clone(),
            self.sizing_value.get(),
        ));
        let thumb_size = self.slider_presenter.borrow().metrics().thumb_size;
        if self.length.get() <= thumb_size {
            self.length.set(thumb_size + 1.0);
        }
        self.sync_presentation();
        self
    }

    pub fn colors(&self, colors: SliderColors) -> &Self {
        self.set_colors(Some(colors))
    }

    pub fn clear_colors(&self) -> &Self {
        self.set_colors(None)
    }

    fn set_colors(&self, colors: Option<SliderColors>) -> &Self {
        self.colors_value.set(colors);
        self.sync_presentation();
        self
    }

    pub fn template(&self, template: Rc<dyn SliderTemplate>) -> &Self {
        self.set_template(Some(template))
    }

    pub fn clear_template(&self) -> &Self {
        self.set_template(None)
    }

    fn set_template(&self, template: Option<Rc<dyn SliderTemplate>>) -> &Self {
        self.template_override.replace(template.clone());
        self.replace_presenter(create_slider_presenter(template, self.sizing_value.get()));
        let thumb_size = self.slider_presenter.borrow().metrics().thumb_size;
        if self.length.get() <= thumb_size {
            logger::warn(
                "Layout",
                "Slider.template() increased the thumb size beyond the current slider length; clamping length to stay interactive.",
            );
            self.length.set(thumb_size + 1.0);
        }
        self.sync_presentation();
        self
    }

    pub fn enabled(&self, enabled: bool) -> &Self {
        self.root.enabled(enabled);
        if !enabled && self.dragging_state.replace(false) {
            if let Some(handle) = upgraded_handle(&self.weak_root) {
                crate::event::release_pointer(handle);
                unsafe { crate::ffi::fui_release_pointer_capture() };
            }
        }
        self.handle_theme_changed();
        self
    }

    pub fn on_changed(&self, handler: impl Fn(SliderChangedEventArgs) + 'static) -> &Self {
        *self.changed.borrow_mut() = Some(Rc::new(handler));
        self
    }

    pub fn current_value(&self) -> f32 {
        self.value.get()
    }

    fn install_visual_subscriptions(&self) {
        let target = SliderEventTarget::from_slider(self);
        let theme_guard = subscribe(move |_theme| {
            target.handle_theme_changed();
        });
        self.root
            .retained_node_ref()
            .retain_attachment(Rc::new(theme_guard));

        let target = SliderEventTarget::from_slider(self);
        let focus_guard = focus_visibility::subscribe(move |_visible| {
            target.sync_focus_chrome();
        });
        self.root
            .retained_node_ref()
            .retain_attachment(Rc::new(focus_guard));
    }

    fn set_value_inner(&self, value: f32, emit: bool, announce: bool) {
        SliderEventTarget::from_slider(self).set_value_inner(value, emit, announce);
    }

    fn create_visual_state(&self) -> SliderVisualState {
        create_slider_visual_state(
            self.value.get(),
            self.min.get(),
            self.max.get(),
            self.orientation.get(),
            self.hovered_state.get(),
            self.dragging_state.get(),
            self.focused_state.get(),
            self.is_enabled(),
        )
    }

    fn sync_presentation(&self) {
        let state = self.create_visual_state();
        sync_slider_geometry_with_state(
            &self.root,
            self.slider_presenter.borrow().as_ref(),
            state,
            self.length.get(),
        );
        sync_slider_visual_state_with_state(
            &self.root,
            self.slider_presenter.borrow().as_ref(),
            state,
            self.colors_value.get(),
        );
    }

    fn sync_semantic_label(&self) {
        self.root
            .default_semantic_label(if self.orientation.get() == Orientation::Vertical {
                "Vertical slider"
            } else {
                "Slider"
            });
    }

    fn handle_theme_changed(&self) {
        self.root.cursor(if self.is_enabled() {
            CursorStyle::Pointer
        } else {
            CursorStyle::Default
        });
        self.sync_presentation();
        self.sync_focus_chrome();
    }

    fn is_enabled(&self) -> bool {
        self.root.retained_node_ref().is_enabled_for_routing()
    }

    fn sync_focus_chrome(&self) {
        sync_slider_focus_chrome(&self.root, self.focused_state.get(), self.is_enabled());
    }

    fn replace_presenter(&self, next_presenter: Rc<dyn SliderPresenter>) {
        let previous = self.slider_presenter.borrow().clone();
        if Rc::ptr_eq(&previous, &next_presenter) {
            return;
        }
        let previous_root = previous.root();
        let next_root = next_presenter.root();
        next_root.position_type(PositionType::Absolute);
        self.root.remove_child(&previous_root);
        self.root.child(&next_root);
        previous_root.dispose();
        self.slider_presenter.replace(next_presenter);
    }
}

impl Node for Slider {
    fn retained_node_ref(&self) -> NodeRef {
        self.root.retained_node_ref()
    }

    fn build_self(&self) {
        self.sync_presentation();
        self.sync_focus_chrome();
        self.root.build_self();
    }
}

impl HasFlexBoxRoot for Slider {
    fn flex_box_root(&self) -> &FlexBox {
        &self.root
    }
}

impl ThemeBindable for Slider {
    fn theme_binding_node(&self) -> NodeRef {
        self.root.retained_node_ref()
    }

    fn weak_theme_target(&self) -> Box<dyn Fn() -> Option<Self>> {
        let target = SliderEventTarget::from_slider(self);
        Box::new(move || target.upgrade())
    }
}

#[derive(Clone)]
struct SliderEventTarget {
    weak_root: Rc<WeakNodeRef>,
    weak_flex_root: WeakFlexBox,
    slider_presenter: Rc<RefCell<Rc<dyn SliderPresenter>>>,
    template_override: Rc<RefCell<Option<Rc<dyn SliderTemplate>>>>,
    sizing_value: Rc<Cell<Option<SliderSizing>>>,
    min: Rc<Cell<f32>>,
    max: Rc<Cell<f32>>,
    step: Rc<Cell<f32>>,
    value: Rc<Cell<f32>>,
    length: Rc<Cell<f32>>,
    orientation: Rc<Cell<Orientation>>,
    changed: Rc<RefCell<Option<SliderChangedCallback>>>,
    focused_state: Rc<Cell<bool>>,
    hovered_state: Rc<Cell<bool>>,
    dragging_state: Rc<Cell<bool>>,
    colors_value: Rc<Cell<Option<SliderColors>>>,
}

impl SliderEventTarget {
    fn from_slider(slider: &Slider) -> Self {
        Self {
            weak_root: slider.weak_root.clone(),
            weak_flex_root: slider.weak_flex_root.clone(),
            slider_presenter: slider.slider_presenter.clone(),
            template_override: slider.template_override.clone(),
            sizing_value: slider.sizing_value.clone(),
            min: slider.min.clone(),
            max: slider.max.clone(),
            step: slider.step.clone(),
            value: slider.value.clone(),
            length: slider.length.clone(),
            orientation: slider.orientation.clone(),
            changed: slider.changed.clone(),
            focused_state: slider.focused_state.clone(),
            hovered_state: slider.hovered_state.clone(),
            dragging_state: slider.dragging_state.clone(),
            colors_value: slider.colors_value.clone(),
        }
    }

    fn upgrade(&self) -> Option<Slider> {
        Some(Slider {
            root: self.weak_flex_root.upgrade()?,
            slider_presenter: self.slider_presenter.clone(),
            template_override: self.template_override.clone(),
            sizing_value: self.sizing_value.clone(),
            colors_value: self.colors_value.clone(),
            min: self.min.clone(),
            max: self.max.clone(),
            step: self.step.clone(),
            value: self.value.clone(),
            length: self.length.clone(),
            orientation: self.orientation.clone(),
            changed: self.changed.clone(),
            hovered_state: self.hovered_state.clone(),
            dragging_state: self.dragging_state.clone(),
            focused_state: self.focused_state.clone(),
            weak_root: self.weak_root.clone(),
            weak_flex_root: self.weak_flex_root.clone(),
        })
    }

    fn is_enabled(&self) -> bool {
        self.weak_flex_root
            .upgrade()
            .map(|root| root.retained_node_ref().is_enabled_for_routing())
            .unwrap_or(false)
    }

    fn handle_pointer_enter(&self) {
        if !self.is_enabled() {
            return;
        }
        self.hovered_state.set(true);
        self.sync_presentation();
    }

    fn handle_pointer_leave(&self) {
        if !self.is_enabled() {
            return;
        }
        self.hovered_state.set(false);
        if !self.dragging_state.get() {
            self.sync_presentation();
        }
    }

    fn handle_pointer_down(&self, event: &mut PointerEventArgs) {
        if !self.is_enabled() {
            return;
        }
        self.hovered_state.set(true);
        self.dragging_state.set(true);
        self.update_value_from_pointer(event.scene_x, event.scene_y, true, true);
        self.sync_presentation();
        event.capture_pointer();
        event.handled = true;
    }

    fn handle_pointer_move(&self, event: &mut PointerEventArgs) {
        if !self.is_enabled() || !self.dragging_state.get() || event.buttons == 0 {
            return;
        }
        self.update_value_from_pointer(event.scene_x, event.scene_y, true, true);
        event.handled = true;
    }

    fn handle_pointer_up(&self, event: &mut PointerEventArgs) {
        if !self.is_enabled() || !self.dragging_state.get() {
            return;
        }
        self.update_value_from_pointer(event.scene_x, event.scene_y, true, true);
        self.dragging_state.set(false);
        self.sync_presentation();
        event.release_pointer_capture();
        event.handled = true;
    }

    fn key_value(&self, event: &mut KeyEventArgs) {
        if !self.is_enabled() || event.modifiers != 0 {
            return;
        }
        let mut next = self.value.get();
        match event.key.as_str() {
            "Home" => next = self.min.get(),
            "End" => next = self.max.get(),
            "ArrowUp" if self.orientation.get() == Orientation::Vertical => next += self.step.get(),
            "ArrowDown" if self.orientation.get() == Orientation::Vertical => {
                next -= self.step.get()
            }
            "ArrowRight" if self.orientation.get() == Orientation::Horizontal => {
                next += self.step.get()
            }
            "ArrowLeft" if self.orientation.get() == Orientation::Horizontal => {
                next -= self.step.get()
            }
            _ => return,
        }
        self.set_value_inner(next, true, true);
        event.handled = true;
    }

    fn apply_persisted_value(&self, value: f32) {
        self.set_value_inner(value, true, false);
    }

    fn set_value_inner(&self, next: f32, emit: bool, announce: bool) {
        let normalized =
            normalize_slider_value(next, self.min.get(), self.max.get(), self.step.get());
        if (normalized - self.value.get()).abs() <= f32::EPSILON {
            self.sync_presentation();
            return;
        }
        self.value.set(normalized);
        if let Some(root) = self.weak_flex_root.upgrade() {
            root.semantic_value_range(normalized, self.min.get(), self.max.get());
            if announce && root.has_built_handle() {
                ui::request_semantic_announcement(root.handle().raw());
            }
        }
        self.sync_presentation();
        if emit {
            if let Some(callback) = self.changed.borrow().clone() {
                callback(SliderChangedEventArgs { value: normalized });
            }
        }
    }

    fn update_value_from_pointer(&self, x: f32, y: f32, emit: bool, announce: bool) {
        let thumb_size = self.slider_presenter.borrow().metrics().thumb_size;
        let available = self.length.get() - thumb_size;
        if available <= 0.0 {
            return;
        }
        let (mut local_x, mut local_y) = (x, y);
        if let Some(handle) = upgraded_handle(&self.weak_root) {
            if let Some(bounds) = ui::get_bounds(handle.raw()) {
                local_x = x - bounds[0];
                local_y = y - bounds[1];
            }
        }
        let leading_inset = SLIDER_OUTER_INSET + (thumb_size * 0.5);
        let offset = if self.orientation.get() == Orientation::Vertical {
            available - clamp(local_y - leading_inset, 0.0, available)
        } else {
            clamp(local_x - leading_inset, 0.0, available)
        };
        let fraction = clamp(offset / available, 0.0, 1.0);
        let next = self.min.get() + ((self.max.get() - self.min.get()) * fraction);
        self.set_value_inner(next, emit, announce);
    }

    fn create_visual_state(&self) -> SliderVisualState {
        create_slider_visual_state(
            self.value.get(),
            self.min.get(),
            self.max.get(),
            self.orientation.get(),
            self.hovered_state.get(),
            self.dragging_state.get(),
            self.focused_state.get(),
            self.is_enabled(),
        )
    }

    fn sync_presentation(&self) {
        let Some(root) = self.weak_flex_root.upgrade() else {
            return;
        };
        let presenter = self.slider_presenter.borrow().clone();
        let state = self.create_visual_state();
        sync_slider_geometry_with_state(&root, presenter.as_ref(), state, self.length.get());
        sync_slider_visual_state_with_state(
            &root,
            presenter.as_ref(),
            state,
            self.colors_value.get(),
        );
    }

    fn handle_theme_changed(&self) {
        let Some(root) = self.weak_flex_root.upgrade() else {
            return;
        };
        root.cursor(if self.is_enabled() {
            CursorStyle::Pointer
        } else {
            CursorStyle::Default
        });
        self.sync_presentation();
        self.sync_focus_chrome();
    }

    fn sync_focus_chrome(&self) {
        let Some(root) = self.weak_flex_root.upgrade() else {
            return;
        };
        sync_slider_focus_chrome(
            &root,
            self.focused_state.get(),
            root.retained_node_ref().is_enabled_for_routing(),
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn create_slider_visual_state(
    value: f32,
    min: f32,
    max: f32,
    orientation: Orientation,
    hovered: bool,
    dragging: bool,
    focused: bool,
    enabled: bool,
) -> SliderVisualState {
    let normalized_value = if max > min {
        clamp((value - min) / (max - min), 0.0, 1.0)
    } else {
        0.0
    };
    SliderVisualState::new(
        value,
        min,
        max,
        normalized_value,
        orientation,
        hovered,
        dragging,
        focused,
        enabled,
    )
}

fn sync_slider_geometry_with_state(
    root: &FlexBox,
    presenter: &dyn SliderPresenter,
    state: SliderVisualState,
    length: f32,
) {
    let metrics = presenter.metrics();
    if state.orientation == Orientation::Vertical {
        root.width(
            metrics.thumb_size + (SLIDER_OUTER_INSET * 2.0) + metrics.cross_axis_extra,
            Unit::Pixel,
        )
        .height(length + (SLIDER_OUTER_INSET * 2.0), Unit::Pixel);
    } else {
        root.width(length + (SLIDER_OUTER_INSET * 2.0), Unit::Pixel)
            .height(
                metrics.thumb_size + (SLIDER_OUTER_INSET * 2.0) + metrics.cross_axis_extra,
                Unit::Pixel,
            );
    }
    presenter
        .root()
        .position_type(PositionType::Absolute)
        .position(SLIDER_CHILD_INSET, SLIDER_CHILD_INSET);
    presenter.layout(state, length);
    root.semantic_value_range(state.value, state.min, state.max);
    root.default_semantic_label(if state.orientation == Orientation::Vertical {
        "Vertical slider"
    } else {
        "Slider"
    });
}

fn sync_slider_visual_state_with_state(
    root: &FlexBox,
    presenter: &dyn SliderPresenter,
    state: SliderVisualState,
    colors: Option<SliderColors>,
) {
    let theme = current_theme();
    root.corner_radius(theme.spacing.sm)
        .border(SLIDER_FOCUS_BORDER_WIDTH, theme.colors.background)
        .padding(
            SLIDER_PADDING,
            SLIDER_PADDING,
            SLIDER_PADDING,
            SLIDER_PADDING,
        )
        .opacity(if state.enabled { 1.0 } else { 0.6 });
    presenter.apply(theme, state, colors);
}

fn sync_slider_focus_chrome(root: &FlexBox, focused: bool, enabled: bool) {
    if focused && enabled && focus_visibility::keyboard_focus_visible() {
        focus_adorner::show_standard(root, current_theme().spacing.sm);
        return;
    }
    focus_adorner::hide_owner(root);
}
