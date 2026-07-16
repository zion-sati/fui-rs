use super::text_input_presenter::{
    create_default_text_input_presenter, TextInputPresenter, TextInputTemplate,
    TextInputVisualState,
};
use crate::bindings::ui;
use crate::controls::control_template_set::get_control_templates;
use crate::controls::TextInputColors;
use crate::event::{
    FocusChangedEventArgs, KeyEventArgs, SelectionChangedEventArgs, TextChangedEventArgs,
};
use crate::ffi::{
    CursorStyle, KeyEventType, KeyModifier, PositionType, SemanticRole, TextVerticalAlign, Unit,
};
use crate::node::{
    flex_box, FlexBox, FlexBoxSurface, HasFlexBoxRoot, Node, NodeHandle, ScrollBarVisibility,
    ScrollBox, TextCore,
};
use crate::persisted::{persisted_value_adapter, PersistedStringCodec};
use crate::signal::SubscriptionGuard;
use crate::theme::{current_theme, subscribe};
use crate::{focus_adorner, focus_visibility};
use std::cell::{Cell, RefCell};
use std::cmp::min;
use std::rc::{Rc, Weak};

const UNLIMITED_TEXT_LENGTH: i32 = i32::MAX;

#[derive(Clone, Copy)]
pub(crate) struct TextInputProfile {
    max_lines: i32,
    multiline: bool,
    wraps_by_default: bool,
    default_semantic_label: &'static str,
}

impl TextInputProfile {
    pub(crate) const fn single_line() -> Self {
        Self {
            max_lines: 1,
            multiline: false,
            wraps_by_default: false,
            default_semantic_label: "Text input",
        }
    }

    pub(crate) const fn multiline() -> Self {
        Self {
            max_lines: 0,
            multiline: true,
            wraps_by_default: true,
            default_semantic_label: "Text area",
        }
    }
}

fn create_presenter(
    profile: TextInputProfile,
    template: Option<Rc<dyn TextInputTemplate>>,
) -> Rc<dyn TextInputPresenter> {
    if let Some(template) = template {
        return template.create();
    }
    let app_template = get_control_templates().and_then(|set| {
        if profile.multiline {
            set.text_area
        } else {
            set.text_input
        }
    });
    if let Some(template) = app_template {
        return template.create();
    }
    create_default_text_input_presenter()
}

fn char_count(text: &str) -> u32 {
    text.chars().count() as u32
}

fn char_to_byte(text: &str, index: u32) -> u32 {
    let target = min(index, char_count(text)) as usize;
    if target == 0 {
        return 0;
    }
    for (position, (byte, _)) in text.char_indices().enumerate() {
        if position == target {
            return byte as u32;
        }
    }
    text.len() as u32
}

fn byte_to_char(text: &str, byte_index: u32) -> u32 {
    let target = min(byte_index as usize, text.len());
    let mut count = 0;
    for (byte, _) in text.char_indices() {
        if byte >= target {
            break;
        }
        count += 1;
    }
    if target == text.len() {
        text.chars().count() as u32
    } else {
        count
    }
}

fn is_enabled(root: &FlexBox) -> bool {
    root.retained_node_ref().is_enabled_for_routing()
}

pub struct TextInputCore {
    self_weak: RefCell<Weak<TextInputCore>>,
    profile: TextInputProfile,
    root: FlexBox,
    editor_text: TextCore,
    editor_scroll_box: Option<ScrollBox>,
    placeholder_text: TextCore,
    placeholder_host: FlexBox,
    placeholder_attached: Cell<bool>,
    presenter: RefCell<Rc<dyn TextInputPresenter>>,
    template: RefCell<Option<Rc<dyn TextInputTemplate>>>,
    colors_value: Cell<Option<TextInputColors>>,
    text_value: RefCell<String>,
    placeholder_value: RefCell<String>,
    max_chars_value: Cell<i32>,
    read_only_value: Cell<bool>,
    password_value: Cell<bool>,
    accepts_tab_value: Cell<bool>,
    host_autofill_hint_value: RefCell<Option<String>>,
    wrapping_value: Cell<bool>,
    vertical_scrollbar_visibility_value: Cell<ScrollBarVisibility>,
    horizontal_scrollbar_visibility_value: Cell<ScrollBarVisibility>,
    font_family_override: RefCell<Option<crate::FontFamily>>,
    font_size_override: Cell<f32>,
    has_font_size_override: Cell<bool>,
    selection_start_chars: Cell<u32>,
    selection_end_chars: Cell<u32>,
    selection_start_bytes: Cell<u32>,
    selection_end_bytes: Cell<u32>,
    focused_state: Cell<bool>,
    changed_callback: RefCell<Option<Rc<dyn Fn(TextChangedEventArgs)>>>,
    selection_changed_callback: RefCell<Option<Rc<dyn Fn(SelectionChangedEventArgs)>>>,
    focus_changed_callback: RefCell<Option<Rc<dyn Fn(FocusChangedEventArgs)>>>,
    theme_guard: RefCell<Option<SubscriptionGuard>>,
    focus_visibility_guard: RefCell<Option<SubscriptionGuard>>,
}

