use crate::assets;
use crate::bindings::ui;
use crate::ffi::{TextAlign, TextOverflow, TextVerticalAlign, Unit};
use crate::frame_scheduler::on_loaded;
use crate::logger::error;
use crate::node::{Node, TextNode};
use crate::theme;
use crate::typography::{FontFamily, FontStack, FontStyle, FontWeight};
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::{Rc, Weak};

const STYLE_RUN_WORD_STRIDE: usize = 7;

thread_local! {
    static PENDING_FONT_LAYOUTS: RefCell<Vec<Weak<RefCell<TextLayoutState>>>> = const { RefCell::new(Vec::new()) };
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TextMetrics {
    pub width: f32,
    pub height: f32,
    pub baseline: f32,
    pub line_count: u32,
    pub max_line_width: f32,
}

#[derive(Clone)]
pub struct TextLayoutReadyEventArgs {
    pub layout: TextLayout,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RichTextSpan {
    text: String,
    font_family: Option<FontFamily>,
    font_size: Option<f32>,
    font_weight: Option<FontWeight>,
    font_style: Option<FontStyle>,
    color: Option<u32>,
    background_color: Option<u32>,
    decoration_flags: u32,
}

impl RichTextSpan {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            font_family: None,
            font_size: None,
            font_weight: None,
            font_style: None,
            color: None,
            background_color: None,
            decoration_flags: 0,
        }
    }

    pub fn font_stack(mut self, stack: FontStack, size: f32) -> Self {
        self.font_family = Some(FontFamily::with_regular_stack(stack));
        self.font_size = Some(size);
        self
    }

    pub fn font_family(mut self, family: FontFamily) -> Self {
        self.font_family = Some(family);
        self
    }

    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = Some(size);
        self
    }

    pub fn font_weight(mut self, weight: FontWeight) -> Self {
        self.font_weight = Some(weight);
        self
    }

    pub fn font_style(mut self, style: FontStyle) -> Self {
        self.font_style = Some(style);
        self
    }

    pub fn bold(self) -> Self {
        self.font_weight(FontWeight::Bold)
    }

    pub fn italic(self) -> Self {
        self.font_style(FontStyle::Italic)
    }

    pub fn text_color(mut self, color: u32) -> Self {
        self.color = Some(color);
        self
    }

    pub fn background_color(mut self, color: u32) -> Self {
        self.background_color = Some(color);
        self
    }

    pub fn underline(mut self) -> Self {
        self.decoration_flags |= 1;
        self
    }

    pub fn strikethrough(mut self) -> Self {
        self.decoration_flags |= 2;
        self
    }
}

pub fn span(text: impl Into<String>) -> RichTextSpan {
    RichTextSpan::new(text)
}

#[derive(Clone)]
pub struct RichText {
    node: TextNode,
    state: Rc<RefCell<RichTextState>>,
}

#[derive(Clone, Debug, PartialEq)]
struct RichTextState {
    fragments: Vec<RichTextSpan>,
    base_font_family: Option<FontFamily>,
    has_base_font_value: bool,
    base_font_size: Option<f32>,
    base_font_weight: Option<FontWeight>,
    base_font_style: Option<FontStyle>,
    base_color: Option<u32>,
}

impl RichText {
    pub fn new(fragments: Vec<RichTextSpan>) -> Self {
        let rich_text = Self {
            node: TextNode::new(""),
            state: Rc::new(RefCell::new(RichTextState {
                fragments: Vec::new(),
                base_font_family: None,
                has_base_font_value: false,
                base_font_size: None,
                base_font_weight: None,
                base_font_style: None,
                base_color: None,
            })),
        };
        rich_text.fragments_value(fragments);
        rich_text
    }

    pub fn from_text(text: impl Into<String>) -> Self {
        Self::new(vec![span(text)])
    }

    pub fn fragments_value(&self, fragments: Vec<RichTextSpan>) -> &Self {
        self.state.borrow_mut().fragments = fragments;
        self.rebuild_attributed_text();
        self
    }

