use crate::ffi;
use crate::image_sampling::ImageSampling;
use crate::logger::error;
use crate::node::Node;
use crate::text::TextLayout;
use std::cell::RefCell;
use std::rc::Rc;

const OP_SAVE: u32 = 1;
const OP_RESTORE: u32 = 2;
const OP_TRANSLATE: u32 = 3;
const OP_SCALE: u32 = 4;
const OP_ROTATE: u32 = 5;
const OP_CLIP_RECT: u32 = 6;
const OP_CLIP_ROUND_RECT: u32 = 7;
const OP_DRAW_RECT: u32 = 10;
const OP_DRAW_CIRCLE: u32 = 11;
const OP_DRAW_LINE: u32 = 12;
const OP_DRAW_ROUND_RECT: u32 = 13;
const OP_DRAW_PATH: u32 = 20;
const OP_DRAW_TEXT_NODE: u32 = 30;
const OP_DRAW_IMAGE: u32 = 31;
const OP_DRAW_SVG: u32 = 32;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Paint {
    pub fill_color: u32,
    pub stroke_color: u32,
    pub stroke_width: f32,
}

impl Paint {
    pub fn fill(color: u32) -> Self {
        Self {
            fill_color: color,
            stroke_color: 0,
            stroke_width: 0.0,
        }
    }

    pub fn stroke(color: u32, width: f32) -> Self {
        Self {
            fill_color: 0,
            stroke_color: color,
            stroke_width: width,
        }
    }

    pub fn filled_stroke(fill_color: u32, stroke_color: u32, stroke_width: f32) -> Self {
        Self {
            fill_color,
            stroke_color,
            stroke_width,
        }
    }

    pub fn has_fill(self) -> bool {
        (self.fill_color & 0xff) != 0
    }

    pub fn has_stroke(self) -> bool {
        self.stroke_width > 0.0 && (self.stroke_color & 0xff) != 0
    }
}

struct PathResource {
    id: u32,
}

impl Drop for PathResource {
    fn drop(&mut self) {
        unsafe { ffi::fui_path_destroy(self.id) };
    }
}

#[derive(Clone)]
pub struct Path {
    resource: Rc<PathResource>,
}

impl Path {
    pub fn new() -> Self {
        let id = unsafe { ffi::fui_path_create() };
        Self {
            resource: Rc::new(PathResource { id }),
        }
    }

    pub fn id(&self) -> u32 {
        self.resource.id
    }

    pub fn move_to(&mut self, x: f32, y: f32) -> &mut Self {
        unsafe { ffi::fui_path_move_to(self.id(), x, y) };
        self
    }

    pub fn line_to(&mut self, x: f32, y: f32) -> &mut Self {
        unsafe { ffi::fui_path_line_to(self.id(), x, y) };
        self
    }

    pub fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) -> &mut Self {
        unsafe { ffi::fui_path_quad_to(self.id(), cx, cy, x, y) };
        self
    }

    pub fn cubic_to(
        &mut self,
        cx1: f32,
        cy1: f32,
        cx2: f32,
        cy2: f32,
        x: f32,
        y: f32,
    ) -> &mut Self {
        unsafe { ffi::fui_path_cubic_to(self.id(), cx1, cy1, cx2, cy2, x, y) };
        self
    }

    pub fn close(&mut self) -> &mut Self {
        unsafe { ffi::fui_path_close(self.id()) };
        self
    }

    pub fn add_rect(&mut self, x: f32, y: f32, w: f32, h: f32) -> &mut Self {
        unsafe { ffi::fui_path_add_rect(self.id(), x, y, w, h) };
        self
    }

    pub fn add_circle(&mut self, cx: f32, cy: f32, r: f32) -> &mut Self {
        unsafe { ffi::fui_path_add_circle(self.id(), cx, cy, r) };
        self
    }
}

#[derive(Default)]
struct DrawContextState {
    canvas_ptr: usize,
    words: Vec<u32>,
    retained_paths: Vec<Path>,
}

#[derive(Clone, Default)]
pub struct DrawContext {
    inner: Rc<RefCell<DrawContextState>>,
}

impl DrawContext {
    pub fn new(canvas_ptr: usize) -> Self {
        Self {
            inner: Rc::new(RefCell::new(DrawContextState {
                canvas_ptr,
                words: Vec::new(),
                retained_paths: Vec::new(),
            })),
        }
    }

