use crate::controls::{DropdownColors, DropdownSizing};
use crate::ffi::{AlignItems, TextVerticalAlign};
use crate::node::{flex_box, text, FlexBox, TextNode};
use crate::theme::Theme;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DropdownOptionRowMetrics {
    pub height: f32,
    pub padding_left: f32,
    pub padding_right: f32,
    pub font_size: f32,
}

impl DropdownOptionRowMetrics {
    pub const fn new(height: f32, padding_left: f32, padding_right: f32, font_size: f32) -> Self {
        Self {
            height,
            padding_left,
            padding_right,
            font_size,
        }
    }
}

pub const DEFAULT_DROPDOWN_OPTION_ROW_METRICS: DropdownOptionRowMetrics =
    DropdownOptionRowMetrics::new(34.0, 10.0, 10.0, 16.0);

fn resolve_option_row_metrics(sizing: Option<DropdownSizing>) -> DropdownOptionRowMetrics {
    let Some(sizing) = sizing else {
        return DEFAULT_DROPDOWN_OPTION_ROW_METRICS;
    };
    if !sizing.has_option_height() && !sizing.has_option_font_size() {
        return DEFAULT_DROPDOWN_OPTION_ROW_METRICS;
    }
    let font_size = if sizing.has_option_font_size() {
        sizing.option_font_size_px()
    } else {
        DEFAULT_DROPDOWN_OPTION_ROW_METRICS.font_size
    };
    let height = if sizing.has_option_height() {
        sizing.option_height_px()
    } else {
        DEFAULT_DROPDOWN_OPTION_ROW_METRICS.height
    };
    DropdownOptionRowMetrics::new(
        height,
        DEFAULT_DROPDOWN_OPTION_ROW_METRICS.padding_left,
        DEFAULT_DROPDOWN_OPTION_ROW_METRICS.padding_right,
        font_size,
    )
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DropdownOptionRowVisualState {
    pub highlighted: bool,
    pub selected: bool,
    pub enabled: bool,
}

impl DropdownOptionRowVisualState {
    pub const fn new(highlighted: bool, selected: bool, enabled: bool) -> Self {
        Self {
            highlighted,
            selected,
            enabled,
        }
    }
}

pub trait DropdownOptionRowPresenter {
    fn root(&self) -> FlexBox;
    fn label_node(&self) -> TextNode;
    fn metrics(&self) -> DropdownOptionRowMetrics;
    fn apply(
        &self,
        theme: Theme,
        state: DropdownOptionRowVisualState,
        colors: Option<DropdownColors>,
    );
}

pub trait DropdownOptionRowTemplate {
    fn create(&self, sizing: Option<DropdownSizing>) -> Rc<dyn DropdownOptionRowPresenter>;
}

#[derive(Clone)]
pub struct DefaultDropdownOptionRowPresenter {
    root: FlexBox,
    label_node: TextNode,
    metrics: DropdownOptionRowMetrics,
}

impl DefaultDropdownOptionRowPresenter {
    pub fn new(metrics: DropdownOptionRowMetrics) -> Self {
        let theme = crate::theme::current_theme();
        let label_node = text("");
        label_node
            .selectable(false, theme.colors.selection)
            .fill_size()
            .text_limits(0, 1)
            .wrapping(false);
        label_node
            .text_overflow_fade(true, false)
            .text_vertical_align(TextVerticalAlign::Center);
        let root = flex_box();
        root.fill_size()
            .align_items(AlignItems::Center)
            .child(&label_node);
        Self {
            root,
            label_node: label_node.clone(),
            metrics,
        }
    }
}

impl DropdownOptionRowPresenter for DefaultDropdownOptionRowPresenter {
    fn root(&self) -> FlexBox {
        self.root.clone()
    }

    fn label_node(&self) -> TextNode {
        self.label_node.clone()
    }

    fn metrics(&self) -> DropdownOptionRowMetrics {
        self.metrics
    }

    fn apply(
        &self,
        theme: Theme,
        state: DropdownOptionRowVisualState,
        colors: Option<DropdownColors>,
    ) {
        let metrics = self.metrics;
        self.root
            .padding(metrics.padding_left, 0.0, metrics.padding_right, 0.0)
            .corner_radius(theme.spacing.xs)
            .bg_color(if state.highlighted {
                theme.context_menu.item.hover_background
            } else {
                0x00000000
            });
        let label_color = if !state.enabled {
            theme.colors.text_muted
        } else if state.selected {
            colors
                .filter(|colors| colors.has_accent())
                .map(|colors| colors.accent_color())
                .unwrap_or(theme.colors.accent)
        } else {
            colors
                .filter(|colors| colors.has_text_primary())
                .map(|colors| colors.text_primary_color())
                .unwrap_or(theme.colors.text_primary)
        };
        self.label_node
            .font_family(theme.fonts.body_family.clone())
            .font_size(metrics.font_size)
            .text_color(label_color);
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DefaultDropdownOptionRowTemplate;

impl DropdownOptionRowTemplate for DefaultDropdownOptionRowTemplate {
    fn create(&self, sizing: Option<DropdownSizing>) -> Rc<dyn DropdownOptionRowPresenter> {
        create_default_dropdown_option_row_presenter(sizing)
    }
}

pub const DEFAULT_DROPDOWN_OPTION_ROW_TEMPLATE: DefaultDropdownOptionRowTemplate =
    DefaultDropdownOptionRowTemplate;

pub fn create_default_dropdown_option_row_presenter(
    sizing: Option<DropdownSizing>,
) -> Rc<dyn DropdownOptionRowPresenter> {
    Rc::new(DefaultDropdownOptionRowPresenter::new(
        resolve_option_row_metrics(sizing),
    ))
}
