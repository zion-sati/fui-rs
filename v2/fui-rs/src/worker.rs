use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ffi;

type ProgressCallback = Rc<dyn Fn(WorkerProgressEventArgs)>;
type CompleteCallback = Rc<dyn Fn(WorkerCompletedEventArgs)>;
type ErrorCallback = Rc<dyn Fn(WorkerErrorEventArgs)>;
const MAX_WORKER_START_INPUT_BYTES: usize = 1024 * 1024;

thread_local! {
    static NEXT_WORKER_ID: RefCell<u32> = const { RefCell::new(1) };
    static ACTIVE_WORKERS: RefCell<HashMap<u32, Rc<RefCell<WorkerInner>>>> = RefCell::new(HashMap::new());
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkerProgressEventArgs {
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkerCompletedEventArgs {
    pub result: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkerErrorEventArgs {
    pub message: String,
}

struct WorkerInner {
    worker_id: u32,
    wasm_path: String,
    entry_name: String,
    on_progress: Option<ProgressCallback>,
    on_complete: Option<CompleteCallback>,
    on_error: Option<ErrorCallback>,
    started: bool,
    finished: bool,
    cancel_requested: bool,
}

pub struct Worker {
    inner: Rc<RefCell<WorkerInner>>,
}

impl Worker {
    pub fn new(wasm_path: impl Into<String>, entry_name: impl Into<String>) -> Self {
        let worker_id = NEXT_WORKER_ID.with(|next| {
            let mut slot = next.borrow_mut();
            let id = *slot;
            *slot += 1;
            id
        });
        let worker = Self {
            inner: Rc::new(RefCell::new(WorkerInner {
                worker_id,
                wasm_path: wasm_path.into(),
                entry_name: entry_name.into(),
                on_progress: None,
                on_complete: None,
                on_error: None,
                started: false,
                finished: false,
                cancel_requested: false,
            })),
        };
        ACTIVE_WORKERS.with(|workers| {
            workers.borrow_mut().insert(worker_id, worker.inner.clone());
        });
        worker
    }

    pub fn on_progress(self, handler: impl Fn(WorkerProgressEventArgs) + 'static) -> Self {
        self.inner.borrow_mut().on_progress = Some(Rc::new(handler));
        self
    }

    pub fn on_complete(self, handler: impl Fn(WorkerCompletedEventArgs) + 'static) -> Self {
        self.inner.borrow_mut().on_complete = Some(Rc::new(handler));
        self
    }

    pub fn on_error(self, handler: impl Fn(WorkerErrorEventArgs) + 'static) -> Self {
        self.inner.borrow_mut().on_error = Some(Rc::new(handler));
        self
    }

    pub fn start(self, input: impl Into<String>) -> Self {
        let input = input.into();
        let already_started = {
            let inner = self.inner.borrow();
            inner.started || inner.finished
        };
        if already_started {
            return self;
        }
        if input.len() > MAX_WORKER_START_INPUT_BYTES {
            let (worker_id, callback) = {
                let mut inner = self.inner.borrow_mut();
                inner.started = true;
                inner.finished = true;
                (inner.worker_id, inner.on_error.clone())
            };
            finish_worker(worker_id);
            if let Some(callback) = callback {
                callback(WorkerErrorEventArgs {
                    message: format!(
                        "Worker.start input exceeds the maximum UTF-8 payload size of {} bytes.",
                        MAX_WORKER_START_INPUT_BYTES
                    ),
                });
            }
            return self;
        }
        let start_info = {
            let mut inner = self.inner.borrow_mut();
            inner.started = true;
            (
                inner.worker_id,
                inner.wasm_path.clone(),
                inner.entry_name.clone(),
            )
        };
        with_utf8(&start_info.1, |wasm_path_ptr, wasm_path_len| {
            with_utf8(&start_info.2, |entry_ptr, entry_len| {
                with_utf8(&input, |input_ptr, input_len| unsafe {
                    ffi::fui_worker_start_string(
                        start_info.0,
                        wasm_path_ptr,
                        wasm_path_len,
                        entry_ptr,
                        entry_len,
                        input_ptr,
                        input_len,
                    );
                })
            })
        });
        self
    }

    pub fn cancel(&self) {
        let worker_id = {
            let mut inner = self.inner.borrow_mut();
            if !inner.started || inner.finished || inner.cancel_requested {
                return;
            }
            inner.cancel_requested = true;
            inner.worker_id
        };
        unsafe { ffi::fui_worker_cancel(worker_id) };
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.cancel();
        finish_worker(self.inner.borrow().worker_id);
    }
}

fn finish_worker(worker_id: u32) -> Option<Rc<RefCell<WorkerInner>>> {
    ACTIVE_WORKERS.with(|workers| workers.borrow_mut().remove(&worker_id))
}

fn with_active_worker(worker_id: u32, callback: impl FnOnce(&mut WorkerInner)) {
    let Some(worker) = ACTIVE_WORKERS.with(|workers| workers.borrow().get(&worker_id).cloned())
    else {
        return;
    };
    let mut inner = worker.borrow_mut();
    callback(&mut inner);
}

#[cfg_attr(not(feature = "worker-runtime"), no_mangle)]
/// # Safety
/// `text_ptr` must be null for an empty message or point to `text_len` readable bytes.
pub unsafe extern "C" fn __fui_on_worker_progress(
    worker_id: u32,
    text_ptr: *const u8,
    text_len: u32,
) {
    let message = if text_ptr.is_null() || text_len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(unsafe { std::slice::from_raw_parts(text_ptr, text_len as usize) })
            .into_owned()
    };
    with_active_worker(worker_id, |inner| {
        if inner.finished || inner.cancel_requested {
            return;
        }
        if let Some(callback) = inner.on_progress.clone() {
            callback(WorkerProgressEventArgs { message });
        }
    });
}

#[cfg_attr(not(feature = "worker-runtime"), no_mangle)]
/// # Safety
/// `text_ptr` must be null for an empty result or point to `text_len` readable bytes.
pub unsafe extern "C" fn __fui_on_worker_complete(
    worker_id: u32,
    text_ptr: *const u8,
    text_len: u32,
) {
    let result = if text_ptr.is_null() || text_len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(unsafe { std::slice::from_raw_parts(text_ptr, text_len as usize) })
            .into_owned()
    };
    let Some(worker) = finish_worker(worker_id) else {
        return;
    };
    let callback = {
        let mut inner = worker.borrow_mut();
        if inner.finished {
            return;
        }
        inner.finished = true;
        inner.on_complete.clone()
    };
    if let Some(callback) = callback {
        callback(WorkerCompletedEventArgs { result });
    }
}

#[cfg_attr(not(feature = "worker-runtime"), no_mangle)]
/// # Safety
/// `text_ptr` must be null for an empty message or point to `text_len` readable bytes.
pub unsafe extern "C" fn __fui_on_worker_error(worker_id: u32, text_ptr: *const u8, text_len: u32) {
    let message = if text_ptr.is_null() || text_len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(unsafe { std::slice::from_raw_parts(text_ptr, text_len as usize) })
            .into_owned()
    };
    let Some(worker) = finish_worker(worker_id) else {
        return;
    };
    let callback = {
        let mut inner = worker.borrow_mut();
        if inner.finished {
            return;
        }
        inner.finished = true;
        inner.on_error.clone()
    };
    if let Some(callback) = callback {
        callback(WorkerErrorEventArgs { message });
    }
}

