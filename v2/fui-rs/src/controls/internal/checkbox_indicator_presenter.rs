use super::pressable_indicator_presenter::{
    PressableIndicatorMetrics, PressableIndicatorPresenter, PressableIndicatorVisualState,
};
use crate::controls::{LabeledControlColors, LabeledControlSizing};
use crate::ffi::{AlignItems, JustifyContent, SemanticCheckedState, Unit};
use crate::node::{flex_box, svg, FlexBox, SvgNode};
use crate::theme::Theme;
use std::rc::Rc;

const CHECKBOX_CHECK_SVG_URL: &str = "data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='20' height='20' viewBox='0 0 14 14'><path d='M2.25 7.15 5.35 10.25 11.75 3.85' fill='none' stroke='%23000000' stroke-width='2.4' stroke-linecap='round' stroke-linejoin='round'/></svg>";

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CheckboxIndicatorMetrics {
    pub indicator_size: f32,
    pub corner_radius: f32,
    pub check_mark_size: f32,
}

impl CheckboxIndicatorMetrics {
    pub const fn new(indicator_size: f32, corner_radius: f32, check_mark_size: f32) -> Self {
        Self {
            indicator_size,
            corner_radius,
            check_mark_size,
        }
    }
}

const DEFAULT_CHECKBOX_METRICS: CheckboxIndicatorMetrics =
    CheckboxIndicatorMetrics::new(20.0, 4.0, 16.0);

pub fn resolve_checkbox_metrics(sizing: Option<LabeledControlSizing>) -> CheckboxIndicatorMetrics {
    let Some(sizing) = sizing else {
        return DEFAULT_CHECKBOX_METRICS;
    };
    if !sizing.has_indicator_size() {
        return DEFAULT_CHECKBOX_METRICS;
    }
    let indicator_size = sizing.indicator_size_px();
    CheckboxIndicatorMetrics::new(indicator_size, indicator_size * 0.2, indicator_size * 0.8)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CheckboxIndicatorVisualState {
    pub checked_state: SemanticCheckedState,
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
    pub enabled: bool,
}

impl CheckboxIndicatorVisualState {
    pub fn new(
        checked_state: SemanticCheckedState,
        hovered: bool,
        pressed: bool,
        focused: bool,
        enabled: bool,
    ) -> Self {
        Self {
            checked_state,
            hovered,
            pressed,
            focused,
            enabled,
        }
    }

    pub fn pressable_state(&self) -> PressableIndicatorVisualState {
        PressableIndicatorVisualState {
            hovered: self.hovered,
            pressed: self.pressed,
            focused: self.focused,
            enabled: self.enabled,
        }
    }
}

pub trait CheckboxIndicatorPresenter: PressableIndicatorPresenter {
    fn apply(
        &self,
        theme: Theme,
        state: CheckboxIndicatorVisualState,
        colors: Option<LabeledControlColors>,
    );
}

pub trait CheckboxIndicatorTemplate {
    fn create(&self, sizing: Option<LabeledControlSizing>) -> Rc<dyn CheckboxIndicatorPresenter>;
}

#[derive(Clone)]
pub struct DefaultCheckboxIndicatorPresenter {
    root: FlexBox,
    metrics: CheckboxIndicatorMetrics,
    mark_node: SvgNode,
}

impl DefaultCheckboxIndicatorPresenter {
    pub fn new(metrics: CheckboxIndicatorMetrics) -> Self {
        let root = flex_box();
        root.width(metrics.indicator_size, Unit::Pixel)
            .height(metrics.indicator_size, Unit::Pixel)
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center);
        let mark_host = flex_box();
        mark_host
            .fill_size()
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center);
        let mark_node = svg(0);
        mark_node
            .width(metrics.check_mark_size, Unit::Pixel)
            .height(metrics.check_mark_size, Unit::Pixel);
        mark_host.child(&mark_node);
        root.child(&mark_host);
        Self {
            root,
            metrics,
            mark_node,
        }
    }
}

impl PressableIndicatorPresenter for DefaultCheckboxIndicatorPresenter {
    fn root(&self) -> FlexBox {
        self.root.clone()
    }

    fn metrics(&self) -> PressableIndicatorMetrics {
        PressableIndicatorMetrics::new(self.metrics.indicator_size, self.metrics.indicator_size)
    }
}

impl CheckboxIndicatorPresenter for DefaultCheckboxIndicatorPresenter {
    fn apply(
        &self,
        theme: Theme,
        state: CheckboxIndicatorVisualState,
        colors: Option<LabeledControlColors>,
    ) {
        let metrics = self.metrics;
        let accent = colors
            .filter(|colors| colors.has_accent())
            .map(|colors| colors.accent_color())
            .unwrap_or_else(|| {
                if state.pressed {
                    theme.colors.accent_pressed
                } else if state.hovered {
                    theme.colors.accent_hovered
                } else {
                    theme.colors.accent
                }
            });
        let mut background = colors
            .filter(|colors| colors.has_background())
            .map(|colors| colors.background_color())
            .unwrap_or(theme.colors.surface);
        let mut border_color = colors
            .filter(|colors| colors.has_border())
            .map(|colors| colors.border_color())
            .unwrap_or(theme.colors.border);
        let mut mark_visible = false;
        let mut mark_color = theme.colors.text_primary;
        if state.checked_state == SemanticCheckedState::True
            || state.checked_state == SemanticCheckedState::Mixed
        {
            background = accent;
            border_color = background;
            mark_visible = state.checked_state == SemanticCheckedState::True;
            mark_color = theme.colors.text_on_accent;
        } else if state.hovered && colors.is_none_or(|colors| !colors.has_background()) {
            background = theme.colors.background;
        }
        self.root
            .corner_radius(metrics.corner_radius)
            .border(1.0, border_color)
            .bg_color(background);
        if mark_visible {
            self.mark_node.source(CHECKBOX_CHECK_SVG_URL);
        } else {
            self.mark_node.clear_source();
        }
        self.mark_node
            .width(metrics.check_mark_size, Unit::Pixel)
            .height(metrics.check_mark_size, Unit::Pixel)
            .opacity(if mark_visible { 1.0 } else { 0.0 })
            .tint(mark_color);
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DefaultCheckboxIndicatorTemplate;

impl CheckboxIndicatorTemplate for DefaultCheckboxIndicatorTemplate {
    fn create(&self, sizing: Option<LabeledControlSizing>) -> Rc<dyn CheckboxIndicatorPresenter> {
        create_default_checkbox_indicator_presenter(sizing)
    }
}

pub const DEFAULT_CHECKBOX_INDICATOR_TEMPLATE: DefaultCheckboxIndicatorTemplate =
    DefaultCheckboxIndicatorTemplate;

pub fn create_default_checkbox_indicator_presenter(
    sizing: Option<LabeledControlSizing>,
) -> Rc<dyn CheckboxIndicatorPresenter> {
    Rc::new(DefaultCheckboxIndicatorPresenter::new(
        resolve_checkbox_metrics(sizing),
    ))
}
