use super::internal::button_presenter::{
    ButtonPresenter, ButtonTemplate, ButtonVisualState, DEFAULT_BUTTON_TEMPLATE,
};
use super::*;
use crate::ffi::CursorStyle;
use crate::node::{TextCore, WeakFlexBox};
use crate::signal::SubscriptionGuard;
use crate::{focus_adorner, focus_visibility};
use std::rc::Rc;

#[derive(Clone)]
pub struct Button {
    root: FlexBox,
    presenter: Rc<RefCell<Rc<dyn ButtonPresenter>>>,
    label: Rc<RefCell<TextCore>>,
    template: Rc<RefCell<Option<Rc<dyn ButtonTemplate>>>>,
    label_value: Rc<RefCell<String>>,
    click: Rc<RefCell<Option<ClickCallback>>>,
    double_click: Rc<RefCell<Option<ClickCallback>>>,
    triple_click: Rc<RefCell<Option<ClickCallback>>>,
    hovered_state: Rc<Cell<bool>>,
    pressed_state: Rc<Cell<bool>>,
    focused_state: Rc<Cell<bool>>,
    keyboard_armed_key: Rc<RefCell<Option<String>>>,
    background_override: Rc<Cell<Option<u32>>>,
    hover_background_override: Rc<Cell<Option<u32>>>,
    pressed_background_override: Rc<Cell<Option<u32>>>,
    border_override: Rc<Cell<Option<Border>>>,
    text_color_override: Rc<Cell<Option<u32>>>,
    colors_value: Rc<Cell<Option<ButtonColors>>>,
    theme_guard: Rc<RefCell<Option<SubscriptionGuard>>>,
    focus_visibility_guard: Rc<RefCell<Option<SubscriptionGuard>>>,
}

impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        let label = label.into();
        let root = row();
        let presenter = create_button_presenter(None);
        let label_node = presenter.label_node();
        label_node.text(label.clone());
        root.flex_direction(crate::FlexDirection::Row)
            .justify_content(JustifyContent::Center)
            .align_items(AlignItems::Center)
            .interactive(true)
            .focusable(true, 0)
            .semantic_role(SemanticRole::Button)
            .semantic_label(label.clone())
            .reflect_semantic_disabled_from_enabled()
            .child(&presenter.content_root());

        let control = Self {
            root,
            presenter: Rc::new(RefCell::new(presenter)),
            label: Rc::new(RefCell::new(label_node)),
            template: Rc::new(RefCell::new(None)),
            label_value: Rc::new(RefCell::new(label.clone())),
            click: Rc::new(RefCell::new(None)),
            double_click: Rc::new(RefCell::new(None)),
            triple_click: Rc::new(RefCell::new(None)),
            hovered_state: Rc::new(Cell::new(false)),
            pressed_state: Rc::new(Cell::new(false)),
            focused_state: Rc::new(Cell::new(false)),
            keyboard_armed_key: Rc::new(RefCell::new(None)),
            background_override: Rc::new(Cell::new(None)),
            hover_background_override: Rc::new(Cell::new(None)),
            pressed_background_override: Rc::new(Cell::new(None)),
            border_override: Rc::new(Cell::new(None)),
            text_color_override: Rc::new(Cell::new(None)),
            colors_value: Rc::new(Cell::new(None)),
            theme_guard: Rc::new(RefCell::new(None)),
            focus_visibility_guard: Rc::new(RefCell::new(None)),
        };
        control.install_visual_subscriptions();
        control.sync_visual_state();
        control.sync_focus_chrome();
        control.bind_events();
        control
    }

    fn bind_events(&self) {
        let event_target = self.event_target();
        self.root.on_pointer_enter(move |_event| {
            event_target.set_hovered(true);
        });
        let event_target = self.event_target();
        self.root.on_pointer_leave(move |_event| {
            event_target.set_hovered(false);
            if event_target.keyboard_armed_key.borrow().is_none() {
                event_target.cancel_press();
            }
        });
        let event_target = self.event_target();
        self.root.on_pointer_down(move |_event| {
            event_target.set_hovered(true);
            event_target.begin_press();
        });
        let event_target = self.event_target();
        self.root.on_pointer_up(move |_event| {
            if event_target.pressed_state.get()
                && event_target.keyboard_armed_key.borrow().is_none()
            {
                event_target.end_press();
            }
        });

        let event_target = self.event_target();
        self.root.on_click(move |event| {
            fire_click_callbacks(
                &event_target.click,
                &event_target.double_click,
                &event_target.triple_click,
                event.click_count.max(1),
            );
            event.handled = true;
        });
        let event_target = self.event_target();
        self.root.on_key_down(move |event| {
            event_target.sync_focus_chrome();
            if !event_target.is_enabled() || event.modifiers != 0 || !is_activation_key(event) {
                return;
            }
            if event_target.keyboard_armed_key.borrow().is_some() {
                event.handled = true;
                return;
            }
            *event_target.keyboard_armed_key.borrow_mut() = Some(event.key.clone());
            event_target.begin_press();
            event.handled = true;
        });
        let event_target = self.event_target();
        self.root.on_key_up(move |event| {
            if !event_target.is_enabled() || event.modifiers != 0 || !is_activation_key(event) {
                return;
            }
            let armed = event_target.keyboard_armed_key.borrow().clone();
            if armed.as_deref() == Some(event.key.as_str()) {
                *event_target.keyboard_armed_key.borrow_mut() = None;
                event_target.end_press();
                fire_click_callbacks(
                    &event_target.click,
                    &event_target.double_click,
                    &event_target.triple_click,
                    1,
                );
                event.handled = true;
            }
        });
        let event_target = self.event_target();
        self.root.on_focus_changed(move |event| {
            if !event.focused && event_target.keyboard_armed_key.borrow().is_some() {
                *event_target.keyboard_armed_key.borrow_mut() = None;
                event_target.cancel_press();
            }
            if event_target.focused_state.get() != event.focused {
                event_target.focused_state.set(event.focused);
                event_target.sync_visual_state();
                event_target.sync_focus_chrome();
            }
        });
    }

    pub fn text(&self, value: impl Into<String>) -> &Self {
        let value = value.into();
        self.label_value.replace(value.clone());
        self.label.borrow().text(value.clone());
        self.root.semantic_label(value);
        self
    }

    pub fn template(&self, template: Option<Rc<dyn ButtonTemplate>>) -> &Self {
        self.template.replace(template.clone());
        self.replace_presenter(create_button_presenter(template));
        self
    }

    pub fn colors(&self, colors: Option<ButtonColors>) -> &Self {
        self.colors_value.set(colors);
        self.sync_visual_state();
        self
    }

    pub fn on_click(&self, handler: impl Fn(ClickEventArgs) + 'static) -> &Self {
        *self.click.borrow_mut() = Some(Rc::new(handler));
        self
    }

    pub fn on_double_click(&self, handler: impl Fn(ClickEventArgs) + 'static) -> &Self {
        *self.double_click.borrow_mut() = Some(Rc::new(handler));
        self
    }

    pub fn on_triple_click(&self, handler: impl Fn(ClickEventArgs) + 'static) -> &Self {
        *self.triple_click.borrow_mut() = Some(Rc::new(handler));
        self
    }

    pub fn enabled(&self, enabled: bool) -> &Self {
        self.root.enabled(enabled);
        if !enabled {
            self.hovered_state.set(false);
            self.focused_state.set(false);
            *self.keyboard_armed_key.borrow_mut() = None;
            self.cancel_press();
        }
        self.sync_visual_state();
        self.sync_focus_chrome();
        self
    }

    pub fn corner_radius(&self, radius: f32) -> &Self {
        self.root.corner_radius(radius);
        self.sync_focus_chrome();
        self
    }

    pub fn corners(&self, tl: f32, tr: f32, br: f32, bl: f32) -> &Self {
        self.root.corners(tl, tr, br, bl);
        self.sync_focus_chrome();
        self
    }

    pub fn hover_bg_color(&self, color: u32) -> &Self {
        self.hover_background_override.set(Some(color));
        self.sync_visual_state();
        self
    }

    pub fn pressed_bg_color(&self, color: u32) -> &Self {
        self.pressed_background_override.set(Some(color));
        self.sync_visual_state();
        self
    }

    pub fn bg_color(&self, color: u32) -> &Self {
        self.background_override.set(Some(color));
        self.sync_visual_state();
        self
    }

    pub fn border(&self, width: f32, color: u32) -> &Self {
        self.border_override.set(Some(Border::solid(width, color)));
        self.sync_visual_state();
        self
    }

    pub fn border_config(&self, border: Border) -> &Self {
        self.border_override.set(Some(border));
        self.sync_visual_state();
        self
    }

    pub fn text_color(&self, color: u32) -> &Self {
        self.text_color_override.set(Some(color));
        self.sync_visual_state();
        self
    }

    fn install_visual_subscriptions(&self) {
        let event_target = self.event_target();
        *self.theme_guard.borrow_mut() = Some(subscribe(move |_theme| {
            event_target.sync_visual_state();
            event_target.sync_focus_chrome();
        }));

        let event_target = self.event_target();
        *self.focus_visibility_guard.borrow_mut() =
            Some(focus_visibility::subscribe(move |_visible| {
                event_target.sync_focus_chrome();
            }));
    }

    fn event_target(&self) -> ButtonEventTarget {
        ButtonEventTarget {
            weak_root: self.root.downgrade(),
            presenter: self.presenter.clone(),
            click: self.click.clone(),
            double_click: self.double_click.clone(),
            triple_click: self.triple_click.clone(),
            hovered_state: self.hovered_state.clone(),
            pressed_state: self.pressed_state.clone(),
            focused_state: self.focused_state.clone(),
            keyboard_armed_key: self.keyboard_armed_key.clone(),
            background_override: self.background_override.clone(),
            hover_background_override: self.hover_background_override.clone(),
            pressed_background_override: self.pressed_background_override.clone(),
            border_override: self.border_override.clone(),
            text_color_override: self.text_color_override.clone(),
            colors_value: self.colors_value.clone(),
        }
    }

    pub(crate) fn is_enabled(&self) -> bool {
        self.root.retained_node_ref().is_enabled_for_routing()
    }

    pub(crate) fn begin_press(&self) {
        self.pressed_state.set(true);
        self.sync_visual_state();
        self.sync_focus_chrome();
    }

    pub(crate) fn end_press(&self, activate: bool) {
        self.pressed_state.set(false);
        self.sync_visual_state();
        self.sync_focus_chrome();
        if activate {
            fire_click_callbacks(&self.click, &self.double_click, &self.triple_click, 1);
        }
    }

    pub(crate) fn cancel_press(&self) {
        if self.pressed_state.replace(false) {
            self.sync_visual_state();
        }
        self.sync_focus_chrome();
    }

    fn sync_visual_state(&self) {
        sync_button_visual_state(
            &self.root,
            &self.presenter,
            self.hovered_state.get(),
            self.pressed_state.get(),
            self.focused_state.get(),
            self.is_enabled(),
            self.background_override.get(),
            self.hover_background_override.get(),
            self.pressed_background_override.get(),
            self.border_override.get(),
            self.text_color_override.get(),
            self.colors_value.get(),
        );
        self.root.cursor(if self.is_enabled() {
            CursorStyle::Pointer
        } else {
            CursorStyle::Default
        });
        self.root
            .opacity(if self.is_enabled() { 1.0 } else { 0.38 });
    }

    fn sync_focus_chrome(&self) {
        sync_button_focus_chrome(&self.root, self.focused_state.get(), self.is_enabled());
    }

    fn replace_presenter(&self, next_presenter: Rc<dyn ButtonPresenter>) {
        let previous_presenter = self.presenter.borrow().clone();
        if Rc::ptr_eq(&previous_presenter, &next_presenter) {
            return;
        }
        next_presenter
            .label_node()
            .text(self.label_value.borrow().clone());
        self.root.child(&next_presenter.content_root());
        self.root.remove_child(&previous_presenter.content_root());
        previous_presenter.content_root().dispose();
        self.label.replace(next_presenter.label_node());
        self.presenter.replace(next_presenter);
        self.sync_visual_state();
        self.sync_focus_chrome();
    }
}

