use crate::{Border, Corners, EdgeInsets, FontFamily, FontStyle, FontWeight, Shadow};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SurfaceAppearance {
    pub(crate) background: Option<u32>,
    pub(crate) background_blur: Option<f32>,
    pub(crate) border: Option<Border>,
    pub(crate) corners: Option<Corners>,
    pub(crate) shadow: Option<Shadow>,
}

impl SurfaceAppearance {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn background(mut self, value: u32) -> Self {
        self.background = Some(value);
        self
    }
    pub fn background_blur(mut self, value: f32) -> Self {
        self.background_blur = Some(value.max(0.0));
        self
    }
    pub fn border(mut self, value: Border) -> Self {
        self.border = Some(value);
        self
    }
    pub fn corners(mut self, value: Corners) -> Self {
        self.corners = Some(value);
        self
    }
    pub fn shadow(mut self, value: Shadow) -> Self {
        self.shadow = Some(value);
        self
    }

    pub(crate) fn presenter_host_style(&self) -> crate::PresenterHostStyle {
        let mut style = crate::PresenterHostStyle::new();
        style.background = self.background;
        style.border = self.border;
        style.corners = self.corners;
        style.shadow = self.shadow;
        style
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct OverlayBackdropAppearance {
    pub(crate) color: Option<u32>,
    pub(crate) blur: Option<f32>,
}

impl OverlayBackdropAppearance {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn color(mut self, value: u32) -> Self {
        self.color = Some(value);
        self
    }
    pub fn blur(mut self, value: f32) -> Self {
        self.blur = Some(value.max(0.0));
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PopupAppearance {
    pub(crate) panel: Option<SurfaceAppearance>,
    pub(crate) backdrop: Option<OverlayBackdropAppearance>,
}

impl PopupAppearance {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn panel(mut self, value: SurfaceAppearance) -> Self {
        self.panel = Some(value);
        self
    }
    pub fn backdrop(mut self, value: OverlayBackdropAppearance) -> Self {
        self.backdrop = Some(value);
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DialogAppearance {
    pub(crate) card: Option<SurfaceAppearance>,
    pub(crate) backdrop: Option<OverlayBackdropAppearance>,
}

impl DialogAppearance {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn card(mut self, value: SurfaceAppearance) -> Self {
        self.card = Some(value);
        self
    }
    pub fn backdrop(mut self, value: OverlayBackdropAppearance) -> Self {
        self.backdrop = Some(value);
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ContextMenuItemAppearance {
    pub(crate) height: Option<f32>,
    pub(crate) padding: Option<EdgeInsets>,
    pub(crate) background: Option<u32>,
    pub(crate) hover_background: Option<u32>,
    pub(crate) text_color: Option<u32>,
    pub(crate) corners: Option<Corners>,
    pub(crate) font_family: Option<FontFamily>,
    pub(crate) font_weight: Option<FontWeight>,
    pub(crate) font_style: Option<FontStyle>,
    pub(crate) font_size: Option<f32>,
}

impl ContextMenuItemAppearance {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn height(mut self, value: f32) -> Self {
        self.height = Some(value.max(1.0));
        self
    }
    pub fn padding(mut self, value: EdgeInsets) -> Self {
        self.padding = Some(value);
        self
    }
    pub fn background(mut self, value: u32) -> Self {
        self.background = Some(value);
        self
    }
    pub fn hover_background(mut self, value: u32) -> Self {
        self.hover_background = Some(value);
        self
    }
    pub fn text_color(mut self, value: u32) -> Self {
        self.text_color = Some(value);
        self
    }
    pub fn corners(mut self, value: Corners) -> Self {
        self.corners = Some(value);
        self
    }
    pub fn font_family(mut self, value: FontFamily) -> Self {
        self.font_family = Some(value);
        self
    }
    pub fn font_weight(mut self, value: FontWeight) -> Self {
        self.font_weight = Some(value);
        self
    }
    pub fn font_style(mut self, value: FontStyle) -> Self {
        self.font_style = Some(value);
        self
    }
    pub fn font_size(mut self, value: f32) -> Self {
        self.font_size = Some(value.max(1.0));
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ContextMenuAppearance {
    pub(crate) width: Option<f32>,
    pub(crate) panel: Option<SurfaceAppearance>,
    pub(crate) backdrop: Option<OverlayBackdropAppearance>,
    pub(crate) item: Option<ContextMenuItemAppearance>,
    pub(crate) separator_color: Option<u32>,
}

impl ContextMenuAppearance {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn width(mut self, value: f32) -> Self {
        self.width = Some(value.max(1.0));
        self
    }
    pub fn panel(mut self, value: SurfaceAppearance) -> Self {
        self.panel = Some(value);
        self
    }
    pub fn backdrop(mut self, value: OverlayBackdropAppearance) -> Self {
        self.backdrop = Some(value);
        self
    }
    pub fn item(mut self, value: ContextMenuItemAppearance) -> Self {
        self.item = Some(value);
        self
    }
    pub fn separator_color(mut self, value: u32) -> Self {
        self.separator_color = Some(value);
        self
    }
}
