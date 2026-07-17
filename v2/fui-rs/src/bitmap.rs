use crate::assets;
use crate::drawing::DrawContext;
use crate::ffi;
use crate::frame_scheduler::on_loaded;
use crate::logger::error;
use crate::node::Node;
use crate::text::TextLayout;
use crate::typography::FontFace;
use std::cell::RefCell;
use std::rc::Rc;

const MAX_DIRTY_RECTS: usize = 16;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BitmapTextReadyEventArgs;

impl BitmapTextReadyEventArgs {
    pub const EMPTY: Self = Self;
}

struct BitmapState {
    width: u32,
    height: u32,
    texture_id: u32,
    pixel_bytes: Vec<u8>,
    offscreen_id: u32,
    canvas_used: bool,
    draw_context: Option<DrawContext>,
    disposed: bool,
    dirty_rects: Vec<(u32, u32, u32, u32)>,
}

#[derive(Clone)]
pub struct Bitmap {
    inner: Rc<RefCell<BitmapState>>,
}

impl Bitmap {
    pub fn new(width: u32, height: u32) -> Self {
        assert!(
            width > 0 && height > 0,
            "Bitmap width and height must be greater than zero."
        );
        let byte_len = (width as usize)
            .checked_mul(height as usize)
            .and_then(|value| value.checked_mul(4))
            .expect("Bitmap byte length overflow.");
        let texture_id = assets::allocate_dynamic_texture_id();
        let offscreen_id = unsafe { ffi::fui_canvas_create_offscreen(width, height) };
        Self {
            inner: Rc::new(RefCell::new(BitmapState {
                width,
                height,
                texture_id,
                pixel_bytes: vec![0; byte_len],
                offscreen_id,
                canvas_used: false,
                draw_context: None,
                disposed: false,
                dirty_rects: Vec::new(),
            })),
        }
    }

    pub fn width(&self) -> u32 {
        self.inner.borrow().width
    }

    pub fn height(&self) -> u32 {
        self.inner.borrow().height
    }

    pub fn texture_id(&self) -> u32 {
        self.inner.borrow().texture_id
    }

    pub fn pixels(&self) -> std::cell::RefMut<'_, Vec<u8>> {
        std::cell::RefMut::map(self.inner.borrow_mut(), |state| {
            assert!(!state.disposed, "Bitmap.pixels() called after dispose.");
            &mut state.pixel_bytes
        })
    }

    pub fn pixel_ptr(&self) -> usize {
        let state = self.inner.borrow();
        if state.pixel_bytes.is_empty() {
            0
        } else {
            state.pixel_bytes.as_ptr() as usize
        }
    }

    pub fn canvas(&self) -> DrawContext {
        let mut state = self.inner.borrow_mut();
        assert!(!state.disposed, "Bitmap.canvas() called after dispose.");
        state.canvas_used = true;
        if let Some(context) = &state.draw_context {
            return context.clone();
        }
        let ptr = unsafe { ffi::fui_canvas_get_offscreen_ptr(state.offscreen_id) };
        let context = DrawContext::new(ptr);
        state.draw_context = Some(context.clone());
        context
    }

    pub fn render<T: Node>(&self, node: &T, x: f32, y: f32, scale: f32) {
        let state = self.inner.borrow();
        assert!(!state.disposed, "Bitmap.render() called after dispose.");
        let handle = node.handle().raw();
        if handle == 0 {
            return;
        }
        unsafe {
            ffi::fui_render_node_to_rgba(
                handle,
                state.width,
                state.height,
                state.pixel_bytes.as_ptr() as usize,
                state.pixel_bytes.len() as u32,
                scale,
                x,
                y,
            )
        };
    }

    pub fn render_text_layout(&self, layout: &TextLayout, x: f32, y: f32, scale: f32) {
        if !layout.is_ready() {
            error(
                "TextLayout",
                "Bitmap.render_text_layout() called before the TextLayout was ready; register on_ready and render after the callback.",
            );
            return;
        }
        let node = layout.draw_node();
        self.render(&node, x, y, scale);
    }

    pub fn prepare_text<T: Node>(node: &T) {
        node.build();
        crate::bindings::ui::prepare_node(node.handle().raw());
    }

    pub fn on_text_ready<T: Node + Clone + 'static>(
        &self,
        node: &T,
        callback: impl FnOnce(BitmapTextReadyEventArgs) + 'static,
    ) -> &Self {
        let node = node.clone();
        let required_font_ids = node.required_font_ids_for_preparation();
        let callback = Rc::new(RefCell::new(Some(callback)));
        FontFace::when_fonts_loaded(&required_font_ids, move |_| {
            let node = node.clone();
            let callback = callback.clone();
            on_loaded(move |_| {
                Self::prepare_text(&node);
                if let Some(callback) = callback.borrow_mut().take() {
                    callback(BitmapTextReadyEventArgs::EMPTY);
                }
            });
        });
        self
    }

    pub fn dirty_rect(&self, x: u32, y: u32, w: u32, h: u32) -> &Self {
        let mut state = self.inner.borrow_mut();
        if w == 0 || h == 0 || x >= state.width || y >= state.height {
            return self;
        }
        let cw = (x + w).min(state.width) - x;
        let ch = (y + h).min(state.height) - y;
        if state.dirty_rects.len() < MAX_DIRTY_RECTS {
            state.dirty_rects.push((x, y, cw, ch));
        }
        self
    }

    pub fn clear_dirty_rects(&self) -> &Self {
        self.inner.borrow_mut().dirty_rects.clear();
        self
    }

    pub fn has_dirty_rects(&self) -> bool {
        !self.inner.borrow().dirty_rects.is_empty()
    }

    pub fn commit(&self) -> u32 {
        let mut state = self.inner.borrow_mut();
        assert!(!state.disposed, "Bitmap.commit() called after dispose.");
        if state.canvas_used {
            if let Some(context) = &state.draw_context {
                context.flush();
            }
            unsafe {
                ffi::fui_canvas_read_offscreen_pixels(
                    state.offscreen_id,
                    state.pixel_bytes.as_mut_ptr() as usize,
                    state.width,
                    state.height,
                )
            };
        }
        if state.dirty_rects.is_empty() {
            unsafe {
                ffi::fui_bitmap_commit(
                    state.texture_id,
                    state.pixel_bytes.as_ptr() as usize,
                    state.pixel_bytes.len() as u32,
                    state.width,
                    state.height,
                )
            };
        } else {
            let dirty_rects = std::mem::take(&mut state.dirty_rects);
            for (x, y, w, h) in dirty_rects {
                let mut rect_bytes = vec![0u8; (w as usize) * (h as usize) * 4];
                for row in 0..h as usize {
                    let src = (((y as usize + row) * state.width as usize) + x as usize) * 4;
                    let dst = row * w as usize * 4;
                    let len = w as usize * 4;
                    rect_bytes[dst..dst + len].copy_from_slice(&state.pixel_bytes[src..src + len]);
                }
                unsafe {
                    ffi::fui_bitmap_commit_dirty(
                        state.texture_id,
                        rect_bytes.as_ptr() as usize,
                        rect_bytes.len() as u32,
                        state.width,
                        state.height,
                        x,
                        y,
                        w,
                        h,
                    )
                };
            }
        }
        assets::mark_texture_asset_ready(state.texture_id, state.width as f32, state.height as f32);
        state.texture_id
    }

    pub fn dispose(&self) {
        let mut state = self.inner.borrow_mut();
        if state.disposed {
            return;
        }
        state.disposed = true;
        unsafe { ffi::fui_canvas_destroy_offscreen(state.offscreen_id) };
        unsafe { ffi::fui_bitmap_release(state.texture_id) };
        state.pixel_bytes.clear();
        state.draw_context = None;
    }
}

