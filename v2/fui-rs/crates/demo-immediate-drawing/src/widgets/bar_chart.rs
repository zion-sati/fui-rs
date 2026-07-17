use super::shared::*;
use fui::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

const COLORS: [u32; 4] = [0x3A6CC5B4, 0x3AC56CB4, 0xC56C3AB4, 0x9E3AC5B4];
const LABEL_COLORS: [u32; 4] = [0xDCE8FFE6, 0xDCFFE8E6, 0xFFE8DCE6, 0xF5DCFFE6];

pub(super) struct BarChart {
    node: CustomDrawable,
    values: Rc<RefCell<[f32; 4]>>,
    labels: [DynamicTextLayout; 4],
}

impl BarChart {
    pub(super) fn new(theme: &Theme) -> Self {
        let values = Rc::new(RefCell::new([0.0; 4]));
        let title = create_plot_title("Bar chart", theme);
        let labels = LABEL_COLORS.map(|color| create_numeric_label(color, theme));
        let node = surface({
            let values = values.clone();
            let title = title.clone();
            let labels = labels.clone();
            move |ctx| {
                let size = 300.0;
                let pad = 14.0;
                let bar_width = 48.0;
                let gap = 16.0;
                let base_y = size - pad;
                ctx.draw_round_rect(0.0, 0.0, size, size, 12.0, 12.0, Paint::fill(CARD));
                draw_plot_title(ctx, &title);
                let values = values.borrow();
                for index in 0..4 {
                    let x = pad + (bar_width + gap) * index as f32;
                    let height = values[index] / 100.0 * (size - pad * 2.0);
                    ctx.draw_rect(
                        x,
                        base_y - height,
                        bar_width,
                        height,
                        Paint::fill(COLORS[index]),
                    );
                    draw_dynamic_label(ctx, &labels[index], x + 2.0, base_y - height - 28.0);
                }
            }
        });
        node.node_id("immediate-drawing-bar-chart")
            .semantic_role(SemanticRole::Image)
            .semantic_label("Animated bar chart drawing sample");
        wake_for_layout(&node, &title);
        for label in &labels {
            wake_for_dynamic(&node, label);
        }
        Self {
            node,
            values,
            labels,
        }
    }

    pub(super) fn node(&self) -> &CustomDrawable {
        &self.node
    }

    pub(super) fn push_values(&self, a: f32, b: f32, c: f32, d: f32) {
        *self.values.borrow_mut() = [a, b, c, d];
        for (label, value) in self.labels.iter().zip([a, b, c, d]) {
            label.set_value(value as f64);
        }
        self.node.mark_dirty();
    }
}
