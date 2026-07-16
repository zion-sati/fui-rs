use crate::ffi::{AlignItems, CursorStyle, FlexDirection, JustifyContent};
use crate::node::Border;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EdgeInsets {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl EdgeInsets {
    pub const fn new(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    pub const fn all(value: f32) -> Self {
        Self::new(value, value, value, value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Corners {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}

impl Corners {
    pub const fn new(top_left: f32, top_right: f32, bottom_right: f32, bottom_left: f32) -> Self {
        Self {
            top_left,
            top_right,
            bottom_right,
            bottom_left,
        }
    }

    pub const fn all(value: f32) -> Self {
        Self::new(value, value, value, value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shadow {
    pub color: u32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur_sigma: f32,
    pub spread: f32,
}

impl Shadow {
    pub fn new(color: u32, offset_x: f32, offset_y: f32, blur_sigma: f32, spread: f32) -> Self {
        Self {
            color,
            offset_x,
            offset_y,
            blur_sigma: blur_sigma.max(0.0),
            spread,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PresenterHostStyle {
    pub flex_direction: Option<FlexDirection>,
    pub justify_content: Option<JustifyContent>,
    pub align_items: Option<AlignItems>,
    pub background: Option<u32>,
    pub padding: Option<EdgeInsets>,
    pub corners: Option<Corners>,
    pub border: Option<Border>,
    pub shadow: Option<Shadow>,
    pub cursor: Option<CursorStyle>,
    pub opacity: Option<f32>,
}

impl PresenterHostStyle {
    pub const fn new() -> Self {
        Self {
            flex_direction: None,
            justify_content: None,
            align_items: None,
            background: None,
            padding: None,
            corners: None,
            border: None,
            shadow: None,
            cursor: None,
            opacity: None,
        }
    }

    pub fn flex_direction(mut self, value: FlexDirection) -> Self {
        self.flex_direction = Some(value);
        self
    }

    pub fn justify_content(mut self, value: JustifyContent) -> Self {
        self.justify_content = Some(value);
        self
    }

    pub fn align_items(mut self, value: AlignItems) -> Self {
        self.align_items = Some(value);
        self
    }

    pub fn background(mut self, value: u32) -> Self {
        self.background = Some(value);
        self
    }

    pub fn padding(mut self, value: EdgeInsets) -> Self {
        self.padding = Some(value);
        self
    }

    pub fn corners(mut self, value: Corners) -> Self {
        self.corners = Some(value);
        self
    }

    pub fn border(mut self, value: Border) -> Self {
        self.border = Some(value);
        self
    }

    pub fn shadow(mut self, value: Shadow) -> Self {
        self.shadow = Some(value);
        self
    }

    pub fn cursor(mut self, value: CursorStyle) -> Self {
        self.cursor = Some(value);
        self
    }

    pub fn opacity(mut self, value: f32) -> Self {
        self.opacity = Some(value.clamp(0.0, 1.0));
        self
    }

    pub(crate) fn overlay(self, fallback: Self) -> Self {
        Self {
            flex_direction: self.flex_direction.or(fallback.flex_direction),
            justify_content: self.justify_content.or(fallback.justify_content),
            align_items: self.align_items.or(fallback.align_items),
            background: self.background.or(fallback.background),
            padding: self.padding.or(fallback.padding),
            corners: self.corners.or(fallback.corners),
            border: self.border.or(fallback.border),
            shadow: self.shadow.or(fallback.shadow),
            cursor: self.cursor.or(fallback.cursor),
            opacity: self.opacity.or(fallback.opacity),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct HostStyleLayers {
    pub local: PresenterHostStyle,
    pub presenter: PresenterHostStyle,
}

impl HostStyleLayers {
    pub fn resolved(self) -> PresenterHostStyle {
        self.local.overlay(self.presenter)
    }
}
