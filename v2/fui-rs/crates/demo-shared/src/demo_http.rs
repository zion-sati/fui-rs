#[cfg(feature = "native-runtime")]
mod implementation {
    use fui::platform::UiDispatcher;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{mpsc, Arc};

    #[derive(Clone, Debug)]
    pub struct DemoHttpResponse {
        pub ok: bool,
        pub status: i32,
        pub status_text: String,
        pub url: String,
    }

    #[derive(Clone, Debug)]
    pub struct DemoHttpError {
        pub message: String,
    }

    type CompleteHandler = Rc<dyn Fn(DemoHttpResponse)>;
    type ErrorHandler = Rc<dyn Fn(DemoHttpError)>;

    pub struct DemoHttpRequest {
        cancelled: Arc<AtomicBool>,
    }

    impl Drop for DemoHttpRequest {
        fn drop(&mut self) {
            self.cancelled.store(true, Ordering::Release);
        }
    }

    pub struct DemoHttpRequestBuilder {
        method: String,
        url: String,
        headers: Vec<(String, String)>,
        body: Vec<u8>,
        on_complete: Option<CompleteHandler>,
        on_error: Option<ErrorHandler>,
    }

    impl DemoHttpRequestBuilder {
        fn new(url: impl Into<String>) -> Self {
            Self {
                method: String::from("GET"),
                url: url.into(),
                headers: Vec::new(),
                body: Vec::new(),
                on_complete: None,
                on_error: None,
            }
        }

        pub fn method(mut self, value: impl Into<String>) -> Self {
            self.method = value.into();
            self
        }

        pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
            self.headers.push((name.into(), value.into()));
            self
        }

        pub fn body_text(mut self, value: impl Into<String>) -> Self {
            self.body = value.into().into_bytes();
            self
        }

        pub fn on_complete(mut self, handler: impl Fn(DemoHttpResponse) + 'static) -> Self {
            self.on_complete = Some(Rc::new(handler));
            self
        }

        pub fn on_error(mut self, handler: impl Fn(DemoHttpError) + 'static) -> Self {
            self.on_error = Some(Rc::new(handler));
            self
        }

        pub fn start(self) -> DemoHttpRequest {
            let Self {
                method,
                url,
                headers,
                body,
                on_complete,
                on_error,
            } = self;
            let cancelled = Arc::new(AtomicBool::new(false));
            let worker_cancelled = cancelled.clone();
            let (sender, receiver) = mpsc::sync_channel(1);
            let result = Rc::new(RefCell::new(Some(receiver)));
            let dispatch = UiDispatcher::prepare({
                let result = result.clone();
                let cancelled = cancelled.clone();
                move || {
                    if cancelled.load(Ordering::Acquire) {
                        return;
                    }
                    let Some(receiver) = result.borrow_mut().take() else {
                        return;
                    };
                    match receiver.recv() {
                        Ok(Ok(response)) => {
                            if let Some(handler) = on_complete {
                                handler(response);
                            }
                        }
                        Ok(Err(error)) => {
                            if let Some(handler) = on_error {
                                handler(DemoHttpError { message: error });
                            }
                        }
                        Err(error) => {
                            if let Some(handler) = on_error {
                                handler(DemoHttpError {
                                    message: format!("Native HTTP result channel failed: {error}"),
                                });
                            }
                        }
                    }
                }
            });
            std::thread::spawn(move || {
                let response = perform_request(
                    &method,
                    &url,
                    &headers,
                    &body,
                );
                if !worker_cancelled.load(Ordering::Acquire) {
                    let _ = sender.send(response);
                    let _ = dispatch.dispatch();
                }
            });
            DemoHttpRequest { cancelled }
        }
    }

    fn perform_request(
        method: &str,
        url: &str,
        headers: &[(String, String)],
        body: &[u8],
    ) -> Result<DemoHttpResponse, String> {
        let mut request = ureq::request(method, url);
        for (name, value) in headers {
            request = request.set(name, value);
        }
        let response = if body.is_empty() {
            request.call()
        } else {
            request.send_bytes(body)
        };
        match response {
            Ok(response) | Err(ureq::Error::Status(_, response)) => Ok(DemoHttpResponse {
                ok: (200..300).contains(&response.status()),
                status: i32::from(response.status()),
                status_text: response.status_text().to_owned(),
                url: response.get_url().to_owned(),
            }),
            Err(ureq::Error::Transport(error)) => Err(error.to_string()),
        }
    }

    pub struct DemoHttp;

    impl DemoHttp {
        pub fn request(url: impl Into<String>) -> DemoHttpRequestBuilder {
            DemoHttpRequestBuilder::new(url)
        }
    }
}

#[cfg(not(feature = "native-runtime"))]
mod implementation {
    pub type DemoHttpRequest = fui::FetchRequest;

    pub struct DemoHttp;

    impl DemoHttp {
        pub fn request(url: impl Into<String>) -> fui::FetchRequest {
            fui::Fetch::request(url)
        }
    }
}

pub use implementation::{DemoHttp, DemoHttpRequest};