impl TextInputCore {
    pub fn new() -> Self {
        Self::with_profile(TextInputProfile::single_line())
    }

    pub(crate) fn multiline() -> Self {
        Self::with_profile(TextInputProfile::multiline())
    }

    fn with_profile(profile: TextInputProfile) -> Self {
        let root = flex_box();
        root.clip_to_bounds(true)
            .interactive(true)
            .reflect_semantic_disabled_from_enabled()
            .selection_area_barrier(true);

        let editor_text = TextCore::new("");
        editor_text
            .semantic_role(SemanticRole::Textbox)
            .reflect_semantic_disabled_from_enabled()
            .focusable(true, 0)
            .selectable(true)
            .editable(true);
        let placeholder_text = TextCore::new("");
        let placeholder_host = flex_box();
        placeholder_host
            .position_type(PositionType::Absolute)
            .clip_to_bounds(false)
            .interactive(true)
            .child(&placeholder_text);

        let editor_scroll_box = if profile.multiline {
            let scroll_box = ScrollBox::new();
            scroll_box
                .fill_size()
                .persist_scroll(false)
                .vertical_scrollbar_visibility(ScrollBarVisibility::Auto)
                .horizontal_scrollbar_visibility(ScrollBarVisibility::Auto)
                .child(&editor_text);
            root.child(&scroll_box);
            Some(scroll_box)
        } else {
            root.child(&editor_text);
            None
        };

        let presenter = create_presenter(profile, None);
        presenter.bind(editor_text.clone(), placeholder_host.clone());

        let this = Self {
            self_weak: RefCell::new(Weak::new()),
            profile,
            root,
            editor_text,
            editor_scroll_box,
            placeholder_text,
            placeholder_host,
            placeholder_attached: Cell::new(false),
            presenter: RefCell::new(presenter),
            template: RefCell::new(None),
            colors_value: Cell::new(None),
            text_value: RefCell::new(String::new()),
            placeholder_value: RefCell::new(String::new()),
            max_chars_value: Cell::new(UNLIMITED_TEXT_LENGTH),
            read_only_value: Cell::new(false),
            password_value: Cell::new(false),
            accepts_tab_value: Cell::new(false),
            host_autofill_hint_value: RefCell::new(None),
            wrapping_value: Cell::new(profile.wraps_by_default),
            vertical_scrollbar_visibility_value: Cell::new(ScrollBarVisibility::Auto),
            horizontal_scrollbar_visibility_value: Cell::new(ScrollBarVisibility::Auto),
            font_family_override: RefCell::new(None),
            font_size_override: Cell::new(0.0),
            has_font_size_override: Cell::new(false),
            selection_start_chars: Cell::new(0),
            selection_end_chars: Cell::new(0),
            selection_start_bytes: Cell::new(0),
            selection_end_bytes: Cell::new(0),
            focused_state: Cell::new(false),
            changed_callback: RefCell::new(None),
            selection_changed_callback: RefCell::new(None),
            focus_changed_callback: RefCell::new(None),
            theme_guard: RefCell::new(None),
            focus_visibility_guard: RefCell::new(None),
        };
        this.sync_semantic_label();
        this.sync_editor_limits();
        this.sync_editor_editability();
        this.sync_browser_input_metadata();
        this.sync_theme_state();
        this.sync_scroll_chrome_state();
        this.sync_placeholder_visibility();
        this
    }

