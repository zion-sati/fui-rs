use super::DialogAppearance;
use super::{Button, Clickable, Form};
use crate::app;
use crate::bindings::ui;
use crate::ffi::{
    AlignItems, BorderStyle, FlexDirection, JustifyContent, PositionType, SemanticRole, Unit,
};
use crate::node::{
    flex_box, portal, Border, ChildContainerSurface, FlexBox, Node, NodeRef, TextNode,
    ThemeBindable, WeakFlexBox,
};
use crate::theme::{current_theme, subscribe, Theme};
use std::cell::RefCell;
use std::rc::Rc;

const DIALOG_CARD_WIDTH: f32 = 420.0;
const DIALOG_BACKGROUND_BLUR: f32 = 16.0;
const DIALOG_SHADOW_OFFSET_X: f32 = 0.0;
const DIALOG_SHADOW_OFFSET_Y: f32 = 8.0;
const DIALOG_SHADOW_BLUR: f32 = 10.0;
const DIALOG_SHADOW_SPREAD: f32 = 0.0;

thread_local! {
    static ACTIVE_DIALOG: RefCell<Option<DialogEventTarget>> = const { RefCell::new(None) };
}

type VoidCallback = Rc<dyn Fn()>;
type ShownCallback = Rc<dyn Fn(DialogShownEventArgs)>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DialogShownEventArgs;

#[derive(Clone)]
pub struct Dialog {
    root: FlexBox,
    title_node: TextNode,
    body_node: TextNode,
    content_host: FlexBox,
    accept_button: Button,
    cancel_button: Button,
    _buttons_row: FlexBox,
    form: Form,
    card: FlexBox,
    overlay: FlexBox,
    state: Rc<RefCell<DialogState>>,
}

#[derive(Clone)]
struct DialogEventTarget {
    root: WeakFlexBox,
    overlay: WeakFlexBox,
    card: WeakFlexBox,
    form: Form,
    state: Rc<RefCell<DialogState>>,
}

struct DialogState {
    visible: bool,
    semantic_scope_token: u32,
    accept_callback: Option<VoidCallback>,
    cancel_callback: Option<VoidCallback>,
    shown_callback: Option<ShownCallback>,
    backdrop_color_value: u32,
    dialog_background_blur_sigma_value: f32,
    shadow_color_value: u32,
    shadow_offset_x_value: f32,
    shadow_offset_y_value: f32,
    shadow_blur_sigma_value: f32,
    shadow_spread_value: f32,
    card_background_color_value: u32,
    card_border_width_value: f32,
    card_border_color_value: u32,
    card_border_style_value: BorderStyle,
    card_border_dash_on_value: f32,
    card_border_dash_off_value: f32,
    card_corner_top_left_value: f32,
    card_corner_top_right_value: f32,
    card_corner_bottom_right_value: f32,
    card_corner_bottom_left_value: f32,
    backdrop_color_overridden: bool,
    shadow_overridden: bool,
    card_background_overridden: bool,
    card_border_overridden: bool,
    card_corner_radius_overridden: bool,
}

impl DialogState {
    fn from_theme(theme: Theme) -> Self {
        Self {
            visible: false,
            semantic_scope_token: 0,
            accept_callback: None,
            cancel_callback: None,
            shown_callback: None,
            backdrop_color_value: theme.colors.dialog_backdrop,
            dialog_background_blur_sigma_value: DIALOG_BACKGROUND_BLUR,
            shadow_color_value: theme.colors.dialog_shadow,
            shadow_offset_x_value: DIALOG_SHADOW_OFFSET_X,
            shadow_offset_y_value: DIALOG_SHADOW_OFFSET_Y,
            shadow_blur_sigma_value: DIALOG_SHADOW_BLUR,
            shadow_spread_value: DIALOG_SHADOW_SPREAD,
            card_background_color_value: theme.colors.surface,
            card_border_width_value: 1.0,
            card_border_color_value: theme.colors.border,
            card_border_style_value: BorderStyle::Solid,
            card_border_dash_on_value: 0.0,
            card_border_dash_off_value: 0.0,
            card_corner_top_left_value: theme.spacing.md,
            card_corner_top_right_value: theme.spacing.md,
            card_corner_bottom_right_value: theme.spacing.md,
            card_corner_bottom_left_value: theme.spacing.md,
            backdrop_color_overridden: false,
            shadow_overridden: false,
            card_background_overridden: false,
            card_border_overridden: false,
            card_corner_radius_overridden: false,
        }
    }
}

