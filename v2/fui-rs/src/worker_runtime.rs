use std::cell::{Cell, RefCell};
thread_local! {
    static WORKER_TERMINAL_SENT: Cell<bool> = const { Cell::new(false) };
    static WORKER_CALLBACK_BUFFER: RefCell<Box<[u8]>> =
        RefCell::new(vec![0u8; 1024 * 1024].into_boxed_slice());
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "fui_worker_host")]
unsafe extern "C" {
    #[link_name = "fui_worker_report_progress"]
    fn host_worker_report_progress(ptr: usize, len: u32);
    #[link_name = "fui_worker_complete_string"]
    fn host_worker_complete_string(ptr: usize, len: u32);
    #[link_name = "fui_worker_fail"]
    fn host_worker_fail(ptr: usize, len: u32);
    #[link_name = "fui_worker_is_cancelled"]
    fn host_worker_is_cancelled() -> bool;
    #[link_name = "fui_worker_request_yield"]
    fn host_worker_request_yield();
    #[link_name = "fui_worker_request_yield_delay"]
    fn host_worker_request_yield_delay(delay_ms: i32);
    #[link_name = "fui_file_read_chunk"]
    fn host_file_read_chunk(offset_low: i32, offset_high: i32, max_bytes: i32) -> i32;
    #[link_name = "fui_file_worker_write_chunk"]
    fn host_file_worker_write_chunk(ptr: usize, len: i32);
}

#[cfg(not(target_arch = "wasm32"))]
unsafe extern "C" {
    #[link_name = "fui_native_worker_report_progress"]
    fn host_native_worker_report_progress(ptr: *const u8, len: u32);
    #[link_name = "fui_native_worker_complete_string"]
    fn host_native_worker_complete_string(ptr: *const u8, len: u32);
    #[link_name = "fui_native_worker_fail"]
    fn host_native_worker_fail(ptr: *const u8, len: u32);
    #[link_name = "fui_native_worker_is_cancelled"]
    fn host_native_worker_is_cancelled() -> bool;
    #[link_name = "fui_native_worker_request_yield"]
    fn host_native_worker_request_yield(delay_ms: i32);
}

fn with_utf8(value: &str, callback: impl FnOnce(usize, u32)) {
    let bytes = value.as_bytes();
    callback(
        if bytes.is_empty() {
            0
        } else {
            bytes.as_ptr() as usize
        },
        bytes.len() as u32,
    );
}

pub struct WorkerRuntime;

impl WorkerRuntime {
    /// # Safety
    /// `input_ptr` must reference at least `input_len` readable bytes when `input_len` is non-zero.
    pub unsafe fn entry_input(input_ptr: usize, input_len: u32) -> String {
        if input_ptr == 0 || input_len == 0 {
            return String::new();
        }
        let bytes =
            unsafe { std::slice::from_raw_parts(input_ptr as *const u8, input_len as usize) };
        String::from_utf8_lossy(bytes).into_owned()
    }

    pub fn report_progress(progress: impl AsRef<str>) {
        if WORKER_TERMINAL_SENT.with(Cell::get) {
            return;
        }
        #[cfg(target_arch = "wasm32")]
        with_utf8(progress.as_ref(), |ptr, len| unsafe {
            host_worker_report_progress(ptr, len);
        });
        #[cfg(not(target_arch = "wasm32"))]
        with_utf8(progress.as_ref(), |ptr, len| unsafe {
            host_native_worker_report_progress(ptr as *const u8, len);
        });
    }

    pub fn complete(result: impl AsRef<str>) {
        if WORKER_TERMINAL_SENT.with(Cell::get) {
            return;
        }
        WORKER_TERMINAL_SENT.with(|sent| sent.set(true));
        #[cfg(target_arch = "wasm32")]
        with_utf8(result.as_ref(), |ptr, len| unsafe {
            host_worker_complete_string(ptr, len);
        });
        #[cfg(not(target_arch = "wasm32"))]
        with_utf8(result.as_ref(), |ptr, len| unsafe {
            host_native_worker_complete_string(ptr as *const u8, len);
        });
    }

