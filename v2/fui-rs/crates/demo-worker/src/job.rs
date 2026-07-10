use crate::runtime::WorkerRuntime;

#[derive(Default)]
pub struct WorkerJobState {
    started: bool,
    finished: bool,
}

impl WorkerJobState {
    pub const fn new() -> Self {
        Self {
            started: false,
            finished: false,
        }
    }
}

pub trait WorkerJob {
    fn state(&mut self) -> &mut WorkerJobState;

    fn on_start(&mut self, _input: String) {}

    fn run(&mut self);

    fn resume(&mut self, input: String) -> bool {
        let should_start = {
            let state = self.state();
            if state.started {
                false
            } else {
                state.started = true;
                true
            }
        };
        if should_start {
            self.on_start(input);
        }
        if self.state().finished {
            return false;
        }
        self.run();
        !self.state().finished
    }

    fn report_progress(&mut self, progress: impl AsRef<str>) {
        if self.state().finished {
            return;
        }
        WorkerRuntime::report_progress(progress);
    }

    fn complete(&mut self, result: impl AsRef<str>) {
        if self.state().finished {
            return;
        }
        self.state().finished = true;
        WorkerRuntime::complete(result);
    }

    fn fail(&mut self, message: impl AsRef<str>) {
        if self.state().finished {
            return;
        }
        self.state().finished = true;
        WorkerRuntime::fail(message);
    }

    fn is_cancelled(&self) -> bool {
        WorkerRuntime::is_cancelled()
    }

    fn r#yield(&mut self, delay_ms: i32) -> bool {
        if self.state().finished {
            return false;
        }
        WorkerRuntime::r#yield(delay_ms)
    }
}