impl Default for Dialog {
    fn default() -> Self {
        Self::new("", "")
    }
}

impl Dialog {
    pub fn new(title: impl Into<String>, body: impl Into<String>) -> Self {
        let theme = current_theme();
        let root = portal();
        let title_node = TextNode::new("");
        title_node
            .selectable(false)
            .semantic_role(SemanticRole::Heading);
        let body_node = TextNode::new("");
        body_node
            .selectable(false)
            .semantic_role(SemanticRole::StaticText);
        let content_host = flex_box();
        content_host
            .width(100.0, Unit::Percent)
            .flex_direction(FlexDirection::Column);
        let cancel_button = Button::new("Cancel");
        let accept_button = Button::new("OK");
        let buttons_row = flex_box();
        buttons_row
            .flex_direction(FlexDirection::Row)
            .justify_content(JustifyContent::End)
            .align_items(AlignItems::Center)
            .width(100.0, Unit::Percent)
            .child(&accept_button)
            .child(
                &flex_box()
                    .width(12.0, Unit::Pixel)
                    .height(1.0, Unit::Pixel)
                    .clone(),
            )
            .child(&cancel_button);
        let form = Form::new();
        form.default_btn(&accept_button)
            .cancel_btn(&cancel_button)
            .child(&buttons_row);
        let card = flex_box();
        card.width(DIALOG_CARD_WIDTH, Unit::Pixel)
            .flex_direction(FlexDirection::Column)
            .interactive(true)
            .child(&title_node)
            .child(
                &flex_box()
                    .width(100.0, Unit::Percent)
                    .height(12.0, Unit::Pixel)
                    .clone(),
            )
            .child(&body_node)
            .child(
                &flex_box()
                    .width(100.0, Unit::Percent)
                    .height(16.0, Unit::Pixel)
                    .clone(),
            )
            .child(&content_host)
            .child(
                &flex_box()
                    .width(100.0, Unit::Percent)
                    .height(24.0, Unit::Pixel)
                    .clone(),
            )
            .child(&form)
            .semantic_role(SemanticRole::Dialog);
        let overlay = flex_box();
        overlay
            .width(100.0, Unit::Percent)
            .height(100.0, Unit::Percent)
            .interactive(true)
            .justify_content(JustifyContent::Center)
            .align_items(AlignItems::Center)
            .child(&card);
        let state = Rc::new(RefCell::new(DialogState::from_theme(theme)));

        root.position_type(PositionType::Absolute)
            .position(0.0, 0.0)
            .width(100.0, Unit::Percent)
            .height(100.0, Unit::Percent);

        let dialog = Self {
            root,
            title_node,
            body_node,
            content_host,
            accept_button,
            cancel_button,
            _buttons_row: buttons_row,
            form,
            card,
            overlay,
            state,
        };
        dialog.content(title, body);
        dialog.bind_events();
        dialog.install_theme_subscription();
        dialog.apply_theme();
        dialog
    }

    pub fn accept_active_dialog() {
        let dialog = ACTIVE_DIALOG.with(|slot| slot.borrow().as_ref().cloned());
        if let Some(dialog) = dialog {
            dialog.accept();
        }
    }

    pub fn cancel_active_dialog() {
        let dialog = ACTIVE_DIALOG.with(|slot| slot.borrow().as_ref().cloned());
        if let Some(dialog) = dialog {
            dialog.cancel();
        }
    }

    pub fn is_open(&self) -> bool {
        self.state.borrow().visible
    }