    pub(crate) fn finish_init(&self, weak: Weak<TextInputCore>) {
        *self.self_weak.borrow_mut() = weak;
        self.install_events();
        self.install_persisted_state();
        self.install_theme_subscription();
        self.install_focus_visibility_subscription();
        self.install_effective_enabled_subscription();
    }

    pub fn text(&self, value: impl Into<String>) -> &Self {
        let value = value.into();
        *self.text_value.borrow_mut() = value.clone();
        self.editor_text.text(value);
        self.caret_to_end();
        self.sync_semantic_label();
        self.sync_placeholder_visibility();
        self
    }

    pub fn value(&self) -> String {
        self.text_value.borrow().clone()
    }

    pub fn selection_start(&self) -> u32 {
        self.selection_start_chars.get()
    }

    pub fn selection_end(&self) -> u32 {
        self.selection_end_chars.get()
    }

    pub fn selection_start_byte_offset(&self) -> u32 {
        self.selection_start_bytes.get()
    }

    pub fn selection_end_byte_offset(&self) -> u32 {
        self.selection_end_bytes.get()
    }

    pub fn placeholder(&self, value: impl Into<String>) -> &Self {
        let value = value.into();
        *self.placeholder_value.borrow_mut() = value.clone();
        self.placeholder_text.text(value);
        self.sync_semantic_label();
        self.sync_placeholder_visibility();
        self
    }

    pub fn max_chars(&self, limit: i32) -> &Self {
        self.max_chars_value.set(if limit < 0 {
            UNLIMITED_TEXT_LENGTH
        } else {
            limit
        });
        self.sync_editor_limits();
        self
    }

    pub fn read_only(&self, flag: bool) -> &Self {
        self.read_only_value.set(flag);
        self.sync_editor_editability();
        self
    }

    pub fn accepts_tab(&self, flag: bool) -> &Self {
        self.accepts_tab_value.set(flag);
        self.editor_text.editor_accepts_tab(flag);
        self
    }

    pub fn password(&self, flag: bool) -> &Self {
        self.password_value.set(flag);
        self.editor_text.obscured(flag);
        self.sync_browser_input_metadata();
        self.sync_semantic_label();
        self
    }

    pub fn host_autofill(&self, hint: Option<&str>) -> &Self {
        *self.host_autofill_hint_value.borrow_mut() =
            hint.filter(|value| !value.is_empty()).map(str::to_string);
        self.sync_browser_input_metadata();
        self
    }

    pub fn selection_range(&self, start: u32, end: u32) -> &Self {
        let text = self.text_value.borrow();
        let start = min(start, char_count(&text));
        let end = min(end, char_count(&text));
        let start_bytes = char_to_byte(&text, start);
        let end_bytes = char_to_byte(&text, end);
        drop(text);
        self.selection_start_chars.set(start);
        self.selection_end_chars.set(end);
        self.selection_start_bytes.set(start_bytes);
        self.selection_end_bytes.set(end_bytes);
        self.editor_text.selection_range(start_bytes, end_bytes);
        self
    }

    pub fn caret(&self, position: u32) -> &Self {
        self.selection_range(position, position)
    }

    pub fn caret_to_end(&self) -> &Self {
        let end = char_count(&self.text_value.borrow());
        self.selection_range(end, end)
    }

    pub fn colors(&self, colors: TextInputColors) -> &Self {
        self.set_colors(Some(colors))
    }

    pub fn clear_colors(&self) -> &Self {
        self.set_colors(None)
    }

    fn set_colors(&self, colors: Option<TextInputColors>) -> &Self {
        self.colors_value.set(colors);
        self.sync_theme_state();
        self
    }

    pub fn enabled(&self, enabled: bool) -> &Self {
        self.root.enabled(enabled);
        self.sync_editor_editability();
        self.sync_theme_state();
        self.sync_placeholder_visibility();
        self
    }

    pub fn focusable(&self, enabled: bool, tab_index: i32) -> &Self {
        self.editor_text.focusable(enabled, tab_index);
        self
    }

    pub fn node_id(&self, id: impl Into<String>) -> &Self {
        self.editor_text.node_id(id);
        self
    }

