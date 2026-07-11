use fui::bridge_callbacks::{
    self, current_route, is_context_menu_visible, last_context_menu_request, last_font_loaded,
    last_scroll_event, last_svg_failed, last_svg_loaded, last_texture_failed, last_texture_loaded,
    persisted_capture_count, persisted_restore_count, AssetFailure, AssetReady, ContextMenuRequest,
    ScrollEvent,
};
use fui::ffi::{self, Call};
use fui::prelude::*;

#[test]
fn viewport_callback_resizes_ui() {
    ffi::test::reset();
    bridge_callbacks::__fui_on_viewport_changed(640.0, 480.0);
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(call, Call::ResizeWindow { logical_w, logical_h } if *logical_w == 640.0 && *logical_h == 480.0)));
}

#[test]
fn route_callback_reads_pointer_length_payload() {
    let route = "/demo?tab=stage4";
    bridge_callbacks::__fui_on_route_changed(route.as_ptr(), route.len() as u32);
    assert_eq!(current_route(), route);
}

#[test]
fn scroll_callback_records_bridge_payload() {
    bridge_callbacks::__fui_on_scroll(42, 1.0, 2.0, 300.0, 400.0, 100.0, 200.0);
    assert_eq!(
        last_scroll_event(),
        Some(ScrollEvent {
            handle: 42,
            offset_x: 1.0,
            offset_y: 2.0,
            content_width: 300.0,
            content_height: 400.0,
            viewport_width: 100.0,
            viewport_height: 200.0,
        }),
    );
}

#[test]
fn context_menu_callbacks_record_and_hide_request() {
    ffi::test::reset();
    let target = text("Context target");
    Application::mount(target.clone());
    let handle = target.handle().raw();

    assert!(bridge_callbacks::__fui_can_show_context_menu(
        fui::ffi::HandleValue::Invalid as u64
    ));
    assert!(bridge_callbacks::__fui_can_show_context_menu(handle));
    bridge_callbacks::__fui_on_context_menu(handle, 12.0, 34.0);
    assert!(is_context_menu_visible());
    assert_eq!(
        last_context_menu_request(),
        Some(ContextMenuRequest {
            handle,
            x: 12.0,
            y: 34.0
        })
    );
    bridge_callbacks::__fui_hide_active_context_menu();
    assert!(!is_context_menu_visible());

    let invalid = fui::ffi::HandleValue::Invalid as u64;
    bridge_callbacks::__fui_on_context_menu(invalid, 56.0, 78.0);
    assert!(is_context_menu_visible());
    assert_eq!(
        last_context_menu_request(),
        Some(ContextMenuRequest {
            handle: invalid,
            x: 56.0,
            y: 78.0
        })
    );
}

#[test]
fn custom_context_menu_handler_without_menu_does_not_mark_bridge_menu_visible() {
    ffi::test::reset();
    let invoked = std::rc::Rc::new(std::cell::Cell::new(0));
    let invoked_clone = invoked.clone();
    let target = text("Custom context target");
    target.on_context_menu(move |_event| {
        invoked_clone.set(invoked_clone.get() + 1);
    });
    Application::mount(target.clone());

    bridge_callbacks::__fui_on_context_menu(target.handle().raw(), 12.0, 34.0);
    assert_eq!(invoked.get(), 1);
    assert!(!is_context_menu_visible());
}

#[test]
fn asset_callbacks_record_success_and_failure_payloads() {
    bridge_callbacks::__fui_on_font_loaded(5);
    bridge_callbacks::__fui_on_svg_loaded(6, 24.0, 32.0);
    bridge_callbacks::__fui_on_texture_loaded(7, 48.0, 64.0);
    let svg_error = "bad svg";
    let texture_error = "bad texture";
    bridge_callbacks::__fui_on_svg_failed(8, svg_error.as_ptr(), svg_error.len() as u32);
    bridge_callbacks::__fui_on_texture_failed(
        9,
        texture_error.as_ptr(),
        texture_error.len() as u32,
    );

    assert_eq!(last_font_loaded(), Some(5));
    assert_eq!(
        last_svg_loaded(),
        Some(AssetReady {
            id: 6,
            width: 24.0,
            height: 32.0
        })
    );
    assert_eq!(
        last_texture_loaded(),
        Some(AssetReady {
            id: 7,
            width: 48.0,
            height: 64.0
        })
    );
    assert_eq!(
        last_svg_failed(),
        Some(AssetFailure {
            id: 8,
            error: svg_error.to_string()
        })
    );
    assert_eq!(
        last_texture_failed(),
        Some(AssetFailure {
            id: 9,
            error: texture_error.to_string()
        })
    );
}

#[test]
fn persisted_callbacks_are_callable() {
    let capture_before = persisted_capture_count();
    let restore_before = persisted_restore_count();
    bridge_callbacks::__fui_capture_persisted_ui_state();
    bridge_callbacks::__fui_restore_persisted_ui_state();
    assert_eq!(persisted_capture_count(), capture_before + 1);
    assert_eq!(persisted_restore_count(), restore_before + 1);
}