    pub fn on_accept(&self, handler: impl Fn() + 'static) -> &Self {
        self.state.borrow_mut().accept_callback = Some(Rc::new(handler));
        self
    }

    pub fn on_cancel(&self, handler: impl Fn() + 'static) -> &Self {
        self.state.borrow_mut().cancel_callback = Some(Rc::new(handler));
        self
    }

    pub fn on_shown(&self, handler: impl Fn(DialogShownEventArgs) + 'static) -> &Self {
        self.state.borrow_mut().shown_callback = Some(Rc::new(handler));
        self
    }

    pub fn title_text(&self) -> TextNode {
        self.title_node.clone()
    }

    pub fn body_text(&self) -> TextNode {
        self.body_node.clone()
    }

    pub fn content_host(&self) -> FlexBox {
        self.content_host.clone()
    }

    pub fn accept_action_button(&self) -> Button {
        self.accept_button.clone()
    }

    pub fn cancel_action_button(&self) -> Button {
        self.cancel_button.clone()
    }

    pub fn appearance(&self, appearance: DialogAppearance) -> &Self {
        let mut state = self.state.borrow_mut();
        let backdrop = appearance.backdrop.unwrap_or_default();
        state.backdrop_color_overridden = backdrop.color.is_some();
        if let Some(color) = backdrop.color {
            state.backdrop_color_value = color;
        }
        state.dialog_background_blur_sigma_value = backdrop.blur.unwrap_or(DIALOG_BACKGROUND_BLUR);

        let card = appearance.card.unwrap_or_default();
        state.card_background_overridden = card.background.is_some();
        if let Some(color) = card.background {
            state.card_background_color_value = color;
        }
        state.card_border_overridden = card.border.is_some();
        if let Some(border) = card.border {
            state.card_border_width_value = border.width;
            state.card_border_color_value = border.color;
            state.card_border_style_value = border.style;
            state.card_border_dash_on_value = border.dash_on;
            state.card_border_dash_off_value = border.dash_off;
        }
        state.card_corner_radius_overridden = card.corners.is_some();
        if let Some(corners) = card.corners {
            state.card_corner_top_left_value = corners.top_left;
            state.card_corner_top_right_value = corners.top_right;
            state.card_corner_bottom_right_value = corners.bottom_right;
            state.card_corner_bottom_left_value = corners.bottom_left;
        }
        state.shadow_overridden = card.shadow.is_some();
        if let Some(shadow) = card.shadow {
            state.shadow_color_value = shadow.color;
            state.shadow_offset_x_value = shadow.offset_x;
            state.shadow_offset_y_value = shadow.offset_y;
            state.shadow_blur_sigma_value = shadow.blur_sigma;
            state.shadow_spread_value = shadow.spread;
        }
        drop(state);
        self.event_target().handle_theme_changed(current_theme());
        self.apply_theme();
        self
    }

    pub fn clear_appearance(&self) -> &Self {
        self.appearance(DialogAppearance::new())
    }

    pub fn content(&self, title: impl Into<String>, body: impl Into<String>) -> &Self {
        let title = title.into();
        let body = body.into();
        self.title_node
            .text(title.clone())
            .semantic_label(title.clone());
        self.body_node.text(body.clone()).semantic_label(body);
        self.card.semantic_label(title);
        self
    }

    pub fn show(&self) {
        self.apply_theme();
        if self.overlay.parent_handle().is_none() {
            self.root.child(&self.overlay);
        }
        {
            let mut state = self.state.borrow_mut();
            if state.semantic_scope_token == 0 && self.overlay.handle().raw() != 0 {
                state.semantic_scope_token = ui::push_semantic_scope(self.overlay.handle().raw());
            }
            state.visible = true;
        }
        self.form.activate();
        if self.accept_button.has_built_handle() {
            ui::request_focus(self.accept_button.handle().raw());
        }
        let event_target = self.event_target();
        ACTIVE_DIALOG.with(|slot| slot.borrow_mut().replace(event_target.clone()));
        app::after_next_commit(move || {
            event_target.fire_shown_after_commit();
        });
    }