#[cfg(test)]
mod tests {
    use super::Worker;
    use crate::ffi::{self, Call};
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn worker_start_emits_host_call() {
        ffi::test::reset();
        let _worker = Worker::new("./workers/test.wasm", "demo").start("hello");
        let calls = ffi::test::take_calls();
        assert!(calls.iter().any(|call| matches!(call, Call::WorkerStartString { wasm_path, entry, input, .. } if wasm_path == "./workers/test.wasm" && entry == "demo" && input == "hello")));
    }

    #[test]
    fn oversized_worker_start_input_reports_error_without_host_call() {
        ffi::test::reset();
        let error = Rc::new(RefCell::new(String::new()));
        let error_clone = error.clone();
        let input = "x".repeat(super::MAX_WORKER_START_INPUT_BYTES + 1);
        let _worker = Worker::new("./workers/test.wasm", "demo")
            .on_error(move |event| {
                error_clone.replace(event.message);
            })
            .start(input);
        let calls = ffi::test::take_calls();
        assert!(!calls
            .iter()
            .any(|call| matches!(call, Call::WorkerStartString { .. })));
        assert!(error.borrow().contains("maximum UTF-8 payload size"));
    }

    #[test]
    fn worker_callbacks_receive_payloads() {
        ffi::test::reset();
        let progress = Rc::new(RefCell::new(String::new()));
        let result = Rc::new(RefCell::new(String::new()));
        let progress_clone = progress.clone();
        let result_clone = result.clone();
        let _worker = Worker::new("./workers/test.wasm", "demo")
            .on_progress(move |event| {
                progress_clone.replace(event.message);
            })
            .on_complete(move |event| {
                result_clone.replace(event.result);
            })
            .start("hello");
        unsafe {
            super::__fui_on_worker_progress(1, b"25%".as_ptr(), 3);
            super::__fui_on_worker_complete(1, b"done".as_ptr(), 4);
        }
        assert_eq!(&*progress.borrow(), "25%");
        assert_eq!(&*result.borrow(), "done");
    }
}
