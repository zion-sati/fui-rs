use crate::PopupPlacement;
use std::cell::Cell;

thread_local! {
    static NEXT_TOOL_TIP_ID: Cell<u64> = const { Cell::new(1) };
}

fn next_tool_tip_id() -> u64 {
    NEXT_TOOL_TIP_ID.with(|slot| {
        let id = slot.get();
        slot.set(id.wrapping_add(1).max(1));
        id
    })
}

#[derive(Clone, Debug)]
pub struct ToolTip {
    id: u64,
    text_value: String,
    initial_show_delay_ms: i32,
    between_show_delay_ms: i32,
    show_duration_ms: i32,
    placement: PopupPlacement,
    horizontal_offset: f32,
    vertical_offset: f32,
    open_on_focus: bool,
    panel_background_color: u32,
    text_color: u32,
    panel_background_overridden: bool,
    text_color_overridden: bool,
}

impl Default for ToolTip {
    fn default() -> Self {
        Self {
            id: next_tool_tip_id(),
            text_value: String::new(),
            initial_show_delay_ms: 700,
            between_show_delay_ms: 100,
            show_duration_ms: 5000,
            placement: PopupPlacement::Top,
            horizontal_offset: 0.0,
            vertical_offset: 0.0,
            open_on_focus: true,
            panel_background_color: 0,
            text_color: 0,
            panel_background_overridden: false,
            text_color_overridden: false,
        }
    }
}

impl PartialEq for ToolTip {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl ToolTip {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn text(value: impl Into<String>) -> Self {
        Self::new().with_text(value)
    }

    pub fn with_text(mut self, value: impl Into<String>) -> Self {
        self.text_value = value.into();
        self
    }

    pub fn content_text(&self) -> &str {
        &self.text_value
    }

    pub fn initial_show_delay(mut self, value: i32) -> Self {
        self.initial_show_delay_ms = value.max(0);
        self
    }

    pub fn initial_show_delay_ms(&self) -> i32 {
        self.initial_show_delay_ms
    }

    pub fn between_show_delay(mut self, value: i32) -> Self {
        self.between_show_delay_ms = value.max(0);
        self
    }

    pub fn between_show_delay_ms(&self) -> i32 {
        self.between_show_delay_ms
    }

    pub fn show_duration(mut self, value: i32) -> Self {
        self.show_duration_ms = value.max(0);
        self
    }

    pub fn show_duration_ms(&self) -> i32 {
        self.show_duration_ms
    }

    pub fn placement(mut self, value: PopupPlacement) -> Self {
        self.placement = value;
        self
    }

    pub fn popup_placement(&self) -> PopupPlacement {
        self.placement
    }

    pub fn horizontal_offset(mut self, value: f32) -> Self {
        self.horizontal_offset = value;
        self
    }

    pub fn horizontal_offset_px(&self) -> f32 {
        self.horizontal_offset
    }

    pub fn vertical_offset(mut self, value: f32) -> Self {
        self.vertical_offset = value;
        self
    }

    pub fn vertical_offset_px(&self) -> f32 {
        self.vertical_offset
    }

    pub fn open_on_focus(mut self, flag: bool) -> Self {
        self.open_on_focus = flag;
        self
    }

    pub fn opens_on_focus(&self) -> bool {
        self.open_on_focus
    }

    pub fn panel_color(mut self, color: u32) -> Self {
        self.panel_background_overridden = true;
        self.panel_background_color = color;
        self
    }

    pub fn has_panel_color_override(&self) -> bool {
        self.panel_background_overridden
    }

    pub fn panel_background_color(&self) -> u32 {
        self.panel_background_color
    }

    pub fn text_color(mut self, color: u32) -> Self {
        self.text_color_overridden = true;
        self.text_color = color;
        self
    }

    pub fn has_text_color_override(&self) -> bool {
        self.text_color_overridden
    }

    pub fn tooltip_text_color(&self) -> u32 {
        self.text_color
    }
}
