use crate::controls::ButtonColors;
use crate::ffi::{AlignItems, FlexDirection, JustifyContent};
use crate::node::{flex_box, FlexBox, TextCore};
use crate::theme::Theme;
use crate::{FontStyle, FontWeight};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, Default)]
pub struct ButtonVisualState {
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
    pub enabled: bool,
}

pub trait ButtonPresenter {
    fn content_root(&self) -> FlexBox;
    fn label_node(&self) -> TextCore;
    fn apply(
        &self,
        host: &FlexBox,
        theme: Theme,
        state: ButtonVisualState,
        colors: Option<ButtonColors>,
    );
}

pub trait ButtonTemplate {
    fn create(&self) -> Rc<dyn ButtonPresenter>;
}

#[derive(Clone)]
pub struct DefaultButtonPresenter {
    content_root: FlexBox,
    label_node: TextCore,
}

impl DefaultButtonPresenter {
    pub fn new() -> Self {
        let label_node = TextCore::new("");
        let content_root = flex_box();
        content_root
            .flex_direction(FlexDirection::Row)
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center)
            .child(&label_node);
        Self {
            content_root,
            label_node,
        }
    }
}

impl ButtonPresenter for DefaultButtonPresenter {
    fn content_root(&self) -> FlexBox {
        self.content_root.clone()
    }

    fn label_node(&self) -> TextCore {
        self.label_node.clone()
    }

    fn apply(
        &self,
        host: &FlexBox,
        theme: Theme,
        state: ButtonVisualState,
        colors: Option<ButtonColors>,
    ) {
        let background = if !state.enabled {
            colors
                .and_then(|colors| colors.background)
                .unwrap_or(theme.colors.accent)
        } else if state.pressed {
            colors
                .and_then(|colors| colors.background_pressed)
                .or_else(|| colors.and_then(|colors| colors.background_hover))
                .or_else(|| colors.and_then(|colors| colors.background))
                .unwrap_or(theme.colors.accent_pressed)
        } else if state.hovered {
            colors
                .and_then(|colors| colors.background_hover)
                .or_else(|| colors.and_then(|colors| colors.background))
                .unwrap_or(theme.colors.accent_hovered)
        } else {
            colors
                .and_then(|colors| colors.background)
                .unwrap_or(theme.colors.accent)
        };
        let border = colors
            .and_then(|colors| colors.border)
            .unwrap_or(theme.colors.border);
        let text_color = if !state.enabled {
            colors
                .and_then(|colors| colors.text_muted)
                .or_else(|| colors.and_then(|colors| colors.text_primary))
                .unwrap_or(theme.colors.text_on_accent)
        } else {
            colors
                .and_then(|colors| colors.text_primary)
                .unwrap_or(theme.colors.text_on_accent)
        };
        host.flex_direction(FlexDirection::Row)
            .justify_content(JustifyContent::Center)
            .align_items(AlignItems::Center)
            .corner_radius(theme.spacing.sm)
            .border(1.0, border)
            .padding(
                theme.spacing.md,
                theme.spacing.sm,
                theme.spacing.md,
                theme.spacing.sm,
            )
            .drop_shadow(0x00000000, 0.0, 0.0, 0.0, 0.0)
            .bg_color(background);
        self.content_root
            .flex_direction(FlexDirection::Row)
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center);
        self.label_node
            .font_family(theme.fonts.body_family.clone())
            .font_weight(FontWeight::Regular)
            .font_style(FontStyle::Normal)
            .font_size(theme.fonts.size_body)
            .text_color(text_color);
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DefaultButtonTemplate;

impl ButtonTemplate for DefaultButtonTemplate {
    fn create(&self) -> Rc<dyn ButtonPresenter> {
        Rc::new(DefaultButtonPresenter::new())
    }
}

pub const DEFAULT_BUTTON_TEMPLATE: DefaultButtonTemplate = DefaultButtonTemplate;

pub fn create_default_button_presenter() -> Rc<dyn ButtonPresenter> {
    Rc::new(DefaultButtonPresenter::new())
}