    pub fn font_stack(&self, stack: FontStack, size: f32) -> &Self {
        let mut state = self.state.borrow_mut();
        state.has_base_font_value = true;
        state.base_font_family = Some(FontFamily::with_regular_stack(stack));
        state.base_font_size = Some(size);
        drop(state);
        self.rebuild_attributed_text();
        self
    }

    pub fn font_family(&self, family: FontFamily) -> &Self {
        let mut state = self.state.borrow_mut();
        state.has_base_font_value = true;
        state.base_font_family = Some(family);
        drop(state);
        self.rebuild_attributed_text();
        self
    }

    pub fn font_weight(&self, weight: FontWeight) -> &Self {
        let mut state = self.state.borrow_mut();
        state.has_base_font_value = true;
        state.base_font_weight = Some(weight);
        drop(state);
        self.rebuild_attributed_text();
        self
    }

    pub fn font_style(&self, style: FontStyle) -> &Self {
        let mut state = self.state.borrow_mut();
        state.has_base_font_value = true;
        state.base_font_style = Some(style);
        drop(state);
        self.rebuild_attributed_text();
        self
    }

    pub fn font_size(&self, size: f32) -> &Self {
        let mut state = self.state.borrow_mut();
        state.has_base_font_value = true;
        state.base_font_size = Some(size);
        drop(state);
        self.rebuild_attributed_text();
        self
    }

    pub fn text_color(&self, color: u32) -> &Self {
        self.state.borrow_mut().base_color = Some(color);
        self.rebuild_attributed_text();
        self
    }

    pub fn push(&self, fragment: RichTextSpan) -> &Self {
        self.state.borrow_mut().fragments.push(fragment);
        self.rebuild_attributed_text();
        self
    }

    pub fn text(&self, content: impl Into<String>) -> &Self {
        self.fragments_value(vec![span(content)])
    }

    fn rebuild_attributed_text(&self) {
        let compiled = self.compile();
        if compiled.apply_base_font {
            self.node
                .font_id(compiled.base_font_id, compiled.base_font_size);
        }
        self.node.text_color(compiled.base_color);
        self.node.text(compiled.content);
        self.node.style_runs(compiled.runs);
    }

    fn compile(&self) -> CompiledRichText {
        let state = self.state.borrow();
        compile_rich_text_state(&state)
    }
}

impl Deref for RichText {
    type Target = TextNode;

    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl Node for RichText {
    fn retained_node_ref(&self) -> crate::node::NodeRef {
        self.node.retained_node_ref()
    }

    fn build_self(&self) {
        self.node.build_self();
    }
}

fn compile_rich_text_state(state: &RichTextState) -> CompiledRichText {
    let mut content = String::new();
    let mut runs = Vec::with_capacity(state.fragments.len() * STYLE_RUN_WORD_STRIDE);
    let theme = theme::current_theme();
    let default_family = state
        .base_font_family
        .clone()
        .unwrap_or_else(|| theme.fonts.body_family.clone());
    let default_weight = state.base_font_weight.unwrap_or(FontWeight::Regular);
    let default_style = state.base_font_style.unwrap_or(FontStyle::Normal);
    let default_size = state.base_font_size.unwrap_or(theme.fonts.size_body);
    let base_font_id = default_family.resolve(default_weight, default_style);
    let base_color = state.base_color.unwrap_or(theme.colors.text_primary);
    let mut start = 0u32;
    for fragment in &state.fragments {
        content.push_str(&fragment.text);
        let end = start + fragment.text.len() as u32;
        let family = fragment
            .font_family
            .clone()
            .unwrap_or_else(|| default_family.clone());
        let weight = fragment.font_weight.unwrap_or(default_weight);
        let style = fragment.font_style.unwrap_or(default_style);
        runs.push(start);
        runs.push(end);
        runs.push(family.resolve(weight, style));
        runs.push(fragment.font_size.unwrap_or(default_size).to_bits());
        runs.push(fragment.color.unwrap_or(base_color));
        runs.push(fragment.background_color.unwrap_or(0));
        runs.push(fragment.decoration_flags);
        start = end;
    }
    CompiledRichText {
        content,
        apply_base_font: state.has_base_font_value || !state.fragments.is_empty(),
        base_font_id,
        base_font_size: default_size,
        base_color,
        runs,
    }
}

struct CompiledRichText {
    content: String,
    apply_base_font: bool,
    base_font_id: u32,
    base_font_size: f32,
    base_color: u32,
    runs: Vec<u32>,
}

type ReadyCallback = Rc<dyn Fn(TextLayoutReadyEventArgs)>;

struct TextLayoutState {
    node: TextNode,
    ready: bool,
    metrics: TextMetrics,
    dynamic_charset: Option<String>,
    ready_callbacks: Vec<ReadyCallback>,
    loaded_callback_registered: bool,
    waiting_for_fonts: bool,
}

#[derive(Clone)]
pub struct TextLayout {
    inner: Rc<RefCell<TextLayoutState>>,
}

impl TextLayout {
    pub fn text(text: impl Into<String>) -> Self {
        Self::from_node(TextNode::new(text.into()))
    }

