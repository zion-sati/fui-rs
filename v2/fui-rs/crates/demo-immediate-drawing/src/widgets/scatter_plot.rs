use super::shared::*;
use fui::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

const COLORS: [u32; 4] = [0x3A6CC5FF, 0x3AC56CFF, 0xC56C3AFF, 0xFFB43CFF];
const LABEL_COLORS: [u32; 4] = [0xDCE8FFE6, 0xDCFFE8E6, 0xFFE8DCE6, 0xFFEEBEE6];

pub(super) struct ScatterPlot {
    node: CustomDrawable,
    points: Rc<RefCell<[(f32, f32); 4]>>,
    labels: [DynamicTextLayout; 4],
}

impl ScatterPlot {
    pub(super) fn new(theme: &Theme) -> Self {
        let points = Rc::new(RefCell::new([(0.0, 0.0); 4]));
        let title = create_plot_title("Scatter plot", theme);
        let labels = LABEL_COLORS.map(|color| create_dynamic_mono_label(color, theme));
        let node = surface({
            let points = points.clone();
            let title = title.clone();
            let labels = labels.clone();
            move |ctx| {
                let size = 300.0;
                ctx.draw_round_rect(0.0, 0.0, size, size, 12.0, 12.0, Paint::fill(CARD));
                draw_plot_title(ctx, &title);
                let points = points.borrow();
                for index in 0..4 {
                    let next = (index + 1) % 4;
                    ctx.draw_line(
                        points[index].0,
                        points[index].1,
                        points[next].0,
                        points[next].1,
                        0xFFFFFF1E,
                        0.5,
                    );
                    ctx.draw_circle(
                        points[index].0,
                        points[index].1,
                        6.0,
                        Paint::fill(COLORS[index]),
                    );
                    draw_dynamic_label(
                        ctx,
                        &labels[index],
                        points[index].0 + 8.0,
                        points[index].1 - 24.0,
                    );
                }
            }
        });
        wake_for_layout(&node, &title);
        for label in &labels {
            wake_for_dynamic(&node, label);
        }
        Self {
            node,
            points,
            labels,
        }
    }

    pub(super) fn node(&self) -> &CustomDrawable {
        &self.node
    }

    pub(super) fn push_values(&self, a: f32, b: f32, c: f32, d: f32) {
        let size = 300.0;
        let pad = 30.0;
        let scale = (size - pad * 2.0) / 2.0;
        let center = size / 2.0;
        *self.points.borrow_mut() = [
            (center + a * scale, center + b * scale),
            (center + c * scale, center + a * scale * 0.7),
            (center + b * scale, center + d * scale),
            (center + d * scale, center + c * scale * 0.7),
        ];
        for (label, text) in self.labels.iter().zip([
            format!("{a:.1},{b:.1}"),
            format!("{c:.1},{:.1}", a * 0.7),
            format!("{b:.1},{d:.1}"),
            format!("{d:.1},{:.1}", c * 0.7),
        ]) {
            label.set_text(text);
        }
        self.node.mark_dirty();
    }
}
