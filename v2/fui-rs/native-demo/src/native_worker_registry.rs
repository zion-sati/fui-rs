#[repr(C)]
pub struct FuiNativeWorkerRegistryEntry {
    pub artifact: *const u8,
    pub entry: *const u8,
    pub host_services: *const FuiNativeWorkerHostServiceEntry,
    pub host_service_count: usize,
    pub invoke: unsafe extern "C" fn(usize, u32),
}
unsafe impl Sync for FuiNativeWorkerRegistryEntry {}

#[repr(C)]
pub struct FuiNativeWorkerHostServiceEntry {
    pub name: *const u8,
}
unsafe impl Sync for FuiNativeWorkerHostServiceEntry {}

static ARTIFACT: &[u8] = b"./workers.wasm\0";
static PRIME_ENTRY: &[u8] = b"stage4PrimeWorker\0";
static FAIL_ENTRY: &[u8] = b"stage4FailWorker\0";
static CLOCK_SERVICE: &[u8] = b"demoWorkerClockWallClockSinceEpochMs\0";
static HOST_SERVICES: &[FuiNativeWorkerHostServiceEntry] =
    &[FuiNativeWorkerHostServiceEntry { name: CLOCK_SERVICE.as_ptr() }];
static WORKERS: &[FuiNativeWorkerRegistryEntry] = &[
    FuiNativeWorkerRegistryEntry {
        artifact: ARTIFACT.as_ptr(),
        entry: PRIME_ENTRY.as_ptr(),
        host_services: HOST_SERVICES.as_ptr(),
        host_service_count: HOST_SERVICES.len(),
        invoke: fui_rs_demo_worker::stage4PrimeWorker,
    },
    FuiNativeWorkerRegistryEntry {
        artifact: ARTIFACT.as_ptr(),
        entry: FAIL_ENTRY.as_ptr(),
        host_services: HOST_SERVICES.as_ptr(),
        host_service_count: HOST_SERVICES.len(),
        invoke: fui_rs_demo_worker::stage4FailWorker,
    },
];

#[no_mangle]
pub unsafe extern "C" fn fui_native_worker_registry(
    count: *mut usize,
) -> *const FuiNativeWorkerRegistryEntry {
    if !count.is_null() {
        unsafe { *count = WORKERS.len() };
    }
    WORKERS.as_ptr()
}
