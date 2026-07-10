mod generated;
mod job;
mod runtime;

use generated::worker_host_services::demo_worker_clock_wall_clock_since_epoch_ms;
use job::{WorkerJob, WorkerJobState};
use runtime::{
    file_read_chunk, file_worker_write_chunk, reset_worker_runtime, worker_text_buffer_ptr,
    worker_text_buffer_size, WorkerRuntime,
};
use std::cell::RefCell;

const PRIME_SEARCH_TOTAL_MS: f64 = 1600.0;
const PRIME_SEARCH_YIELD_INTERVAL_MS: f64 = 250.0;
const PRIME_TIME_CHECK_MASK: i32 = 127;

thread_local! {
    static ACTIVE_PRIME_JOB: RefCell<Option<Stage4PrimeWorkerJob>> = const { RefCell::new(None) };
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
    }

    fn run(&mut self) {
        if self.is_cancelled() {
            self.fail(format!(
                "cancelled:{}",
                parse_prime_search_percent(
                    self.started_at_ms,
                    demo_worker_clock_wall_clock_since_epoch_ms()
                )
            ));
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
                now = demo_worker_clock_wall_clock_since_epoch_ms();
            }
        }
        now = demo_worker_clock_wall_clock_since_epoch_ms();
        self.report_progress(parse_prime_search_percent(self.started_at_ms, now).to_string());
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

#[no_mangle]
pub extern "C" fn stage4PrimeWorker(input_ptr: usize, input_len: u32) {
    let input = WorkerRuntime::entry_input(input_ptr, input_len);
    ACTIVE_PRIME_JOB.with(|slot| {
        let mut state = slot.borrow_mut();
        let mut job = state.take().unwrap_or_default();
        if job.resume(input) {
            *state = Some(job);
        }
    });
}

#[no_mangle]
pub extern "C" fn stage4FailWorker(input_ptr: usize, input_len: u32) {
    reset_worker_runtime();
    let _ = WorkerRuntime::entry_input(input_ptr, input_len);
    let now = demo_worker_clock_wall_clock_since_epoch_ms();
    WorkerRuntime::fail(format!("worker failure clock={:.0}", now));
}

#[no_mangle]
pub extern "C" fn stage4FileProcessorWorker(input_ptr: usize, input_len: u32) {
    reset_worker_runtime();
    let _ = WorkerRuntime::entry_input(input_ptr, input_len);
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

#[no_mangle]
pub extern "C" fn __fui_worker_text_buffer() -> usize {
    worker_text_buffer_ptr()
}

#[no_mangle]
pub extern "C" fn __fui_worker_text_buffer_size() -> u32 {
    worker_text_buffer_size()
}

#[no_mangle]
pub extern "C" fn __fui_on_fetch_complete(
    _request_id: u32,
    _ok: bool,
    _status: i32,
    _payload_ptr: *const u8,
    _payload_len: u32,
) {
}

#[no_mangle]
pub extern "C" fn __fui_on_fetch_error(
    _request_id: u32,
    _payload_ptr: *const u8,
    _payload_len: u32,
) {
}
