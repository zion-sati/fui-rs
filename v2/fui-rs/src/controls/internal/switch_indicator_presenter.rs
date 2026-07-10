use super::pressable_indicator_presenter::{
    PressableIndicatorMetrics, PressableIndicatorPresenter, PressableIndicatorVisualState,
};
use crate::controls::{LabeledControlColors, LabeledControlSizing};
use crate::ffi::{AlignItems, PositionType, Unit};
use crate::node::{flex_box, FlexBox};
use crate::theme::Theme;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SwitchIndicatorVisualState {
    pub checked: bool,
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
    pub enabled: bool,
}

impl SwitchIndicatorVisualState {
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

pub trait SwitchIndicatorPresenter: PressableIndicatorPresenter {
    fn apply(
        &self,
        theme: Theme,
        state: SwitchIndicatorVisualState,
        colors: Option<LabeledControlColors>,
    );
}

pub trait SwitchIndicatorTemplate {
    fn create(&self, sizing: Option<LabeledControlSizing>) -> Rc<dyn SwitchIndicatorPresenter>;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwitchIndicatorMetrics {
    pub track_width: f32,
    pub track_height: f32,
    pub thumb_size: f32,
    pub thumb_x: f32,
    pub thumb_checked_x: f32,
    pub thumb_y: f32,
}

impl SwitchIndicatorMetrics {
    pub const fn new(
        track_width: f32,
        track_height: f32,
        thumb_size: f32,
        thumb_x: f32,
        thumb_checked_x: f32,
        thumb_y: f32,
    ) -> Self {
        Self {
            track_width,
            track_height,
            thumb_size,
            thumb_x,
            thumb_checked_x,
            thumb_y,
        }
    }
}

const DEFAULT_SWITCH_METRICS: SwitchIndicatorMetrics =
    SwitchIndicatorMetrics::new(44.0, 26.0, 20.0, 3.0, 21.0, 2.0);

pub fn resolve_switch_metrics(sizing: Option<LabeledControlSizing>) -> SwitchIndicatorMetrics {
    let Some(sizing) = sizing else {
        return DEFAULT_SWITCH_METRICS;
    };
    if !sizing.has_indicator_size() {
        return DEFAULT_SWITCH_METRICS;
    }
    let scale = sizing.indicator_size_px() / DEFAULT_SWITCH_METRICS.track_height;
    let track_width = DEFAULT_SWITCH_METRICS.track_width * scale;
    let track_height = DEFAULT_SWITCH_METRICS.track_height * scale;
    let thumb_size = DEFAULT_SWITCH_METRICS.thumb_size * scale;
    let thumb_x = DEFAULT_SWITCH_METRICS.thumb_x * scale;
    let thumb_y = DEFAULT_SWITCH_METRICS.thumb_y * scale;
    SwitchIndicatorMetrics::new(
        track_width,
        track_height,
        thumb_size,
        thumb_x,
        track_width - thumb_size - thumb_x,
        thumb_y,
    )
}

#[derive(Clone)]
pub struct DefaultSwitchIndicatorPresenter {
    root: FlexBox,
    metrics: SwitchIndicatorMetrics,
    thumb_node: FlexBox,
}

impl DefaultSwitchIndicatorPresenter {
    pub fn new(metrics: SwitchIndicatorMetrics) -> Self {
        let root = flex_box();
        root.width(metrics.track_width, Unit::Pixel)
            .height(metrics.track_height, Unit::Pixel)
            .clip_to_bounds(true);
        let thumb_node = flex_box();
        thumb_node
            .position_type(PositionType::Absolute)
            .position(metrics.thumb_x, metrics.thumb_y)
            .width(metrics.thumb_size, Unit::Pixel)
            .height(metrics.thumb_size, Unit::Pixel);
        root.align_items(AlignItems::Center).child(&thumb_node);
        Self {
            root,
            metrics,
            thumb_node,
        }
    }
}

impl PressableIndicatorPresenter for DefaultSwitchIndicatorPresenter {
    fn root(&self) -> FlexBox {
        self.root.clone()
    }

    fn metrics(&self) -> PressableIndicatorMetrics {
        PressableIndicatorMetrics::new(self.metrics.track_width, self.metrics.track_height)
    }
}

impl SwitchIndicatorPresenter for DefaultSwitchIndicatorPresenter {
    fn apply(
        &self,
        theme: Theme,
        state: SwitchIndicatorVisualState,
        colors: Option<LabeledControlColors>,
    ) {
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
        let track_color = if state.checked {
            accent
        } else {
            colors
                .filter(|colors| colors.has_background())
                .map(|colors| colors.background_color())
                .unwrap_or_else(|| {
                    if state.hovered {
                        theme.colors.background
                    } else {
                        theme.colors.surface
                    }
                })
        };
        let border_color = colors
            .filter(|colors| colors.has_border())
            .map(|colors| colors.border_color())
            .unwrap_or_else(|| {
                if state.checked {
                    track_color
                } else {
                    theme.colors.border
                }
            });
        let metrics = self.metrics;
        self.root
            .corner_radius(metrics.track_height * 0.5)
            .border(1.0, border_color)
            .bg_color(track_color);
        self.thumb_node
            .position(
                if state.checked {
                    metrics.thumb_checked_x
                } else {
                    metrics.thumb_x
                },
                metrics.thumb_y,
            )
            .width(metrics.thumb_size, Unit::Pixel)
            .height(metrics.thumb_size, Unit::Pixel)
            .corner_radius(metrics.thumb_size * 0.5)
            .bg_color(
                colors
                    .filter(|colors| colors.has_background())
                    .map(|colors| colors.background_color())
                    .unwrap_or(theme.colors.surface),
            )
            .border(1.0, border_color);
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DefaultSwitchIndicatorTemplate;

impl SwitchIndicatorTemplate for DefaultSwitchIndicatorTemplate {
    fn create(&self, sizing: Option<LabeledControlSizing>) -> Rc<dyn SwitchIndicatorPresenter> {
        create_default_switch_indicator_presenter(sizing)
    }
}

pub const DEFAULT_SWITCH_INDICATOR_TEMPLATE: DefaultSwitchIndicatorTemplate =
    DefaultSwitchIndicatorTemplate;

pub fn create_default_switch_indicator_presenter(
    sizing: Option<LabeledControlSizing>,
) -> Rc<dyn SwitchIndicatorPresenter> {
    Rc::new(DefaultSwitchIndicatorPresenter::new(
        resolve_switch_metrics(sizing),
    ))
}
