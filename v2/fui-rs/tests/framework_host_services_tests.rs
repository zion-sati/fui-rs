use fui::ffi;
use fui::generated::framework_host_services;

#[test]
fn framework_host_services_read_non_wasm_test_state() {
    ffi::test::reset();
    ffi::test::set_host_now_ms(1234.5);
    ffi::test::set_system_dark_mode(false);
    ffi::test::set_system_accent_color(0xAABBCCDD);
    ffi::test::set_platform_family(2);
    ffi::test::set_host_environment(1);
    ffi::test::set_host_capabilities(0x7f);
    ffi::test::set_coarse_pointer(true);

    assert_eq!(framework_host_services::fui_now_ms(), 1234.5);
    assert!(!framework_host_services::fui_is_dark_mode());
    assert_eq!(framework_host_services::fui_get_accent_color(), 0xAABBCCDD);
    assert_eq!(framework_host_services::fui_get_platform_family(), 2);
    assert_eq!(framework_host_services::fui_get_host_environment(), 1);
    assert_eq!(framework_host_services::fui_get_host_capabilities(), 0x7f);
    assert!(framework_host_services::fui_is_coarse_pointer());
}
