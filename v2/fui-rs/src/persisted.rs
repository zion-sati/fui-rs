use crate::event;
use crate::ffi;
use std::rc::Rc;

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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PersistedScrollOffset {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersistedTextState {
    pub version: u32,
    pub payload: String,
}

pub trait PersistedStateAdapter {
    fn kind(&self) -> &str;
    fn version(&self) -> u32;
    fn capture(&self) -> Option<String>;
    fn restore(&self, payload: &str, version: u32);
}

pub trait PersistedStateCodec<T>: 'static {
    fn encode(&self, value: T) -> String;
    fn decode(&self, payload: &str, version: u32) -> T;
}

pub struct PersistedStringCodec;
pub struct PersistedBoolCodec;
pub struct PersistedInt32Codec;
pub struct PersistedFloat32Codec;

impl PersistedStateCodec<String> for PersistedStringCodec {
    fn encode(&self, value: String) -> String {
        value
    }

    fn decode(&self, payload: &str, _version: u32) -> String {
        payload.to_string()
    }
}

impl PersistedStateCodec<bool> for PersistedBoolCodec {
    fn encode(&self, value: bool) -> String {
        value.to_string()
    }

    fn decode(&self, payload: &str, _version: u32) -> bool {
        payload.parse::<bool>().unwrap_or(false)
    }
}

impl PersistedStateCodec<i32> for PersistedInt32Codec {
    fn encode(&self, value: i32) -> String {
        value.to_string()
    }

    fn decode(&self, payload: &str, _version: u32) -> i32 {
        payload.parse::<i32>().unwrap_or(0)
    }
}

impl PersistedStateCodec<f32> for PersistedFloat32Codec {
    fn encode(&self, value: f32) -> String {
        value.to_string()
    }

    fn decode(&self, payload: &str, _version: u32) -> f32 {
        payload.parse::<f32>().unwrap_or(0.0)
    }
}

struct PersistedValueAdapter<T, TCodec>
where
    T: 'static,
    TCodec: PersistedStateCodec<T>,
{
    kind: String,
    version: u32,
    codec: TCodec,
    capture_value: Rc<dyn Fn() -> Option<T>>,
    restore_value: Rc<dyn Fn(T)>,
}

impl<T, TCodec> PersistedStateAdapter for PersistedValueAdapter<T, TCodec>
where
    T: 'static,
    TCodec: PersistedStateCodec<T>,
{
    fn kind(&self) -> &str {
        &self.kind
    }

    fn version(&self) -> u32 {
        self.version
    }

    fn capture(&self) -> Option<String> {
        (self.capture_value)().map(|value| self.codec.encode(value))
    }

    fn restore(&self, payload: &str, version: u32) {
        (self.restore_value)(self.codec.decode(payload, version));
    }
}

pub fn persisted_value_adapter<T, TCodec>(
    kind: impl Into<String>,
    codec: TCodec,
    version: u32,
    capture_value: impl Fn() -> Option<T> + 'static,
    restore_value: impl Fn(T) + 'static,
) -> Rc<dyn PersistedStateAdapter>
where
    T: 'static,
    TCodec: PersistedStateCodec<T>,
{
    let kind = kind.into();
    assert!(
        !kind.is_empty(),
        "PersistedStateAdapter requires a non-empty kind."
    );
    Rc::new(PersistedValueAdapter {
        kind,
        version,
        codec,
        capture_value: Rc::new(capture_value),
        restore_value: Rc::new(restore_value),
    })
}

pub fn store_scroll_offset(node_id: &str, x: f32, y: f32) {
    with_utf8(node_id, |node_id_ptr, node_id_len| unsafe {
        ffi::fui_set_persisted_scroll_offset(node_id_ptr, node_id_len, x, y);
    });
}

pub fn store_text_state(node_id: &str, kind: &str, version: u32, payload: &str) {
    with_utf8(node_id, |node_id_ptr, node_id_len| {
        with_utf8(kind, |kind_ptr, kind_len| {
            with_utf8(payload, |payload_ptr, payload_len| unsafe {
                ffi::fui_set_persisted_state(
                    node_id_ptr,
                    node_id_len,
                    kind_ptr,
                    kind_len,
                    version,
                    payload_ptr,
                    payload_len,
                );
            })
        })
    });
}