    pub fn hide(&self) {
        self.event_target().hide();
    }

    fn bind_events(&self) {
        self.card.on_pointer_down(|event| {
            event.handled = true;
        });

        let event_target = self.event_target();
        self.overlay.on_pointer_click(move |event| {
            event_target.handle_backdrop_click(event);
        });

        self.accept_button.on_click(|_event| {
            Dialog::accept_active_dialog();
        });

        self.cancel_button.on_click(|_event| {
            Dialog::cancel_active_dialog();
        });
    }

    fn install_theme_subscription(&self) {
        let event_target = self.event_target();
        let title = self.title_node.clone();
        let body = self.body_node.clone();
        let guard = subscribe(move |theme| {
            event_target.handle_theme_changed(theme);
            event_target.apply_theme(&title, &body);
        });
        self.root
            .retained_node_ref()
            .retain_attachment(Rc::new(guard));
    }

    fn apply_theme(&self) {
        self.event_target()
            .apply_theme(&self.title_node, &self.body_node);
    }

    fn event_target(&self) -> DialogEventTarget {
        DialogEventTarget {
            root: self.root.downgrade(),
            overlay: self.overlay.downgrade(),
            card: self.card.downgrade(),
            form: self.form.clone(),
            state: self.state.clone(),
        }
    }
}

impl Node for Dialog {
    fn retained_node_ref(&self) -> NodeRef {
        self.root.retained_node_ref()
    }

    fn retained_owner_attachment(&self) -> Option<Rc<dyn std::any::Any>> {
        Some(Rc::new(self.clone()))
    }

    fn build_self(&self) {
        self.apply_theme();
        self.root.build_self();
    }

    fn dispose(&self) {
        self.hide();
        if self.overlay.handle().raw() != 0 {
            self.overlay.dispose();
        }
        self.root.dispose();
    }
}

impl crate::node::HasFlexBoxRoot for Dialog {
    fn flex_box_root(&self) -> &FlexBox {
        &self.root
    }
}

impl ThemeBindable for Dialog {
    fn theme_binding_node(&self) -> NodeRef {
        self.root.retained_node_ref()
    }

    fn weak_theme_target(&self) -> Box<dyn Fn() -> Option<Self>> {
        let root = self.root.downgrade();
        let title_node = self.title_node.clone();
        let body_node = self.body_node.clone();
        let content_host = self.content_host.clone();
        let accept_button = self.accept_button.clone();
        let cancel_button = self.cancel_button.clone();
        let buttons_row = self._buttons_row.clone();
        let form = self.form.clone();
        let card = self.card.clone();
        let overlay = self.overlay.clone();
        let state = self.state.clone();
        Box::new(move || {
            Some(Self {
                root: root.upgrade()?,
                title_node: title_node.clone(),
                body_node: body_node.clone(),
                content_host: content_host.clone(),
                accept_button: accept_button.clone(),
                cancel_button: cancel_button.clone(),
                _buttons_row: buttons_row.clone(),
                form: form.clone(),
                card: card.clone(),
                overlay: overlay.clone(),
                state: state.clone(),
            })
        })
    }
}

impl DialogEventTarget {
    fn hide(&self) {
        let Some(root) = self.root.upgrade() else {
            return;
        };
        let Some(overlay) = self.overlay.upgrade() else {
            return;
        };
        let mut state = self.state.borrow_mut();
        if !state.visible && overlay.parent_handle().is_none() {
            return;
        }
        if state.semantic_scope_token != 0 {
            ui::remove_semantic_scope(state.semantic_scope_token);
            state.semantic_scope_token = 0;
        }
        state.visible = false;
        drop(state);
        self.form.deactivate();
        root.remove_child(&overlay);
        ACTIVE_DIALOG.with(|slot| {
            let should_clear = {
                let borrowed = slot.borrow();
                borrowed
                    .as_ref()
                    .is_some_and(|active| Rc::ptr_eq(&active.state, &self.state))
            };
            if should_clear {
                slot.borrow_mut().take();
            }
        });
    }

