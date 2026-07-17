use fui::prelude::*;

pub(super) const ACCENT: u32 = 0x3A6CC5FF;
pub(super) const GRAY: u32 = 0xC8C8C8FF;
pub(super) const NEEDLE: u32 = 0xDC3232FF;
pub(super) const CARD: u32 = 0x232332FF;

pub(super) fn create_plot_title(title: &str, theme: &Theme) -> TextLayout {
    let layout = TextLayout::text(title);
    layout
        .font_family(theme.fonts.body_family.clone())
        .font_size(13.0)
        .text_color(0xEBEEF5D2)
        .width(180.0, Unit::Pixel)
        .height(24.0, Unit::Pixel);
    layout
}

pub(super) fn create_numeric_label(color: u32, theme: &Theme) -> DynamicTextLayout {
    let layout = DynamicTextLayout::numeric();
    layout
        .precision(0)
        .font_family(theme.fonts.body_family.clone())
        .font_size(12.0)
        .text_color(color)
        .width(72.0, Unit::Pixel)
        .height(20.0, Unit::Pixel);
    layout.set_value(0.0);
    layout
}

pub(super) fn create_dynamic_mono_label(color: u32, theme: &Theme) -> DynamicTextLayout {
    let layout = DynamicTextLayout::fixed_charset("0123456789.-, ");
    layout
        .font_family(theme.fonts.mono_family.clone())
        .font_size(12.0)
        .text_color(color)
        .width(72.0, Unit::Pixel)
        .height(20.0, Unit::Pixel);
    layout.set_text("0.0,0.0");
    layout
}

pub(super) fn wake_for_layout(node: &CustomDrawable, layout: &TextLayout) {
    let invalidator = node.invalidator();
    layout.on_ready(move |_| invalidator.mark_dirty());
}

pub(super) fn wake_for_dynamic(node: &CustomDrawable, layout: &DynamicTextLayout) {
    let invalidator = node.invalidator();
    layout.on_ready(move |_| invalidator.mark_dirty());
}

pub(super) fn draw_plot_title(ctx: &DrawContext, title: &TextLayout) {
    if title.is_ready() {
        ctx.draw_text_layout(title, 16.0, 24.0);
    }
}

pub(super) fn draw_dynamic_label(ctx: &DrawContext, label: &DynamicTextLayout, x: f32, y: f32) {
    if !label.is_ready() {
        return;
    }
    let width = label.measure().width + 10.0;
    ctx.draw_round_rect(x, y, width, 22.0, 5.0, 5.0, Paint::fill(0x080A12E1));
    ctx.draw_dynamic_text_layout(label, x + 5.0, y + 4.0);
}

pub(super) fn surface(draw: impl Fn(&mut DrawContext) + 'static) -> CustomDrawable {
    let node = custom_drawable(draw);
    node.width(300.0, Unit::Pixel).height(300.0, Unit::Pixel);
    node
}