    pub fn line_height(&self, value: f32) -> &Self {
        self.editor_text.line_height(value);
        self.placeholder_text.line_height(value);
        self
    }

    pub fn wrapping(&self, flag: bool) -> &Self {
        if self.wrapping_value.get() == flag {
            return self;
        }
        self.wrapping_value.set(flag);
        self.sync_editor_wrapping();
        self.sync_scroll_chrome_state();
        self.sync_theme_state();
        self
    }

    pub fn vertical_scrollbar_visibility(&self, mode: ScrollBarVisibility) -> &Self {
        self.vertical_scrollbar_visibility_value.set(mode);
        self.sync_scroll_chrome_state();
        self
    }

    pub fn horizontal_scrollbar_visibility(&self, mode: ScrollBarVisibility) -> &Self {
        self.horizontal_scrollbar_visibility_value.set(mode);
        self.sync_scroll_chrome_state();
        self
    }

    pub fn font_family(&self, family: crate::FontFamily) -> &Self {
        *self.font_family_override.borrow_mut() = Some(family);
        self.sync_theme_state();
        self
    }

    pub fn font_size(&self, size: f32) -> &Self {
        self.font_size_override.set(size);
        self.has_font_size_override.set(size > 0.0);
        self.sync_theme_state();
        self
    }

    pub fn template(&self, template: Rc<dyn TextInputTemplate>) -> &Self {
        self.set_template(Some(template))
    }

    pub fn clear_template(&self) -> &Self {
        self.set_template(None)
    }

    fn set_template(&self, template: Option<Rc<dyn TextInputTemplate>>) -> &Self {
        *self.template.borrow_mut() = template.clone();
        let presenter = create_presenter(self.profile, template);
        presenter.bind(self.editor_text.clone(), self.placeholder_host.clone());
        *self.presenter.borrow_mut() = presenter;
        self.sync_theme_state();
        self
    }

    pub fn on_changed(&self, handler: impl Fn(TextChangedEventArgs) + 'static) -> &Self {
        *self.changed_callback.borrow_mut() = Some(Rc::new(handler));
        self
    }

    pub fn on_text_changed(&self, handler: impl Fn(TextChangedEventArgs) + 'static) -> &Self {
        self.on_changed(handler)
    }

    pub fn on_selection_changed(
        &self,
        handler: impl Fn(SelectionChangedEventArgs) + 'static,
    ) -> &Self {
        *self.selection_changed_callback.borrow_mut() = Some(Rc::new(handler));
        self
    }

    pub fn on_focus_changed(&self, handler: impl Fn(FocusChangedEventArgs) + 'static) -> &Self {
        *self.focus_changed_callback.borrow_mut() = Some(Rc::new(handler));
        self
    }

    pub fn focus_now(&self) -> &Self {
        self.focus_editor();
        self.editor_text.selection_range(
            self.selection_start_bytes.get(),
            self.selection_end_bytes.get(),
        );
        self
    }

    pub fn scroll_offset_x(&self) -> f32 {
        self.editor_scroll_box
            .as_ref()
            .map(|scroll_box| scroll_box.scroll_state().offset_x())
            .unwrap_or(0.0)
    }

    pub fn scroll_offset_y(&self) -> f32 {
        self.editor_scroll_box
            .as_ref()
            .map(|scroll_box| scroll_box.scroll_state().offset_y())
            .unwrap_or(0.0)
    }

    pub fn scroll_to(&self, x: f32, y: f32) -> &Self {
        if let Some(scroll_box) = &self.editor_scroll_box {
            scroll_box.scroll_to(x, y);
        }
        self
    }