    pub fn fail(message: impl AsRef<str>) {
        if WORKER_TERMINAL_SENT.with(Cell::get) {
            return;
        }
        WORKER_TERMINAL_SENT.with(|sent| sent.set(true));
        #[cfg(target_arch = "wasm32")]
        with_utf8(message.as_ref(), |ptr, len| unsafe {
            host_worker_fail(ptr, len);
        });
        #[cfg(not(target_arch = "wasm32"))]
        with_utf8(message.as_ref(), |ptr, len| unsafe {
            host_native_worker_fail(ptr as *const u8, len);
        });
    }

    pub fn is_cancelled() -> bool {
        #[cfg(target_arch = "wasm32")]
        {
            unsafe { host_worker_is_cancelled() }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            unsafe { host_native_worker_is_cancelled() }
        }
    }

    pub fn r#yield(delay_ms: i32) -> bool {
        if WORKER_TERMINAL_SENT.with(Cell::get) {
            return false;
        }
        #[cfg(target_arch = "wasm32")]
        unsafe {
            if delay_ms > 0 {
                host_worker_request_yield_delay(delay_ms);
            } else {
                host_worker_request_yield();
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        unsafe {
            host_native_worker_request_yield(delay_ms.max(0));
        }
        true
    }

    pub fn yield_now(delay_ms: i32) -> bool {
        Self::r#yield(delay_ms)
    }
}

pub fn file_read_chunk(offset_low: i32, offset_high: i32, max_bytes: i32) -> i32 {
    #[cfg(target_arch = "wasm32")]
    {
        unsafe { host_file_read_chunk(offset_low, offset_high, max_bytes) }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (offset_low, offset_high, max_bytes);
        0
    }
}

pub fn file_worker_write_chunk(ptr: usize, len: i32) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        host_file_worker_write_chunk(ptr, len);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (ptr, len);
    }
}

pub fn worker_text_buffer_ptr() -> usize {
    WORKER_CALLBACK_BUFFER.with(|buffer| buffer.borrow().as_ptr() as usize)
}

pub fn worker_text_buffer_size() -> u32 {
    WORKER_CALLBACK_BUFFER.with(|buffer| buffer.borrow().len() as u32)
}

pub fn reset_worker_runtime() {
    WORKER_TERMINAL_SENT.with(|sent| sent.set(false));
}

#[cfg(test)]
mod tests {
    use super::{reset_worker_runtime, WorkerRuntime};
    use crate::{fui_worker, WorkerJob, WorkerJobState};
    use std::cell::{Cell, RefCell};

    thread_local! {
        static START_INPUT: RefCell<String> = const { RefCell::new(String::new()) };
        static RUN_COUNT: Cell<u32> = const { Cell::new(0) };
        static HOST_EVENTS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
        static HOST_CANCELLED: Cell<bool> = const { Cell::new(false) };
    }

    unsafe fn host_text(ptr: *const u8, len: u32) -> String {
        if ptr.is_null() || len == 0 {
            return String::new();
        }
        String::from_utf8_lossy(unsafe { std::slice::from_raw_parts(ptr, len as usize) })
            .into_owned()
    }

    fn record(event: impl Into<String>) {
        HOST_EVENTS.with(|events| events.borrow_mut().push(event.into()));
    }

    fn reset_host() {
        START_INPUT.with(|value| value.borrow_mut().clear());
        RUN_COUNT.with(|value| value.set(0));
        HOST_EVENTS.with(|events| events.borrow_mut().clear());
        HOST_CANCELLED.with(|cancelled| cancelled.set(false));
        reset_worker_runtime();
    }

