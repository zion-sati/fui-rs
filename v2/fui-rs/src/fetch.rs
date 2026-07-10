use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ffi;

type ResponseCallback = Rc<dyn Fn(FetchResponse)>;
type ErrorCallback = Rc<dyn Fn(FetchErrorEventArgs)>;

thread_local! {
    static NEXT_FETCH_ID: RefCell<u32> = const { RefCell::new(1) };
    static ACTIVE_REQUESTS: RefCell<HashMap<u32, Rc<RefCell<FetchRequestInner>>>> = RefCell::new(HashMap::new());
}

fn encode_text_parts(values: &[String]) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&(values.len() as u32).to_le_bytes());
    for value in values {
        let encoded = value.as_bytes();
        bytes.extend_from_slice(&(encoded.len() as u32).to_le_bytes());
        bytes.extend_from_slice(encoded);
    }
    bytes
}

fn decode_text_parts(bytes: &[u8]) -> Vec<String> {
    if bytes.len() < 4 {
        return Vec::new();
    }
    let mut cursor = 0usize;
    let count =
        u32::from_le_bytes(bytes[cursor..cursor + 4].try_into().unwrap_or([0, 0, 0, 0])) as usize;
    cursor += 4;
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        if cursor + 4 > bytes.len() {
            break;
        }
        let len = u32::from_le_bytes(bytes[cursor..cursor + 4].try_into().unwrap_or([0, 0, 0, 0]))
            as usize;
        cursor += 4;
        if cursor + len > bytes.len() {
            break;
        }
        values.push(String::from_utf8_lossy(&bytes[cursor..cursor + len]).into_owned());
        cursor += len;
    }
    values
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
pub struct FetchResponse {
    pub ok: bool,
    pub status: i32,
    pub status_text: String,
    pub url: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FetchErrorEventArgs {
    pub message: String,
}

struct FetchRequestInner {
    method: String,
    url: String,
    headers: Vec<String>,
    body: Vec<u8>,
    on_complete: Option<ResponseCallback>,
    on_error: Option<ErrorCallback>,
    request_id: u32,
    started: bool,
    finished: bool,
}

pub struct FetchRequest {
    inner: Rc<RefCell<FetchRequestInner>>,
}

impl FetchRequest {
    fn new(url: impl Into<String>) -> Self {
        Self {
            inner: Rc::new(RefCell::new(FetchRequestInner {
                method: "GET".to_string(),
                url: url.into(),
                headers: Vec::new(),
                body: Vec::new(),
                on_complete: None,
                on_error: None,
                request_id: 0,
                started: false,
                finished: false,
            })),
        }
    }

    pub fn method(self, value: impl Into<String>) -> Self {
        self.inner.borrow_mut().method = value.into();
        self
    }

    pub fn header(self, name: impl Into<String>, value: impl Into<String>) -> Self {
        let mut inner = self.inner.borrow_mut();
        inner.headers.push(name.into());
        inner.headers.push(value.into());
        drop(inner);
        self
    }

    pub fn body_bytes(self, value: Vec<u8>) -> Self {
        self.inner.borrow_mut().body = value;
        self
    }

    pub fn body_text(self, value: impl Into<String>) -> Self {
        self.body_bytes(value.into().into_bytes())
    }

    pub fn on_complete(self, handler: impl Fn(FetchResponse) + 'static) -> Self {
        self.inner.borrow_mut().on_complete = Some(Rc::new(handler));
        self
    }

    pub fn on_error(self, handler: impl Fn(FetchErrorEventArgs) + 'static) -> Self {
        self.inner.borrow_mut().on_error = Some(Rc::new(handler));
        self
    }

    pub fn start(self) -> Self {
        let already_started = {
            let inner = self.inner.borrow();
            inner.started || inner.finished
        };
        if already_started {
            return self;
        }
        let empty_url_error = {
            let inner = self.inner.borrow();
            if inner.url.is_empty() {
                inner.on_error.clone()
            } else {
                None
            }
        };
        if self.inner.borrow().url.is_empty() {
            if let Some(callback) = empty_url_error {
                callback(FetchErrorEventArgs {
                    message: "FetchRequest.start: url must not be empty.".to_string(),
                });
            }
            return self;
        }
        let (method, url, headers, body, request_id) = {
            let mut inner = self.inner.borrow_mut();
            let request_id = NEXT_FETCH_ID.with(|next| {
                let mut slot = next.borrow_mut();
                let id = *slot;
                *slot += 1;
                id
            });
            inner.request_id = request_id;
            inner.started = true;
            ACTIVE_REQUESTS.with(|requests| {
                requests.borrow_mut().insert(request_id, self.inner.clone());
            });
            (
                inner.method.clone(),
                inner.url.clone(),
                inner.headers.clone(),
                inner.body.clone(),
                request_id,
            )
        };
        let header_bytes = encode_text_parts(&headers);
        with_utf8(&method, |method_ptr, method_len| {
            with_utf8(&url, |url_ptr, url_len| unsafe {
                ffi::fui_fetch_start(
                    request_id,
                    method_ptr,
                    method_len,
                    url_ptr,
                    url_len,
                    if header_bytes.is_empty() {
                        0
                    } else {
                        header_bytes.as_ptr() as usize
                    },
                    header_bytes.len() as u32,
                    if body.is_empty() {
                        0
                    } else {
                        body.as_ptr() as usize
                    },
                    body.len() as u32,
                );
            })
        });
        self
    }

