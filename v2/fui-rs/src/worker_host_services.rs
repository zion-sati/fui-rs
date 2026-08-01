use crate::worker_runtime::WorkerRuntime;
use std::collections::HashMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex, OnceLock};

#[derive(Clone, Debug, PartialEq)]
#[doc(hidden)]
pub enum NativeWorkerHostServiceValue {
    String(String),
    Bool(bool),
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
    F64(f64),
    Bytes(Vec<u8>),
    I32Array(Vec<i32>),
    U32Array(Vec<u32>),
    I64Array(Vec<i64>),
    U64Array(Vec<u64>),
    F64Array(Vec<f64>),
    Void,
}

#[doc(hidden)]
pub trait NativeWorkerHostServiceType: Default + Sized {
    const TYPE_NAME: &'static str;
    fn into_native_worker_host_service_value(self) -> NativeWorkerHostServiceValue;
    fn from_native_worker_host_service_value(
        value: NativeWorkerHostServiceValue,
    ) -> Result<Self, String>;
}

macro_rules! native_worker_host_service_type {
    ($type:ty, $variant:ident, $name:literal) => {
        impl NativeWorkerHostServiceType for $type {
            const TYPE_NAME: &'static str = $name;

            fn into_native_worker_host_service_value(self) -> NativeWorkerHostServiceValue {
                NativeWorkerHostServiceValue::$variant(self)
            }

            fn from_native_worker_host_service_value(
                value: NativeWorkerHostServiceValue,
            ) -> Result<Self, String> {
                match value {
                    NativeWorkerHostServiceValue::$variant(value) => Ok(value),
                    _ => Err(format!("expected {}", Self::TYPE_NAME)),
                }
            }
        }
    };
}

native_worker_host_service_type!(String, String, "string");
native_worker_host_service_type!(bool, Bool, "bool");
native_worker_host_service_type!(i32, I32, "i32");
native_worker_host_service_type!(u32, U32, "u32");
native_worker_host_service_type!(i64, I64, "i64");
native_worker_host_service_type!(u64, U64, "u64");
native_worker_host_service_type!(f64, F64, "f64");
native_worker_host_service_type!(Vec<u8>, Bytes, "bytes");
native_worker_host_service_type!(Vec<i32>, I32Array, "i32_array");
native_worker_host_service_type!(Vec<u32>, U32Array, "u32_array");
native_worker_host_service_type!(Vec<i64>, I64Array, "i64_array");
native_worker_host_service_type!(Vec<u64>, U64Array, "u64_array");
native_worker_host_service_type!(Vec<f64>, F64Array, "f64_array");

impl NativeWorkerHostServiceType for () {
    const TYPE_NAME: &'static str = "void";

    fn into_native_worker_host_service_value(self) -> NativeWorkerHostServiceValue {
        NativeWorkerHostServiceValue::Void
    }

    fn from_native_worker_host_service_value(
        value: NativeWorkerHostServiceValue,
    ) -> Result<Self, String> {
        match value {
            NativeWorkerHostServiceValue::Void => Ok(()),
            _ => Err(format!("expected {}", Self::TYPE_NAME)),
        }
    }
}

type Handler = dyn Fn(Vec<NativeWorkerHostServiceValue>) -> Result<NativeWorkerHostServiceValue, String>
    + Send
    + Sync
    + 'static;

struct RegisteredHandler {
    generation: u64,
    handler: Arc<Handler>,
}

#[derive(Default)]
struct Registry {
    next_generation: u64,
    handlers: HashMap<&'static str, RegisteredHandler>,
}

fn registry() -> &'static Mutex<Registry> {
    static REGISTRY: OnceLock<Mutex<Registry>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(Registry::default()))
}

pub struct NativeWorkerHostServiceRegistration {
    import_name: &'static str,
    generation: u64,
}

impl Drop for NativeWorkerHostServiceRegistration {
    fn drop(&mut self) {
        let mut registry = registry()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if registry
            .handlers
            .get(self.import_name)
            .is_some_and(|registered| registered.generation == self.generation)
        {
            registry.handlers.remove(self.import_name);
        }
    }
}