    #[no_mangle]
    extern "C" fn fui_native_worker_report_progress(ptr: *const u8, len: u32) {
        record(format!("progress:{}", unsafe { host_text(ptr, len) }));
    }

    #[no_mangle]
    extern "C" fn fui_native_worker_complete_string(ptr: *const u8, len: u32) {
        record(format!("complete:{}", unsafe { host_text(ptr, len) }));
    }

    #[no_mangle]
    extern "C" fn fui_native_worker_fail(ptr: *const u8, len: u32) {
        record(format!("error:{}", unsafe { host_text(ptr, len) }));
    }

    #[no_mangle]
    extern "C" fn fui_native_worker_is_cancelled() -> bool {
        HOST_CANCELLED.with(Cell::get)
    }

    #[no_mangle]
    extern "C" fn fui_native_worker_request_yield(delay_ms: i32) {
        record(format!("yield:{delay_ms}"));
    }

    #[derive(Default)]
    struct TestJob {
        state: WorkerJobState,
    }

    impl WorkerJob for TestJob {
        fn state(&mut self) -> &mut WorkerJobState {
            &mut self.state
        }

        fn on_start(&mut self, input: String) {
            START_INPUT.with(|value| value.replace(input));
        }

        fn run(&mut self) {
            let run_count = RUN_COUNT.with(|value| {
                let next = value.get() + 1;
                value.set(next);
                next
            });
            if run_count == 1 {
                self.report_progress("halfway ✅");
                self.r#yield(-10);
            } else {
                self.complete("complete");
            }
        }
    }

    #[derive(Default)]
    struct FailedJob {
        state: WorkerJobState,
    }

    impl WorkerJob for FailedJob {
        fn state(&mut self) -> &mut WorkerJobState {
            &mut self.state
        }

        fn run(&mut self) {
            self.fail("failed ❌");
        }
    }

    #[derive(Default)]
    struct PanickingJob {
        state: WorkerJobState,
    }

    impl WorkerJob for PanickingJob {
        fn state(&mut self) -> &mut WorkerJobState {
            &mut self.state
        }

        fn run(&mut self) {
            panic!("native worker panic");
        }
    }

    fui_worker!(
        test_worker_entry => TestJob,
        failed_worker_entry => FailedJob,
        panicking_worker_entry => PanickingJob,
    );

    #[test]
    fn native_worker_entry_preserves_utf8_state_progress_yield_and_completion() {
        reset_host();
        let input = "start 🌍";

        unsafe {
            test_worker_entry(input.as_ptr() as usize, input.len() as u32);
            test_worker_entry(0, 0);
        }

        assert_eq!(START_INPUT.with(|value| value.borrow().clone()), input);
        assert_eq!(RUN_COUNT.with(Cell::get), 2);
        assert_eq!(
            HOST_EVENTS.with(|events| events.borrow().clone()),
            ["progress:halfway ✅", "yield:0", "complete:complete"]
        );
    }

    #[test]
    fn native_worker_failure_and_terminal_calls_are_delivered_once() {
        reset_host();
        unsafe { failed_worker_entry(0, 0) };
        WorkerRuntime::report_progress("late progress");
        WorkerRuntime::complete("late complete");
        WorkerRuntime::fail("late error");

        assert_eq!(
            HOST_EVENTS.with(|events| events.borrow().clone()),
            ["error:failed ❌"]
        );
    }

    #[test]
    fn native_worker_observes_host_cancellation() {
        reset_host();
        assert!(!WorkerRuntime::is_cancelled());
        HOST_CANCELLED.with(|cancelled| cancelled.set(true));
        assert!(WorkerRuntime::is_cancelled());
    }

    #[test]
    fn native_worker_panic_becomes_one_normalized_error() {
        reset_host();
        unsafe { panicking_worker_entry(0, 0) };

        assert_eq!(
            HOST_EVENTS.with(|events| events.borrow().clone()),
            ["error:Worker panicked."]
        );
    }
}