    pub fn cancel(&self) {
        let request_id = {
            let inner = self.inner.borrow();
            if !inner.started || inner.finished || inner.request_id == 0 {
                return;
            }
            inner.request_id
        };
        unsafe { ffi::fui_fetch_cancel(request_id) };
        finish_request(&self.inner);
    }
}

impl Drop for FetchRequest {
    fn drop(&mut self) {
        self.cancel();
    }
}

fn finish_request(request: &Rc<RefCell<FetchRequestInner>>) {
    let request_id = {
        let mut inner = request.borrow_mut();
        if inner.finished {
            return;
        }
        let request_id = inner.request_id;
        inner.request_id = 0;
        inner.finished = true;
        inner.on_complete = None;
        inner.on_error = None;
        request_id
    };
    if request_id != 0 {
        ACTIVE_REQUESTS.with(|requests| {
            requests.borrow_mut().remove(&request_id);
        });
    }
}

pub struct Fetch;

impl Fetch {
    pub fn request(url: impl Into<String>) -> FetchRequest {
        FetchRequest::new(url)
    }
}

fn complete_request(request_id: u32, response: FetchResponse) {
    let request = ACTIVE_REQUESTS.with(|requests| requests.borrow_mut().remove(&request_id));
    let Some(request) = request else {
        return;
    };
    let callback = {
        let mut inner = request.borrow_mut();
        if inner.finished {
            return;
        }
        inner.finished = true;
        inner.request_id = 0;
        inner.on_error = None;
        inner.on_complete.clone()
    };
    request.borrow_mut().on_complete = None;
    if let Some(callback) = callback {
        callback(response);
    }
}

fn fail_request(request_id: u32, message: String) {
    let request = ACTIVE_REQUESTS.with(|requests| requests.borrow_mut().remove(&request_id));
    let Some(request) = request else {
        return;
    };
    let callback = {
        let mut inner = request.borrow_mut();
        if inner.finished {
            return;
        }
        inner.finished = true;
        inner.request_id = 0;
        inner.on_complete = None;
        inner.on_error.clone()
    };
    request.borrow_mut().on_error = None;
    if let Some(callback) = callback {
        callback(FetchErrorEventArgs { message });
    }
}

pub fn dispose_all_fetch_requests() {
    let requests = ACTIVE_REQUESTS.with(|requests| {
        requests
            .borrow_mut()
            .drain()
            .map(|(_, request)| request)
            .collect::<Vec<_>>()
    });
    for request in requests {
        let request_id = {
            let mut inner = request.borrow_mut();
            if inner.finished || !inner.started || inner.request_id == 0 {
                inner.finished = true;
                inner.on_complete = None;
                inner.on_error = None;
                0
            } else {
                let request_id = inner.request_id;
                inner.request_id = 0;
                inner.finished = true;
                inner.on_complete = None;
                inner.on_error = None;
                request_id
            }
        };
        if request_id != 0 {
            unsafe { ffi::fui_fetch_cancel(request_id) };
        }
    }
}

pub fn reset_fetch_runtime() {
    dispose_all_fetch_requests();
    NEXT_FETCH_ID.with(|next| {
        *next.borrow_mut() = 1;
    });
}

#[no_mangle]
pub extern "C" fn __fui_on_fetch_complete(
    request_id: u32,
    ok: bool,
    status: i32,
    payload_ptr: *const u8,
    payload_len: u32,
) {
    let parts = if payload_ptr.is_null() || payload_len == 0 {
        Vec::new()
    } else {
        decode_text_parts(unsafe { std::slice::from_raw_parts(payload_ptr, payload_len as usize) })
    };
    complete_request(
        request_id,
        FetchResponse {
            ok,
            status,
            status_text: parts.first().cloned().unwrap_or_default(),
            url: parts.get(1).cloned().unwrap_or_default(),
        },
    );
}

#[no_mangle]
pub extern "C" fn __fui_on_fetch_error(request_id: u32, payload_ptr: *const u8, payload_len: u32) {
    let message = if payload_ptr.is_null() || payload_len == 0 {
        "Fetch request failed.".to_string()
    } else {
        String::from_utf8_lossy(unsafe {
            std::slice::from_raw_parts(payload_ptr, payload_len as usize)
        })
        .into_owned()
    };
    fail_request(request_id, message);
}

#[cfg(test)]
mod tests {
    use super::Fetch;
    use crate::ffi::{self, Call};
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn fetch_request_emits_host_call() {
        ffi::test::reset();
        let request = Fetch::request("https://example.com")
            .method("POST")
            .header("Accept", "application/json")
            .body_text("hello")
            .start();
        let calls = ffi::test::take_calls();
        assert!(calls.iter().any(|call| matches!(call, Call::FetchStart { method, url, .. } if method == "POST" && url == "https://example.com")));
        drop(request);
    }