pub fn try_load_scroll_offset(node_id: &str) -> Option<PersistedScrollOffset> {
    let mut x = 0.0f32;
    let mut y = 0.0f32;
    let found = with_utf8_result(node_id, |node_id_ptr, node_id_len| unsafe {
        ffi::fui_try_get_persisted_scroll_offset(
            node_id_ptr,
            node_id_len,
            (&mut x as *mut f32) as usize,
            (&mut y as *mut f32) as usize,
        )
    });
    if found {
        Some(PersistedScrollOffset { x, y })
    } else {
        None
    }
}

pub fn try_load_text_state(node_id: &str, kind: &str) -> Option<PersistedTextState> {
    let mut version = 0u32;
    let payload_ptr = event::__fui_text_buffer() as usize;
    let payload_capacity = event::__fui_text_buffer_size();
    let copied = with_utf8_result(node_id, |node_id_ptr, node_id_len| {
        with_utf8_result(kind, |kind_ptr, kind_len| unsafe {
            ffi::fui_copy_persisted_state(
                node_id_ptr,
                node_id_len,
                kind_ptr,
                kind_len,
                (&mut version as *mut u32) as usize,
                payload_ptr,
                payload_capacity,
            )
        })
    });
    if copied < 0 {
        return None;
    }
    let copied_len = copied as usize;
    if copied_len > payload_capacity as usize {
        panic!("Persisted state payload exceeded shared Rust text buffer capacity.");
    }
    let payload = if copied_len == 0 {
        String::new()
    } else {
        let bytes = unsafe { std::slice::from_raw_parts(payload_ptr as *const u8, copied_len) };
        String::from_utf8_lossy(bytes).into_owned()
    };
    Some(PersistedTextState { version, payload })
}

fn with_utf8_result<T>(value: &str, callback: impl FnOnce(usize, u32) -> T) -> T {
    let bytes = value.as_bytes();
    callback(
        if bytes.is_empty() {
            0
        } else {
            bytes.as_ptr() as usize
        },
        bytes.len() as u32,
    )
}

#[cfg(test)]
mod tests {
    use super::{
        persisted_value_adapter, store_scroll_offset, store_text_state, try_load_scroll_offset,
        try_load_text_state, PersistedBoolCodec, PersistedScrollOffset, PersistedTextState,
    };
    use crate::ffi::{self, Call};

    #[test]
    fn persisted_scroll_round_trips_through_host() {
        ffi::test::reset();
        store_scroll_offset("list", 12.0, 34.0);
        assert_eq!(
            try_load_scroll_offset("list"),
            Some(PersistedScrollOffset { x: 12.0, y: 34.0 }),
        );
        let calls = ffi::test::take_calls();
        assert!(calls.iter().any(|call| matches!(call, Call::SetPersistedScrollOffset { node_id, x, y } if node_id == "list" && *x == 12.0 && *y == 34.0)));
        assert!(calls.iter().any(|call| matches!(call, Call::TryGetPersistedScrollOffset { node_id } if node_id == "list")));
    }

    #[test]
    fn persisted_text_round_trips_through_host() {
        ffi::test::reset();
        store_text_state("input", "text", 2, "hello");
        assert_eq!(
            try_load_text_state("input", "text"),
            Some(PersistedTextState {
                version: 2,
                payload: "hello".to_string(),
            }),
        );
        let calls = ffi::test::take_calls();
        assert!(calls.iter().any(|call| matches!(call, Call::SetPersistedState { node_id, kind, version, payload } if node_id == "input" && kind == "text" && *version == 2 && payload == "hello")));
        assert!(calls.iter().any(|call| matches!(call, Call::CopyPersistedState { node_id, kind } if node_id == "input" && kind == "text")));
    }

    #[test]
    #[should_panic(expected = "PersistedStateAdapter requires a non-empty kind.")]
    fn persisted_value_adapter_rejects_empty_kind() {
        let _ = persisted_value_adapter("", PersistedBoolCodec, 1, || Some(true), |_| {});
    }
}
