use super::shared::*;
use fui::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

const SPARK_LINE: u32 = 0xFFB43CFF;

struct SparklineHistory {
    values: [f32; 80],
    write_position: usize,
    length: usize,
}

impl Default for SparklineHistory {
    fn default() -> Self {
        Self {
            values: [0.0; 80],
            write_position: 0,
            length: 0,
        }
    }
}

pub(super) struct Sparkline {
    node: CustomDrawable,
    history: Rc<RefCell<SparklineHistory>>,
}

impl Sparkline {
    pub(super) fn new(theme: &Theme) -> Self {
        let history = Rc::new(RefCell::new(SparklineHistory::default()));
        let title = create_plot_title("Sparkline", theme);
        let node = surface({
            let history = history.clone();
            let title = title.clone();
            move |ctx| {
                let size = 300.0;
                let pad = 14.0;
                ctx.draw_round_rect(0.0, 0.0, size, size, 12.0, 12.0, Paint::fill(CARD));
                draw_plot_title(ctx, &title);
                let history = history.borrow();
                if history.length < 2 {
                    return;
                }
                let step_x = (size - pad * 2.0) / (history.length - 1) as f32;
                for index in 1..history.length {
                    let previous = (history.write_position + 80 - history.length + index - 1) % 80;
                    let current = (history.write_position + 80 - history.length + index) % 80;
                    let x0 = pad + step_x * (index - 1) as f32;
                    let y0 = size - pad - history.values[previous] / 100.0 * (size - pad * 2.0);
                    let x1 = pad + step_x * index as f32;
                    let y1 = size - pad - history.values[current] / 100.0 * (size - pad * 2.0);
                    ctx.draw_line(x0, y0, x1, y1, SPARK_LINE, 2.0);
                }
            }
        });
        wake_for_layout(&node, &title);
        Self { node, history }
    }

    pub(super) fn node(&self) -> &CustomDrawable {
        &self.node
    }

    pub(super) fn push(&self, value: f32) {
        let mut history = self.history.borrow_mut();
        let position = history.write_position;
        history.values[position] = value;
        history.write_position = (position + 1) % history.values.len();
        history.length = (history.length + 1).min(history.values.len());
        drop(history);
        self.node.mark_dirty();
    }
}
