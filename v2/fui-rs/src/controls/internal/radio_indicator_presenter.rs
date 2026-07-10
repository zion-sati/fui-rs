use super::pressable_indicator_presenter::{
    PressableIndicatorMetrics, PressableIndicatorPresenter, PressableIndicatorVisualState,
};
use crate::controls::{LabeledControlColors, LabeledControlSizing};
use crate::ffi::{AlignItems, JustifyContent, Unit};
use crate::node::{flex_box, FlexBox};
use crate::theme::Theme;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RadioIndicatorMetrics {
    pub indicator_size: f32,
    pub dot_size: f32,
    pub border_width: f32,
}

impl RadioIndicatorMetrics {
    pub const fn new(indicator_size: f32, dot_size: f32, border_width: f32) -> Self {
        Self {
            indicator_size,
            dot_size,
            border_width,
        }
    }
}

const DEFAULT_RADIO_METRICS: RadioIndicatorMetrics = RadioIndicatorMetrics::new(20.0, 8.0, 1.0);

fn centered_inset(outer_size: f32, inner_size: f32) -> f32 {
    if outer_size > inner_size {
        (outer_size - inner_size) * 0.5
    } else {
        0.0
    }
}

fn dot_inset(metrics: RadioIndicatorMetrics) -> f32 {
    let inset = centered_inset(metrics.indicator_size, metrics.dot_size);
    if inset > metrics.border_width {
        inset - metrics.border_width
    } else {
        0.0
    }
}

pub fn resolve_radio_metrics(sizing: Option<LabeledControlSizing>) -> RadioIndicatorMetrics {
    let Some(sizing) = sizing else {
        return DEFAULT_RADIO_METRICS;
    };
    if !sizing.has_indicator_size() {
        return DEFAULT_RADIO_METRICS;
    }
    let indicator_size = sizing.indicator_size_px();
    RadioIndicatorMetrics::new(
        indicator_size,
        indicator_size * 0.4,
        if indicator_size >= 24.0 { 2.0 } else { 1.0 },
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RadioIndicatorVisualState {
    pub checked: bool,
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
    pub enabled: bool,
}

impl RadioIndicatorVisualState {
    pub fn new(checked: bool, hovered: bool, pressed: bool, focused: bool, enabled: bool) -> Self {
        Self {
            checked,
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

pub trait RadioIndicatorPresenter: PressableIndicatorPresenter {
    fn apply(
        &self,
        theme: Theme,
        state: RadioIndicatorVisualState,
        colors: Option<LabeledControlColors>,
    );
}

pub trait RadioIndicatorTemplate {
    fn create(&self, sizing: Option<LabeledControlSizing>) -> Rc<dyn RadioIndicatorPresenter>;
}

#[derive(Clone)]
pub struct DefaultRadioIndicatorPresenter {
    root: FlexBox,
    metrics: RadioIndicatorMetrics,
    dot_node: FlexBox,
}

impl DefaultRadioIndicatorPresenter {
    pub fn new(metrics: RadioIndicatorMetrics) -> Self {
        let root = flex_box();
        root.width(metrics.indicator_size, Unit::Pixel)
            .height(metrics.indicator_size, Unit::Pixel)
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center);
        let dot_node = flex_box();
        dot_node
            .position_type(crate::ffi::PositionType::Absolute)
            .position(dot_inset(metrics), dot_inset(metrics))
            .width(metrics.dot_size, Unit::Pixel)
            .height(metrics.dot_size, Unit::Pixel);
        root.child(&dot_node);
        Self {
            root,
            metrics,
            dot_node,
        }
    }
}

impl PressableIndicatorPresenter for DefaultRadioIndicatorPresenter {
    fn root(&self) -> FlexBox {
        self.root.clone()
    }

    fn metrics(&self) -> PressableIndicatorMetrics {
        PressableIndicatorMetrics::new(self.metrics.indicator_size, self.metrics.indicator_size)
    }
}

impl RadioIndicatorPresenter for DefaultRadioIndicatorPresenter {
    fn apply(
        &self,
        theme: Theme,
        state: RadioIndicatorVisualState,
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
        let outer_color = if state.checked {
            accent
        } else {
            colors
                .filter(|colors| colors.has_border())
                .map(|colors| colors.border_color())
                .unwrap_or(theme.colors.border)
        };
        self.root
            .corner_radius(metrics.indicator_size * 0.5)
            .border(metrics.border_width, outer_color)
            .bg_color(
                colors
                    .filter(|colors| colors.has_background())
                    .map(|colors| colors.background_color())
                    .unwrap_or(theme.colors.surface),
            );
        self.dot_node
            .corner_radius(metrics.dot_size * 0.5)
            .position(dot_inset(metrics), dot_inset(metrics))
            .width(metrics.dot_size, Unit::Pixel)
            .height(metrics.dot_size, Unit::Pixel)
            .bg_color(outer_color)
            .opacity(if state.checked { 1.0 } else { 0.0 });
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DefaultRadioIndicatorTemplate;

impl RadioIndicatorTemplate for DefaultRadioIndicatorTemplate {
    fn create(&self, sizing: Option<LabeledControlSizing>) -> Rc<dyn RadioIndicatorPresenter> {
        create_default_radio_indicator_presenter(sizing)
    }
}

pub const DEFAULT_RADIO_INDICATOR_TEMPLATE: DefaultRadioIndicatorTemplate =
    DefaultRadioIndicatorTemplate;

pub fn create_default_radio_indicator_presenter(
    sizing: Option<LabeledControlSizing>,
) -> Rc<dyn RadioIndicatorPresenter> {
    Rc::new(DefaultRadioIndicatorPresenter::new(resolve_radio_metrics(
        sizing,
    )))
}