    fn accept(&self) {
        let callback = self.state.borrow().accept_callback.clone();
        self.hide();
        if let Some(callback) = callback {
            callback();
        }
    }

    fn cancel(&self) {
        let callback = self.state.borrow().cancel_callback.clone();
        self.hide();
        if let Some(callback) = callback {
            callback();
        }
    }

    fn fire_shown_after_commit(&self) {
        let state = self.state.borrow();
        if !state.visible {
            return;
        }
        let Some(overlay) = self.overlay.upgrade() else {
            return;
        };
        if overlay.parent_handle().is_none() {
            return;
        }
        let callback = state.shown_callback.clone();
        drop(state);
        if let Some(callback) = callback {
            callback(DialogShownEventArgs);
        }
    }

    fn handle_theme_changed(&self, theme: Theme) {
        let mut state = self.state.borrow_mut();
        if !state.backdrop_color_overridden {
            state.backdrop_color_value = theme.colors.dialog_backdrop;
        }
        if !state.shadow_overridden {
            state.shadow_color_value = theme.colors.dialog_shadow;
        }
        if !state.card_background_overridden {
            state.card_background_color_value = theme.colors.surface;
        }
        if !state.card_border_overridden {
            state.card_border_width_value = 1.0;
            state.card_border_color_value = theme.colors.border;
            state.card_border_style_value = BorderStyle::Solid;
            state.card_border_dash_on_value = 0.0;
            state.card_border_dash_off_value = 0.0;
        }
        if !state.card_corner_radius_overridden {
            state.card_corner_top_left_value = theme.spacing.md;
            state.card_corner_top_right_value = theme.spacing.md;
            state.card_corner_bottom_right_value = theme.spacing.md;
            state.card_corner_bottom_left_value = theme.spacing.md;
        }
    }

    fn apply_theme(&self, title: &TextNode, body: &TextNode) {
        let theme = current_theme();
        let state = self.state.borrow();
        if let Some(overlay) = self.overlay.upgrade() {
            overlay
                .bg_color(state.backdrop_color_value)
                .background_blur(state.dialog_background_blur_sigma_value);
        }
        if let Some(card) = self.card.upgrade() {
            card.bg_color(state.card_background_color_value)
                .corners(
                    state.card_corner_top_left_value,
                    state.card_corner_top_right_value,
                    state.card_corner_bottom_right_value,
                    state.card_corner_bottom_left_value,
                )
                .border_config(Border {
                    width: state.card_border_width_value,
                    color: state.card_border_color_value,
                    style: state.card_border_style_value,
                    dash_on: state.card_border_dash_on_value,
                    dash_off: state.card_border_dash_off_value,
                })
                .drop_shadow(
                    state.shadow_color_value,
                    state.shadow_offset_x_value,
                    state.shadow_offset_y_value,
                    state.shadow_blur_sigma_value,
                    state.shadow_spread_value,
                )
                .padding(24.0, 24.0, 24.0, 24.0);
        }
        drop(state);
        title
            .font_family(theme.fonts.heading_family.clone())
            .font_size(theme.fonts.size_heading)
            .text_color(theme.colors.text_primary);
        body.font_family(theme.fonts.body_family.clone())
            .font_size(theme.fonts.size_body)
            .text_color(theme.colors.text_muted);
    }

    fn handle_backdrop_click(&self, event: &mut crate::PointerEventArgs) {
        let Some(card) = self.card.upgrade() else {
            return;
        };
        if let Some(bounds) = ui::get_bounds(card.handle().raw()) {
            let left = bounds[0];
            let top = bounds[1];
            let right = left + bounds[2];
            let bottom = top + bounds[3];
            if event.scene_x >= left
                && event.scene_x <= right
                && event.scene_y >= top
                && event.scene_y <= bottom
            {
                return;
            }
        }
        event.handled = true;
        self.cancel();
    }
}