#[doc(hidden)]
pub fn register_native_worker_host_service(
    import_name: &'static str,
    handler: impl Fn(Vec<NativeWorkerHostServiceValue>) -> Result<NativeWorkerHostServiceValue, String>
        + Send
        + Sync
        + 'static,
) -> Result<NativeWorkerHostServiceRegistration, String> {
    let mut registry = registry()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if registry.handlers.contains_key(import_name) {
        return Err(format!(
            "Native Worker host service {import_name} is already registered."
        ));
    }
    registry.next_generation = registry.next_generation.wrapping_add(1).max(1);
    let generation = registry.next_generation;
    registry.handlers.insert(
        import_name,
        RegisteredHandler {
            generation,
            handler: Arc::new(handler),
        },
    );
    Ok(NativeWorkerHostServiceRegistration {
        import_name,
        generation,
    })
}

#[doc(hidden)]
pub fn native_worker_host_service_arg<T: NativeWorkerHostServiceType>(
    args: &mut std::vec::IntoIter<NativeWorkerHostServiceValue>,
    import_name: &str,
    index: usize,
) -> Result<T, String> {
    let value = args.next().ok_or_else(|| {
        format!("Native Worker host service {import_name} is missing argument {index}.")
    })?;
    T::from_native_worker_host_service_value(value).map_err(|message| {
        format!("Native Worker host service {import_name} argument {index} {message}.")
    })
}

#[cfg(not(target_arch = "wasm32"))]
unsafe extern "C" {
    fn fui_native_worker_host_service_is_allowed(name: *const u8, length: u32) -> bool;
}

fn is_allowed(import_name: &str) -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = import_name;
        false
    }
    #[cfg(not(target_arch = "wasm32"))]
    unsafe {
        fui_native_worker_host_service_is_allowed(
            import_name.as_ptr(),
            import_name.len().try_into().unwrap_or(u32::MAX),
        )
    }
}

fn invoke(
    import_name: &'static str,
    args: Vec<NativeWorkerHostServiceValue>,
) -> Result<NativeWorkerHostServiceValue, String> {
    if !is_allowed(import_name) {
        return Err(format!(
            "Native Worker host service {import_name} is not allowed by the active Worker declaration."
        ));
    }
    let handler = {
        let registry = registry()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        registry
            .handlers
            .get(import_name)
            .map(|registered| Arc::clone(&registered.handler))
    }
    .ok_or_else(|| format!("Native Worker host service {import_name} is not registered."))?;
    match catch_unwind(AssertUnwindSafe(|| handler(args))) {
        Ok(result) => result,
        Err(_) => Err(format!(
            "Native Worker host service {import_name} panicked."
        )),
    }
}

