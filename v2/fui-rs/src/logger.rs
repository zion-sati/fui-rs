use crate::ffi;

fn write_log(category: &str, message: &str) {
    let category_bytes = category.as_bytes();
    let message_bytes = message.as_bytes();
    unsafe {
        ffi::fui_log(
            if category_bytes.is_empty() {
                0
            } else {
                category_bytes.as_ptr() as usize
            },
            category_bytes.len() as u32,
            if message_bytes.is_empty() {
                0
            } else {
                message_bytes.as_ptr() as usize
            },
            message_bytes.len() as u32,
        )
    }
}

pub fn logs_enabled() -> bool {
    unsafe { ffi::fui_logs_enabled() }
}

pub fn log(category: &str, message: &str) {
    if !logs_enabled() {
        return;
    }
    write_log(category, message);
}

pub fn warn(category: &str, message: &str) {
    write_log(&format!("Warning/{category}"), message);
}

pub fn error(category: &str, message: &str) {
    write_log(&format!("Error/{category}"), message);
}

#[cfg(test)]
mod tests {
    use crate::ffi::{self, Call};

    #[test]
    fn log_respects_logs_enabled() {
        ffi::test::reset();
        ffi::test::set_logs_enabled(false);
        super::log("test", "message");
        let calls = ffi::test::take_calls();
        assert!(calls.iter().any(|call| matches!(call, Call::LogsEnabled)));
        assert!(!calls.iter().any(|call| matches!(call, Call::Log { .. })));
    }
}