    fn install_events(&self) {
        let weak = self.self_weak.borrow().clone();
        self.root.on_pointer_down(move |event| {
            if let Some(this) = weak.upgrade() {
                if event.target_handle() == this.root.handle() {
                    this.handle_shell_pointer_down();
                    event.handled = true;
                }
            }
        });

        let weak = self.self_weak.borrow().clone();
        self.placeholder_host.on_pointer_down(move |event| {
            if let Some(this) = weak.upgrade() {
                this.handle_shell_pointer_down();
                event.handled = true;
            }
        });

        if let Some(scroll_box) = &self.editor_scroll_box {
            let weak = self.self_weak.borrow().clone();
            scroll_box.viewport().on_pointer_down(move |_event| {
                if let Some(this) = weak.upgrade() {
                    this.handle_viewport_pointer_down();
                }
            });
        }

        let weak = self.self_weak.borrow().clone();
        self.editor_text
            .on_text_replaced(move |_start, _end, _text| {
                if let Some(this) = weak.upgrade() {
                    this.handle_editor_text_changed();
                }
            });

        let weak = self.self_weak.borrow().clone();
        self.editor_text.on_text_changed(move |_event| {
            if let Some(this) = weak.upgrade() {
                this.handle_editor_text_changed();
            }
        });

        let weak = self.self_weak.borrow().clone();
        self.editor_text.on_selection_changed(move |event| {
            if let Some(this) = weak.upgrade() {
                this.handle_editor_selection_changed(event.start, event.end);
            }
        });

        let weak = self.self_weak.borrow().clone();
        self.editor_text.on_key_down(move |event| {
            if let Some(this) = weak.upgrade() {
                this.handle_editor_key_down(event);
            }
        });

        let weak = self.self_weak.borrow().clone();
        self.editor_text.on_focus_changed(move |event| {
            if let Some(this) = weak.upgrade() {
                this.handle_editor_focus_changed(event.focused);
            }
        });
    }

    fn install_theme_subscription(&self) {
        let weak = self.self_weak.borrow().clone();
        let guard = subscribe(move |_theme| {
            if let Some(this) = weak.upgrade() {
                this.sync_theme_state();
            }
        });
        *self.theme_guard.borrow_mut() = Some(guard);
    }

    fn install_focus_visibility_subscription(&self) {
        let weak = self.self_weak.borrow().clone();
        let guard = focus_visibility::subscribe(move |_visible| {
            if let Some(this) = weak.upgrade() {
                this.sync_focus_chrome();
            }
        });
        *self.focus_visibility_guard.borrow_mut() = Some(guard);
    }

    fn install_effective_enabled_subscription(&self) {
        let weak = self.self_weak.borrow().clone();
        self.root
            .retained_node_ref()
            .on_effective_enabled_changed(Rc::new(move |_enabled| {
                let Some(this) = weak.upgrade() else {
                    return;
                };
                this.sync_editor_editability();
                this.sync_theme_state();
                this.sync_placeholder_visibility();
            }));
    }

    fn install_persisted_state(&self) {
        let weak = self.self_weak.borrow().clone();
        self.editor_text.persist_state(persisted_value_adapter(
            "text-input-value",
            PersistedStringCodec,
            1,
            {
                let weak = weak.clone();
                move || {
                    let this = weak.upgrade()?;
                    if this.password_value.get() {
                        None
                    } else {
                        Some(this.value())
                    }
                }
            },
            move |value| {
                let Some(this) = weak.upgrade() else {
                    return;
                };
                if this.password_value.get() {
                    return;
                }
                this.apply_persisted_text(value);
            },
        ));
    }

