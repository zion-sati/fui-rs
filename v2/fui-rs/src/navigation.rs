use crate::ffi;

pub fn can_navigate_back() -> bool {
    unsafe { ffi::fui_can_navigate_back() }
}

pub fn can_navigate_forward() -> bool {
    unsafe { ffi::fui_can_navigate_forward() }
}

pub fn navigate_back() {
    unsafe { ffi::fui_navigate_back() }
}

pub fn navigate_forward() {
    unsafe { ffi::fui_navigate_forward() }
}

pub fn navigate_to(target: &str, open_in_new_tab: bool) {
    let bytes = target.as_bytes();
    unsafe {
        ffi::fui_navigate_to(
            if bytes.is_empty() {
                0
            } else {
                bytes.as_ptr() as usize
            },
            bytes.len() as u32,
            open_in_new_tab,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::ffi::{self, Call};

    #[test]
    fn navigate_to_emits_host_call() {
        ffi::test::reset();
        super::navigate_to("/docs", true);
        let calls = ffi::test::take_calls();
        assert!(calls.iter().any(|call| matches!(
            call,
            Call::NavigateTo { target, open_in_new_tab } if target == "/docs" && *open_in_new_tab
        )));
    }
}
