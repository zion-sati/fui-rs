use crate::controls::TextInputColors;
use crate::ffi::{CursorStyle, Unit};
use crate::node::{FlexBox, TextCore};
use crate::theme::Theme;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextInputVisualState {
    pub multiline: bool,
    pub enabled: bool,
    pub wrapping: bool,
}

pub trait TextInputPresenter {
    fn bind(&self, host: FlexBox, editor_host: TextCore, placeholder_host: FlexBox);
    fn apply(&self, theme: Theme, state: &TextInputVisualState, colors: Option<TextInputColors>);
}

pub trait TextInputTemplate {
    fn create(&self) -> Rc<dyn TextInputPresenter>;
}

#[derive(Clone, Default)]
pub struct DefaultTextInputPresenter {
    host: RefCell<Option<FlexBox>>,
    editor_host: RefCell<Option<TextCore>>,
    placeholder_host: RefCell<Option<FlexBox>>,
}

impl DefaultTextInputPresenter {
    pub fn new() -> Self {
        Self::default()
    }
}

impl TextInputPresenter for DefaultTextInputPresenter {
    fn bind(&self, host: FlexBox, editor_host: TextCore, placeholder_host: FlexBox) {
        *self.host.borrow_mut() = Some(host);
        *self.editor_host.borrow_mut() = Some(editor_host);
        *self.placeholder_host.borrow_mut() = Some(placeholder_host);
    }

    fn apply(&self, theme: Theme, state: &TextInputVisualState, colors: Option<TextInputColors>) {
        let Some(host) = self.host.borrow().clone() else {
            return;
        };
        let Some(editor_host) = self.editor_host.borrow().clone() else {
            return;
        };
        let Some(placeholder_host) = self.placeholder_host.borrow().clone() else {
            return;
        };

        let horizontal_padding = theme.spacing.md;
        let vertical_padding = theme.spacing.sm;
        let editable_cursor = if state.enabled {
            CursorStyle::Text
        } else {
            CursorStyle::Default
        };
        let shell_cursor = if !state.multiline && state.enabled {
            CursorStyle::Text
        } else {
            CursorStyle::Default
        };
        let bg = colors
            .filter(|value| value.has_background())
            .map(|value| value.background_color())
            .unwrap_or(theme.colors.surface);
        let border_color = colors
            .filter(|value| value.has_border())
            .map(|value| value.border_color())
            .unwrap_or(theme.colors.border);

        host.bg_color(bg)
            .corner_radius(theme.spacing.sm)
            .border(1.0, border_color)
            .padding(
                horizontal_padding,
                vertical_padding,
                horizontal_padding,
                vertical_padding,
            )
            .cursor(shell_cursor)
            .opacity(if state.enabled { 1.0 } else { 0.6 });
        editor_host.cursor(editable_cursor);
        placeholder_host
            .position(horizontal_padding, vertical_padding)
            .width(100.0, Unit::Percent)
            .cursor(editable_cursor);
    }
}

#[derive(Clone)]
pub struct DefaultTextInputTemplate;

impl TextInputTemplate for DefaultTextInputTemplate {
    fn create(&self) -> Rc<dyn TextInputPresenter> {
        Rc::new(DefaultTextInputPresenter::new())
    }
}

pub const DEFAULT_TEXT_INPUT_TEMPLATE: DefaultTextInputTemplate = DefaultTextInputTemplate;

pub fn create_default_text_input_presenter() -> Rc<dyn TextInputPresenter> {
    DEFAULT_TEXT_INPUT_TEMPLATE.create()
}