    fn sync_theme_state(&self) {
        let theme = current_theme();
        let resolved_font_family = self
            .font_family_override
            .borrow()
            .clone()
            .unwrap_or_else(|| theme.fonts.body_family.clone());
        let resolved_font_size = if self.has_font_size_override.get() {
            self.font_size_override.get()
        } else {
            theme.fonts.size_body
        };
        let line_height = resolved_font_size + theme.spacing.sm;
        let host_style = self.presenter.borrow().present(
            theme.clone(),
            &TextInputVisualState {
                multiline: self.profile.multiline,
                enabled: is_enabled(&self.root),
                wrapping: self.wrapping_value.get(),
            },
            self.colors_value.get(),
        );
        self.root.apply_presenter_style(host_style);

        let colors = self.colors_value.get();
        let text_color = if is_enabled(&self.root) {
            colors
                .filter(|value| value.has_text_primary())
                .map(|value| value.text_primary_color())
                .unwrap_or(theme.colors.text_primary)
        } else {
            colors
                .filter(|value| value.has_text_muted())
                .map(|value| value.text_muted_color())
                .unwrap_or(theme.colors.text_muted)
        };
        let caret_color = colors
            .filter(|value| value.has_caret())
            .map(|value| value.caret_color())
            .unwrap_or(theme.colors.accent);
        let placeholder_color = colors
            .filter(|value| value.has_placeholder())
            .map(|value| value.placeholder_color())
            .unwrap_or(theme.colors.text_muted);

        self.editor_text
            .width(
                if self.should_editor_track_viewport_width() {
                    100.0
                } else {
                    0.0
                },
                if self.should_editor_track_viewport_width() {
                    Unit::Percent
                } else {
                    Unit::Auto
                },
            )
            .height(
                if self.profile.multiline {
                    0.0
                } else {
                    line_height
                },
                if self.profile.multiline {
                    Unit::Auto
                } else {
                    Unit::Pixel
                },
            )
            .font_family(resolved_font_family.clone())
            .font_size(resolved_font_size)
            .text_vertical_align(if self.profile.multiline {
                TextVerticalAlign::Top
            } else {
                TextVerticalAlign::Center
            })
            .text_color(text_color)
            .caret_color(caret_color)
            .wrapping(self.wrapping_value.get());
        self.placeholder_host.width(100.0, Unit::Percent).height(
            if self.profile.multiline {
                0.0
            } else {
                line_height
            },
            if self.profile.multiline {
                Unit::Auto
            } else {
                Unit::Pixel
            },
        );
        self.placeholder_text
            .width(100.0, Unit::Percent)
            .height(
                if self.profile.multiline {
                    0.0
                } else {
                    line_height
                },
                if self.profile.multiline {
                    Unit::Auto
                } else {
                    Unit::Pixel
                },
            )
            .font_family(resolved_font_family)
            .font_size(resolved_font_size)
            .text_vertical_align(if self.profile.multiline {
                TextVerticalAlign::Top
            } else {
                TextVerticalAlign::Center
            })
            .text_color(placeholder_color)
            .wrapping(self.wrapping_value.get());
        if let Some(scroll_box) = &self.editor_scroll_box {
            scroll_box.flex_box_root().cursor(CursorStyle::Default);
            scroll_box.fill_size();
            self.sync_scroll_chrome_state();
        }
        self.sync_focus_chrome();
    }

    fn handle_editor_focus_changed(&self, focused: bool) {
        if self.focused_state.get() == focused {
            return;
        }
        self.focused_state.set(focused);
        self.sync_focus_chrome();
        self.sync_placeholder_visibility();
        if let Some(callback) = self.focus_changed_callback.borrow().clone() {
            callback(FocusChangedEventArgs { focused });
        }
    }

    fn handle_editor_text_changed(&self) {
        let value = self.editor_text.text_value();
        if *self.text_value.borrow() == value {
            return;
        }
        *self.text_value.borrow_mut() = value.clone();
        if self.password_value.get() {
            self.editor_text.obscured(true);
        }
        self.clamp_selection_to_text();
        self.sync_semantic_label();
        self.sync_placeholder_visibility();
        if let Some(callback) = self.changed_callback.borrow().clone() {
            callback(TextChangedEventArgs { text: value });
        }
    }

    fn handle_editor_selection_changed(&self, start: u32, end: u32) {
        self.selection_start_bytes.set(start);
        self.selection_end_bytes.set(end);
        let text = self.text_value.borrow();
        let start_chars = byte_to_char(&text, start);
        let end_chars = byte_to_char(&text, end);
        let start_bytes = char_to_byte(&text, start_chars);
        let end_bytes = char_to_byte(&text, end_chars);
        drop(text);
        self.selection_start_bytes.set(start_bytes);
        self.selection_end_bytes.set(end_bytes);
        self.selection_start_chars.set(start_chars);
        self.selection_end_chars.set(end_chars);
        if let Some(callback) = self.selection_changed_callback.borrow().clone() {
            callback(SelectionChangedEventArgs {
                start: start_chars,
                end: end_chars,
            });
        }
    }

    fn handle_editor_key_down(&self, event: &mut KeyEventArgs) {
        if !self.accepts_tab_value.get() && !Self::is_text_navigation_key(event) {
            return;
        }
        if event.event_type != KeyEventType::Down || !is_enabled(&self.root) {
            return;
        }
        if Self::is_text_navigation_key(event) {
            event.handled = true;
            return;
        }
        if event.key != "Tab" || event.modifiers != 0 || self.read_only_value.get() {
            return;
        }
        self.replace_selection_with_text("\t");
        event.handled = true;
    }

