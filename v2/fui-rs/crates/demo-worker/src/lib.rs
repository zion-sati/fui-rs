mod generated;

use fui::worker_runtime::{
    file_read_chunk, file_worker_write_chunk, reset_worker_runtime, worker_text_buffer_ptr,
};
use fui::{fui_worker, WorkerJob, WorkerJobState, WorkerRuntime};
use generated::worker_host_services::demo_worker_clock_wall_clock_since_epoch_ms;
#[cfg(not(target_arch = "wasm32"))]
use generated::worker_host_services::register_native_demo_worker_clock_wall_clock_since_epoch_ms;

const PRIME_SEARCH_TOTAL_MS: f64 = 1600.0;
const PRIME_SEARCH_YIELD_INTERVAL_MS: f64 = 250.0;
const PRIME_TIME_CHECK_MASK: i32 = 127;

#[cfg(not(target_arch = "wasm32"))]
pub fn register_native_worker_host_services(
) -> Result<fui::worker_host_services::NativeWorkerHostServiceRegistration, String> {
    register_native_worker_host_services_with_clock(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_secs_f64() * 1000.0)
            .map_err(|error| format!("Native wall clock is unavailable: {error}"))
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn register_native_worker_host_services_with_clock(
    clock: impl Fn() -> Result<f64, String> + Send + Sync + 'static,
) -> Result<fui::worker_host_services::NativeWorkerHostServiceRegistration, String> {
    register_native_demo_worker_clock_wall_clock_since_epoch_ms(clock)
}

fn parse_prime_search_percent(started_at_ms: f64, now_ms: f64) -> i32 {
    let elapsed_ms = now_ms - started_at_ms;
    if elapsed_ms <= 0.0 {
        return 0;
    }
    if elapsed_ms >= PRIME_SEARCH_TOTAL_MS {
        return 100;
    }
    ((elapsed_ms * 100.0) / PRIME_SEARCH_TOTAL_MS) as i32
}

fn is_prime(value: i32) -> bool {
    if value < 2 {
        return false;
    }
    if value == 2 {
        return true;
    }
    if (value & 1) == 0 {
        return false;
    }
    let mut divisor = 3;
    while divisor <= value / divisor {
        if value % divisor == 0 {
            return false;
        }
        divisor += 2;
    }
    true
}

struct Stage4PrimeWorkerJob {
    state: WorkerJobState,
    started_at_ms: f64,
    deadline_ms: f64,
    next_yield_at_ms: f64,
    candidate: i32,
    largest_prime: i32,
    last_percent: i32,
}

impl Default for Stage4PrimeWorkerJob {
    fn default() -> Self {
        Self {
            state: WorkerJobState::new(),
            started_at_ms: 0.0,
            deadline_ms: 0.0,
            next_yield_at_ms: 0.0,
            candidate: 2,
            largest_prime: 2,
            last_percent: 0,
        }
    }
}

impl WorkerJob for Stage4PrimeWorkerJob {
    fn state(&mut self) -> &mut WorkerJobState {
        &mut self.state
    }

    fn on_start(&mut self, input: String) {
        let _ = input;
        let now = demo_worker_clock_wall_clock_since_epoch_ms();
        self.started_at_ms = now;
        self.deadline_ms = now + PRIME_SEARCH_TOTAL_MS;
        self.next_yield_at_ms = now + PRIME_SEARCH_YIELD_INTERVAL_MS;
        self.candidate = 2;
        self.largest_prime = 2;
        self.last_percent = 0;
    }

    fn run(&mut self) {
        if self.is_cancelled() {
            self.fail(format!("cancelled:{}", self.last_percent));
            return;
        }
        let mut now = demo_worker_clock_wall_clock_since_epoch_ms();
        let slice_deadline = self.next_yield_at_ms.min(self.deadline_ms);
        while now < slice_deadline {
            if is_prime(self.candidate) {
                self.largest_prime = self.candidate;
            }
            self.candidate += 1;
            if (self.candidate & PRIME_TIME_CHECK_MASK) == 0 {
                if self.is_cancelled() {
                    self.fail(format!("cancelled:{}", self.last_percent));
                    return;
                }
                now = demo_worker_clock_wall_clock_since_epoch_ms();
            }
        }
        now = demo_worker_clock_wall_clock_since_epoch_ms();
        self.last_percent = parse_prime_search_percent(self.started_at_ms, now);
        self.report_progress(self.last_percent.to_string());
        if now >= self.deadline_ms {
            self.complete(format!(
                "prime={} clock={:.0}",
                self.largest_prime, self.started_at_ms
            ));
            return;
        }
        self.next_yield_at_ms += PRIME_SEARCH_YIELD_INTERVAL_MS;
        if self.next_yield_at_ms > self.deadline_ms {
            self.next_yield_at_ms = self.deadline_ms;
        }
        self.r#yield(0);
    }
}

fui_worker!(stage4PrimeWorker => Stage4PrimeWorkerJob);

/// # Safety
/// `input_ptr` must reference `input_len` readable bytes when `input_len` is non-zero.
#[no_mangle]
pub unsafe extern "C" fn stage4FailWorker(input_ptr: usize, input_len: u32) {
    reset_worker_runtime();
    let _ = unsafe { WorkerRuntime::entry_input(input_ptr, input_len) };
    let now = demo_worker_clock_wall_clock_since_epoch_ms();
    WorkerRuntime::fail(format!("worker failure clock={:.0}", now));
}

/// # Safety
/// `input_ptr` must reference `input_len` readable bytes when `input_len` is non-zero.
#[no_mangle]
pub unsafe extern "C" fn stage4FileProcessorWorker(input_ptr: usize, input_len: u32) {
    reset_worker_runtime();
    let _ = unsafe { WorkerRuntime::entry_input(input_ptr, input_len) };
    const READ_CHUNK_SIZE: i32 = 64 * 1024;
    let buffer_ptr = worker_text_buffer_ptr();
    let mut offset: u64 = 0;
    let mut hash: u32 = 5381;

    loop {
        let bytes_read = file_read_chunk(offset as i32, (offset >> 32) as i32, READ_CHUNK_SIZE);
        if bytes_read <= 0 {
            break;
        }
        for index in 0..bytes_read as usize {
            let byte = unsafe { *((buffer_ptr + index) as *const u8) } as u32;
            hash = ((hash << 5).wrapping_add(hash)).wrapping_add(byte);
        }
        file_worker_write_chunk(buffer_ptr, bytes_read);
        offset += bytes_read as u64;
        WorkerRuntime::report_progress(offset.to_string());
    }

    WorkerRuntime::complete(format!(
        "{{\"hash\":{},\"algo\":\"djb2\",\"bytes\":{}}}",
        hash, offset
    ));
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod native_host_service_tests {
    use super::*;
    use std::cell::RefCell;
    use std::sync::Mutex;

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    thread_local! {
        static ERRORS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    }

    unsafe fn text(ptr: *const u8, len: u32) -> String {
        if ptr.is_null() || len == 0 {
            return String::new();
        }
        String::from_utf8_lossy(unsafe { std::slice::from_raw_parts(ptr, len as usize) })
            .into_owned()
    }

    #[no_mangle]
    extern "C" fn fui_native_worker_host_service_is_allowed(
        _name: *const u8,
        _length: u32,
    ) -> bool {
        true
    }

    #[no_mangle]
    extern "C" fn fui_native_worker_report_progress(_ptr: *const u8, _len: u32) {}

    #[no_mangle]
    extern "C" fn fui_native_worker_complete_string(_ptr: *const u8, _len: u32) {}

    #[no_mangle]
    extern "C" fn fui_native_worker_fail(ptr: *const u8, len: u32) {
        ERRORS.with(|errors| errors.borrow_mut().push(unsafe { text(ptr, len) }));
    }

    #[no_mangle]
    extern "C" fn fui_native_worker_is_cancelled() -> bool {
        false
    }

    #[no_mangle]
    extern "C" fn fui_native_worker_request_yield(_delay_ms: i32) {}

    #[test]
    fn generated_demo_clock_binding_returns_a_native_value() {
        let _guard = TEST_LOCK.lock().expect("lock native service test");
        fui::worker_runtime::reset_worker_runtime();
        let _registration = register_native_worker_host_services().expect("register native clock");
        let before = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock before")
            .as_secs_f64()
            * 1000.0;
        let actual = demo_worker_clock_wall_clock_since_epoch_ms();
        let after = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock after")
            .as_secs_f64()
            * 1000.0;
        assert!((before..=after).contains(&actual));
    }

    #[test]
    fn generated_binding_turns_service_errors_and_panics_into_worker_errors() {
        let _guard = TEST_LOCK.lock().expect("lock native service test");
        fui::worker_runtime::reset_worker_runtime();
        ERRORS.with(|errors| errors.borrow_mut().clear());
        assert_eq!(demo_worker_clock_wall_clock_since_epoch_ms(), 0.0);
        assert!(ERRORS.with(|errors| errors
            .borrow()
            .iter()
            .any(|error| error.contains("not registered"))));

        for (handler, expected) in [
            (
                Box::new(|| Err("clock unavailable".to_owned()))
                    as Box<dyn Fn() -> Result<f64, String> + Send + Sync>,
                "clock unavailable",
            ),
            (
                Box::new(|| -> Result<f64, String> { panic!("clock panic") })
                    as Box<dyn Fn() -> Result<f64, String> + Send + Sync>,
                "panicked",
            ),
        ] {
            fui::worker_runtime::reset_worker_runtime();
            ERRORS.with(|errors| errors.borrow_mut().clear());
            let registration = register_native_demo_worker_clock_wall_clock_since_epoch_ms(handler)
                .expect("register failing clock");
            assert_eq!(demo_worker_clock_wall_clock_since_epoch_ms(), 0.0);
            assert!(
                ERRORS.with(|errors| errors.borrow().iter().any(|error| error.contains(expected)))
            );
            drop(registration);
        }
    }
}