    fn push_float(words: &mut Vec<u32>, value: f32) {
        words.push(value.to_bits());
    }

    pub fn flush(&self) {
        let mut state = self.inner.borrow_mut();
        if state.words.is_empty() {
            return;
        }
        unsafe {
            ffi::fui_canvas_draw_batch(
                state.canvas_ptr,
                state.words.as_ptr() as usize,
                state.words.len() as u32,
            )
        };
        state.words.clear();
        state.retained_paths.clear();
    }

    pub fn save(&self) {
        self.inner.borrow_mut().words.push(OP_SAVE);
    }

    pub fn restore(&self) {
        self.inner.borrow_mut().words.push(OP_RESTORE);
    }

    pub fn translate(&self, x: f32, y: f32) {
        let mut state = self.inner.borrow_mut();
        state.words.push(OP_TRANSLATE);
        Self::push_float(&mut state.words, x);
        Self::push_float(&mut state.words, y);
    }

    pub fn scale(&self, sx: f32, sy: f32) {
        let mut state = self.inner.borrow_mut();
        state.words.push(OP_SCALE);
        Self::push_float(&mut state.words, sx);
        Self::push_float(&mut state.words, sy);
    }

    pub fn rotate(&self, degrees: f32) {
        let mut state = self.inner.borrow_mut();
        state.words.push(OP_ROTATE);
        Self::push_float(&mut state.words, degrees);
    }

    pub fn clip_rect(&self, x: f32, y: f32, w: f32, h: f32) {
        let mut state = self.inner.borrow_mut();
        state.words.push(OP_CLIP_RECT);
        Self::push_float(&mut state.words, x);
        Self::push_float(&mut state.words, y);
        Self::push_float(&mut state.words, w);
        Self::push_float(&mut state.words, h);
    }

    pub fn clip_round_rect(
        &self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        tl: f32,
        tr: f32,
        br: f32,
        bl: f32,
    ) {
        let mut state = self.inner.borrow_mut();
        state.words.push(OP_CLIP_ROUND_RECT);
        Self::push_float(&mut state.words, x);
        Self::push_float(&mut state.words, y);
        Self::push_float(&mut state.words, w);
        Self::push_float(&mut state.words, h);
        Self::push_float(&mut state.words, tl);
        Self::push_float(&mut state.words, tr);
        Self::push_float(&mut state.words, br);
        Self::push_float(&mut state.words, bl);
    }

    pub fn draw_rect(&self, x: f32, y: f32, w: f32, h: f32, paint: Paint) {
        let mut state = self.inner.borrow_mut();
        state.words.push(OP_DRAW_RECT);
        Self::push_float(&mut state.words, x);
        Self::push_float(&mut state.words, y);
        Self::push_float(&mut state.words, w);
        Self::push_float(&mut state.words, h);
        state.words.push(paint.fill_color);
        state.words.push(paint.stroke_color);
        Self::push_float(&mut state.words, paint.stroke_width);
    }

    pub fn draw_circle(&self, cx: f32, cy: f32, radius: f32, paint: Paint) {
        let mut state = self.inner.borrow_mut();
        state.words.push(OP_DRAW_CIRCLE);
        Self::push_float(&mut state.words, cx);
        Self::push_float(&mut state.words, cy);
        Self::push_float(&mut state.words, radius);
        state.words.push(paint.fill_color);
        state.words.push(paint.stroke_color);
        Self::push_float(&mut state.words, paint.stroke_width);
    }

    pub fn draw_line(&self, x1: f32, y1: f32, x2: f32, y2: f32, color: u32, stroke_width: f32) {
        let mut state = self.inner.borrow_mut();
        state.words.push(OP_DRAW_LINE);
        Self::push_float(&mut state.words, x1);
        Self::push_float(&mut state.words, y1);
        Self::push_float(&mut state.words, x2);
        Self::push_float(&mut state.words, y2);
        state.words.push(color);
        Self::push_float(&mut state.words, stroke_width);
    }