    pub fn rich(fragments: Vec<RichTextSpan>) -> Self {
        let rich_text = RichText::new(fragments);
        Self::from_node(rich_text.node.clone())
    }

    fn from_node(node: TextNode) -> Self {
        Self {
            inner: Rc::new(RefCell::new(TextLayoutState {
                node,
                ready: false,
                metrics: TextMetrics::default(),
                dynamic_charset: None,
                ready_callbacks: Vec::new(),
                loaded_callback_registered: false,
                waiting_for_fonts: false,
            })),
        }
    }

    pub fn is_ready(&self) -> bool {
        self.inner.borrow().ready
    }

    pub fn draw_node(&self) -> TextNode {
        self.ensure_built();
        self.inner.borrow().node.clone()
    }

    pub fn measure(&self) -> TextMetrics {
        if !self.is_ready() {
            error(
                "TextLayout",
                "TextLayout.measure() called before the TextLayout was ready; register on_ready and measure after the callback.",
            );
            return TextMetrics::default();
        }
        self.inner.borrow().metrics
    }

    pub fn measured_width(&self) -> f32 {
        self.measure().width
    }

    pub fn measured_height(&self) -> f32 {
        self.measure().height
    }

    pub fn on_ready(&self, callback: impl Fn(TextLayoutReadyEventArgs) + 'static) -> &Self {
        if self.is_ready() {
            callback(TextLayoutReadyEventArgs {
                layout: self.clone(),
            });
            return self;
        }
        self.inner
            .borrow_mut()
            .ready_callbacks
            .push(Rc::new(callback) as ReadyCallback);
        self.schedule_ready();
        self
    }

    pub fn set_text(&self, value: impl Into<String>) -> &Self {
        self.inner.borrow().node.text(value.into());
        self.mark_dirty();
        self
    }

    pub fn width(&self, value: f32, unit: Unit) -> &Self {
        self.inner.borrow().node.width(value, unit);
        self.mark_dirty();
        self
    }

    pub fn height(&self, value: f32, unit: Unit) -> &Self {
        self.inner.borrow().node.height(value, unit);
        self.mark_dirty();
        self
    }

    pub fn font_stack(&self, stack: FontStack, size: f32) -> &Self {
        self.inner.borrow().node.font_stack(stack, size);
        self.mark_dirty();
        self
    }

    pub fn font_family(&self, family: FontFamily) -> &Self {
        self.inner.borrow().node.font_family(family);
        self.mark_dirty();
        self
    }

    pub fn font_weight(&self, weight: FontWeight) -> &Self {
        self.inner.borrow().node.font_weight(weight);
        self.mark_dirty();
        self
    }