impl Drop for Bitmap {
    fn drop(&mut self) {
        if Rc::strong_count(&self.inner) == 1 {
            self.dispose();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Bitmap;
    use crate::drawing::Paint;
    use crate::ffi::{self, Call};
    use crate::frame_scheduler;
    use crate::node::{Node, TextNode};
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn bitmap_canvas_commit_flushes_offscreen_and_marks_asset_ready() {
        ffi::test::reset();
        let bitmap = Bitmap::new(32, 24);
        let canvas = bitmap.canvas();
        canvas.draw_rect(0.0, 0.0, 10.0, 12.0, Paint::fill(0xFF00FFFF));
        bitmap.commit();

        let calls = ffi::test::take_calls();
        assert!(calls.iter().any(|call| matches!(
            call,
            Call::CanvasCreateOffscreen {
                width: 32,
                height: 24,
                ..
            }
        )));
        assert!(calls
            .iter()
            .any(|call| matches!(call, Call::CanvasDrawBatch { .. })));
        assert!(calls.iter().any(|call| matches!(
            call,
            Call::CanvasReadOffscreenPixels {
                width: 32,
                height: 24,
                ..
            }
        )));
        assert!(calls.iter().any(|call| matches!(
            call,
            Call::BitmapCommit {
                width: 32,
                height: 24,
                ..
            }
        )));
    }

    #[test]
    fn bitmap_dirty_commit_uses_subrect_upload() {
        ffi::test::reset();
        let bitmap = Bitmap::new(8, 6);
        bitmap.dirty_rect(2, 1, 3, 2).commit();

        let calls = ffi::test::take_calls();
        assert!(calls.iter().any(|call| matches!(
            call,
            Call::BitmapCommitDirty {
                full_width: 8,
                full_height: 6,
                sub_x: 2,
                sub_y: 1,
                sub_w: 3,
                sub_h: 2,
                ..
            }
        )));
    }

    #[test]
    fn bitmap_text_ready_builds_detached_text_before_preparing_it() {
        ffi::test::reset();
        frame_scheduler::reset_commit_state();
        let bitmap = Bitmap::new(64, 32);
        let text = TextNode::new("Detached bitmap text");
        let fired = Rc::new(Cell::new(false));
        bitmap.on_text_ready(&text, {
            let fired = fired.clone();
            move |_| fired.set(true)
        });

        frame_scheduler::fire_loaded_callbacks();

        assert!(fired.get());
        assert!(text.has_built_handle());
        let calls = ffi::test::take_calls();
        assert!(calls
            .iter()
            .any(|call| matches!(call, Call::PrepareNode { .. })));
    }
}