impl Node for Button {
    fn retained_node_ref(&self) -> NodeRef {
        self.root.retained_node_ref()
    }

    fn build_self(&self) {
        self.sync_visual_state();
        self.sync_focus_chrome();
        self.root.build_self();
    }
}

impl HasFlexBoxRoot for Button {
    fn flex_box_root(&self) -> &FlexBox {
        &self.root
    }
}

fn sync_button_visual_state(
    root: &FlexBox,
    presenter: &Rc<RefCell<Rc<dyn ButtonPresenter>>>,
    hovered: bool,
    pressed: bool,
    focused: bool,
    enabled: bool,
    background_override: Option<u32>,
    hover_background_override: Option<u32>,
    pressed_background_override: Option<u32>,
    border_override: Option<Border>,
    text_color_override: Option<u32>,
    colors: Option<ButtonColors>,
) {
    let presenter = presenter.borrow().clone();
    presenter.apply(
        root,
        current_theme(),
        ButtonVisualState {
            hovered,
            pressed,
            focused,
            enabled,
        },
        colors,
    );
    let override_background = if !enabled {
        background_override
    } else if pressed {
        pressed_background_override.or(background_override)
    } else if hovered {
        hover_background_override.or(background_override)
    } else {
        background_override
    };
    if let Some(color) = override_background {
        root.bg_color(color);
    }
    if let Some(border) = border_override {
        root.border_config(border);
    }
    if let Some(color) = text_color_override {
        presenter.label_node().text_color(color);
    }
}

