use super::shared::*;
use fui::platform::device_pixel_ratio;
use fui::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

struct PaintState {
    bitmap: Bitmap,
    bitmap_scale: f32,
    hint_label: Option<RichText>,
    hint_baked: bool,
    painting: bool,
}

pub(super) struct PaintCanvas {
    node: CustomDrawable,
    _state: Rc<RefCell<PaintState>>,
}

impl PaintCanvas {
    pub(super) fn new(theme: &Theme) -> Self {
        let bitmap_scale: f32 = device_pixel_ratio().max(1.0);
        let backing_size = (300.0_f32 * bitmap_scale).ceil().max(1.0) as u32;
        let bitmap = Bitmap::new(backing_size, backing_size);
        bitmap.commit();
        let state = Rc::new(RefCell::new(PaintState {
            bitmap,
            bitmap_scale,
            hint_label: None,
            hint_baked: false,
            painting: false,
        }));
        let node = surface({
            let state = state.clone();
            move |ctx| draw_paint_canvas(ctx, &mut state.borrow_mut())
        });
        node.node_id("widget-paint")
            .semantic_role(SemanticRole::Image)
            .semantic_label("Paint canvas - drag to draw");
        node.on_pointer_down({
            let state = state.clone();
            let invalidator = node.invalidator();
            move |event| {
                let mut state = state.borrow_mut();
                state.painting = true;
                brush_at_logical(&mut state, event.x, event.y, 8.0, NEEDLE);
                state.bitmap.clear_dirty_rects().commit();
                invalidator.mark_dirty();
            }
        });
        node.on_pointer_move({
            let state = state.clone();
            let invalidator = node.invalidator();
            move |event| {
                let mut state = state.borrow_mut();
                if !state.painting {
                    return;
                }
                brush_at_logical(&mut state, event.x, event.y, 8.0, NEEDLE);
                state.bitmap.clear_dirty_rects().commit();
                invalidator.mark_dirty();
            }
        });
        node.on_pointer_up({
            let state = state.clone();
            move |_| state.borrow_mut().painting = false
        });

        let hint = RichText::new(vec![
            span("Draw ").italic(),
            span("here")
                .font_size(30.0)
                .text_color(0x3AC56CFF)
                .font_family(theme.fonts.mono_family.clone())
                .bold()
                .underline(),
            span(" on this canvas!").bold(),
        ]);
        hint.font_family(theme.fonts.body_family.clone())
            .font_size(24.0)
            .width(268.0, Unit::Pixel)
            .height(268.0, Unit::Pixel);
        let weak_state = Rc::downgrade(&state);
        let invalidator = node.invalidator();
        state.borrow().bitmap.on_text_ready(&hint, {
            let hint = hint.clone();
            move |_| {
                let Some(state) = weak_state.upgrade() else {
                    return;
                };
                let mut state = state.borrow_mut();
                state.hint_label = Some(hint.clone());
                state.hint_baked = false;
                invalidator.mark_dirty();
            }
        });
        Self {
            node,
            _state: state,
        }
    }

    pub(super) fn node(&self) -> &CustomDrawable {
        &self.node
    }
}

fn brush_at_logical(state: &mut PaintState, x: f32, y: f32, radius: f32, color: u32) {
    let center_x = (x * state.bitmap_scale).round() as i32;
    let center_y = (y * state.bitmap_scale).round() as i32;
    let radius = (radius * state.bitmap_scale).round().max(1.0) as i32;
    let width = state.bitmap.width() as i32;
    let height = state.bitmap.height() as i32;
    let mut pixels = state.bitmap.pixels();
    for delta_y in -radius..=radius {
        for delta_x in -radius..=radius {
            if delta_x * delta_x + delta_y * delta_y > radius * radius {
                continue;
            }
            let pixel_x = center_x + delta_x;
            let pixel_y = center_y + delta_y;
            if pixel_x < 0 || pixel_x >= width || pixel_y < 0 || pixel_y >= height {
                continue;
            }
            let offset = ((pixel_y * width + pixel_x) * 4) as usize;
            pixels[offset] = ((color >> 24) & 0xff) as u8;
            pixels[offset + 1] = ((color >> 16) & 0xff) as u8;
            pixels[offset + 2] = ((color >> 8) & 0xff) as u8;
            pixels[offset + 3] = 0xff;
        }
    }
}

fn draw_paint_canvas(ctx: &mut DrawContext, state: &mut PaintState) {
    let size = 300.0;
    ctx.draw_round_rect(0.0, 0.0, size, size, 12.0, 12.0, Paint::fill(CARD));
    ctx.draw_image(state.bitmap.texture_id(), 0.0, 0.0, size, size);
    if !state.hint_baked {
        if let Some(hint) = state.hint_label.as_ref() {
            state.bitmap.render(hint, 16.0, 32.0, state.bitmap_scale);
            state.bitmap.commit();
            state.hint_baked = true;
        }
    }
    let hint = 0x9696AA78;
    let length = 16.0;
    let margin = 8.0;
    ctx.draw_line(margin, length, margin, margin, hint, 1.5);
    ctx.draw_line(margin, margin, length, margin, hint, 1.5);
    ctx.draw_line(size - length, margin, size - margin, margin, hint, 1.5);
    ctx.draw_line(size - margin, margin, size - margin, length, hint, 1.5);
    ctx.draw_line(margin, size - length, margin, size - margin, hint, 1.5);
    ctx.draw_line(margin, size - margin, length, size - margin, hint, 1.5);
    ctx.draw_line(
        size - length,
        size - margin,
        size - margin,
        size - margin,
        hint,
        1.5,
    );
    ctx.draw_line(
        size - margin,
        size - margin,
        size - margin,
        size - length,
        hint,
        1.5,
    );
}