#[doc(hidden)]
pub fn invoke_native_worker_host_service<T: NativeWorkerHostServiceType>(
    import_name: &'static str,
    args: Vec<NativeWorkerHostServiceValue>,
) -> T {
    match invoke(import_name, args).and_then(|value| {
        T::from_native_worker_host_service_value(value).map_err(|message| {
            format!("Native Worker host service {import_name} returned {message}.")
        })
    }) {
        Ok(value) => value,
        Err(message) => {
            WorkerRuntime::fail(message);
            T::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::thread;

    static ALLOWED: AtomicBool = AtomicBool::new(true);

    #[no_mangle]
    extern "C" fn fui_native_worker_host_service_is_allowed(
        _name: *const u8,
        _length: u32,
    ) -> bool {
        ALLOWED.load(Ordering::SeqCst)
    }

    #[test]
    fn native_registry_normalizes_all_values_and_releases_owned_buffers() {
        ALLOWED.store(true, Ordering::SeqCst);
        let registration = register_native_worker_host_service("allValues", |args| {
            assert_eq!(args.len(), 13);
            assert_eq!(
                args[0],
                NativeWorkerHostServiceValue::String("hello".into())
            );
            assert_eq!(args[1], NativeWorkerHostServiceValue::Bool(true));
            assert_eq!(
                args[12],
                NativeWorkerHostServiceValue::F64Array(vec![1.5, 2.5])
            );
            Ok(NativeWorkerHostServiceValue::Bytes(vec![7, 8, 9]))
        })
        .expect("register service");
        let result: Vec<u8> = invoke_native_worker_host_service(
            "allValues",
            vec![
                NativeWorkerHostServiceValue::String("hello".into()),
                NativeWorkerHostServiceValue::Bool(true),
                NativeWorkerHostServiceValue::I32(-2),
                NativeWorkerHostServiceValue::U32(2),
                NativeWorkerHostServiceValue::I64(-3),
                NativeWorkerHostServiceValue::U64(3),
                NativeWorkerHostServiceValue::F64(4.5),
                NativeWorkerHostServiceValue::Bytes(vec![1]),
                NativeWorkerHostServiceValue::I32Array(vec![-1]),
                NativeWorkerHostServiceValue::U32Array(vec![1]),
                NativeWorkerHostServiceValue::I64Array(vec![-2]),
                NativeWorkerHostServiceValue::U64Array(vec![2]),
                NativeWorkerHostServiceValue::F64Array(vec![1.5, 2.5]),
            ],
        );
        assert_eq!(result, [7, 8, 9]);
        drop(registration);
    }

    #[test]
    fn native_registry_enforces_allowlist_before_invocation() {
        let invoked = Arc::new(AtomicUsize::new(0));
        let invoked_by_handler = Arc::clone(&invoked);
        let _registration = register_native_worker_host_service("denied", move |_| {
            invoked_by_handler.fetch_add(1, Ordering::SeqCst);
            Ok(NativeWorkerHostServiceValue::Void)
        })
        .expect("register service");
        ALLOWED.store(false, Ordering::SeqCst);
        let _: () = invoke_native_worker_host_service("denied", vec![]);
        ALLOWED.store(true, Ordering::SeqCst);
        assert_eq!(invoked.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn native_registry_normalizes_unknown_malformed_error_and_panic_results() {
        ALLOWED.store(true, Ordering::SeqCst);
        assert!(invoke("unknown", vec![])
            .unwrap_err()
            .contains("not registered"));

        let mut missing = Vec::new().into_iter();
        assert!(
            native_worker_host_service_arg::<u32>(&mut missing, "malformed", 0)
                .unwrap_err()
                .contains("missing argument 0")
        );
        let mut wrong = vec![NativeWorkerHostServiceValue::String("wrong".into())].into_iter();
        assert!(
            native_worker_host_service_arg::<u32>(&mut wrong, "malformed", 0)
                .unwrap_err()
                .contains("expected u32")
        );

        let malformed = register_native_worker_host_service("malformed", |_| {
            Ok(NativeWorkerHostServiceValue::String("wrong".into()))
        })
        .expect("register malformed service");
        assert!(
            <u32 as NativeWorkerHostServiceType>::from_native_worker_host_service_value(
                invoke("malformed", vec![]).expect("invoke malformed service")
            )
            .unwrap_err()
            .contains("expected u32")
        );
        drop(malformed);

        let error = register_native_worker_host_service("error", |_| Err("unavailable".into()))
            .expect("register error service");
        assert_eq!(invoke("error", vec![]), Err("unavailable".into()));
        drop(error);

        let panicking = register_native_worker_host_service("panicking", |_| panic!("boom"))
            .expect("register panicking service");
        assert!(invoke("panicking", vec![])
            .unwrap_err()
            .contains("panicked"));
        drop(panicking);
    }

    #[test]
    fn native_registry_supports_concurrent_worker_calls() {
        ALLOWED.store(true, Ordering::SeqCst);
        let _registration = register_native_worker_host_service("concurrent", |args| {
            let mut args = args.into_iter();
            let value: u32 = native_worker_host_service_arg(&mut args, "concurrent", 0)?;
            Ok(NativeWorkerHostServiceValue::U32(value + 1))
        })
        .expect("register concurrent service");
        let threads: Vec<_> = (0..8)
            .map(|value| {
                thread::spawn(move || {
                    invoke("concurrent", vec![NativeWorkerHostServiceValue::U32(value)])
                })
            })
            .collect();
        for (value, thread) in threads.into_iter().enumerate() {
            assert_eq!(
                thread.join().expect("join worker"),
                Ok(NativeWorkerHostServiceValue::U32(value as u32 + 1))
            );
        }
    }
}
