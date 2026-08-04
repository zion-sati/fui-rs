#[cfg(feature = "native-runtime")]
mod implementation {
    use fui::platform::{
        show_open_file_dialog, show_save_file_dialog, NativeFileDialogOptions,
        NativeFileDialogRequest, NativeFileDialogResult, NativeFileFilter, UiDispatcher,
    };
    use fui::{
        FileCapabilities, FileErrorEventArgs, FileSaveMode, FileSaveResult,
        FileWorkerProcessProgress, FileWorkerProcessResult,
    };
    use std::cell::RefCell;
    use std::path::{Path, PathBuf};
    use std::rc::Rc;
    use std::sync::mpsc;

    #[derive(Clone, Debug)]
    pub struct DemoPickedFile {
        path: PathBuf,
        name: String,
        size_bytes: u64,
    }

    impl DemoPickedFile {
        pub fn name(&self) -> String {
            self.name.clone()
        }

        pub fn size_bytes(&self) -> u64 {
            self.size_bytes
        }
    }

    #[must_use = "retain the guard while the native file dialog is active"]
    pub struct DemoFileRequestGuard {
        _request: Option<NativeFileDialogRequest>,
    }

    pub struct DemoFileOpenRequest {
        multiple: bool,
    }

    impl DemoFileOpenRequest {
        pub fn multiple(mut self, flag: bool) -> Self {
            self.multiple = flag;
            self
        }

        pub fn pick_with_error(
            self,
            on_complete: impl Fn(DemoFileOpenEvent) + 'static,
            on_error: Option<impl Fn(FileErrorEventArgs) + 'static>,
        ) -> DemoFileRequestGuard {
            let on_error = on_error.map(|handler| Rc::new(handler) as Rc<dyn Fn(FileErrorEventArgs)>);
            let show_error = on_error.clone();
            let request = show_open_file_dialog(
                NativeFileDialogOptions {
                    allow_multiple: self.multiple,
                    ..NativeFileDialogOptions::default()
                },
                move |result| match result {
                    NativeFileDialogResult::Selected { paths, .. } => {
                        let files = paths.into_iter().filter_map(picked_file).collect();
                        on_complete(DemoFileOpenEvent { files });
                    }
                    NativeFileDialogResult::Cancelled => on_complete(DemoFileOpenEvent {
                        files: Vec::new(),
                    }),
                    NativeFileDialogResult::Error(message) => {
                        if let Some(handler) = &on_error {
                            handler(FileErrorEventArgs { message });
                        }
                    }
                },
            );
            if request.is_none() {
                if let Some(handler) = show_error {
                    handler(FileErrorEventArgs {
                        message: String::from("Native open-file dialog could not be shown."),
                    });
                }
            }
            DemoFileRequestGuard { _request: request }
        }
    }

    pub struct DemoFileOpenEvent {
        pub files: Vec<DemoPickedFile>,
    }

    pub struct DemoFileSaveRequest {
        suggested_name: String,
        file_extension: String,
    }

    impl DemoFileSaveRequest {
        pub fn suggested_name(mut self, value: impl Into<String>) -> Self {
            self.suggested_name = value.into();
            self
        }

        pub fn mime_type(self, _value: impl Into<String>) -> Self {
            self
        }

        pub fn file_extension(mut self, value: impl Into<String>) -> Self {
            self.file_extension = value.into();
            self
        }

        pub fn save_text_with_error(
            self,
            text: impl Into<String>,
            on_complete: impl Fn(FileSaveResult) + 'static,
            on_error: Option<impl Fn(FileErrorEventArgs) + 'static>,
        ) -> DemoFileRequestGuard {
            self.save_bytes_with_error(text.into().as_bytes(), on_complete, on_error)
        }