    fn is_text_navigation_key(event: &KeyEventArgs) -> bool {
        let non_shift_modifiers = event.modifiers
            & ((KeyModifier::Ctrl as u32) | (KeyModifier::Alt as u32) | (KeyModifier::Meta as u32));
        event.event_type == KeyEventType::Down
            && non_shift_modifiers == 0
            && matches!(
                event.key.as_str(),
                "ArrowLeft"
                    | "ArrowRight"
                    | "ArrowUp"
                    | "ArrowDown"
                    | "Home"
                    | "End"
                    | "PageUp"
                    | "PageDown"
            )
    }

    fn replace_selection_with_text(&self, inserted: &str) {
        let text = self.text_value.borrow().clone();
        let start = min(
            self.selection_start_chars.get(),
            self.selection_end_chars.get(),
        );
        let end = self
            .selection_start_chars
            .get()
            .max(self.selection_end_chars.get());
        let current_len = char_count(&text);
        let inserted_len = char_count(inserted);
        let replaced_len = end - start;
        if current_len - replaced_len + inserted_len > self.max_chars_value.get() as u32 {
            return;
        }

        let start_byte = char_to_byte(&text, start) as usize;
        let end_byte = char_to_byte(&text, end) as usize;
        let mut value =
            String::with_capacity(text.len() - (end_byte - start_byte) + inserted.len());
        value.push_str(&text[..start_byte]);
        value.push_str(inserted);
        value.push_str(&text[end_byte..]);
        let caret = start + inserted_len;

        *self.text_value.borrow_mut() = value.clone();
        let handle = self.editor_text.handle();
        if handle != NodeHandle::INVALID {
            let caret_byte = char_to_byte(&value, caret);
            ui::replace_text_range(
                handle.raw(),
                start_byte as u32,
                end_byte as u32,
                inserted,
                caret_byte,
            );
            self.selection_start_chars.set(caret);
            self.selection_end_chars.set(caret);
            self.selection_start_bytes.set(caret_byte);
            self.selection_end_bytes.set(caret_byte);
        } else {
            self.editor_text.text(value.clone());
            if self.password_value.get() {
                self.editor_text.obscured(true);
            }
            self.selection_range(caret, caret);
        }
        self.sync_semantic_label();
        self.sync_placeholder_visibility();
        if let Some(callback) = self.changed_callback.borrow().clone() {
            callback(TextChangedEventArgs { text: value });
        }
        if let Some(callback) = self.selection_changed_callback.borrow().clone() {
            callback(SelectionChangedEventArgs {
                start: caret,
                end: caret,
            });
        }
    }

    fn sync_editor_limits(&self) {
        self.editor_text
            .text_limits(self.max_chars_value.get(), self.profile.max_lines);
    }

    fn sync_editor_editability(&self) {
        self.editor_text.selectable(is_enabled(&self.root));
        self.editor_text
            .editable(!self.read_only_value.get() && is_enabled(&self.root));
    }

    fn sync_browser_input_metadata(&self) {
        let handle = self.editor_text.handle();
        if handle != NodeHandle::INVALID {
            ui::register_text_input_metadata(
                handle.raw(),
                self.password_value.get(),
                self.host_autofill_hint_value.borrow().as_deref(),
            );
        }
    }

    fn sync_semantic_label(&self) {
        let label = if !self.placeholder_value.borrow().is_empty() {
            self.placeholder_value.borrow().clone()
        } else if self.read_only_value.get() && !self.text_value.borrow().is_empty() {
            self.text_value.borrow().clone()
        } else if self.password_value.get() {
            "Password input".to_string()
        } else {
            self.profile.default_semantic_label.to_string()
        };
        self.editor_text.default_semantic_label(label);
    }

    fn sync_editor_wrapping(&self) {
        self.editor_text.wrapping(self.wrapping_value.get());
        self.placeholder_text.wrapping(self.wrapping_value.get());
        if self.profile.multiline {
            self.editor_text.width(
                if self.should_editor_track_viewport_width() {
                    100.0
                } else {
                    0.0
                },
                if self.should_editor_track_viewport_width() {
                    Unit::Percent
                } else {
                    Unit::Auto
                },
            );
        }
    }

