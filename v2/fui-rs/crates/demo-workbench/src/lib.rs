#[cfg(feature = "web-route")]
mod generated;

use fui::prelude::*;
#[cfg(feature = "web-route")]
use fui_rs_demo_shared::Stage4WorkerTestApi;
use fui_rs_demo_shared::{clear_demo_shared_state, Stage4Showcase};
use fui_rs_demo_universal::{DemoEnvironment, DemoPageId, UniversalDemoPage};
#[cfg(feature = "web-route")]
use std::cell::RefCell;
use std::rc::Rc;

#[cfg(feature = "web-route")]
thread_local! {
    static WORKER_DETAIL_TEXT_BUFFER: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

#[cfg(feature = "web-route")]
fn with_worker_api<T>(callback: impl FnOnce(&Stage4WorkerTestApi) -> T) -> Option<T> {
    __fui_rs_with_app(|app| {
        app.get_active_page()
            .as_deref()
            .and_then(|shell| shell.content().state::<Stage4WorkerTestApi>())
            .map(callback)
    })
}

#[cfg(feature = "web-route")]
fn worker_status_code(value: &str) -> i32 {
    if value.starts_with("Stage 4 worker status: complete") {
        2
    } else if value.starts_with("Stage 4 worker status: error") {
        3
    } else if value.starts_with("Stage 4 worker status: cancelled") {
        4
    } else if value.starts_with("Stage 4 worker status: cancelling") {
        5
    } else if value.starts_with("Stage 4 worker status: running") {
        1
    } else {
        0
    }
}

fn build_workbench_page() -> Stage4Showcase {
    Stage4Showcase::new(
        "FUI-RS workbench",
        Rc::new(|_is_wide| {}),
        Rc::new(|_accent| {}),
        Rc::new(|_opacity| {}),
        Rc::new(|| {}),
    )
}

pub fn build_universal_page(_environment: &DemoEnvironment) -> UniversalDemoPage {
    let page = build_workbench_page();
    let root = page.root.clone();
    let worker_test_api = page.worker_test_api.clone();
    UniversalDemoPage::new(
        DemoPageId::TextAndFonts.metadata(),
        retained_view(&root)
            .keep_alive(page)
            .on_dispose(clear_demo_shared_state),
    )
    .with_state(worker_test_api)
}

#[cfg(feature = "web-route")]
fn web_environment() -> DemoEnvironment {
    DemoEnvironment::browser(
        fui::platform::platform_family(),
        fui_rs_demo_universal::DemoLinks::new(
            "https://github.com/zion-sati/fui-rs",
            "https://docs.rs/fui-rs/latest/fui",
        ),
    )
}

#[cfg(feature = "web-route")]
fn build_web_page() -> fui_rs_demo_shared::RoutedDemoShell<UniversalDemoPage> {
    Application::caption(DemoPageId::TextAndFonts.metadata().title);
    let page = build_universal_page(&web_environment());
    let root = page.view().root();
    fui_rs_demo_shared::routed_demo_shell(page, root, DemoPageId::TextAndFonts)
}

#[cfg(feature = "web-route")]
fn dispose_workbench_page(page: &fui_rs_demo_shared::RoutedDemoShell<UniversalDemoPage>) {
    page.content().view().dispose();
    WORKER_DETAIL_TEXT_BUFFER.with(|buffer| {
        buffer.borrow_mut().clear();
    });
}

#[cfg(feature = "web-route")]
fui_managed_app!(
    fui_rs_demo_shared::RoutedDemoShell<UniversalDemoPage>,
    build_web_page,
    |page: &fui_rs_demo_shared::RoutedDemoShell<UniversalDemoPage>| page.root.clone(),
    dispose: dispose_workbench_page
);

#[cfg(feature = "web-route")]
#[no_mangle]
pub extern "C" fn __startWorkerDemo() {
    let _ = with_worker_api(|worker_test_api| {
        (worker_test_api.start_prime)();
    });
}

#[cfg(feature = "web-route")]
#[no_mangle]
pub extern "C" fn __startFailingWorkerDemo() {
    let _ = with_worker_api(|worker_test_api| {
        (worker_test_api.start_fail)();
    });
}

#[cfg(feature = "web-route")]
#[no_mangle]
pub extern "C" fn __getWorkerDemoStatusCode() -> i32 {
    with_worker_api(|worker_test_api| worker_status_code(&(worker_test_api.status)())).unwrap_or(0)
}

#[cfg(feature = "web-route")]
#[no_mangle]
pub extern "C" fn __workerDemoDetailHasPrimeAndClock() -> bool {
    with_worker_api(|worker_test_api| {
        let detail = (worker_test_api.detail)();
        detail.contains("Stage 4 worker detail: complete • prime=") && detail.contains(" clock=")
    })
    .unwrap_or(false)
}

#[cfg(feature = "web-route")]
#[no_mangle]
pub extern "C" fn __workerDemoDetailHasErrorClock() -> bool {
    with_worker_api(|worker_test_api| {
        (worker_test_api.detail)().contains("Stage 4 worker detail: error • worker failure clock=")
    })
    .unwrap_or(false)
}

#[cfg(feature = "web-route")]
#[no_mangle]
pub extern "C" fn __getWorkerDemoDetailTextPtr() -> usize {
    let detail = with_worker_api(|worker_test_api| (worker_test_api.detail)()).unwrap_or_default();
    WORKER_DETAIL_TEXT_BUFFER.with(|buffer| {
        let mut bytes = buffer.borrow_mut();
        bytes.clear();
        bytes.extend_from_slice(detail.as_bytes());
        if bytes.is_empty() {
            0
        } else {
            bytes.as_ptr() as usize
        }
    })
}

#[cfg(feature = "web-route")]
#[no_mangle]
pub extern "C" fn __getWorkerDemoDetailTextLength() -> u32 {
    WORKER_DETAIL_TEXT_BUFFER.with(|buffer| buffer.borrow().len() as u32)
}