        pub fn save_bytes_with_error(
            self,
            bytes: &[u8],
            on_complete: impl Fn(FileSaveResult) + 'static,
            on_error: Option<impl Fn(FileErrorEventArgs) + 'static>,
        ) -> DemoFileRequestGuard {
            let bytes = bytes.to_vec();
            let on_error = on_error.map(|handler| Rc::new(handler) as Rc<dyn Fn(FileErrorEventArgs)>);
            let show_error = on_error.clone();
            let suggested_path = suggested_path(&self.suggested_name, &self.file_extension);
            let request = show_save_file_dialog(
                NativeFileDialogOptions {
                    filters: file_filters(&self.file_extension),
                    default_location: Some(suggested_path),
                    ..NativeFileDialogOptions::default()
                },
                move |result| match result {
                    NativeFileDialogResult::Selected { paths, .. } => {
                        let Some(path) = paths.into_iter().next() else {
                            return;
                        };
                        match std::fs::write(&path, &bytes) {
                            Ok(()) => on_complete(FileSaveResult {
                                file_name: file_name(&path),
                                mode: FileSaveMode::NativePicker,
                                written_bytes: bytes.len() as u64,
                            }),
                            Err(error) => {
                                if let Some(handler) = &on_error {
                                    handler(FileErrorEventArgs {
                                        message: format!("Could not save {}: {error}", path.display()),
                                    });
                                }
                            }
                        }
                    }
                    NativeFileDialogResult::Cancelled => {}
                    NativeFileDialogResult::Error(message) => {
                        if let Some(handler) = &on_error {
                            handler(FileErrorEventArgs { message });
                        }
                    }
                },
            );
            if request.is_none() {
                if let Some(handler) = show_error {
                    handler(FileErrorEventArgs {
                        message: String::from("Native save-file dialog could not be shown."),
                    });
                }
            }
            DemoFileRequestGuard { _request: request }
        }
    }

    type ProgressHandler = Rc<dyn Fn(FileWorkerProcessProgress)>;
    type CompleteHandler = Rc<dyn Fn(FileWorkerProcessResult)>;
    type ErrorHandler = Rc<dyn Fn(FileErrorEventArgs)>;

    #[derive(Default)]
    struct CopyState {
        file: Option<DemoPickedFile>,
        suggested_name: String,
        on_progress: Option<ProgressHandler>,
        on_complete: Option<CompleteHandler>,
        on_error: Option<ErrorHandler>,
        dialog: Option<NativeFileDialogRequest>,
    }

    #[derive(Clone)]
    pub struct DemoFileCopyRequest {
        state: Rc<RefCell<CopyState>>,
    }

    impl DemoFileCopyRequest {
        fn new(file: DemoPickedFile) -> Self {
            Self {
                state: Rc::new(RefCell::new(CopyState {
                    suggested_name: file.name(),
                    file: Some(file),
                    ..CopyState::default()
                })),
            }
        }

        pub fn worker(self, _artifact: impl Into<String>, _entry: impl Into<String>) -> Self {
            self
        }

        pub fn save_to_picked_file(self, value: impl Into<String>) -> Self {
            self.state.borrow_mut().suggested_name = value.into();
            self
        }

        pub fn on_progress(self, handler: impl Fn(FileWorkerProcessProgress) + 'static) -> Self {
            self.state.borrow_mut().on_progress = Some(Rc::new(handler));
            self
        }

        pub fn on_complete(self, handler: impl Fn(FileWorkerProcessResult) + 'static) -> Self {
            self.state.borrow_mut().on_complete = Some(Rc::new(handler));
            self
        }

        pub fn on_error(self, handler: impl Fn(FileErrorEventArgs) + 'static) -> Self {
            self.state.borrow_mut().on_error = Some(Rc::new(handler));
            self
        }

        pub fn start(self) -> Self {
            let (source, suggested_name) = {
                let state = self.state.borrow();
                (state.file.clone(), state.suggested_name.clone())
            };
            let Some(source) = source else {
                return self;
            };
            let state = self.state.clone();
            let request = show_save_file_dialog(
                NativeFileDialogOptions {
                    default_location: Some(PathBuf::from(suggested_name)),
                    ..NativeFileDialogOptions::default()
                },
                move |result| match result {
                    NativeFileDialogResult::Selected { paths, .. } => {
                        let Some(destination) = paths.into_iter().next() else {
                            return;
                        };
                        copy_in_background(source.clone(), destination, state.clone());
                    }
                    NativeFileDialogResult::Cancelled => {}
                    NativeFileDialogResult::Error(message) => emit_copy_error(&state, message),
                },
            );
            self.state.borrow_mut().dialog = request;
            self
        }
    }

