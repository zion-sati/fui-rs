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

#[cfg(target_arch = "wasm32")]
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
        let _ = progress.as_ref();
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
        let _ = result.as_ref();
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
        let _ = message.as_ref();
    }

    pub fn is_cancelled() -> bool {
        #[cfg(target_arch = "wasm32")]
        {
            unsafe { host_worker_is_cancelled() }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            false
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
        let _ = delay_ms;
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
    use crate::{fui_worker, WorkerJob, WorkerJobState};
    use std::cell::{Cell, RefCell};

    thread_local! {
        static START_INPUT: RefCell<String> = const { RefCell::new(String::new()) };
        static RUN_COUNT: Cell<u32> = const { Cell::new(0) };
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
            if run_count == 2 {
                self.complete("complete");
            }
        }
    }

    fui_worker!(test_worker_entry => TestJob);

    #[test]
    fn worker_macro_keeps_job_state_across_resumes_and_exports_shared_buffer() {
        RUN_COUNT.with(|value| value.set(0));
        START_INPUT.with(|value| value.borrow_mut().clear());
        let input = "start";

        unsafe {
            test_worker_entry(input.as_ptr() as usize, input.len() as u32);
            test_worker_entry(0, 0);
        }

        assert_eq!(START_INPUT.with(|value| value.borrow().clone()), "start");
        assert_eq!(RUN_COUNT.with(Cell::get), 2);
        assert_ne!(__fui_worker_text_buffer(), 0);
        assert!(__fui_worker_text_buffer_size() > 0);
    }
}