    pub fn draw_round_rect(&self, x: f32, y: f32, w: f32, h: f32, rx: f32, ry: f32, paint: Paint) {
        let mut state = self.inner.borrow_mut();
        state.words.push(OP_DRAW_ROUND_RECT);
        Self::push_float(&mut state.words, x);
        Self::push_float(&mut state.words, y);
        Self::push_float(&mut state.words, w);
        Self::push_float(&mut state.words, h);
        Self::push_float(&mut state.words, rx);
        Self::push_float(&mut state.words, ry);
        state.words.push(paint.fill_color);
        state.words.push(paint.stroke_color);
        Self::push_float(&mut state.words, paint.stroke_width);
    }

    pub fn draw_path(&self, path: &Path, paint: Paint) {
        let mut state = self.inner.borrow_mut();
        state.words.push(OP_DRAW_PATH);
        state.words.push(path.id());
        state.words.push(paint.fill_color);
        state.words.push(paint.stroke_color);
        Self::push_float(&mut state.words, paint.stroke_width);
        state.retained_paths.push(path.clone());
    }

    pub fn draw_text_node<T: Node>(&self, node: &T, x: f32, y: f32) {
        let handle = node.handle().raw();
        let mut state = self.inner.borrow_mut();
        state.words.push(OP_DRAW_TEXT_NODE);
        state.words.push(handle as u32);
        state.words.push((handle >> 32) as u32);
        Self::push_float(&mut state.words, x);
        Self::push_float(&mut state.words, y);
    }

    pub fn draw_text_layout(&self, layout: &TextLayout, x: f32, y: f32) {
        if !layout.is_ready() {
            error(
                "TextLayout",
                "DrawContext.draw_text_layout() called before the TextLayout was ready; register on_ready and draw after the callback.",
            );
            return;
        }
        let node = layout.draw_node();
        self.draw_text_node(&node, x, y);
    }

    pub fn draw_image(&self, texture_id: u32, x: f32, y: f32, w: f32, h: f32) {
        self.draw_image_sampling(texture_id, x, y, w, h, ImageSampling::linear());
    }

    pub fn draw_image_sampling(
        &self,
        texture_id: u32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        sampling: ImageSampling,
    ) {
        let mut state = self.inner.borrow_mut();
        state.words.push(OP_DRAW_IMAGE);
        state.words.push(texture_id);
        Self::push_float(&mut state.words, x);
        Self::push_float(&mut state.words, y);
        Self::push_float(&mut state.words, w);
        Self::push_float(&mut state.words, h);
        state.words.push(sampling.ffi_kind() as u32);
        state.words.push(sampling.max_aniso());
    }

    pub fn draw_svg(&self, svg_id: u32, x: f32, y: f32, w: f32, h: f32) {
        let mut state = self.inner.borrow_mut();
        state.words.push(OP_DRAW_SVG);
        state.words.push(svg_id);
        Self::push_float(&mut state.words, x);
        Self::push_float(&mut state.words, y);
        Self::push_float(&mut state.words, w);
        Self::push_float(&mut state.words, h);
    }
}

#[cfg(test)]
mod tests {
    use super::{DrawContext, Paint, Path};
    use crate::ffi::{self, Call};
    use crate::image_sampling::ImageSampling;

    #[test]
    fn path_and_draw_context_emit_batched_host_calls() {
        ffi::test::reset();

        let mut path = Path::new();
        path.move_to(1.0, 2.0)
            .line_to(3.0, 4.0)
            .add_circle(8.0, 9.0, 10.0);

        let ctx = DrawContext::new(77);
        ctx.draw_rect(0.0, 1.0, 20.0, 30.0, Paint::fill(0xFF00FFFF));
        ctx.draw_path(&path, Paint::stroke(0xFFFFFFFF, 2.0));
        ctx.draw_image_sampling(9, 0.0, 0.0, 40.0, 50.0, ImageSampling::linear());
        ctx.flush();

        let calls = ffi::test::take_calls();
        assert!(calls
            .iter()
            .any(|call| matches!(call, Call::PathCreate { .. })));
        assert!(calls.iter().any(|call| matches!(
            call,
            Call::PathMoveTo { x, y, .. } if (*x - 1.0).abs() < f32::EPSILON && (*y - 2.0).abs() < f32::EPSILON
        )));
        assert!(calls.iter().any(|call| matches!(
            call,
            Call::CanvasDrawBatch { canvas_ptr: 77, words } if !words.is_empty()
        )));
    }
}
