use crate::controls::{DropdownColors, DropdownSizing};
use crate::ffi::{AlignItems, FlexDirection, JustifyContent, TextVerticalAlign, Unit};
use crate::node::{flex_box, text, FlexBox, TextNode};
use crate::theme::Theme;
use std::rc::Rc;

const DEFAULT_CHEVRON_BOX_SIZE: f32 = 16.0;
const DEFAULT_FIELD_PADDING_X: f32 = 16.0;
const DEFAULT_FIELD_FONT_SIZE: f32 = 16.0;
const DEFAULT_FIELD_HEIGHT: f32 = 32.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DropdownFieldMetrics {
    pub height: f32,
    pub font_size: f32,
    pub chevron_box_size: f32,
    pub padding_left: f32,
    pub padding_top: f32,
    pub padding_right: f32,
    pub padding_bottom: f32,
}

impl DropdownFieldMetrics {
    pub const fn new(
        height: f32,
        font_size: f32,
        chevron_box_size: f32,
        padding_left: f32,
        padding_top: f32,
        padding_right: f32,
        padding_bottom: f32,
    ) -> Self {
        Self {
            height,
            font_size,
            chevron_box_size,
            padding_left,
            padding_top,
            padding_right,
            padding_bottom,
        }
    }
}

pub const DEFAULT_DROPDOWN_FIELD_METRICS: DropdownFieldMetrics = DropdownFieldMetrics::new(
    DEFAULT_FIELD_HEIGHT,
    DEFAULT_FIELD_FONT_SIZE,
    DEFAULT_CHEVRON_BOX_SIZE,
    DEFAULT_FIELD_PADDING_X,
    0.0,
    DEFAULT_FIELD_PADDING_X,
    0.0,
);

