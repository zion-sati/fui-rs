mod generated;

use fui::prelude::*;
use fui_rs_demo_shared::{clear_demo_shared_state, Stage4Showcase};
use std::cell::RefCell;
use std::rc::Rc;

thread_local! {
    static WORKER_DETAIL_TEXT_BUFFER: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

fn with_showcase<T>(callback: impl FnOnce(&Stage4Showcase) -> T) -> Option<T> {
    __fui_rs_with_app(|app| app.get_active_page().as_deref().map(callback))
}

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
    Application::caption("EffinDOM FUI-RS Demo • Workbench");
    Stage4Showcase::new(
        "FUI-RS workbench",
        Rc::new(|_is_wide| {}),
        Rc::new(|_accent| {}),
        Rc::new(|_opacity| {}),
        Rc::new(|| {}),
    )
}

fn dispose_workbench_page(_: &Stage4Showcase) {
    clear_demo_shared_state();
    WORKER_DETAIL_TEXT_BUFFER.with(|buffer| {
        buffer.borrow_mut().clear();
    });
}

fui_managed_app!(
    Stage4Showcase,
    build_workbench_page,
    |page: &Stage4Showcase| page.root.clone(),
    dispose: dispose_workbench_page
);

#[no_mangle]
pub extern "C" fn __startWorkerDemo() {
    let _ = with_showcase(|showcase| {
        (showcase.worker_test_api.start_prime)();
    });
}

#[no_mangle]
pub extern "C" fn __startFailingWorkerDemo() {
    let _ = with_showcase(|showcase| {
        (showcase.worker_test_api.start_fail)();
    });
}

#[no_mangle]
pub extern "C" fn __getWorkerDemoStatusCode() -> i32 {
    with_showcase(|showcase| worker_status_code(&(showcase.worker_test_api.status)())).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn __workerDemoDetailHasPrimeAndClock() -> bool {
    with_showcase(|showcase| {
        let detail = (showcase.worker_test_api.detail)();
        detail.contains("Stage 4 worker detail: complete • prime=") && detail.contains(" clock=")
    })
    .unwrap_or(false)
}

#[no_mangle]
pub extern "C" fn __workerDemoDetailHasErrorClock() -> bool {
    with_showcase(|showcase| {
        (showcase.worker_test_api.detail)()
            .contains("Stage 4 worker detail: error • worker failure clock=")
    })
    .unwrap_or(false)
}

#[no_mangle]
pub extern "C" fn __getWorkerDemoDetailTextPtr() -> usize {
    let detail = with_showcase(|showcase| (showcase.worker_test_api.detail)()).unwrap_or_default();
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

#[no_mangle]
pub extern "C" fn __getWorkerDemoDetailTextLength() -> u32 {
    WORKER_DETAIL_TEXT_BUFFER.with(|buffer| buffer.borrow().len() as u32)
}