    pub fn font_style(&self, style: FontStyle) -> &Self {
        self.inner.borrow().node.font_style(style);
        self.mark_dirty();
        self
    }

    pub fn font_size(&self, size: f32) -> &Self {
        self.inner.borrow().node.font_size(size);
        self.mark_dirty();
        self
    }

    pub fn line_height(&self, line_height: f32) -> &Self {
        self.inner.borrow().node.line_height(line_height);
        self.mark_dirty();
        self
    }

    pub fn text_color(&self, color: u32) -> &Self {
        self.inner.borrow().node.text_color(color);
        self.mark_dirty();
        self
    }

    pub fn text_align(&self, align: TextAlign) -> &Self {
        self.inner.borrow().node.text_align(align);
        self.mark_dirty();
        self
    }

    pub fn text_vertical_align(&self, align: TextVerticalAlign) -> &Self {
        self.inner.borrow().node.text_vertical_align(align);
        self.mark_dirty();
        self
    }

    pub fn text_limits(&self, max_chars: i32, max_lines: i32) -> &Self {
        self.inner.borrow().node.text_limits(max_chars, max_lines);
        self.mark_dirty();
        self
    }

    pub fn max_lines(&self, max_lines: i32) -> &Self {
        self.text_limits(i32::MAX, max_lines)
    }

    pub fn wrapping(&self, wrap: bool) -> &Self {
        self.inner.borrow().node.wrapping(wrap);
        self.mark_dirty();
        self
    }

    pub fn wrap(&self, wrap: bool) -> &Self {
        self.wrapping(wrap)
    }

    pub fn text_overflow(&self, overflow: TextOverflow) -> &Self {
        self.inner.borrow().node.text_overflow(overflow);
        self.mark_dirty();
        self
    }

    pub fn overflow(&self, overflow: TextOverflow) -> &Self {
        self.text_overflow(overflow)
    }

    pub fn text_overflow_fade(&self, horizontal: bool, vertical: bool) -> &Self {
        self.inner
            .borrow()
            .node
            .text_overflow_fade(horizontal, vertical);
        self.mark_dirty();
        self
    }

    fn set_dynamic_charset_internal(&self, charset: String) {
        self.inner.borrow_mut().dynamic_charset = Some(charset);
        self.mark_dirty();
    }

    fn ensure_built(&self) {
        let node = self.inner.borrow().node.clone();
        node.build();
    }

    fn required_font_ids(&self) -> Vec<u32> {
        self.inner.borrow().node.required_font_ids()
    }

    fn fonts_ready(&self) -> bool {
        let required = self.required_font_ids();
        if required.is_empty() {
            return true;
        }
        required.into_iter().all(assets::is_font_loaded)
    }

    fn mark_dirty(&self) {
        let mut state = self.inner.borrow_mut();
        state.ready = false;
        state.metrics = TextMetrics::default();
        state.waiting_for_fonts = false;
    }

    fn schedule_ready(&self) {
        let already_registered = {
            let mut state = self.inner.borrow_mut();
            if state.loaded_callback_registered {
                true
            } else {
                state.loaded_callback_registered = true;
                false
            }
        };
        if already_registered {
            return;
        }
        let layout = self.clone();
        on_loaded(move |_| {
            layout.inner.borrow_mut().loaded_callback_registered = false;
            layout.prepare_or_wait();
        });
    }

    fn prepare_or_wait(&self) {
        self.ensure_built();
        if !self.fonts_ready() {
            self.register_waiting_for_fonts();
            return;
        }
        self.prepare_now();
    }

    fn register_waiting_for_fonts(&self) {
        let already_waiting = {
            let mut state = self.inner.borrow_mut();
            if state.waiting_for_fonts {
                true
            } else {
                state.waiting_for_fonts = true;
                false
            }
        };
        if already_waiting {
            return;
        }
        let weak = Rc::downgrade(&self.inner);
        PENDING_FONT_LAYOUTS.with(|layouts| layouts.borrow_mut().push(weak));
    }

