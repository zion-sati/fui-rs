use super::internal::text_input_presenter::TextInputTemplate;
use super::TextInputColors;
use crate::event::{SelectionChangedEventArgs, TextChangedEventArgs};
use crate::FontFamily;
use std::rc::Rc;

/// Shared editable-text behavior implemented by [`TextInput`](super::TextInput)
/// and [`TextArea`](super::TextArea).
///
/// Selection and caret positions use Unicode scalar-value indices. The
/// explicitly named byte-offset getters expose the corresponding UTF-8 runtime
/// offsets for diagnostics and low-level interop only.
pub trait TextEditorSurface: Sized {
    fn text(&self, value: impl Into<String>) -> &Self;
    fn value(&self) -> String;
    fn selection_start(&self) -> u32;
    fn selection_end(&self) -> u32;
    fn selection_start_byte_offset(&self) -> u32;
    fn selection_end_byte_offset(&self) -> u32;
    fn placeholder(&self, value: impl Into<String>) -> &Self;
    fn max_chars(&self, limit: i32) -> &Self;
    fn read_only(&self, flag: bool) -> &Self;
    fn accepts_tab(&self, flag: bool) -> &Self;
    fn selection_range(&self, start: u32, end: u32) -> &Self;
    fn caret(&self, position: u32) -> &Self;
    fn caret_to_end(&self) -> &Self;
    fn colors(&self, colors: TextInputColors) -> &Self;
    fn clear_colors(&self) -> &Self;
    fn template(&self, template: Rc<dyn TextInputTemplate>) -> &Self;
    fn clear_template(&self) -> &Self;
    fn line_height(&self, value: f32) -> &Self;
    fn font_family(&self, family: FontFamily) -> &Self;
    fn font_size(&self, size: f32) -> &Self;
    fn on_changed(&self, handler: impl Fn(TextChangedEventArgs) + 'static) -> &Self;
    fn on_text_changed(&self, handler: impl Fn(TextChangedEventArgs) + 'static) -> &Self;
    fn on_selection_changed(&self, handler: impl Fn(SelectionChangedEventArgs) + 'static) -> &Self;
}

macro_rules! impl_text_editor_surface {
    ($control:ty) => {
        impl crate::controls::TextEditorSurface for $control {
            fn text(&self, value: impl Into<String>) -> &Self {
                self.core.text(value);
                self
            }

            fn value(&self) -> String {
                self.core.value()
            }

            fn selection_start(&self) -> u32 {
                self.core.selection_start()
            }

            fn selection_end(&self) -> u32 {
                self.core.selection_end()
            }

            fn selection_start_byte_offset(&self) -> u32 {
                self.core.selection_start_byte_offset()
            }

            fn selection_end_byte_offset(&self) -> u32 {
                self.core.selection_end_byte_offset()
            }

            fn placeholder(&self, value: impl Into<String>) -> &Self {
                self.core.placeholder(value);
                self
            }

            fn max_chars(&self, limit: i32) -> &Self {
                self.core.max_chars(limit);
                self
            }

            fn read_only(&self, flag: bool) -> &Self {
                self.core.read_only(flag);
                self
            }

            fn accepts_tab(&self, flag: bool) -> &Self {
                self.core.accepts_tab(flag);
                self
            }

            fn selection_range(&self, start: u32, end: u32) -> &Self {
                self.core.selection_range(start, end);
                self
            }

            fn caret(&self, position: u32) -> &Self {
                self.core.caret(position);
                self
            }

            fn caret_to_end(&self) -> &Self {
                self.core.caret_to_end();
                self
            }

            fn colors(&self, colors: crate::controls::TextInputColors) -> &Self {
                self.core.colors(colors);
                self
            }

            fn clear_colors(&self) -> &Self {
                self.core.clear_colors();
                self
            }

            fn template(
                &self,
                template: std::rc::Rc<dyn crate::controls::TextInputTemplate>,
            ) -> &Self {
                self.core.template(template);
                self
            }

            fn clear_template(&self) -> &Self {
                self.core.clear_template();
                self
            }

            fn line_height(&self, value: f32) -> &Self {
                self.core.line_height(value);
                self
            }

            fn font_family(&self, family: crate::FontFamily) -> &Self {
                self.core.font_family(family);
                self
            }

            fn font_size(&self, size: f32) -> &Self {
                self.core.font_size(size);
                self
            }

            fn on_changed(
                &self,
                handler: impl Fn(crate::event::TextChangedEventArgs) + 'static,
            ) -> &Self {
                self.core.on_changed(handler);
                self
            }

            fn on_text_changed(
                &self,
                handler: impl Fn(crate::event::TextChangedEventArgs) + 'static,
            ) -> &Self {
                self.core.on_text_changed(handler);
                self
            }

            fn on_selection_changed(
                &self,
                handler: impl Fn(crate::event::SelectionChangedEventArgs) + 'static,
            ) -> &Self {
                self.core.on_selection_changed(handler);
                self
            }
        }
    };
}

pub(crate) use impl_text_editor_surface;