    fn sync_scroll_chrome_state(&self) {
        if let Some(scroll_box) = &self.editor_scroll_box {
            let allow_horizontal_scroll = !self.wrapping_value.get();
            scroll_box
                .vertical_scrollbar_visibility(self.vertical_scrollbar_visibility_value.get())
                .horizontal_scrollbar_visibility(if allow_horizontal_scroll {
                    self.horizontal_scrollbar_visibility_value.get()
                } else {
                    ScrollBarVisibility::Never
                });
            if !self.profile.multiline {
                scroll_box.scroll_enabled_y(false);
            } else {
                scroll_box.scroll_enabled_y(true);
            }
            scroll_box.scroll_enabled_x(allow_horizontal_scroll);
        }
    }

    fn should_editor_track_viewport_width(&self) -> bool {
        !self.profile.multiline || self.wrapping_value.get()
    }

    fn sync_placeholder_visibility(&self) {
        let should_show =
            self.text_value.borrow().is_empty() && !self.placeholder_value.borrow().is_empty();
        self.placeholder_text.text(if should_show {
            self.placeholder_value.borrow().clone()
        } else {
            String::new()
        });
        if should_show {
            if !self.placeholder_attached.get() {
                self.placeholder_attached.set(true);
                self.root.child(&self.placeholder_host);
            }
            return;
        }
        if !self.placeholder_attached.get() {
            return;
        }
        self.placeholder_attached.set(false);
        self.root.remove_child(&self.placeholder_host);
    }

    fn clamp_selection_to_text(&self) {
        let text = self.text_value.borrow();
        let char_len = char_count(&text);
        let start_chars = min(self.selection_start_chars.get(), char_len);
        let end_chars = min(self.selection_end_chars.get(), char_len);
        let start_bytes = char_to_byte(&text, start_chars);
        let end_bytes = char_to_byte(&text, end_chars);
        drop(text);
        self.selection_start_chars.set(start_chars);
        self.selection_end_chars.set(end_chars);
        self.selection_start_bytes.set(start_bytes);
        self.selection_end_bytes.set(end_bytes);
    }

    fn apply_persisted_text(&self, value: String) {
        if *self.text_value.borrow() == value {
            return;
        }
        self.text(value.clone());
        if let Some(callback) = self.changed_callback.borrow().clone() {
            callback(TextChangedEventArgs { text: value });
        }
    }

    fn sync_focus_chrome(&self) {
        if self.focused_state.get()
            && is_enabled(&self.root)
            && focus_visibility::keyboard_focus_visible()
        {
            let corners = self
                .root
                .resolved_host_style()
                .corners
                .unwrap_or_else(|| crate::Corners::all(current_theme().spacing.sm));
            focus_adorner::show_standard_corners(
                &self.root,
                corners.top_left,
                corners.top_right,
                corners.bottom_right,
                corners.bottom_left,
            );
            return;
        }
        focus_adorner::hide_owner(&self.root);
    }

    fn focus_editor(&self) {
        if is_enabled(&self.root) {
            self.editor_text.focus_now();
        }
    }

    fn handle_shell_pointer_down(&self) {
        if !is_enabled(&self.root) {
            return;
        }
        if !self.profile.multiline {
            if self.focused_state.get() {
                self.focus_editor();
            } else {
                self.focus_now();
            }
            return;
        }
        self.handle_viewport_pointer_down();
    }

    fn handle_viewport_pointer_down(&self) {
        if !is_enabled(&self.root) {
            return;
        }
        if self.focused_state.get() {
            self.focus_editor();
            return;
        }
        self.focus_now();
    }
}

impl TextInputCore {
    pub(crate) fn build_control(&self) {
        self.root.build();
        self.sync_browser_input_metadata();
        self.editor_text.selection_range(
            self.selection_start_bytes.get(),
            self.selection_end_bytes.get(),
        );
    }

    pub(crate) fn editor_node(&self) -> TextCore {
        self.editor_text.clone()
    }
}

impl HasFlexBoxRoot for TextInputCore {
    fn flex_box_root(&self) -> &FlexBox {
        &self.root
    }
}
