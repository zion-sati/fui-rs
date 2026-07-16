use crate::controls::TextInputColors;
use crate::ffi::{CursorStyle, Unit};
use crate::node::{Border, Corners, EdgeInsets, FlexBox, PresenterHostStyle, TextCore};
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
    fn bind(&self, editor_host: TextCore, placeholder_host: FlexBox);
    fn present(
        &self,
        theme: Theme,
        state: &TextInputVisualState,
        colors: Option<TextInputColors>,
    ) -> PresenterHostStyle;
}

pub trait TextInputTemplate {
    fn create(&self) -> Rc<dyn TextInputPresenter>;
}

#[derive(Clone, Default)]
pub struct DefaultTextInputPresenter {
    editor_host: RefCell<Option<TextCore>>,
    placeholder_host: RefCell<Option<FlexBox>>,
}

impl DefaultTextInputPresenter {
    pub fn new() -> Self {
        Self::default()
    }
}

impl TextInputPresenter for DefaultTextInputPresenter {
    fn bind(&self, editor_host: TextCore, placeholder_host: FlexBox) {
        *self.editor_host.borrow_mut() = Some(editor_host);
        *self.placeholder_host.borrow_mut() = Some(placeholder_host);
    }

    fn present(
        &self,
        theme: Theme,
        state: &TextInputVisualState,
        colors: Option<TextInputColors>,
    ) -> PresenterHostStyle {
        let Some(editor_host) = self.editor_host.borrow().clone() else {
            return PresenterHostStyle::new();
        };
        let Some(placeholder_host) = self.placeholder_host.borrow().clone() else {
            return PresenterHostStyle::new();
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

        editor_host.cursor(editable_cursor);
        placeholder_host
            .position(horizontal_padding, vertical_padding)
            .width(100.0, Unit::Percent)
            .cursor(editable_cursor);
        PresenterHostStyle::new()
            .background(bg)
            .corners(Corners::all(theme.spacing.sm))
            .border(Border::solid(1.0, border_color))
            .padding(EdgeInsets::new(
                horizontal_padding,
                vertical_padding,
                horizontal_padding,
                vertical_padding,
            ))
            .cursor(shell_cursor)
            .opacity(if state.enabled { 1.0 } else { 0.6 })
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
