use crate::animation::AnimationTiming;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct NodeTransitions {
    opacity_timing: Option<AnimationTiming>,
    background_color_timing: Option<AnimationTiming>,
    scroll_offset_timing: Option<AnimationTiming>,
}

impl NodeTransitions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn opacity(mut self, timing: AnimationTiming) -> Self {
        self.opacity_timing = Some(timing);
        self
    }

    pub fn bg_color(mut self, timing: AnimationTiming) -> Self {
        self.background_color_timing = Some(timing);
        self
    }

    pub fn scroll_offset(mut self, timing: AnimationTiming) -> Self {
        self.scroll_offset_timing = Some(timing);
        self
    }

    pub(crate) fn opacity_timing(&self) -> Option<AnimationTiming> {
        self.opacity_timing
    }

    pub(crate) fn background_color_timing(&self) -> Option<AnimationTiming> {
        self.background_color_timing
    }

    pub(crate) fn scroll_offset_timing(&self) -> Option<AnimationTiming> {
        self.scroll_offset_timing
    }
}
