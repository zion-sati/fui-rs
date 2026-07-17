use super::shared::*;
use fui::prelude::*;
use std::cell::RefCell;
use std::f32::consts::PI;
use std::rc::Rc;

const YARN_A: u32 = 0xFF72A8E6;
const YARN_C: u32 = 0xFFD86FE6;

struct YarnState {
    noise_x: f32,
    noise_y: f32,
    noise_z: f32,
    direction_x: f32,
    direction_y: f32,
    direction_z: f32,
    dragging: bool,
    pointer_x: f32,
    pointer_y: f32,
    pull: f32,
}

pub(super) struct DancingYarn {
    node: CustomDrawable,
    state: Rc<RefCell<YarnState>>,
}

impl DancingYarn {
    pub(super) fn new(theme: &Theme) -> Self {
        let state = Rc::new(RefCell::new(YarnState {
            noise_x: 0.0,
            noise_y: 2.7,
            noise_z: 5.1,
            direction_x: 1.0,
            direction_y: 1.0,
            direction_z: -1.0,
            dragging: false,
            pointer_x: 150.0,
            pointer_y: 150.0,
            pull: 0.0,
        }));
        let title = create_plot_title("Dancing yarn", theme);
        let node = surface({
            let state = state.clone();
            let title = title.clone();
            move |ctx| draw_yarn(ctx, &state.borrow(), &title)
        });
        node.node_id("widget-yarn")
            .semantic_role(SemanticRole::Image)
            .semantic_label("Dancing yarn interactive noise panel");
        node.on_pointer_down({
            let state = state.clone();
            let invalidator = node.invalidator();
            move |event| {
                event.capture_pointer();
                let mut state = state.borrow_mut();
                state.dragging = true;
                state.pointer_x = event.x;
                state.pointer_y = event.y;
                state.pull = 1.0;
                invalidator.mark_dirty();
            }
        });
        node.on_pointer_move({
            let state = state.clone();
            let invalidator = node.invalidator();
            move |event| {
                let mut state = state.borrow_mut();
                if !state.dragging {
                    return;
                }
                state.pointer_x = event.x;
                state.pointer_y = event.y;
                state.pull = 1.0;
                invalidator.mark_dirty();
            }
        });
        let end_drag = {
            let state = state.clone();
            let invalidator = node.invalidator();
            move || {
                state.borrow_mut().dragging = false;
                invalidator.mark_dirty();
            }
        };
        node.on_pointer_up({
            let end_drag = end_drag.clone();
            move |event| {
                end_drag();
                event.release_pointer_capture();
            }
        });
        node.on_pointer_cancel(move |event| {
            end_drag();
            event.release_pointer_capture();
        });
        wake_for_layout(&node, &title);
        Self { node, state }
    }

    pub(super) fn node(&self) -> &CustomDrawable {
        &self.node
    }

    pub(super) fn tick(&self) {
        let mut state = self.state.borrow_mut();
        (state.noise_x, state.direction_x) = advance_noise(state.noise_x, state.direction_x, 0.032);
        (state.noise_y, state.direction_y) = advance_noise(state.noise_y, state.direction_y, 0.021);
        (state.noise_z, state.direction_z) = advance_noise(state.noise_z, state.direction_z, 0.027);
        if !state.dragging && state.pull > 0.0 {
            state.pull *= 0.88;
            if state.pull < 0.02 {
                state.pull = 0.0;
            }
        }
        drop(state);
        self.node.mark_dirty();
    }
}

fn advance_noise(mut value: f32, mut direction: f32, step: f32) -> (f32, f32) {
    value += step * direction;
    if value > 9.0 {
        value = 9.0;
        direction = -1.0;
    } else if value < 0.0 {
        value = 0.0;
        direction = 1.0;
    }
    (value, direction)
}

