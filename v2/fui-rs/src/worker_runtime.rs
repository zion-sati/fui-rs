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
    pub fn entry_input(input_ptr: usize, input_len: u32) -> String {
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
        send_text(progress.as_ref(), |ptr, len| unsafe {
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
        send_text(result.as_ref(), |ptr, len| unsafe {
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
        send_text(message.as_ref(), |ptr, len| unsafe {
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

pub fn handle_fetch_complete(
    request_id: u32,
    ok: bool,
    status: i32,
    payload_ptr: *const u8,
    payload_len: u32,
) {
    crate::fetch::__fui_on_fetch_complete(request_id, ok, status, payload_ptr, payload_len);
}

pub fn handle_fetch_error(request_id: u32, payload_ptr: *const u8, payload_len: u32) {
    crate::fetch::__fui_on_fetch_error(request_id, payload_ptr, payload_len);
}

pub fn reset_worker_runtime() {
    WORKER_TERMINAL_SENT.with(|sent| sent.set(false));
}