    fn copy_in_background(source: DemoPickedFile, destination: PathBuf, state: Rc<RefCell<CopyState>>) {
        let (sender, receiver) = mpsc::sync_channel(1);
        let ui_source = source.clone();
        let ui_destination = destination.clone();
        let dispatch = UiDispatcher::prepare(move || match receiver.recv() {
            Ok(Ok(written)) => {
                let state = state.borrow();
                if let Some(handler) = &state.on_progress {
                    handler(FileWorkerProcessProgress {
                        processed_bytes: written,
                        total_bytes: ui_source.size_bytes,
                        output_file_name: Some(file_name(&ui_destination)),
                    });
                }
                if let Some(handler) = &state.on_complete {
                    handler(FileWorkerProcessResult {
                        processed_bytes: written,
                        output_file_name: Some(file_name(&ui_destination)),
                        worker_result: Some(String::from("copied with native Rust filesystem I/O")),
                    });
                }
            }
            Ok(Err(message)) => emit_copy_error(&state, message),
            Err(error) => emit_copy_error(&state, format!("Native copy result failed: {error}")),
        });
        std::thread::spawn(move || {
            let result = std::fs::copy(&source.path, &destination)
                .map_err(|error| format!("Could not copy {}: {error}", source.path.display()));
            let _ = sender.send(result);
            let _ = dispatch.dispatch();
        });
    }

    fn emit_copy_error(state: &Rc<RefCell<CopyState>>, message: String) {
        if let Some(handler) = &state.borrow().on_error {
            handler(FileErrorEventArgs { message });
        }
    }

    fn picked_file(path: PathBuf) -> Option<DemoPickedFile> {
        let metadata = std::fs::metadata(&path).ok()?;
        Some(DemoPickedFile {
            name: file_name(&path),
            size_bytes: metadata.len(),
            path,
        })
    }

    fn suggested_path(name: &str, extension: &str) -> PathBuf {
        let extension = extension.trim_start_matches('.');
        if extension.is_empty() || name.ends_with(&format!(".{extension}")) {
            PathBuf::from(name)
        } else {
            PathBuf::from(format!("{name}.{extension}"))
        }
    }

    fn file_filters(extension: &str) -> Vec<NativeFileFilter> {
        let extension = extension.trim_start_matches('.');
        if extension.is_empty() {
            Vec::new()
        } else {
            vec![NativeFileFilter::new("Files", [extension])]
        }
    }

    fn file_name(path: &Path) -> String {
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("file")
            .to_owned()
    }

    pub struct DemoFile;

    impl DemoFile {
        pub fn capabilities() -> FileCapabilities {
            FileCapabilities {
                can_pick_open: true,
                can_read: true,
                can_save: true,
                can_read_chunks: true,
                can_write_chunks: true,
                can_use_native_save_picker: true,
                can_process_in_worker_to_picked_file: true,
            }
        }

        pub fn open() -> DemoFileOpenRequest {
            DemoFileOpenRequest { multiple: false }
        }

        pub fn save() -> DemoFileSaveRequest {
            DemoFileSaveRequest {
                suggested_name: String::from("file"),
                file_extension: String::new(),
            }
        }

        pub fn process_file_in_worker(file: DemoPickedFile) -> DemoFileCopyRequest {
            DemoFileCopyRequest::new(file)
        }
    }
}

#[cfg(not(feature = "native-runtime"))]
mod implementation {
    pub type DemoPickedFile = fui::BrowserFile;
    pub type DemoFileRequestGuard = fui::FileRequestGuard;
    pub type DemoFileCopyRequest = fui::FileWorkerProcessRequest;

    pub struct DemoFile;

    impl DemoFile {
        pub fn capabilities() -> fui::FileCapabilities {
            fui::File::capabilities()
        }

        pub fn open() -> fui::FileOpenRequest {
            fui::File::open()
        }

        pub fn save() -> fui::FileSaveRequest {
            fui::File::save()
        }

        pub fn process_file_in_worker(file: DemoPickedFile) -> DemoFileCopyRequest {
            fui::File::process_file_in_worker(file)
        }
    }
}

pub use implementation::{DemoFile, DemoFileCopyRequest, DemoFileRequestGuard, DemoPickedFile};