    #[test]
    fn fetch_callbacks_receive_result() {
        ffi::test::reset();
        let result = Rc::new(RefCell::new(String::new()));
        let result_clone = result.clone();
        let request = Fetch::request("https://example.com")
            .on_complete(move |response| {
                result_clone.replace(response.status_text);
            })
            .start();
        let payload = {
            let mut bytes = Vec::new();
            bytes.extend_from_slice(&2u32.to_le_bytes());
            for part in ["OK", "https://example.com"] {
                bytes.extend_from_slice(&(part.len() as u32).to_le_bytes());
                bytes.extend_from_slice(part.as_bytes());
            }
            bytes
        };
        super::__fui_on_fetch_complete(1, true, 200, payload.as_ptr(), payload.len() as u32);
        assert_eq!(&*result.borrow(), "OK");
        drop(request);
    }

    #[test]
    fn fetch_empty_url_reports_error_without_host_call() {
        ffi::test::reset();
        super::reset_fetch_runtime();
        let result = Rc::new(RefCell::new(String::new()));
        let result_clone = result.clone();
        let request = Fetch::request("")
            .on_error(move |event| {
                result_clone.replace(event.message);
            })
            .start();
        assert_eq!(
            &*result.borrow(),
            "FetchRequest.start: url must not be empty."
        );
        assert!(ffi::test::take_calls()
            .iter()
            .all(|call| !matches!(call, Call::FetchStart { .. })));
        drop(request);
    }

    #[test]
    fn fetch_cancel_finishes_and_suppresses_late_completion() {
        ffi::test::reset();
        super::reset_fetch_runtime();
        let result = Rc::new(RefCell::new(String::new()));
        let result_clone = result.clone();
        let request = Fetch::request("https://example.com")
            .on_complete(move |response| {
                result_clone.replace(response.status_text);
            })
            .start();
        request.cancel();
        let calls = ffi::test::take_calls();
        assert!(calls
            .iter()
            .any(|call| matches!(call, Call::FetchCancel { request_id } if *request_id == 1)));
        let payload = {
            let mut bytes = Vec::new();
            bytes.extend_from_slice(&2u32.to_le_bytes());
            for part in ["OK", "https://example.com"] {
                bytes.extend_from_slice(&(part.len() as u32).to_le_bytes());
                bytes.extend_from_slice(part.as_bytes());
            }
            bytes
        };
        super::__fui_on_fetch_complete(1, true, 200, payload.as_ptr(), payload.len() as u32);
        assert_eq!(&*result.borrow(), "");
        drop(request);
    }

    #[test]
    fn fetch_drop_cancels_active_request() {
        ffi::test::reset();
        super::reset_fetch_runtime();
        {
            let _request = Fetch::request("https://example.com").start();
        }
        let calls = ffi::test::take_calls();
        assert!(calls
            .iter()
            .any(|call| matches!(call, Call::FetchCancel { request_id } if *request_id == 1)));
    }

    #[test]
    fn fetch_error_callback_receives_default_message() {
        ffi::test::reset();
        super::reset_fetch_runtime();
        let result = Rc::new(RefCell::new(String::new()));
        let result_clone = result.clone();
        let request = Fetch::request("https://example.com")
            .on_error(move |event| {
                result_clone.replace(event.message);
            })
            .start();
        super::__fui_on_fetch_error(1, std::ptr::null(), 0);
        assert_eq!(&*result.borrow(), "Fetch request failed.");
        drop(request);
    }
}
