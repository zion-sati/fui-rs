use crate::node::FlexBox;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PressableIndicatorMetrics {
    pub width: f32,
    pub height: f32,
}

impl PressableIndicatorMetrics {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PressableIndicatorVisualState {
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
    pub enabled: bool,
}

pub trait PressableIndicatorPresenter {
    fn root(&self) -> FlexBox;
    fn metrics(&self) -> PressableIndicatorMetrics;
}
