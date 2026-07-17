use super::shared::*;
use fui::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

const WAVE_LINE: u32 = 0x3AC59EFF;
const WAVE_DIM: u32 = 0x3AC59E3C;

pub(super) struct Waveform {
    node: CustomDrawable,
    values: Rc<RefCell<[f32; 4]>>,
}

impl Waveform {
    pub(super) fn new(theme: &Theme) -> Self {
        let values = Rc::new(RefCell::new([0.0; 4]));
        let title = create_plot_title("Waveform", theme);
        let node = surface({
            let values = values.clone();
            let title = title.clone();
            move |ctx| {
                let size = 300.0;
                let pad = 14.0;
                let mid = size / 2.0;
                let amplitude = size * 0.33;
                let steps = 40;
                ctx.draw_round_rect(0.0, 0.0, size, size, 12.0, 12.0, Paint::fill(CARD));
                draw_plot_title(ctx, &title);
                let values = values.borrow();
                for (index, value) in values.iter().enumerate() {
                    let phase = index as f32 * 1.2;
                    let color = if index == 2 { WAVE_LINE } else { WAVE_DIM };
                    for step in 1..steps {
                        let x0 = pad + (step - 1) as f32 / (steps - 1) as f32 * (size - pad * 2.0);
                        let x1 = pad + step as f32 / (steps - 1) as f32 * (size - pad * 2.0);
                        let t0 = (step - 1) as f32 * 0.3 + phase;
                        let t1 = step as f32 * 0.3 + phase;
                        let scale = *value / 100.0;
                        ctx.draw_line(
                            x0,
                            mid + t0.sin() * amplitude * scale,
                            x1,
                            mid + t1.sin() * amplitude * scale,
                            color,
                            1.5,
                        );
                    }
                }
            }
        });
        wake_for_layout(&node, &title);
        Self { node, values }
    }

    pub(super) fn node(&self) -> &CustomDrawable {
        &self.node
    }

    pub(super) fn push_values(&self, a: f32, b: f32, c: f32, d: f32) {
        *self.values.borrow_mut() = [a, b, c, d];
        self.node.mark_dirty();
    }
}
