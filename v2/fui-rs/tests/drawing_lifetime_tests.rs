use fui::ffi::{self, Call};
use fui::prelude::*;

#[test]
fn queued_draw_path_retains_resource_until_batch_flush() {
    ffi::test::reset();
    let context = DrawContext::new(77);
    let path_id;

    {
        let mut path = Path::new();
        path.move_to(0.0, 0.0).line_to(10.0, 10.0);
        path_id = path.id();
        context.draw_path(&path, Paint::stroke(0xFFFFFFFF, 2.0));
    }

    let queued_calls = ffi::test::take_calls();
    assert!(!queued_calls
        .iter()
        .any(|call| matches!(call, Call::PathDestroy { path_id: id } if *id == path_id)));

    context.flush();
    let flushed_calls = ffi::test::take_calls();
    assert!(flushed_calls
        .iter()
        .any(|call| matches!(call, Call::CanvasDrawBatch { canvas_ptr: 77, .. })));
    assert!(flushed_calls
        .iter()
        .any(|call| matches!(call, Call::PathDestroy { path_id: id } if *id == path_id)));
}
