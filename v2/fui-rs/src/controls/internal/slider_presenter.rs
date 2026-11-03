use crate::controls::{SliderColors, SliderSizing};
use crate::ffi::{AlignItems, FlexDirection, JustifyContent, Orientation, PositionType, Unit};
use crate::node::{flex_box, FlexBox};
use crate::theme::Theme;
use std::rc::Rc;

fn clamp(value: f32, min: f32, max: f32) -> f32 {
    if value < min {
        return min;
    }
    if value > max {
        return max;
    }
    value
}

fn resolve_thumb_size(metrics: SliderPresenterMetrics) -> f32 {
    if metrics.thumb_size > 1.0 {
        metrics.thumb_size
    } else {
        1.0
    }
}

fn resolve_track_thickness(metrics: SliderPresenterMetrics) -> f32 {
    clamp(metrics.track_thickness, 1.0, resolve_thumb_size(metrics))
}

fn resolve_cross_axis_extra(metrics: SliderPresenterMetrics) -> f32 {
    if metrics.cross_axis_extra > 0.0 {
        metrics.cross_axis_extra
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SliderPresenterMetrics {
    pub thumb_size: f32,
    pub track_thickness: f32,
    pub cross_axis_extra: f32,
}

impl SliderPresenterMetrics {
    pub const fn new(thumb_size: f32, track_thickness: f32) -> Self {
        Self {
            thumb_size,
            track_thickness,
            cross_axis_extra: 2.0,
        }
    }

    pub const fn with_cross_axis_extra(mut self, cross_axis_extra: f32) -> Self {
        self.cross_axis_extra = cross_axis_extra;
        self
    }

    pub const fn with_values(thumb_size: f32, track_thickness: f32, cross_axis_extra: f32) -> Self {
        Self {
            thumb_size,
            track_thickness,
            cross_axis_extra,
        }
    }
}

const DEFAULT_SLIDER_METRICS: SliderPresenterMetrics = SliderPresenterMetrics::new(18.0, 6.0);

fn resolve_slider_metrics(sizing: Option<SliderSizing>) -> SliderPresenterMetrics {
    let Some(sizing) = sizing else {
        return DEFAULT_SLIDER_METRICS;
    };
    let thumb_size = if sizing.has_thumb_size() {
        sizing.thumb_size_px()
    } else {
        DEFAULT_SLIDER_METRICS.thumb_size
    };
    let track_thickness = if sizing.has_track_thickness() {
        sizing.track_thickness_px()
    } else {
        DEFAULT_SLIDER_METRICS.track_thickness
    };
    if (thumb_size - DEFAULT_SLIDER_METRICS.thumb_size).abs() <= f32::EPSILON
        && (track_thickness - DEFAULT_SLIDER_METRICS.track_thickness).abs() <= f32::EPSILON
    {
        return DEFAULT_SLIDER_METRICS;
    }
    SliderPresenterMetrics::new(thumb_size, track_thickness)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SliderVisualState {
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub normalized_value: f32,
    pub orientation: Orientation,
    pub hovered: bool,
    pub dragging: bool,
    pub focused: bool,
    pub enabled: bool,
}

impl SliderVisualState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        value: f32,
        min: f32,
        max: f32,
        normalized_value: f32,
        orientation: Orientation,
        hovered: bool,
        dragging: bool,
        focused: bool,
        enabled: bool,
    ) -> Self {
        Self {
            value,
            min,
            max,
            normalized_value,
            orientation,
            hovered,
            dragging,
            focused,
            enabled,
        }
    }
}

/// A slider's visuals. The slider sizes the presenter's root to its track area, a fixed length or the length it
/// fills, so a presenter lays out relative to its root rather than in pixels along the track.
pub trait SliderPresenter {
    fn root(&self) -> FlexBox;
    fn metrics(&self) -> SliderPresenterMetrics;
    fn layout(&self, state: SliderVisualState);
    fn apply(&self, theme: Theme, state: SliderVisualState, colors: Option<SliderColors>);
}

pub trait SliderTemplate {
    fn create(&self, sizing: Option<SliderSizing>) -> Rc<dyn SliderPresenter>;
}

#[derive(Clone)]
pub struct DefaultSliderPresenter {
    root: FlexBox,
    metrics: SliderPresenterMetrics,
    rail: FlexBox,
    track_node: FlexBox,
    fill_box: FlexBox,
    fill_node: FlexBox,
    spacer: FlexBox,
    thumb_node: FlexBox,
}

impl DefaultSliderPresenter {
    pub fn new(metrics: SliderPresenterMetrics) -> Self {
        // The rail runs between the thumb's centre at each end. The track and fill lie on it; a spacer as long as the
        // value's share of the rail puts the thumb, shifted back half its size onto the spacer's end, where the value
        // is.
        let root = flex_box();
        let rail = flex_box();
        let track_node = flex_box();
        track_node.position_type(PositionType::Absolute);
        let fill_box = flex_box();
        fill_box.position_type(PositionType::Absolute);
        let fill_node = flex_box();
        fill_box.child(&fill_node);
        let spacer = flex_box();
        let thumb_node = flex_box();
        thumb_node
            .width(metrics.thumb_size, Unit::Pixel)
            .height(metrics.thumb_size, Unit::Pixel);
        rail.align_items(AlignItems::Center)
            .child(&track_node)
            .child(&fill_box)
            .child(&spacer)
            .child(&thumb_node);
        root.child(&rail);
        Self {
            root,
            metrics,
            rail,
            track_node,
            fill_box,
            fill_node,
            spacer,
            thumb_node,
        }
    }
}

impl SliderPresenter for DefaultSliderPresenter {
    fn root(&self) -> FlexBox {
        self.root.clone()
    }

    fn metrics(&self) -> SliderPresenterMetrics {
        self.metrics
    }

    fn layout(&self, state: SliderVisualState) {
        let metrics = self.metrics;
        let thumb_size = resolve_thumb_size(metrics);
        let track_thickness = resolve_track_thickness(metrics);
        let cross_axis_inset = resolve_cross_axis_extra(metrics) * 0.5;
        let fraction = clamp(state.normalized_value, 0.0, 1.0);
        let half_thumb = thumb_size * 0.5;
        let track_offset = (thumb_size - track_thickness) * 0.5;
        if state.orientation == Orientation::Vertical {
            self.root.flex_direction(FlexDirection::Row);
            self.rail
                .flex_direction(FlexDirection::Column)
                .width(thumb_size, Unit::Pixel)
                .fill_height()
                .margin(cross_axis_inset, half_thumb, cross_axis_inset, half_thumb);
            self.track_node
                .width(track_thickness, Unit::Pixel)
                .height(100.0, Unit::Percent)
                .position(track_offset, 0.0);
            // The fill grows up from the rail's bottom.
            self.fill_box
                .flex_direction(FlexDirection::Column)
                .justify_content(JustifyContent::End)
                .width(track_thickness, Unit::Pixel)
                .height(100.0, Unit::Percent)
                .position(track_offset, 0.0);
            self.fill_node
                .width(100.0, Unit::Percent)
                .height(fraction * 100.0, Unit::Percent);
            self.spacer
                .width(1.0, Unit::Pixel)
                .height((1.0 - fraction) * 100.0, Unit::Percent);
            self.thumb_node.position(0.0, -half_thumb);
            return;
        }

        self.root.flex_direction(FlexDirection::Column);
        self.rail
            .flex_direction(FlexDirection::Row)
            .height(thumb_size, Unit::Pixel)
            .fill_width()
            .margin(half_thumb, cross_axis_inset, half_thumb, cross_axis_inset);
        self.track_node
            .width(100.0, Unit::Percent)
            .height(track_thickness, Unit::Pixel)
            .position(0.0, track_offset);
        self.fill_box
            .flex_direction(FlexDirection::Row)
            .justify_content(JustifyContent::Start)
            .width(100.0, Unit::Percent)
            .height(track_thickness, Unit::Pixel)
            .position(0.0, track_offset);
        self.fill_node
            .width(fraction * 100.0, Unit::Percent)
            .height(100.0, Unit::Percent);
        self.spacer
            .width(fraction * 100.0, Unit::Percent)
            .height(1.0, Unit::Pixel);
        self.thumb_node.position(-half_thumb, 0.0);
    }

    fn apply(&self, theme: Theme, state: SliderVisualState, colors: Option<SliderColors>) {
        let accent = if state.dragging {
            theme.colors.accent_pressed
        } else if state.hovered {
            theme.colors.accent_hovered
        } else {
            theme.colors.accent
        };
        let metrics = self.metrics;
        let thumb_size = resolve_thumb_size(metrics);
        let track_thickness = resolve_track_thickness(metrics);
        let track_radius = track_thickness * 0.5;
        self.track_node.corner_radius(track_radius);
        let track_color = colors
            .filter(|colors| colors.has_track())
            .map(|colors| colors.track_color())
            .unwrap_or(theme.colors.scrollbar_track);
        self.track_node.bg_color(track_color);
        self.fill_node.corner_radius(track_radius);
        let fill_color = colors
            .filter(|colors| colors.has_fill())
            .map(|colors| colors.fill_color())
            .unwrap_or(accent);
        self.fill_node.bg_color(fill_color);
        self.thumb_node
            .width(thumb_size, Unit::Pixel)
            .height(thumb_size, Unit::Pixel)
            .corner_radius(thumb_size * 0.5);
        let thumb_color = colors
            .filter(|colors| colors.has_thumb())
            .map(|colors| colors.thumb_color())
            .unwrap_or(accent);
        self.thumb_node
            .bg_color(thumb_color)
            .border(1.0, theme.colors.surface);
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DefaultSliderTemplate;

impl SliderTemplate for DefaultSliderTemplate {
    fn create(&self, sizing: Option<SliderSizing>) -> Rc<dyn SliderPresenter> {
        create_default_slider_presenter(sizing)
    }
}

pub const DEFAULT_SLIDER_TEMPLATE: DefaultSliderTemplate = DefaultSliderTemplate;

pub fn create_default_slider_presenter(sizing: Option<SliderSizing>) -> Rc<dyn SliderPresenter> {
    Rc::new(DefaultSliderPresenter::new(resolve_slider_metrics(sizing)))
}