fn clamp_unit(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

fn fade_noise(value: f32) -> f32 {
    value * value * value * (value * (value * 6.0 - 15.0) + 10.0)
}

fn hash3(x: i32, y: i32, z: i32) -> f32 {
    let mut hash = x
        .wrapping_mul(374_761_393)
        .wrapping_add(y.wrapping_mul(668_265_263))
        .wrapping_add(z.wrapping_mul(2_147_483_647));
    hash = (hash ^ (hash >> 13)).wrapping_mul(1_274_126_177);
    hash ^= hash >> 16;
    (hash & 0x7fff_ffff) as f32 / 2_147_483_647.0
}

fn value_noise3(x: f32, y: f32, z: f32) -> f32 {
    let ix = x.floor() as i32;
    let iy = y.floor() as i32;
    let iz = z.floor() as i32;
    let ux = fade_noise(x - ix as f32);
    let uy = fade_noise(y - iy as f32);
    let uz = fade_noise(z - iz as f32);
    let x00 = hash3(ix, iy, iz) + (hash3(ix + 1, iy, iz) - hash3(ix, iy, iz)) * ux;
    let x10 = hash3(ix, iy + 1, iz) + (hash3(ix + 1, iy + 1, iz) - hash3(ix, iy + 1, iz)) * ux;
    let x01 = hash3(ix, iy, iz + 1) + (hash3(ix + 1, iy, iz + 1) - hash3(ix, iy, iz + 1)) * ux;
    let x11 = hash3(ix, iy + 1, iz + 1)
        + (hash3(ix + 1, iy + 1, iz + 1) - hash3(ix, iy + 1, iz + 1)) * ux;
    let y0 = x00 + (x10 - x00) * uy;
    let y1 = x01 + (x11 - x01) * uy;
    y0 + (y1 - y0) * uz
}

fn yarn_color(value: f32) -> u32 {
    let warm = clamp_unit(0.5 + (value * PI * 2.0).sin() * 0.5);
    let cool = 1.0 - warm;
    let red = (116.0 * cool + 255.0 * warm).round() as u32;
    let green = (222.0 * cool + 132.0 * warm).round() as u32;
    let blue = (255.0 * cool + 150.0 * warm).round() as u32;
    (red << 24) | (green << 16) | (blue << 8) | 230
}

fn draw_yarn(ctx: &mut DrawContext, state: &YarnState, title: &TextLayout) {
    let size = 300.0;
    let center_x = size / 2.0;
    let center_y = size / 2.0 + 8.0;
    let pointer_bias_y = (state.pointer_x / size - 0.5) * 1.7 * state.pull;
    let pointer_bias_z = (state.pointer_y / size - 0.5) * 1.7 * state.pull;
    ctx.draw_round_rect(0.0, 0.0, size, size, 12.0, 12.0, Paint::fill(CARD));
    draw_plot_title(ctx, title);
    ctx.save();
    ctx.clip_round_rect(0.0, 0.0, size, size, 12.0, 12.0, 12.0, 12.0);
    ctx.draw_circle(
        state.pointer_x,
        state.pointer_y,
        22.0 + 10.0 * state.pull,
        Paint::stroke(0xFFFFFF00 | (36.0 + 70.0 * state.pull).round() as u32, 1.0),
    );
    let mut previous = (0.0, 0.0);
    for index in 0..132 {
        let t = index as f32 / 131.0;
        let x_base = 28.0 + t * (size - 56.0);
        let centered = (t - 0.5) * 2.0;
        let envelope = 1.0 - centered * centered;
        let n0 = value_noise3(
            t * 4.2 + state.noise_x,
            state.noise_y + pointer_bias_y,
            state.noise_z + pointer_bias_z,
        );
        let n1 = value_noise3(
            t * 5.6 + state.noise_x + 6.0,
            state.noise_y + 3.0 + pointer_bias_y,
            state.noise_z + 1.4 + pointer_bias_z,
        );
        let n2 = value_noise3(
            t * 7.3 + state.noise_x + 2.0,
            state.noise_y + 8.0 + pointer_bias_y,
            state.noise_z + 4.0 + pointer_bias_z,
        );
        let angle = (n0 * 2.0 - 1.0) * PI * 1.35;
        let radius = (24.0 + n1 * 46.0) * (0.35 + envelope * 0.95);
        let mut x = x_base + angle.cos() * radius * 0.72;
        let mut y = center_y + angle.sin() * radius + (n2 - 0.5) * 42.0;
        let delta_x = state.pointer_x - x;
        let delta_y = state.pointer_y - y;
        let distance = (delta_x * delta_x + delta_y * delta_y).sqrt();
        let influence = clamp_unit(1.0 - distance / 145.0).powi(2) * state.pull;
        x += delta_x * influence * 0.62 + (t * 18.0 + state.noise_z).sin() * influence * 20.0;
        y += delta_y * influence * 0.62 + (t * 17.0 + state.noise_y).cos() * influence * 20.0;
        if index > 0 {
            ctx.draw_line(
                previous.0,
                previous.1,
                x,
                y,
                yarn_color(t + state.noise_x * 0.07),
                1.3 + n2 * 2.2 + influence * 1.4,
            );
        }
        if index % 19 == 0 {
            ctx.draw_circle(
                x,
                y,
                2.2 + n1 * 2.2,
                Paint::fill(if index % 38 == 0 { YARN_C } else { YARN_A }),
            );
        }
        previous = (x, y);
    }
    ctx.draw_circle(center_x, center_y, 76.0, Paint::stroke(0xFFFFFF16, 1.0));
    ctx.restore();
}