fn resolve_field_metrics(sizing: Option<DropdownSizing>) -> DropdownFieldMetrics {
    let Some(sizing) = sizing else {
        return DEFAULT_DROPDOWN_FIELD_METRICS;
    };
    if !sizing.has_field_height() && !sizing.has_field_font_size() && !sizing.has_chevron_box_size()
    {
        return DEFAULT_DROPDOWN_FIELD_METRICS;
    }
    let font_size = if sizing.has_field_font_size() {
        sizing.field_font_size_px()
    } else {
        DEFAULT_DROPDOWN_FIELD_METRICS.font_size
    };
    let chevron_box_size = if sizing.has_chevron_box_size() {
        sizing.chevron_box_size_px()
    } else {
        DEFAULT_DROPDOWN_FIELD_METRICS.chevron_box_size
    };
    let content_height = font_size.max(chevron_box_size);
    let height = if sizing.has_field_height() {
        sizing.field_height_px()
    } else {
        DEFAULT_DROPDOWN_FIELD_METRICS.height.max(content_height)
    };
    DropdownFieldMetrics::new(
        height,
        font_size,
        chevron_box_size,
        DEFAULT_FIELD_PADDING_X,
        0.0,
        DEFAULT_FIELD_PADDING_X,
        0.0,
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DropdownFieldVisualState {
    pub open: bool,
    pub focused: bool,
    pub enabled: bool,
    pub pressed: bool,
    pub selected_label: String,
}

impl DropdownFieldVisualState {
    pub fn new(
        open: bool,
        focused: bool,
        enabled: bool,
        pressed: bool,
        selected_label: impl Into<String>,
    ) -> Self {
        Self {
            open,
            focused,
            enabled,
            pressed,
            selected_label: selected_label.into(),
        }
    }
}

pub trait DropdownFieldPresenter {
    fn root(&self) -> FlexBox;
    fn value_host(&self) -> FlexBox;
    fn value_node(&self) -> TextNode;
    fn chevron_host(&self) -> FlexBox;
    fn metrics(&self) -> DropdownFieldMetrics;
    fn apply(&self, theme: Theme, state: &DropdownFieldVisualState, colors: Option<DropdownColors>);
}

pub trait DropdownFieldTemplate {
    fn create(&self, sizing: Option<DropdownSizing>) -> Rc<dyn DropdownFieldPresenter>;
}

#[derive(Clone)]
pub struct DefaultDropdownFieldPresenter {
    root: FlexBox,
    value_host: FlexBox,
    value_node: TextNode,
    chevron_host: FlexBox,
    metrics: DropdownFieldMetrics,
}

impl DefaultDropdownFieldPresenter {
    pub fn new(metrics: DropdownFieldMetrics) -> Self {
        let theme = crate::theme::current_theme();
        let value_node = text("");
        value_node
            .selectable(false, theme.colors.selection)
            .fill_size()
            .text_limits(0, 1)
            .wrapping(false);
        value_node
            .text_overflow_fade(true, false)
            .text_vertical_align(TextVerticalAlign::Center);
        let value_host = flex_box();
        value_host.fill_size().child(&value_node);
        let chevron_host = flex_box();
        chevron_host
            .width(metrics.chevron_box_size, Unit::Pixel)
            .height(metrics.chevron_box_size, Unit::Pixel)
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center);
        let root = flex_box();
        root.flex_direction(FlexDirection::Row)
            .align_items(AlignItems::Center)
            .child(&value_host)
            .child(&chevron_host);
        Self {
            root,
            value_host,
            value_node: value_node.clone(),
            chevron_host,
            metrics,
        }
    }
}

impl DropdownFieldPresenter for DefaultDropdownFieldPresenter {
    fn root(&self) -> FlexBox {
        self.root.clone()
    }

    fn value_host(&self) -> FlexBox {
        self.value_host.clone()
    }

    fn value_node(&self) -> TextNode {
        self.value_node.clone()
    }

    fn chevron_host(&self) -> FlexBox {
        self.chevron_host.clone()
    }

    fn metrics(&self) -> DropdownFieldMetrics {
        self.metrics
    }

    fn apply(
        &self,
        theme: Theme,
        state: &DropdownFieldVisualState,
        colors: Option<DropdownColors>,
    ) {
        let metrics = self.metrics;
        let content_height = metrics
            .font_size
            .max(metrics.height - metrics.padding_top - metrics.padding_bottom);
        let bg = if colors.is_some_and(|colors| colors.has_background()) {
            colors.unwrap().background_color()
        } else if state.pressed && state.enabled {
            theme.colors.background
        } else {
            theme.colors.surface
        };
        let border_color = colors
            .filter(|colors| colors.has_border())
            .map(|colors| colors.border_color())
            .unwrap_or(theme.colors.border);
        self.root
            .flex_direction(FlexDirection::Row)
            .align_items(AlignItems::Center)
            .height(metrics.height, Unit::Pixel)
            .corner_radius(theme.spacing.sm)
            .border(2.0, border_color)
            .padding(
                metrics.padding_left,
                metrics.padding_top,
                metrics.padding_right,
                metrics.padding_bottom,
            )
            .bg_color(bg);
        self.value_host.fill_size();
        let text_color = if !state.enabled {
            theme.colors.text_muted
        } else {
            colors
                .filter(|colors| colors.has_text_primary())
                .map(|colors| colors.text_primary_color())
                .unwrap_or(theme.colors.text_primary)
        };
        self.value_node
            .font_family(theme.fonts.body_family.clone())
            .font_size(metrics.font_size)
            .line_height(content_height)
            .text_color(text_color);
        self.chevron_host
            .width(metrics.chevron_box_size, Unit::Pixel)
            .height(metrics.chevron_box_size, Unit::Pixel)
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center);
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DefaultDropdownFieldTemplate;

impl DropdownFieldTemplate for DefaultDropdownFieldTemplate {
    fn create(&self, sizing: Option<DropdownSizing>) -> Rc<dyn DropdownFieldPresenter> {
        create_default_dropdown_field_presenter(sizing)
    }
}

pub const DEFAULT_DROPDOWN_FIELD_TEMPLATE: DefaultDropdownFieldTemplate =
    DefaultDropdownFieldTemplate;

pub fn create_default_dropdown_field_presenter(
    sizing: Option<DropdownSizing>,
) -> Rc<dyn DropdownFieldPresenter> {
    Rc::new(DefaultDropdownFieldPresenter::new(resolve_field_metrics(
        sizing,
    )))
}