    fn prepare_now(&self) {
        self.ensure_built();
        let (node, dynamic_charset) = {
            let state = self.inner.borrow();
            (state.node.clone(), state.dynamic_charset.clone())
        };
        let handle = node.handle().raw();
        if let Some(charset) = dynamic_charset {
            ui::set_dynamic_text_charset(handle, &charset);
        }
        if ui::prepare_node(handle) == 0 {
            let mut state = self.inner.borrow_mut();
            state.ready = false;
            state.metrics = TextMetrics::default();
            return;
        }
        let metrics = ui::get_text_metrics(handle).unwrap_or([0.0, 0.0, 0.0, 0.0, 0.0]);
        let callbacks = {
            let mut state = self.inner.borrow_mut();
            state.ready = true;
            state.metrics = TextMetrics {
                width: metrics[0],
                height: metrics[1],
                baseline: metrics[2],
                line_count: metrics[3] as u32,
                max_line_width: metrics[4],
            };
            state.waiting_for_fonts = false;
            std::mem::take(&mut state.ready_callbacks)
        };
        for callback in callbacks {
            callback(TextLayoutReadyEventArgs {
                layout: self.clone(),
            });
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DynamicTextOverflow {
    Reject = 0,
    FallbackShape = 1,
}

struct DynamicTextLayoutState {
    layout: TextLayout,
    charset: String,
    overflow: DynamicTextOverflow,
    current_text: String,
    numeric_mode: bool,
    numeric_precision: i32,
    numeric_prefix: String,
    numeric_suffix: String,
    has_numeric_value: bool,
    numeric_value: f64,
}

#[derive(Clone)]
pub struct DynamicTextLayout {
    inner: Rc<RefCell<DynamicTextLayoutState>>,
}

impl DynamicTextLayout {
    pub fn fixed_charset(charset: impl Into<String>) -> Self {
        let layout = TextLayout::text("");
        let charset = charset.into();
        layout.set_dynamic_charset_internal(charset.clone());
        Self {
            inner: Rc::new(RefCell::new(DynamicTextLayoutState {
                layout,
                charset,
                overflow: DynamicTextOverflow::FallbackShape,
                current_text: String::new(),
                numeric_mode: false,
                numeric_precision: -1,
                numeric_prefix: String::new(),
                numeric_suffix: String::new(),
                has_numeric_value: false,
                numeric_value: 0.0,
            })),
        }
    }

    pub fn numeric() -> Self {
        let layout = Self::fixed_charset("0123456789.-");
        layout.inner.borrow_mut().numeric_mode = true;
        layout
    }

    pub fn is_ready(&self) -> bool {
        self.inner.borrow().layout.is_ready()
    }

    pub fn measure(&self) -> TextMetrics {
        self.inner.borrow().layout.measure()
    }

    pub fn current_text(&self) -> String {
        self.inner.borrow().current_text.clone()
    }

    pub fn on_ready(&self, callback: impl Fn(TextLayoutReadyEventArgs) + 'static) -> &Self {
        self.inner.borrow().layout.on_ready(callback);
        self
    }

    pub fn width(&self, value: f32, unit: Unit) -> &Self {
        self.inner.borrow().layout.width(value, unit);
        self
    }

    pub fn height(&self, value: f32, unit: Unit) -> &Self {
        self.inner.borrow().layout.height(value, unit);
        self
    }

    pub fn font_stack(&self, stack: FontStack, size: f32) -> &Self {
        self.inner.borrow().layout.font_stack(stack, size);
        self
    }

    pub fn font_family(&self, family: FontFamily) -> &Self {
        self.inner.borrow().layout.font_family(family);
        self
    }

    pub fn font_weight(&self, weight: FontWeight) -> &Self {
        self.inner.borrow().layout.font_weight(weight);
        self
    }

    pub fn font_style(&self, style: FontStyle) -> &Self {
        self.inner.borrow().layout.font_style(style);
        self
    }

    pub fn font_size(&self, size: f32) -> &Self {
        self.inner.borrow().layout.font_size(size);
        self
    }

    pub fn line_height(&self, line_height: f32) -> &Self {
        self.inner.borrow().layout.line_height(line_height);
        self
    }

    pub fn text_color(&self, color: u32) -> &Self {
        self.inner.borrow().layout.text_color(color);
        self
    }

    pub fn text_align(&self, align: TextAlign) -> &Self {
        self.inner.borrow().layout.text_align(align);
        self
    }

    pub fn text_vertical_align(&self, align: TextVerticalAlign) -> &Self {
        self.inner.borrow().layout.text_vertical_align(align);
        self
    }

    pub fn text_limits(&self, max_chars: i32, max_lines: i32) -> &Self {
        self.inner.borrow().layout.text_limits(max_chars, max_lines);
        self
    }

    pub fn max_lines(&self, max_lines: i32) -> &Self {
        self.inner.borrow().layout.max_lines(max_lines);
        self
    }

    pub fn wrap(&self, wrap: bool) -> &Self {
        self.inner.borrow().layout.wrap(wrap);
        self
    }

    pub fn wrapping(&self, wrap: bool) -> &Self {
        self.wrap(wrap)
    }

    pub fn text_overflow(&self, overflow: TextOverflow) -> &Self {
        self.inner.borrow().layout.text_overflow(overflow);
        self
    }

    pub fn overflow(&self, mode: DynamicTextOverflow) -> &Self {
        self.inner.borrow_mut().overflow = mode;
        self
    }

    pub fn set_text(&self, value: impl Into<String>) -> bool {
        let value = value.into();
        if !self.supports_text(&value)
            && self.inner.borrow().overflow == DynamicTextOverflow::Reject
        {
            return false;
        }
        let layout = self.inner.borrow().layout.clone();
        let was_ready = layout.is_ready();
        self.inner.borrow_mut().current_text = value.clone();
        layout.set_text(value);
        if was_ready {
            layout.prepare_or_wait();
        }
        true
    }

    pub fn text(&self, value: impl Into<String>) -> &Self {
        let _ = self.set_text(value);
        self
    }

    pub fn precision(&self, digits: i32) -> &Self {
        let mut state = self.inner.borrow_mut();
        state.numeric_mode = true;
        state.numeric_precision = digits.max(0);
        drop(state);
        self.refresh_numeric_text();
        self
    }

    pub fn prefix(&self, value: impl Into<String>) -> &Self {
        let value = value.into();
        {
            let mut state = self.inner.borrow_mut();
            state.numeric_mode = true;
            state.numeric_prefix = value.clone();
            Self::include_in_charset(&mut state, &value);
            let layout = state.layout.clone();
            let charset = state.charset.clone();
            drop(state);
            layout.set_dynamic_charset_internal(charset);
        }
        self.refresh_numeric_text();
        self
    }

    pub fn suffix(&self, value: impl Into<String>) -> &Self {
        let value = value.into();
        {
            let mut state = self.inner.borrow_mut();
            state.numeric_mode = true;
            state.numeric_suffix = value.clone();
            Self::include_in_charset(&mut state, &value);
            let layout = state.layout.clone();
            let charset = state.charset.clone();
            drop(state);
            layout.set_dynamic_charset_internal(charset);
        }
        self.refresh_numeric_text();
        self
    }

    pub fn set_value(&self, value: f64) -> bool {
        {
            let mut state = self.inner.borrow_mut();
            state.numeric_mode = true;
            state.has_numeric_value = true;
            state.numeric_value = value;
        }
        self.set_text(self.compose_numeric_text(value))
    }

    pub fn draw_node(&self) -> TextNode {
        self.inner.borrow().layout.draw_node()
    }

    fn supports_text(&self, value: &str) -> bool {
        let charset = self.inner.borrow().charset.clone();
        if charset.is_empty() {
            return true;
        }
        value.chars().all(|ch| charset.contains(ch))
    }

    fn include_in_charset(state: &mut DynamicTextLayoutState, value: &str) {
        for ch in value.chars() {
            if !state.charset.contains(ch) {
                state.charset.push(ch);
            }
        }
    }

    fn refresh_numeric_text(&self) {
        let (numeric_mode, has_value, numeric_value) = {
            let state = self.inner.borrow();
            (
                state.numeric_mode,
                state.has_numeric_value,
                state.numeric_value,
            )
        };
        if !numeric_mode || !has_value {
            return;
        }
        let _ = self.set_text(self.compose_numeric_text(numeric_value));
    }

    fn compose_numeric_text(&self, value: f64) -> String {
        let mut state = self.inner.borrow_mut();
        let prefix = state.numeric_prefix.clone();
        let suffix = state.numeric_suffix.clone();
        Self::include_in_charset(&mut state, &prefix);
        Self::include_in_charset(&mut state, &suffix);
        let charset = state.charset.clone();
        let layout = state.layout.clone();
        drop(state);
        layout.set_dynamic_charset_internal(charset);
        format!("{prefix}{}{suffix}", self.format_numeric_value(value))
    }

    fn format_numeric_value(&self, value: f64) -> String {
        let precision = self.inner.borrow().numeric_precision;
        if value.is_nan() || !value.is_finite() || precision < 0 {
            return value.to_string();
        }
        format!("{value:.precision$}", precision = precision as usize)
    }
}

pub(crate) fn notify_font_loaded(_font_id: u32) {
    PENDING_FONT_LAYOUTS.with(|layouts| {
        let mut layouts = layouts.borrow_mut();
        layouts.retain(|weak| {
            let Some(inner) = weak.upgrade() else {
                return false;
            };
            let layout = TextLayout { inner };
            if layout.fonts_ready() {
                layout.prepare_now();
                false
            } else {
                true
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::{
        notify_font_loaded, span, DynamicTextLayout, DynamicTextOverflow, RichText, TextLayout,
    };
    use crate::assets;
    use crate::ffi::{self, Call, TextAlign, TextOverflow, TextVerticalAlign, Unit};
    use crate::frame_scheduler;
    use crate::node::Node;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn rich_text_emits_style_runs() {
        ffi::test::reset();
        let node = RichText::new(vec![span("Hello")
            .font_stack(crate::typography::FontStack::from_id(7), 18.0)
            .text_color(0xFF00FFFF)]);
        node.build();
        let calls = ffi::test::take_calls();
        assert!(calls
            .iter()
            .any(|call| matches!(call, Call::SetTextStyleRuns { run_count: 1, .. })));
    }

    #[test]
    fn rich_text_retained_mutations_update_text_and_style_runs() {
        ffi::test::reset();
        let node = RichText::from_text("Before");
        node.build();
        ffi::test::take_calls();

        node.fragments_value(vec![
            span("After").font_stack(crate::typography::FontStack::from_id(8), 19.0),
            span(" ✅").text_color(0x00FF00FF),
        ]);

        let calls = ffi::test::take_calls();
        assert!(calls
            .iter()
            .any(|call| matches!(call, Call::SetText { text, .. } if text == "After ✅")));
        assert!(calls
            .iter()
            .any(|call| matches!(call, Call::SetTextStyleRuns { run_count: 2, .. })));
    }

    #[test]
    fn rich_text_inherits_fui_as_max_lines_surface() {
        ffi::test::reset();
        let node = RichText::from_text("One line");
        node.max_lines(1).build();
        assert!(ffi::test::take_calls().iter().any(|call| matches!(
            call,
            Call::SetTextLimits {
                max_chars,
                max_lines,
                ..
            } if *max_chars == i32::MAX && *max_lines == 1
        )));
    }

    #[test]
    fn text_layout_waits_for_loaded_and_fonts_before_reporting_ready() {
        ffi::test::reset();
        frame_scheduler::reset_commit_state();
        ffi::test::set_text_metrics(88.0, 22.0, 16.0, 2, 64.0);

        let ready_count = Rc::new(RefCell::new(0));
        let layout = TextLayout::text("Layout");
        layout
            .font_stack(crate::typography::FontStack::from_id(99), 16.0)
            .on_ready({
                let ready_count = ready_count.clone();
                move |_| *ready_count.borrow_mut() += 1
            });

        frame_scheduler::fire_loaded_callbacks();
        assert_eq!(*ready_count.borrow(), 0);

        assets::on_font_loaded(99);
        notify_font_loaded(99);
        assert_eq!(*ready_count.borrow(), 1);

        let calls = ffi::test::take_calls();
        assert!(calls
            .iter()
            .any(|call| matches!(call, Call::PrepareNode { .. })));
        assert!(calls
            .iter()
            .any(|call| matches!(call, Call::GetTextMetrics { .. })));
    }

    #[test]
    fn text_layout_emits_display_configuration() {
        ffi::test::reset();
        frame_scheduler::reset_commit_state();
        assets::on_font_loaded(1);
        TextLayout::text("Layout")
            .line_height(26.0)
            .text_align(TextAlign::Right)
            .text_vertical_align(TextVerticalAlign::Center)
            .text_limits(12, 3)
            .wrapping(true)
            .text_overflow(TextOverflow::Ellipsis)
            .text_overflow_fade(true, false)
            .on_ready(|_| {});
        frame_scheduler::fire_loaded_callbacks();
        let calls = ffi::test::take_calls();
        assert!(calls.iter().any(|call| matches!(call, Call::SetLineHeight { line_height, .. } if (*line_height - 26.0).abs() < f32::EPSILON)));
        assert!(calls.iter().any(|call| matches!(call, Call::SetTextAlign { align_enum, .. } if *align_enum == TextAlign::Right as u32)));
        assert!(calls.iter().any(|call| matches!(call, Call::SetTextVerticalAlign { align_enum, .. } if *align_enum == TextVerticalAlign::Center as u32)));
        assert!(calls.iter().any(|call| matches!(call, Call::SetTextLimits { max_chars, max_lines, .. } if *max_chars == 12 && *max_lines == 3)));
        assert!(calls
            .iter()
            .any(|call| matches!(call, Call::SetTextWrapping { wrap: true, .. })));
        assert!(calls.iter().any(|call| matches!(call, Call::SetTextOverflow { overflow_enum, .. } if *overflow_enum == TextOverflow::Ellipsis as u32)));
        assert!(calls.iter().any(|call| matches!(
            call,
            Call::SetTextOverflowFade {
                horizontal: true,
                vertical: false,
                ..
            }
        )));
    }

    #[test]
    fn dynamic_text_layout_rejects_unsupported_text_when_configured() {
        ffi::test::reset();
        let layout = DynamicTextLayout::fixed_charset("0123456789");
        layout.overflow(DynamicTextOverflow::Reject);
        assert!(layout.set_text("123"));
        assert!(!layout.set_text("12a"));
    }

    #[test]
    fn dynamic_text_layout_reprepares_after_ready_text_update() {
        ffi::test::reset();
        frame_scheduler::reset_commit_state();
        assets::test_reset();

        let layout = DynamicTextLayout::fixed_charset("0123456789");
        layout
            .font_stack(crate::typography::FontStack::from_id(1), 16.0)
            .width(120.0, Unit::Pixel)
            .height(24.0, Unit::Pixel)
            .on_ready(|_| {});
        frame_scheduler::fire_loaded_callbacks();
        assert!(layout.is_ready());
        ffi::test::take_calls();

        assert!(layout.set_text("42"));
        assert!(layout.is_ready());
        let calls = ffi::test::take_calls();
        assert!(calls
            .iter()
            .any(|call| matches!(call, Call::PrepareNode { .. })));
        assert!(calls
            .iter()
            .any(|call| matches!(call, Call::SetDynamicTextCharset { charset, .. } if charset == "0123456789")));
    }
}
