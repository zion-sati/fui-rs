use crate::logger;

fn sanitize_positive(owner: &str, property: &str, value: f32) -> f32 {
    if value <= 0.0 {
        logger::warn(
            "Layout",
            &format!("{owner}.{property}() received {value}; ignoring."),
        );
        return 0.0;
    }
    value
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LabeledControlSizing {
    indicator_size: f32,
    label_font_size: f32,
}

impl LabeledControlSizing {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn indicator_size(mut self, value: f32) -> Self {
        self.indicator_size = sanitize_positive("LabeledControlSizing", "indicator_size", value);
        self
    }

    pub fn label_font_size(mut self, value: f32) -> Self {
        self.label_font_size = sanitize_positive("LabeledControlSizing", "label_font_size", value);
        self
    }

    pub fn has_indicator_size(&self) -> bool {
        self.indicator_size > 0.0
    }

    pub fn has_label_font_size(&self) -> bool {
        self.label_font_size > 0.0
    }

    pub fn indicator_size_px(&self) -> f32 {
        self.indicator_size
    }

    pub fn label_font_size_px(&self) -> f32 {
        self.label_font_size
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SliderSizing {
    thumb_size: f32,
    track_thickness: f32,
}

impl SliderSizing {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn thumb_size(mut self, value: f32) -> Self {
        self.thumb_size = sanitize_positive("SliderSizing", "thumb_size", value);
        self
    }

    pub fn track_thickness(mut self, value: f32) -> Self {
        self.track_thickness = sanitize_positive("SliderSizing", "track_thickness", value);
        self
    }

    pub fn has_thumb_size(&self) -> bool {
        self.thumb_size > 0.0
    }

    pub fn has_track_thickness(&self) -> bool {
        self.track_thickness > 0.0
    }

    pub fn thumb_size_px(&self) -> f32 {
        self.thumb_size
    }

    pub fn track_thickness_px(&self) -> f32 {
        self.track_thickness
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DropdownSizing {
    field_font_size: f32,
    option_font_size: f32,
    field_height: f32,
    option_height: f32,
    chevron_box_size: f32,
    chevron_icon_size: f32,
}

impl DropdownSizing {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn field_font_size(mut self, value: f32) -> Self {
        self.field_font_size = sanitize_positive("DropdownSizing", "field_font_size", value);
        self
    }

    pub fn option_font_size(mut self, value: f32) -> Self {
        self.option_font_size = sanitize_positive("DropdownSizing", "option_font_size", value);
        self
    }

    pub fn field_height(mut self, value: f32) -> Self {
        self.field_height = sanitize_positive("DropdownSizing", "field_height", value);
        self
    }

    pub fn option_height(mut self, value: f32) -> Self {
        self.option_height = sanitize_positive("DropdownSizing", "option_height", value);
        self
    }

    pub fn chevron_box_size(mut self, value: f32) -> Self {
        self.chevron_box_size = sanitize_positive("DropdownSizing", "chevron_box_size", value);
        self
    }

    pub fn chevron_icon_size(mut self, value: f32) -> Self {
        self.chevron_icon_size = sanitize_positive("DropdownSizing", "chevron_icon_size", value);
        self
    }

    pub fn has_field_font_size(&self) -> bool {
        self.field_font_size > 0.0
    }

    pub fn has_option_font_size(&self) -> bool {
        self.option_font_size > 0.0
    }

    pub fn has_field_height(&self) -> bool {
        self.field_height > 0.0
    }

    pub fn has_option_height(&self) -> bool {
        self.option_height > 0.0
    }

    pub fn has_chevron_box_size(&self) -> bool {
        self.chevron_box_size > 0.0
    }

    pub fn has_chevron_icon_size(&self) -> bool {
        self.chevron_icon_size > 0.0
    }

    pub fn field_font_size_px(&self) -> f32 {
        self.field_font_size
    }

    pub fn option_font_size_px(&self) -> f32 {
        self.option_font_size
    }

    pub fn field_height_px(&self) -> f32 {
        self.field_height
    }

    pub fn option_height_px(&self) -> f32 {
        self.option_height
    }

    pub fn chevron_box_size_px(&self) -> f32 {
        self.chevron_box_size
    }

    pub fn chevron_icon_size_px(&self) -> f32 {
        self.chevron_icon_size
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ButtonColors {
    pub(crate) background: Option<u32>,
    pub(crate) background_hover: Option<u32>,
    pub(crate) background_pressed: Option<u32>,
    pub(crate) text_primary: Option<u32>,
    pub(crate) text_muted: Option<u32>,
    pub(crate) border: Option<u32>,
}

impl ButtonColors {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn background(mut self, color: u32) -> Self {
        self.background = Some(color);
        self
    }

    pub fn background_hover(mut self, color: u32) -> Self {
        self.background_hover = Some(color);
        self
    }

    pub fn background_pressed(mut self, color: u32) -> Self {
        self.background_pressed = Some(color);
        self
    }

    pub fn text_primary(mut self, color: u32) -> Self {
        self.text_primary = Some(color);
        self
    }

    pub fn text_muted(mut self, color: u32) -> Self {
        self.text_muted = Some(color);
        self
    }

    pub fn border(mut self, color: u32) -> Self {
        self.border = Some(color);
        self
    }

    pub fn has_background(&self) -> bool {
        self.background.is_some()
    }

    pub fn background_color(&self) -> u32 {
        self.background.unwrap_or(0)
    }

    pub fn has_background_hover(&self) -> bool {
        self.background_hover.is_some()
    }

    pub fn background_hover_color(&self) -> u32 {
        self.background_hover.unwrap_or(0)
    }

    pub fn has_background_pressed(&self) -> bool {
        self.background_pressed.is_some()
    }

    pub fn background_pressed_color(&self) -> u32 {
        self.background_pressed.unwrap_or(0)
    }

    pub fn has_text_primary(&self) -> bool {
        self.text_primary.is_some()
    }

    pub fn text_primary_color(&self) -> u32 {
        self.text_primary.unwrap_or(0)
    }

    pub fn has_text_muted(&self) -> bool {
        self.text_muted.is_some()
    }

    pub fn text_muted_color(&self) -> u32 {
        self.text_muted.unwrap_or(0)
    }

    pub fn has_border(&self) -> bool {
        self.border.is_some()
    }

    pub fn border_color(&self) -> u32 {
        self.border.unwrap_or(0)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LabeledControlColors {
    pub(crate) background: Option<u32>,
    pub(crate) border: Option<u32>,
    pub(crate) accent: Option<u32>,
    pub(crate) text_primary: Option<u32>,
    pub(crate) text_muted: Option<u32>,
}

impl LabeledControlColors {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn background(mut self, color: u32) -> Self {
        self.background = Some(color);
        self
    }

    pub fn border(mut self, color: u32) -> Self {
        self.border = Some(color);
        self
    }

    pub fn accent(mut self, color: u32) -> Self {
        self.accent = Some(color);
        self
    }

    pub fn text_primary(mut self, color: u32) -> Self {
        self.text_primary = Some(color);
        self
    }

    pub fn text_muted(mut self, color: u32) -> Self {
        self.text_muted = Some(color);
        self
    }

    pub fn has_background(&self) -> bool {
        self.background.is_some()
    }

    pub fn background_color(&self) -> u32 {
        self.background.unwrap_or(0)
    }

    pub fn has_border(&self) -> bool {
        self.border.is_some()
    }

    pub fn border_color(&self) -> u32 {
        self.border.unwrap_or(0)
    }

    pub fn has_accent(&self) -> bool {
        self.accent.is_some()
    }

    pub fn accent_color(&self) -> u32 {
        self.accent.unwrap_or(0)
    }

    pub fn has_text_primary(&self) -> bool {
        self.text_primary.is_some()
    }

    pub fn text_primary_color(&self) -> u32 {
        self.text_primary.unwrap_or(0)
    }

    pub fn has_text_muted(&self) -> bool {
        self.text_muted.is_some()
    }

    pub fn text_muted_color(&self) -> u32 {
        self.text_muted.unwrap_or(0)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SliderColors {
    pub(crate) track: Option<u32>,
    pub(crate) fill: Option<u32>,
    pub(crate) thumb: Option<u32>,
}

impl SliderColors {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn track(mut self, color: u32) -> Self {
        self.track = Some(color);
        self
    }

    pub fn fill(mut self, color: u32) -> Self {
        self.fill = Some(color);
        self
    }

    pub fn thumb(mut self, color: u32) -> Self {
        self.thumb = Some(color);
        self
    }

    pub fn has_track(&self) -> bool {
        self.track.is_some()
    }

    pub fn track_color(&self) -> u32 {
        self.track.unwrap_or(0)
    }

    pub fn has_fill(&self) -> bool {
        self.fill.is_some()
    }

    pub fn fill_color(&self) -> u32 {
        self.fill.unwrap_or(0)
    }

    pub fn has_thumb(&self) -> bool {
        self.thumb.is_some()
    }

    pub fn thumb_color(&self) -> u32 {
        self.thumb.unwrap_or(0)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DropdownColors {
    pub(crate) background: Option<u32>,
    pub(crate) text_primary: Option<u32>,
    pub(crate) placeholder: Option<u32>,
    pub(crate) border: Option<u32>,
    pub(crate) accent: Option<u32>,
}

impl DropdownColors {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn background(mut self, color: u32) -> Self {
        self.background = Some(color);
        self
    }

    pub fn text_primary(mut self, color: u32) -> Self {
        self.text_primary = Some(color);
        self
    }

    pub fn placeholder(mut self, color: u32) -> Self {
        self.placeholder = Some(color);
        self
    }

    pub fn border(mut self, color: u32) -> Self {
        self.border = Some(color);
        self
    }

    pub fn accent(mut self, color: u32) -> Self {
        self.accent = Some(color);
        self
    }

    pub fn has_background(&self) -> bool {
        self.background.is_some()
    }

    pub fn background_color(&self) -> u32 {
        self.background.unwrap_or(0)
    }

    pub fn has_text_primary(&self) -> bool {
        self.text_primary.is_some()
    }

    pub fn text_primary_color(&self) -> u32 {
        self.text_primary.unwrap_or(0)
    }

    pub fn has_placeholder(&self) -> bool {
        self.placeholder.is_some()
    }

    pub fn placeholder_color(&self) -> u32 {
        self.placeholder.unwrap_or(0)
    }

    pub fn has_border(&self) -> bool {
        self.border.is_some()
    }

    pub fn border_color(&self) -> u32 {
        self.border.unwrap_or(0)
    }

    pub fn has_accent(&self) -> bool {
        self.accent.is_some()
    }

    pub fn accent_color(&self) -> u32 {
        self.accent.unwrap_or(0)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextInputColors {
    pub(crate) background: Option<u32>,
    pub(crate) text_primary: Option<u32>,
    pub(crate) text_muted: Option<u32>,
    pub(crate) placeholder: Option<u32>,
    pub(crate) caret: Option<u32>,
    pub(crate) border: Option<u32>,
    pub(crate) accent: Option<u32>,
}

impl TextInputColors {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn background(mut self, color: u32) -> Self {
        self.background = Some(color);
        self
    }

    pub fn text_primary(mut self, color: u32) -> Self {
        self.text_primary = Some(color);
        self
    }

    pub fn text_muted(mut self, color: u32) -> Self {
        self.text_muted = Some(color);
        self
    }

    pub fn placeholder(mut self, color: u32) -> Self {
        self.placeholder = Some(color);
        self
    }

    pub fn caret(mut self, color: u32) -> Self {
        self.caret = Some(color);
        self
    }

    pub fn border(mut self, color: u32) -> Self {
        self.border = Some(color);
        self
    }

    pub fn accent(mut self, color: u32) -> Self {
        self.accent = Some(color);
        self
    }

    pub fn has_background(&self) -> bool {
        self.background.is_some()
    }

    pub fn background_color(&self) -> u32 {
        self.background.unwrap_or(0)
    }

    pub fn has_text_primary(&self) -> bool {
        self.text_primary.is_some()
    }

    pub fn text_primary_color(&self) -> u32 {
        self.text_primary.unwrap_or(0)
    }

    pub fn has_text_muted(&self) -> bool {
        self.text_muted.is_some()
    }

    pub fn text_muted_color(&self) -> u32 {
        self.text_muted.unwrap_or(0)
    }

    pub fn has_placeholder(&self) -> bool {
        self.placeholder.is_some()
    }

    pub fn placeholder_color(&self) -> u32 {
        self.placeholder.unwrap_or(0)
    }

    pub fn has_caret(&self) -> bool {
        self.caret.is_some()
    }

    pub fn caret_color(&self) -> u32 {
        self.caret.unwrap_or(0)
    }

    pub fn has_border(&self) -> bool {
        self.border.is_some()
    }

    pub fn border_color(&self) -> u32 {
        self.border.unwrap_or(0)
    }

    pub fn has_accent(&self) -> bool {
        self.accent.is_some()
    }

    pub fn accent_color(&self) -> u32 {
        self.accent.unwrap_or(0)
    }
}
