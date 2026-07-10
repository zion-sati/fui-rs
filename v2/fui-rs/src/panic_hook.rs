#[cfg(all(debug_assertions, target_arch = "wasm32"))]
use crate::logger;
#[cfg(all(debug_assertions, target_arch = "wasm32"))]
use std::panic;
#[cfg(all(debug_assertions, target_arch = "wasm32"))]
use std::sync::Once;

#[cfg(all(debug_assertions, target_arch = "wasm32"))]
static INSTALL_PANIC_HOOK: Once = Once::new();

#[cfg(all(debug_assertions, target_arch = "wasm32"))]
pub(crate) fn install() {
    INSTALL_PANIC_HOOK.call_once(|| {
        panic::set_hook(Box::new(|info| {
            let mut message = String::from("Rust panic");
            if let Some(location) = info.location() {
                message.push_str(" at ");
                message.push_str(location.file());
                message.push(':');
                message.push_str(&location.line().to_string());
            }

            if let Some(payload) = info.payload().downcast_ref::<&str>() {
                message.push_str(": ");
                message.push_str(payload);
            } else if let Some(payload) = info.payload().downcast_ref::<String>() {
                message.push_str(": ");
                message.push_str(payload);
            }
            logger::error("Panic", &message);
        }));
    });
}

#[cfg(any(not(debug_assertions), not(target_arch = "wasm32")))]
pub(crate) fn install() {}