fn sync_button_focus_chrome(root: &FlexBox, focused: bool, enabled: bool) {
    if focused && enabled && focus_visibility::keyboard_focus_visible() {
        let radius = current_theme().spacing.sm;
        focus_adorner::show_standard_corners(root, radius, radius, radius, radius);
        return;
    }
    focus_adorner::hide_owner(root);
}

fn create_button_presenter(template: Option<Rc<dyn ButtonTemplate>>) -> Rc<dyn ButtonPresenter> {
    if let Some(template) = template {
        return template.create();
    }
    if let Some(template_set) = get_control_templates() {
        if let Some(template) = template_set.button {
            return template.create();
        }
    }
    DEFAULT_BUTTON_TEMPLATE.create()
}

#[derive(Clone)]
struct ButtonEventTarget {
    weak_root: WeakFlexBox,
    presenter: Rc<RefCell<Rc<dyn ButtonPresenter>>>,
    click: Rc<RefCell<Option<ClickCallback>>>,
    double_click: Rc<RefCell<Option<ClickCallback>>>,
    triple_click: Rc<RefCell<Option<ClickCallback>>>,
    hovered_state: Rc<Cell<bool>>,
    pressed_state: Rc<Cell<bool>>,
    focused_state: Rc<Cell<bool>>,
    keyboard_armed_key: Rc<RefCell<Option<String>>>,
    background_override: Rc<Cell<Option<u32>>>,
    hover_background_override: Rc<Cell<Option<u32>>>,
    pressed_background_override: Rc<Cell<Option<u32>>>,
    border_override: Rc<Cell<Option<Border>>>,
    text_color_override: Rc<Cell<Option<u32>>>,
    colors_value: Rc<Cell<Option<ButtonColors>>>,
}

impl ButtonEventTarget {
    fn is_enabled(&self) -> bool {
        self.weak_root
            .upgrade()
            .is_some_and(|root| root.retained_node_ref().is_enabled_for_routing())
    }

    fn sync_visual_state(&self) {
        let Some(root) = self.weak_root.upgrade() else {
            return;
        };
        sync_button_visual_state(
            &root,
            &self.presenter,
            self.hovered_state.get(),
            self.pressed_state.get(),
            self.focused_state.get(),
            root.retained_node_ref().is_enabled_for_routing(),
            self.background_override.get(),
            self.hover_background_override.get(),
            self.pressed_background_override.get(),
            self.border_override.get(),
            self.text_color_override.get(),
            self.colors_value.get(),
        );
        root.cursor(if root.retained_node_ref().is_enabled_for_routing() {
            CursorStyle::Pointer
        } else {
            CursorStyle::Default
        });
        root.opacity(if root.retained_node_ref().is_enabled_for_routing() {
            1.0
        } else {
            0.38
        });
    }

    fn sync_focus_chrome(&self) {
        let Some(root) = self.weak_root.upgrade() else {
            return;
        };
        sync_button_focus_chrome(
            &root,
            self.focused_state.get(),
            root.retained_node_ref().is_enabled_for_routing(),
        );
    }

    fn begin_press(&self) {
        self.pressed_state.set(true);
        self.sync_visual_state();
        self.sync_focus_chrome();
    }

    fn end_press(&self) {
        self.pressed_state.set(false);
        self.sync_visual_state();
        self.sync_focus_chrome();
    }

    fn cancel_press(&self) {
        if self.pressed_state.replace(false) {
            self.sync_visual_state();
        }
        self.sync_focus_chrome();
    }

    fn set_hovered(&self, hovered: bool) {
        if self.hovered_state.replace(hovered) != hovered {
            self.sync_visual_state();
        }
        self.sync_focus_chrome();
    }
}
