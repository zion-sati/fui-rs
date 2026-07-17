use super::shared::*;
use fui::prelude::*;
use std::cell::Cell;
use std::f32::consts::PI;
use std::rc::Rc;

pub(super) struct Gauge {
    node: CustomDrawable,
    value: Rc<Cell<f32>>,
}

impl Gauge {
    pub(super) fn new(theme: &Theme) -> Self {
        let value = Rc::new(Cell::new(0.0));
        let title = create_plot_title("Gauge", theme);
        let node = surface({
            let value = value.clone();
            let title = title.clone();
            move |ctx| {
                let size = 300.0;
                let center = size * 0.5;
                ctx.draw_round_rect(0.0, 0.0, size, size, 12.0, 12.0, Paint::fill(CARD));
                draw_plot_title(ctx, &title);
                ctx.draw_circle(center, center, size * 0.4, Paint::stroke(GRAY, 6.0));
                let angle = ((value.get() / 100.0) * 270.0 - 135.0) * PI / 180.0;
                let length = size * 0.3;
                ctx.draw_line(
                    center,
                    center,
                    center + angle.cos() * length,
                    center + angle.sin() * length,
                    NEEDLE,
                    3.0,
                );
                ctx.draw_circle(center, center, 6.0, Paint::fill(ACCENT));
            }
        });
        node.node_id("immediate-drawing-gauge")
            .semantic_role(SemanticRole::Image)
            .semantic_label("Animated gauge drawing sample");
        wake_for_layout(&node, &title);
        Self { node, value }
    }

    pub(super) fn node(&self) -> &CustomDrawable {
        &self.node
    }

    pub(super) fn set_value(&self, value: f32) {
        self.value.set(value);
        self.node.mark_dirty();
    }
}
