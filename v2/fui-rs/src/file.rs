use crate::ffi;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

const FILE_STATUS_SUCCESS: u32 = 1;
const FILE_STATUS_CANCELLED: u32 = 2;

const FILE_CAPABILITY_OPEN: u32 = 1 << 0;
const FILE_CAPABILITY_READ: u32 = 1 << 1;
const FILE_CAPABILITY_SAVE: u32 = 1 << 2;
const FILE_CAPABILITY_CHUNKED_READ: u32 = 1 << 3;
const FILE_CAPABILITY_CHUNKED_WRITE: u32 = 1 << 4;
const FILE_CAPABILITY_NATIVE_SAVE_PICKER: u32 = 1 << 5;
const FILE_CAPABILITY_PROCESS_WORKER_SAVE: u32 = 1 << 6;

type FileOpenCallback = Rc<dyn Fn(FileOpenEventArgs)>;
type FileErrorCallback = Rc<dyn Fn(FileErrorEventArgs)>;
type FileReadCallback = Rc<dyn Fn(FileReadChunk)>;
type FileSaveCallback = Rc<dyn Fn(FileSaveResult)>;
type FileWriteCallback = Rc<dyn Fn(FileWriteProgress)>;
type FileWriterCreatedCallback = Rc<dyn Fn(BrowserFileWriter)>;
type FileWorkerChunkCallback = Rc<dyn Fn(FileReadChunk)>;
type FileWorkerProgressCallback = Rc<dyn Fn(FileWorkerProcessProgress)>;
type FileWorkerCompleteCallback = Rc<dyn Fn(FileWorkerProcessResult)>;

thread_local! {
    static NEXT_FILE_REQUEST_ID: RefCell<u32> = const { RefCell::new(1) };
    static BROWSER_FILES: RefCell<HashMap<String, Rc<RefCell<BrowserFileState>>>> = RefCell::new(HashMap::new());
    static PENDING_OPEN_REQUESTS: RefCell<HashMap<u32, PendingOpenRequest>> = RefCell::new(HashMap::new());
    static PENDING_READ_REQUESTS: RefCell<HashMap<u32, PendingReadRequest>> = RefCell::new(HashMap::new());
    static PENDING_SAVE_REQUESTS: RefCell<HashMap<u32, PendingSaveRequest>> = RefCell::new(HashMap::new());
    static PENDING_WRITER_CREATE_REQUESTS: RefCell<HashMap<u32, PendingWriterCreateRequest>> = RefCell::new(HashMap::new());
    static PENDING_WRITER_WRITE_REQUESTS: RefCell<HashMap<u32, PendingWriterWriteRequest>> = RefCell::new(HashMap::new());
    static PENDING_WRITER_FINISH_REQUESTS: RefCell<HashMap<u32, PendingWriterFinishRequest>> = RefCell::new(HashMap::new());
    static ACTIVE_WORKER_PROCESS_REQUESTS: RefCell<HashMap<u32, Rc<RefCell<FileWorkerProcessRequestState>>>> = RefCell::new(HashMap::new());
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

fn read_utf8(ptr: *const u8, len: u32) -> String {
    if ptr.is_null() || len == 0 {
        return String::new();
    }
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len as usize) };
    String::from_utf8_lossy(bytes).into_owned()
}

fn next_request_id() -> u32 {
    NEXT_FILE_REQUEST_ID.with(|slot| {
        let mut slot = slot.borrow_mut();
        let id = *slot;
        *slot += 1;
        id
    })
}

fn describe_file_failure(status: u32, fallback: &str) -> String {
    if status == FILE_STATUS_CANCELLED {
        return "File operation was cancelled.".to_string();
    }
    fallback.to_string()
}

fn dispatch_file_error(callback: Option<FileErrorCallback>, message: String) {
    if let Some(callback) = callback {
        callback(FileErrorEventArgs { message });
        return;
    }
    crate::logger::warn("File", &message);
}

fn decode_length_prefixed_text(bytes: &[u8], cursor: &mut usize) -> Option<String> {
    if *cursor + 4 > bytes.len() {
        return None;
    }
    let len = u32::from_le_bytes(bytes[*cursor..*cursor + 4].try_into().ok()?) as usize;
    *cursor += 4;
    if *cursor + len > bytes.len() {
        return None;
    }
    let value = String::from_utf8_lossy(&bytes[*cursor..*cursor + len]).into_owned();
    *cursor += len;
    Some(value)
}

fn decode_file_list_payload(bytes: &[u8]) -> Vec<BrowserFile> {
    if bytes.len() < 4 {
        return Vec::new();
    }
    let mut cursor = 0usize;
    let count = u32::from_le_bytes(bytes[cursor..cursor + 4].try_into().unwrap_or([0; 4])) as usize;
    cursor += 4;
    let mut files = Vec::with_capacity(count);
    for _ in 0..count {
        let Some(id) = decode_length_prefixed_text(bytes, &mut cursor) else {
            break;
        };
        if cursor + 16 > bytes.len() {
            break;
        }
        let size_bytes = u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().unwrap_or([0; 8]));
        cursor += 8;
        let last_modified_ms =
            u64::from_le_bytes(bytes[cursor..cursor + 8].try_into().unwrap_or([0; 8]));
        cursor += 8;
        let Some(name) = decode_length_prefixed_text(bytes, &mut cursor) else {
            break;
        };
        let Some(mime_type) = decode_length_prefixed_text(bytes, &mut cursor) else {
            break;
        };
        files.push(register_browser_file(
            id,
            name,
            if mime_type.is_empty() {
                None
            } else {
                Some(mime_type)
            },
            size_bytes,
            last_modified_ms,
        ));
    }
    files
}

fn decode_writer_payload(bytes: &[u8]) -> Option<(FileSaveMode, String, Option<String>)> {
    if bytes.len() < 4 {
        return None;
    }
    let mut cursor = 0usize;
    let mode_raw = u32::from_le_bytes(bytes[cursor..cursor + 4].try_into().ok()?);
    cursor += 4;
    let first = decode_length_prefixed_text(bytes, &mut cursor)?;
    let second = if cursor < bytes.len() {
        decode_length_prefixed_text(bytes, &mut cursor)
    } else {
        None
    };
    Some((FileSaveMode::from_raw(mode_raw), first, second))
}

fn split_worker_process_complete_payload(text: String) -> (Option<String>, Option<String>) {
    let mut parts = text.splitn(2, '\0');
    let output_file_name = parts.next().unwrap_or_default().to_string();
    let worker_result = parts.next().unwrap_or_default().to_string();
    (
        if output_file_name.is_empty() {
            None
        } else {
            Some(output_file_name)
        },
        if worker_result.is_empty() {
            None
        } else {
            Some(worker_result)
        },
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FileRequestKind {
    Open,
    Read,
    Save,
    WriterCreate,
    WriterWrite,
    WriterFinish,
}

#[must_use = "dropping the guard unregisters the pending file callback"]
pub struct FileRequestGuard {
    kind: FileRequestKind,
    request_id: u32,
}

impl FileRequestGuard {
    fn inactive(kind: FileRequestKind) -> Self {
        Self {
            kind,
            request_id: 0,
        }
    }

    pub fn cancel(&mut self) {
        if self.request_id == 0 {
            return;
        }
        let request_id = self.request_id;
        self.request_id = 0;
        match self.kind {
            FileRequestKind::Open => {
                PENDING_OPEN_REQUESTS.with(|requests| {
                    requests.borrow_mut().remove(&request_id);
                });
            }
            FileRequestKind::Read => {
                PENDING_READ_REQUESTS.with(|requests| {
                    requests.borrow_mut().remove(&request_id);
                });
            }
            FileRequestKind::Save => {
                PENDING_SAVE_REQUESTS.with(|requests| {
                    requests.borrow_mut().remove(&request_id);
                });
            }
            FileRequestKind::WriterCreate => {
                PENDING_WRITER_CREATE_REQUESTS.with(|requests| {
                    requests.borrow_mut().remove(&request_id);
                });
            }
            FileRequestKind::WriterWrite => {
                PENDING_WRITER_WRITE_REQUESTS.with(|requests| {
                    requests.borrow_mut().remove(&request_id);
                });
            }
            FileRequestKind::WriterFinish => {
                PENDING_WRITER_FINISH_REQUESTS.with(|requests| {
                    requests.borrow_mut().remove(&request_id);
                });
            }
        }
    }
}

impl Drop for FileRequestGuard {
    fn drop(&mut self) {
        self.cancel();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileSaveMode {
    Download = 1,
    NativePicker = 2,
}

impl FileSaveMode {
    fn from_raw(value: u32) -> Self {
        match value {
            2 => Self::NativePicker,
            _ => Self::Download,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileCapabilities {
    pub can_pick_open: bool,
    pub can_read: bool,
    pub can_save: bool,
    pub can_read_chunks: bool,
    pub can_write_chunks: bool,
    pub can_use_native_save_picker: bool,
    pub can_process_in_worker_to_picked_file: bool,
}

impl FileCapabilities {
    fn from_bits(bits: u32) -> Self {
        Self {
            can_pick_open: (bits & FILE_CAPABILITY_OPEN) != 0,
            can_read: (bits & FILE_CAPABILITY_READ) != 0,
            can_save: (bits & FILE_CAPABILITY_SAVE) != 0,
            can_read_chunks: (bits & FILE_CAPABILITY_CHUNKED_READ) != 0,
            can_write_chunks: (bits & FILE_CAPABILITY_CHUNKED_WRITE) != 0,
            can_use_native_save_picker: (bits & FILE_CAPABILITY_NATIVE_SAVE_PICKER) != 0,
            can_process_in_worker_to_picked_file: (bits & FILE_CAPABILITY_PROCESS_WORKER_SAVE) != 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileOpenEventArgs {
    pub files: Vec<BrowserFile>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileErrorEventArgs {
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileReadChunk {
    pub offset_bytes: u64,
    pub file_size_bytes: u64,
    pub bytes: Vec<u8>,
}

impl FileReadChunk {
    pub fn next_offset_bytes(&self) -> u64 {
        self.offset_bytes + self.bytes.len() as u64
    }

    pub fn reached_eof(&self) -> bool {
        self.next_offset_bytes() >= self.file_size_bytes
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileWriteProgress {
    pub written_bytes: u64,
    pub total_written_bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileSaveResult {
    pub file_name: String,
    pub mode: FileSaveMode,
    pub written_bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileWorkerProcessProgress {
    pub processed_bytes: u64,
    pub total_bytes: u64,
    pub output_file_name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileWorkerProcessResult {
    pub processed_bytes: u64,
    pub output_file_name: Option<String>,
    pub worker_result: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
struct BrowserFileState {
    id: String,
    name: String,
    mime_type: Option<String>,
    size_bytes: u64,
    last_modified_ms: u64,
}

#[derive(Clone, Debug)]
pub struct BrowserFile {
    inner: Rc<RefCell<BrowserFileState>>,
}

impl Default for BrowserFile {
    fn default() -> Self {
        Self {
            inner: Rc::new(RefCell::new(BrowserFileState {
                id: String::new(),
                name: String::new(),
                mime_type: None,
                size_bytes: 0,
                last_modified_ms: 0,
            })),
        }
    }
}

impl PartialEq for BrowserFile {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Eq for BrowserFile {}

impl BrowserFile {
    pub fn id(&self) -> String {
        self.inner.borrow().id.clone()
    }

    pub fn name(&self) -> String {
        self.inner.borrow().name.clone()
    }

    pub fn mime_type(&self) -> Option<String> {
        self.inner.borrow().mime_type.clone()
    }

    pub fn size_bytes(&self) -> u64 {
        self.inner.borrow().size_bytes
    }

    pub fn last_modified_ms(&self) -> u64 {
        self.inner.borrow().last_modified_ms
    }

    pub fn read_bytes_chunk(
        &self,
        offset_bytes: u64,
        max_bytes: u32,
        on_complete: impl Fn(FileReadChunk) + 'static,
    ) -> FileRequestGuard {
        self.read_bytes_chunk_with_error(
            offset_bytes,
            max_bytes,
            on_complete,
            None::<fn(FileErrorEventArgs)>,
        )
    }

    pub fn read_bytes_chunk_with_error(
        &self,
        offset_bytes: u64,
        max_bytes: u32,
        on_complete: impl Fn(FileReadChunk) + 'static,
        on_error: Option<impl Fn(FileErrorEventArgs) + 'static>,
    ) -> FileRequestGuard {
        if max_bytes == 0 {
            dispatch_file_error(
                on_error.map(|handler| Rc::new(handler) as FileErrorCallback),
                "BrowserFile.read_bytes_chunk: max_bytes must be greater than zero.".to_string(),
            );
            return FileRequestGuard::inactive(FileRequestKind::Read);
        }
        let request_id = next_request_id();
        PENDING_READ_REQUESTS.with(|requests| {
            requests.borrow_mut().insert(
                request_id,
                PendingReadRequest {
                    on_complete: Rc::new(on_complete),
                    on_error: on_error.map(|handler| Rc::new(handler) as FileErrorCallback),
                },
            );
        });
        let file_id = self.id();
        with_utf8(&file_id, |file_id_ptr, file_id_len| unsafe {
            ffi::fui_file_read_chunk(
                request_id,
                file_id_ptr,
                file_id_len,
                offset_bytes,
                max_bytes,
            );
        });
        FileRequestGuard {
            kind: FileRequestKind::Read,
            request_id,
        }
    }
}

struct PendingOpenRequest {
    on_complete: FileOpenCallback,
    on_error: Option<FileErrorCallback>,
}

struct PendingReadRequest {
    on_complete: FileReadCallback,
    on_error: Option<FileErrorCallback>,
}

struct PendingSaveRequest {
    on_complete: FileSaveCallback,
    on_error: Option<FileErrorCallback>,
}

struct PendingWriterCreateRequest {
    on_complete: FileWriterCreatedCallback,
    on_error: Option<FileErrorCallback>,
}

struct PendingWriterWriteRequest {
    on_complete: FileWriteCallback,
    on_error: Option<FileErrorCallback>,
}

struct PendingWriterFinishRequest {
    on_complete: FileSaveCallback,
    on_error: Option<FileErrorCallback>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrowserFileWriter {
    writer_id: String,
    pub file_name: String,
    pub mode: FileSaveMode,
}

impl BrowserFileWriter {
    pub fn write_text_chunk(
        &self,
        text: impl Into<String>,
        on_complete: impl Fn(FileWriteProgress) + 'static,
    ) -> FileRequestGuard {
        self.write_text_chunk_with_error(text, on_complete, None::<fn(FileErrorEventArgs)>)
    }

    pub fn write_text_chunk_with_error(
        &self,
        text: impl Into<String>,
        on_complete: impl Fn(FileWriteProgress) + 'static,
        on_error: Option<impl Fn(FileErrorEventArgs) + 'static>,
    ) -> FileRequestGuard {
        let text = text.into();
        let request_id = next_request_id();
        PENDING_WRITER_WRITE_REQUESTS.with(|requests| {
            requests.borrow_mut().insert(
                request_id,
                PendingWriterWriteRequest {
                    on_complete: Rc::new(on_complete),
                    on_error: on_error.map(|handler| Rc::new(handler) as FileErrorCallback),
                },
            );
        });
        with_utf8(&self.writer_id, |writer_id_ptr, writer_id_len| {
            with_utf8(&text, |text_ptr, text_len| unsafe {
                ffi::fui_file_writer_write_text(
                    request_id,
                    writer_id_ptr,
                    writer_id_len,
                    text_ptr,
                    text_len,
                );
            })
        });
        FileRequestGuard {
            kind: FileRequestKind::WriterWrite,
            request_id,
        }
    }

    pub fn write_bytes_chunk(
        &self,
        bytes: &[u8],
        on_complete: impl Fn(FileWriteProgress) + 'static,
    ) -> FileRequestGuard {
        self.write_bytes_chunk_with_error(bytes, on_complete, None::<fn(FileErrorEventArgs)>)
    }

    pub fn write_bytes_chunk_with_error(
        &self,
        bytes: &[u8],
        on_complete: impl Fn(FileWriteProgress) + 'static,
        on_error: Option<impl Fn(FileErrorEventArgs) + 'static>,
    ) -> FileRequestGuard {
        let owned = bytes.to_vec();
        let request_id = next_request_id();
        PENDING_WRITER_WRITE_REQUESTS.with(|requests| {
            requests.borrow_mut().insert(
                request_id,
                PendingWriterWriteRequest {
                    on_complete: Rc::new(on_complete),
                    on_error: on_error.map(|handler| Rc::new(handler) as FileErrorCallback),
                },
            );
        });
        with_utf8(&self.writer_id, |writer_id_ptr, writer_id_len| unsafe {
            ffi::fui_file_writer_write_bytes(
                request_id,
                writer_id_ptr,
                writer_id_len,
                if owned.is_empty() {
                    0
                } else {
                    owned.as_ptr() as usize
                },
                owned.len() as u32,
            );
        });
        FileRequestGuard {
            kind: FileRequestKind::WriterWrite,
            request_id,
        }
    }

    pub fn finish(&self, on_complete: impl Fn(FileSaveResult) + 'static) -> FileRequestGuard {
        self.finish_with_error(on_complete, None::<fn(FileErrorEventArgs)>)
    }

    pub fn finish_with_error(
        &self,
        on_complete: impl Fn(FileSaveResult) + 'static,
        on_error: Option<impl Fn(FileErrorEventArgs) + 'static>,
    ) -> FileRequestGuard {
        let request_id = next_request_id();
        PENDING_WRITER_FINISH_REQUESTS.with(|requests| {
            requests.borrow_mut().insert(
                request_id,
                PendingWriterFinishRequest {
                    on_complete: Rc::new(on_complete),
                    on_error: on_error.map(|handler| Rc::new(handler) as FileErrorCallback),
                },
            );
        });
        with_utf8(&self.writer_id, |writer_id_ptr, writer_id_len| unsafe {
            ffi::fui_file_writer_finish(request_id, writer_id_ptr, writer_id_len);
        });
        FileRequestGuard {
            kind: FileRequestKind::WriterFinish,
            request_id,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct FileOpenRequest {
    accept: String,
    multiple: bool,
}

impl FileOpenRequest {
    pub fn accept(mut self, value: impl Into<String>) -> Self {
        self.accept = value.into();
        self
    }

    pub fn multiple(mut self, flag: bool) -> Self {
        self.multiple = flag;
        self
    }

    pub fn pick(self, on_complete: impl Fn(FileOpenEventArgs) + 'static) -> FileRequestGuard {
        self.pick_with_error(on_complete, None::<fn(FileErrorEventArgs)>)
    }

    pub fn pick_with_error(
        self,
        on_complete: impl Fn(FileOpenEventArgs) + 'static,
        on_error: Option<impl Fn(FileErrorEventArgs) + 'static>,
    ) -> FileRequestGuard {
        let request_id = next_request_id();
        PENDING_OPEN_REQUESTS.with(|requests| {
            requests.borrow_mut().insert(
                request_id,
                PendingOpenRequest {
                    on_complete: Rc::new(on_complete),
                    on_error: on_error.map(|handler| Rc::new(handler) as FileErrorCallback),
                },
            );
        });
        with_utf8(&self.accept, |accept_ptr, accept_len| unsafe {
            ffi::fui_file_pick(request_id, accept_ptr, accept_len, self.multiple);
        });
        FileRequestGuard {
            kind: FileRequestKind::Open,
            request_id,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct FileSaveRequest {
    suggested_name: String,
    mime_type: String,
    file_extension: String,
}

impl FileSaveRequest {
    pub fn suggested_name(mut self, value: impl Into<String>) -> Self {
        self.suggested_name = value.into();
        self
    }

    pub fn mime_type(mut self, value: impl Into<String>) -> Self {
        self.mime_type = value.into();
        self
    }

    pub fn file_extension(mut self, value: impl Into<String>) -> Self {
        self.file_extension = value.into();
        self
    }

    pub fn save_text(
        self,
        text: impl Into<String>,
        on_complete: impl Fn(FileSaveResult) + 'static,
    ) -> FileRequestGuard {
        self.save_text_with_error(text, on_complete, None::<fn(FileErrorEventArgs)>)
    }

    pub fn save_text_with_error(
        self,
        text: impl Into<String>,
        on_complete: impl Fn(FileSaveResult) + 'static,
        on_error: Option<impl Fn(FileErrorEventArgs) + 'static>,
    ) -> FileRequestGuard {
        let text = text.into();
        let request_id = next_request_id();
        PENDING_SAVE_REQUESTS.with(|requests| {
            requests.borrow_mut().insert(
                request_id,
                PendingSaveRequest {
                    on_complete: Rc::new(on_complete),
                    on_error: on_error.map(|handler| Rc::new(handler) as FileErrorCallback),
                },
            );
        });
        with_utf8(
            &self.suggested_name,
            |suggested_name_ptr, suggested_name_len| {
                with_utf8(&self.mime_type, |mime_type_ptr, mime_type_len| {
                    with_utf8(
                        &self.file_extension,
                        |file_extension_ptr, file_extension_len| {
                            with_utf8(&text, |text_ptr, text_len| unsafe {
                                ffi::fui_file_save_text(
                                    request_id,
                                    suggested_name_ptr,
                                    suggested_name_len,
                                    mime_type_ptr,
                                    mime_type_len,
                                    file_extension_ptr,
                                    file_extension_len,
                                    text_ptr,
                                    text_len,
                                );
                            })
                        },
                    )
                })
            },
        );
        FileRequestGuard {
            kind: FileRequestKind::Save,
            request_id,
        }
    }

    pub fn save_bytes(
        self,
        bytes: &[u8],
        on_complete: impl Fn(FileSaveResult) + 'static,
    ) -> FileRequestGuard {
        self.save_bytes_with_error(bytes, on_complete, None::<fn(FileErrorEventArgs)>)
    }

    pub fn save_bytes_with_error(
        self,
        bytes: &[u8],
        on_complete: impl Fn(FileSaveResult) + 'static,
        on_error: Option<impl Fn(FileErrorEventArgs) + 'static>,
    ) -> FileRequestGuard {
        let owned = bytes.to_vec();
        let request_id = next_request_id();
        PENDING_SAVE_REQUESTS.with(|requests| {
            requests.borrow_mut().insert(
                request_id,
                PendingSaveRequest {
                    on_complete: Rc::new(on_complete),
                    on_error: on_error.map(|handler| Rc::new(handler) as FileErrorCallback),
                },
            );
        });
        with_utf8(
            &self.suggested_name,
            |suggested_name_ptr, suggested_name_len| {
                with_utf8(&self.mime_type, |mime_type_ptr, mime_type_len| {
                    with_utf8(
                        &self.file_extension,
                        |file_extension_ptr, file_extension_len| unsafe {
                            ffi::fui_file_save_bytes(
                                request_id,
                                suggested_name_ptr,
                                suggested_name_len,
                                mime_type_ptr,
                                mime_type_len,
                                file_extension_ptr,
                                file_extension_len,
                                if owned.is_empty() {
                                    0
                                } else {
                                    owned.as_ptr() as usize
                                },
                                owned.len() as u32,
                            );
                        },
                    )
                })
            },
        );
        FileRequestGuard {
            kind: FileRequestKind::Save,
            request_id,
        }
    }

    pub fn create_writer(
        self,
        on_complete: impl Fn(BrowserFileWriter) + 'static,
    ) -> FileRequestGuard {
        self.create_writer_with_error(on_complete, None::<fn(FileErrorEventArgs)>)
    }

    pub fn create_writer_with_error(
        self,
        on_complete: impl Fn(BrowserFileWriter) + 'static,
        on_error: Option<impl Fn(FileErrorEventArgs) + 'static>,
    ) -> FileRequestGuard {
        let request_id = next_request_id();
        PENDING_WRITER_CREATE_REQUESTS.with(|requests| {
            requests.borrow_mut().insert(
                request_id,
                PendingWriterCreateRequest {
                    on_complete: Rc::new(on_complete),
                    on_error: on_error.map(|handler| Rc::new(handler) as FileErrorCallback),
                },
            );
        });
        with_utf8(
            &self.suggested_name,
            |suggested_name_ptr, suggested_name_len| {
                with_utf8(&self.mime_type, |mime_type_ptr, mime_type_len| {
                    with_utf8(
                        &self.file_extension,
                        |file_extension_ptr, file_extension_len| unsafe {
                            ffi::fui_file_create_writer(
                                request_id,
                                suggested_name_ptr,
                                suggested_name_len,
                                mime_type_ptr,
                                mime_type_len,
                                file_extension_ptr,
                                file_extension_len,
                            );
                        },
                    )
                })
            },
        );
        FileRequestGuard {
            kind: FileRequestKind::WriterCreate,
            request_id,
        }
    }
}

#[derive(Default)]
struct FileWorkerProcessRequestState {
    file: BrowserFile,
    worker_wasm_path: String,
    worker_entry_name: String,
    suggested_name: String,
    save_to_picked_file: bool,
    chunk_bytes: u32,
    on_chunk: Option<FileWorkerChunkCallback>,
    on_progress: Option<FileWorkerProgressCallback>,
    on_complete: Option<FileWorkerCompleteCallback>,
    on_error: Option<FileErrorCallback>,
    request_id: u32,
    started: bool,
    finished: bool,
}

#[derive(Clone, Default)]
pub struct FileWorkerProcessRequest {
    inner: Rc<RefCell<FileWorkerProcessRequestState>>,
}

impl FileWorkerProcessRequest {
    fn new(file: BrowserFile) -> Self {
        Self {
            inner: Rc::new(RefCell::new(FileWorkerProcessRequestState {
                suggested_name: file.name(),
                file,
                chunk_bytes: 64 * 1024,
                ..FileWorkerProcessRequestState::default()
            })),
        }
    }

    pub fn suggested_name(self, value: impl Into<String>) -> Self {
        self.inner.borrow_mut().suggested_name = value.into();
        self
    }

    pub fn worker(self, wasm_path: impl Into<String>, entry_name: impl Into<String>) -> Self {
        let mut inner = self.inner.borrow_mut();
        inner.worker_wasm_path = wasm_path.into();
        inner.worker_entry_name = entry_name.into();
        drop(inner);
        self
    }

    pub fn save_to_picked_file(self, value: impl Into<String>) -> Self {
        let mut inner = self.inner.borrow_mut();
        inner.save_to_picked_file = true;
        inner.suggested_name = value.into();
        drop(inner);
        self
    }

    pub fn chunk_bytes(self, value: u32) -> Self {
        self.inner.borrow_mut().chunk_bytes = value;
        self
    }

    pub fn on_chunk(self, handler: impl Fn(FileReadChunk) + 'static) -> Self {
        self.inner.borrow_mut().on_chunk = Some(Rc::new(handler));
        self
    }

    pub fn on_progress(self, handler: impl Fn(FileWorkerProcessProgress) + 'static) -> Self {
        self.inner.borrow_mut().on_progress = Some(Rc::new(handler));
        self
    }

    pub fn on_complete(self, handler: impl Fn(FileWorkerProcessResult) + 'static) -> Self {
        self.inner.borrow_mut().on_complete = Some(Rc::new(handler));
        self
    }

    pub fn on_error(self, handler: impl Fn(FileErrorEventArgs) + 'static) -> Self {
        self.inner.borrow_mut().on_error = Some(Rc::new(handler));
        self
    }

    pub fn start(self) -> Self {
        let (
            request_id,
            worker_wasm_path,
            worker_entry_name,
            file_id,
            suggested_name,
            chunk_bytes,
            save_to_picked_file,
            on_chunk,
            on_error,
            already_finished,
            already_started,
        ) = {
            let inner = self.inner.borrow();
            (
                inner.request_id,
                inner.worker_wasm_path.clone(),
                inner.worker_entry_name.clone(),
                inner.file.id(),
                inner.suggested_name.clone(),
                inner.chunk_bytes,
                inner.save_to_picked_file,
                inner.on_chunk.is_some(),
                inner.on_error.clone(),
                inner.finished,
                inner.started,
            )
        };
        if already_finished {
            crate::logger::warn(
                "File",
                "FileWorkerProcessRequest.start ignored after the worker process already finished.",
            );
            return self;
        }
        if already_started {
            crate::logger::warn(
                "File",
                "FileWorkerProcessRequest.start ignored because the request already started.",
            );
            return self;
        }
        if chunk_bytes == 0 {
            dispatch_file_error(
                on_error,
                "FileWorkerProcessRequest.start: chunk_bytes must be greater than zero."
                    .to_string(),
            );
            return self;
        }
        if worker_wasm_path.is_empty() || worker_entry_name.is_empty() {
            dispatch_file_error(
                on_error,
                "FileWorkerProcessRequest.start: worker(wasm_path, entry_name) is required."
                    .to_string(),
            );
            return self;
        }
        if !save_to_picked_file && !on_chunk {
            dispatch_file_error(
                on_error,
                "FileWorkerProcessRequest.start: either save_to_picked_file(...) or on_chunk(...) is required.".to_string(),
            );
            return self;
        }
        let request_id = if request_id == 0 {
            let request_id = next_request_id();
            let mut inner = self.inner.borrow_mut();
            inner.request_id = request_id;
            inner.started = true;
            drop(inner);
            ACTIVE_WORKER_PROCESS_REQUESTS.with(|requests| {
                requests.borrow_mut().insert(request_id, self.inner.clone());
            });
            request_id
        } else {
            request_id
        };
        with_utf8(
            &worker_wasm_path,
            |worker_wasm_path_ptr, worker_wasm_path_len| {
                with_utf8(&worker_entry_name, |worker_entry_ptr, worker_entry_len| {
                    with_utf8(&file_id, |file_id_ptr, file_id_len| {
                        with_utf8(
                            &suggested_name,
                            |suggested_name_ptr, suggested_name_len| unsafe {
                                ffi::fui_file_process_worker_start(
                                    request_id,
                                    worker_wasm_path_ptr,
                                    worker_wasm_path_len,
                                    worker_entry_ptr,
                                    worker_entry_len,
                                    file_id_ptr,
                                    file_id_len,
                                    suggested_name_ptr,
                                    suggested_name_len,
                                    chunk_bytes,
                                    save_to_picked_file,
                                );
                            },
                        )
                    })
                })
            },
        );
        self
    }

    pub fn cancel(&self) {
        let request_id = {
            let mut inner = self.inner.borrow_mut();
            if inner.finished {
                return;
            }
            let request_id = inner.request_id;
            if request_id == 0 {
                inner.finished = true;
                return;
            }
            inner.request_id = 0;
            inner.started = false;
            inner.finished = true;
            inner.on_chunk = None;
            inner.on_progress = None;
            inner.on_complete = None;
            inner.on_error = None;
            request_id
        };
        ACTIVE_WORKER_PROCESS_REQUESTS.with(|requests| {
            requests.borrow_mut().remove(&request_id);
        });
        unsafe {
            ffi::fui_file_process_worker_cancel(request_id);
        }
    }
}

impl Drop for FileWorkerProcessRequest {
    fn drop(&mut self) {
        self.cancel();
    }
}

pub struct File;

impl File {
    pub fn open() -> FileOpenRequest {
        FileOpenRequest::default()
    }

    pub fn save() -> FileSaveRequest {
        FileSaveRequest::default()
    }

    pub fn process_file_in_worker(file: BrowserFile) -> FileWorkerProcessRequest {
        FileWorkerProcessRequest::new(file)
    }

    pub fn capabilities() -> FileCapabilities {
        FileCapabilities::from_bits(unsafe { ffi::fui_file_capabilities() })
    }

    pub fn try_get_file(id: &str) -> Option<BrowserFile> {
        BROWSER_FILES.with(|files| {
            files
                .borrow()
                .get(id)
                .cloned()
                .map(|inner| BrowserFile { inner })
        })
    }
}

pub(crate) fn register_browser_file(
    id: String,
    name: String,
    mime_type: Option<String>,
    size_bytes: u64,
    last_modified_ms: u64,
) -> BrowserFile {
    BROWSER_FILES.with(|files| {
        let mut files = files.borrow_mut();
        if let Some(existing) = files.get(&id).cloned() {
            let mut existing = existing.borrow_mut();
            existing.name = name;
            existing.mime_type = mime_type;
            existing.size_bytes = size_bytes;
            existing.last_modified_ms = last_modified_ms;
            drop(existing);
            return BrowserFile {
                inner: files.get(&id).unwrap().clone(),
            };
        }
        let inner = Rc::new(RefCell::new(BrowserFileState {
            id: id.clone(),
            name,
            mime_type,
            size_bytes,
            last_modified_ms,
        }));
        files.insert(id, inner.clone());
        BrowserFile { inner }
    })
}

fn finish_worker_process(
    request_id: u32,
) -> Option<(
    Option<FileWorkerCompleteCallback>,
    Option<FileErrorCallback>,
)> {
    let request = ACTIVE_WORKER_PROCESS_REQUESTS
        .with(|requests| requests.borrow_mut().remove(&request_id))?;
    let mut inner = request.borrow_mut();
    if inner.finished {
        return None;
    }
    inner.started = false;
    inner.finished = true;
    inner.request_id = 0;
    let on_complete = inner.on_complete.take();
    let on_error = inner.on_error.take();
    inner.on_chunk = None;
    inner.on_progress = None;
    Some((on_complete, on_error))
}

pub fn reset_file_runtime() {
    let active_requests = ACTIVE_WORKER_PROCESS_REQUESTS
        .with(|requests| requests.borrow().values().cloned().collect::<Vec<_>>());
    for request in active_requests {
        FileWorkerProcessRequest { inner: request }.cancel();
    }
    BROWSER_FILES.with(|files| files.borrow_mut().clear());
    PENDING_OPEN_REQUESTS.with(|requests| requests.borrow_mut().clear());
    PENDING_READ_REQUESTS.with(|requests| requests.borrow_mut().clear());
    PENDING_SAVE_REQUESTS.with(|requests| requests.borrow_mut().clear());
    PENDING_WRITER_CREATE_REQUESTS.with(|requests| requests.borrow_mut().clear());
    PENDING_WRITER_WRITE_REQUESTS.with(|requests| requests.borrow_mut().clear());
    PENDING_WRITER_FINISH_REQUESTS.with(|requests| requests.borrow_mut().clear());
    ACTIVE_WORKER_PROCESS_REQUESTS.with(|requests| requests.borrow_mut().clear());
    NEXT_FILE_REQUEST_ID.with(|slot| *slot.borrow_mut() = 1);
}

#[cfg_attr(not(feature = "worker-runtime"), no_mangle)]
/// # Safety
/// `payload_ptr` must be null for an empty payload or point to `payload_len` readable bytes.
pub unsafe extern "C" fn __fui_on_file_pick_result(
    request_id: u32,
    status: u32,
    payload_ptr: *const u8,
    payload_len: u32,
) {
    let request = PENDING_OPEN_REQUESTS.with(|requests| requests.borrow_mut().remove(&request_id));
    let Some(request) = request else {
        return;
    };
    if status == FILE_STATUS_SUCCESS {
        let payload = if payload_ptr.is_null() || payload_len == 0 {
            &[][..]
        } else {
            unsafe { std::slice::from_raw_parts(payload_ptr, payload_len as usize) }
        };
        let files = decode_file_list_payload(payload);
        (request.on_complete)(FileOpenEventArgs { files });
        return;
    }
    dispatch_file_error(
        request.on_error,
        if payload_ptr.is_null() || payload_len == 0 {
            describe_file_failure(status, "File picker failed.")
        } else {
            read_utf8(payload_ptr, payload_len)
        },
    );
}

#[cfg_attr(not(feature = "worker-runtime"), no_mangle)]
/// # Safety
/// `payload_ptr` must be null for an empty payload or point to `payload_len` readable bytes.
pub unsafe extern "C" fn __fui_on_file_read_result(
    request_id: u32,
    status: u32,
    offset_bytes: u64,
    file_size_bytes: u64,
    payload_ptr: *const u8,
    payload_len: u32,
) {
    let request = PENDING_READ_REQUESTS.with(|requests| requests.borrow_mut().remove(&request_id));
    let Some(request) = request else {
        return;
    };
    if status == FILE_STATUS_SUCCESS {
        let bytes = if payload_ptr.is_null() || payload_len == 0 {
            Vec::new()
        } else {
            unsafe { std::slice::from_raw_parts(payload_ptr, payload_len as usize) }.to_vec()
        };
        (request.on_complete)(FileReadChunk {
            offset_bytes,
            file_size_bytes,
            bytes,
        });
        return;
    }
    dispatch_file_error(
        request.on_error,
        if payload_ptr.is_null() || payload_len == 0 {
            describe_file_failure(status, "File read failed.")
        } else {
            read_utf8(payload_ptr, payload_len)
        },
    );
}

#[cfg_attr(not(feature = "worker-runtime"), no_mangle)]
/// # Safety
/// `payload_ptr` must be null for an empty payload or point to `payload_len` readable bytes.
pub unsafe extern "C" fn __fui_on_file_save_result(
    request_id: u32,
    status: u32,
    written_bytes: u64,
    payload_ptr: *const u8,
    payload_len: u32,
) {
    let request = PENDING_SAVE_REQUESTS.with(|requests| requests.borrow_mut().remove(&request_id));
    let Some(request) = request else {
        return;
    };
    if status == FILE_STATUS_SUCCESS {
        let payload = if payload_ptr.is_null() || payload_len == 0 {
            &[][..]
        } else {
            unsafe { std::slice::from_raw_parts(payload_ptr, payload_len as usize) }
        };
        if let Some((mode, file_name, _)) = decode_writer_payload(payload) {
            (request.on_complete)(FileSaveResult {
                file_name,
                mode,
                written_bytes,
            });
            return;
        }
    }
    dispatch_file_error(
        request.on_error,
        if payload_ptr.is_null() || payload_len == 0 {
            describe_file_failure(status, "File save failed.")
        } else {
            read_utf8(payload_ptr, payload_len)
        },
    );
}

#[cfg_attr(not(feature = "worker-runtime"), no_mangle)]
/// # Safety
/// `payload_ptr` must be null for an empty payload or point to `payload_len` readable bytes.
pub unsafe extern "C" fn __fui_on_file_writer_created(
    request_id: u32,
    status: u32,
    payload_ptr: *const u8,
    payload_len: u32,
) {
    let request =
        PENDING_WRITER_CREATE_REQUESTS.with(|requests| requests.borrow_mut().remove(&request_id));
    let Some(request) = request else {
        return;
    };
    if status == FILE_STATUS_SUCCESS {
        let payload = if payload_ptr.is_null() || payload_len == 0 {
            &[][..]
        } else {
            unsafe { std::slice::from_raw_parts(payload_ptr, payload_len as usize) }
        };
        if let Some((mode, writer_id, Some(file_name))) = decode_writer_payload(payload) {
            (request.on_complete)(BrowserFileWriter {
                writer_id,
                file_name,
                mode,
            });
            return;
        }
    }
    dispatch_file_error(
        request.on_error,
        if payload_ptr.is_null() || payload_len == 0 {
            describe_file_failure(status, "Creating a file writer failed.")
        } else {
            read_utf8(payload_ptr, payload_len)
        },
    );
}

#[cfg_attr(not(feature = "worker-runtime"), no_mangle)]
/// # Safety
/// `payload_ptr` must be null for an empty payload or point to `payload_len` readable bytes.
pub unsafe extern "C" fn __fui_on_file_write_result(
    request_id: u32,
    status: u32,
    written_bytes: u64,
    total_written_bytes: u64,
    payload_ptr: *const u8,
    payload_len: u32,
) {
    let request =
        PENDING_WRITER_WRITE_REQUESTS.with(|requests| requests.borrow_mut().remove(&request_id));
    let Some(request) = request else {
        return;
    };
    if status == FILE_STATUS_SUCCESS {
        (request.on_complete)(FileWriteProgress {
            written_bytes,
            total_written_bytes,
        });
        return;
    }
    dispatch_file_error(
        request.on_error,
        if payload_ptr.is_null() || payload_len == 0 {
            describe_file_failure(status, "File write failed.")
        } else {
            read_utf8(payload_ptr, payload_len)
        },
    );
}

#[cfg_attr(not(feature = "worker-runtime"), no_mangle)]
/// # Safety
/// `payload_ptr` must be null for an empty payload or point to `payload_len` readable bytes.
pub unsafe extern "C" fn __fui_on_file_finish_result(
    request_id: u32,
    status: u32,
    written_bytes: u64,
    payload_ptr: *const u8,
    payload_len: u32,
) {
    let request =
        PENDING_WRITER_FINISH_REQUESTS.with(|requests| requests.borrow_mut().remove(&request_id));
    let Some(request) = request else {
        return;
    };
    if status == FILE_STATUS_SUCCESS {
        let payload = if payload_ptr.is_null() || payload_len == 0 {
            &[][..]
        } else {
            unsafe { std::slice::from_raw_parts(payload_ptr, payload_len as usize) }
        };
        if let Some((mode, file_name, _)) = decode_writer_payload(payload) {
            (request.on_complete)(FileSaveResult {
                file_name,
                mode,
                written_bytes,
            });
            return;
        }
    }
    dispatch_file_error(
        request.on_error,
        if payload_ptr.is_null() || payload_len == 0 {
            describe_file_failure(status, "Finishing the file writer failed.")
        } else {
            read_utf8(payload_ptr, payload_len)
        },
    );
}

#[cfg_attr(not(feature = "worker-runtime"), no_mangle)]
/// # Safety
/// `payload_ptr` must be null for an empty payload or point to `payload_len` readable bytes.
pub unsafe extern "C" fn __fui_on_file_worker_process_progress(
    request_id: u32,
    processed_bytes: u64,
    total_bytes: u64,
    payload_ptr: *const u8,
    payload_len: u32,
) {
    let text = if payload_ptr.is_null() || payload_len == 0 {
        String::new()
    } else {
        read_utf8(payload_ptr, payload_len)
    };
    let request =
        ACTIVE_WORKER_PROCESS_REQUESTS.with(|requests| requests.borrow().get(&request_id).cloned());
    let Some(request) = request else {
        return;
    };
    let callback = request.borrow().on_progress.clone();
    if let Some(callback) = callback {
        callback(FileWorkerProcessProgress {
            processed_bytes,
            total_bytes,
            output_file_name: if text.is_empty() { None } else { Some(text) },
        });
    }
}

#[cfg_attr(not(feature = "worker-runtime"), no_mangle)]
/// # Safety
/// `payload_ptr` must be null for an empty payload or point to `payload_len` readable bytes.
pub unsafe extern "C" fn __fui_on_file_worker_process_chunk(
    request_id: u32,
    offset_bytes: u64,
    file_size_bytes: u64,
    payload_ptr: *const u8,
    payload_len: u32,
) {
    let request =
        ACTIVE_WORKER_PROCESS_REQUESTS.with(|requests| requests.borrow().get(&request_id).cloned());
    let Some(request) = request else {
        return;
    };
    let callback = request.borrow().on_chunk.clone();
    if let Some(callback) = callback {
        let bytes = if payload_ptr.is_null() || payload_len == 0 {
            Vec::new()
        } else {
            unsafe { std::slice::from_raw_parts(payload_ptr, payload_len as usize) }.to_vec()
        };
        callback(FileReadChunk {
            offset_bytes,
            file_size_bytes,
            bytes,
        });
    }
}

#[cfg_attr(not(feature = "worker-runtime"), no_mangle)]
/// # Safety
/// `payload_ptr` must be null for an empty payload or point to `payload_len` readable bytes.
pub unsafe extern "C" fn __fui_on_file_worker_process_complete(
    request_id: u32,
    processed_bytes: u64,
    payload_ptr: *const u8,
    payload_len: u32,
) {
    let text = if payload_ptr.is_null() || payload_len == 0 {
        String::new()
    } else {
        read_utf8(payload_ptr, payload_len)
    };
    let (output_file_name, worker_result) = split_worker_process_complete_payload(text);
    let Some((on_complete, _)) = finish_worker_process(request_id) else {
        return;
    };
    if let Some(callback) = on_complete {
        callback(FileWorkerProcessResult {
            processed_bytes,
            output_file_name,
            worker_result,
        });
    }
}

#[cfg_attr(not(feature = "worker-runtime"), no_mangle)]
/// # Safety
/// `payload_ptr` must be null for an empty payload or point to `payload_len` readable bytes.
pub unsafe extern "C" fn __fui_on_file_worker_process_error(
    request_id: u32,
    status: u32,
    payload_ptr: *const u8,
    payload_len: u32,
) {
    let message = if payload_ptr.is_null() || payload_len == 0 {
        describe_file_failure(status, "Worker file processing failed.")
    } else {
        read_utf8(payload_ptr, payload_len)
    };
    let Some((_, on_error)) = finish_worker_process(request_id) else {
        return;
    };
    dispatch_file_error(on_error, message);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::{self, Call};
    use std::cell::RefCell;
    use std::rc::Rc;

    fn writer_payload(mode: u32, first: &str, second: Option<&str>) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&mode.to_le_bytes());
        bytes.extend_from_slice(&(first.len() as u32).to_le_bytes());
        bytes.extend_from_slice(first.as_bytes());
        if let Some(second) = second {
            bytes.extend_from_slice(&(second.len() as u32).to_le_bytes());
            bytes.extend_from_slice(second.as_bytes());
        }
        bytes
    }

    #[test]
    fn open_request_emits_host_call_and_pick_result_registers_files() {
        reset_file_runtime();
        ffi::test::reset();
        let picked = Rc::new(RefCell::new(Vec::<BrowserFile>::new()));
        let picked_clone = picked.clone();
        let _guard = File::open()
            .accept(".txt")
            .multiple(true)
            .pick(move |event| {
                picked_clone.replace(event.files);
            });
        let calls = ffi::test::take_calls();
        assert!(calls.iter().any(|call| matches!(call, Call::FilePick { request_id, accept, multiple } if *request_id == 1 && accept == ".txt" && *multiple)));

        let mut payload = Vec::new();
        payload.extend_from_slice(&1u32.to_le_bytes());
        payload.extend_from_slice(&5u32.to_le_bytes());
        payload.extend_from_slice(b"file-");
        payload.extend_from_slice(&12u64.to_le_bytes());
        payload.extend_from_slice(&34u64.to_le_bytes());
        payload.extend_from_slice(&8u32.to_le_bytes());
        payload.extend_from_slice(b"note.txt");
        payload.extend_from_slice(&10u32.to_le_bytes());
        payload.extend_from_slice(b"text/plain");
        unsafe {
            super::__fui_on_file_pick_result(
                1,
                FILE_STATUS_SUCCESS,
                payload.as_ptr(),
                payload.len() as u32,
            );
        }

        let picked = picked.borrow();
        assert_eq!(picked.len(), 1);
        assert_eq!(picked[0].name(), "note.txt");
        assert_eq!(picked[0].mime_type(), Some("text/plain".to_string()));
        assert_eq!(picked[0].size_bytes(), 12);
    }

    #[test]
    fn browser_file_read_chunk_emits_host_call_and_callback_receives_bytes() {
        reset_file_runtime();
        ffi::test::reset();
        let file =
            register_browser_file("picked-1".to_string(), "demo.bin".to_string(), None, 100, 0);
        let chunk = Rc::new(RefCell::new(None::<FileReadChunk>));
        let chunk_clone = chunk.clone();
        let _guard = file.read_bytes_chunk(10, 8, move |value| {
            chunk_clone.replace(Some(value));
        });
        let calls = ffi::test::take_calls();
        assert!(calls.iter().any(|call| matches!(call, Call::FileReadChunk { request_id, file_id, offset_bytes, max_bytes } if *request_id == 1 && file_id == "picked-1" && *offset_bytes == 10 && *max_bytes == 8)));

        let bytes = b"1234".to_vec();
        unsafe {
            super::__fui_on_file_read_result(
                1,
                FILE_STATUS_SUCCESS,
                10,
                100,
                bytes.as_ptr(),
                bytes.len() as u32,
            );
        }
        let chunk = chunk.borrow().clone().expect("chunk");
        assert_eq!(chunk.offset_bytes, 10);
        assert_eq!(chunk.file_size_bytes, 100);
        assert_eq!(chunk.bytes, b"1234");
    }

    #[test]
    fn save_text_emits_host_call() {
        reset_file_runtime();
        ffi::test::reset();
        let _guard = File::save()
            .suggested_name("report")
            .mime_type("text/plain")
            .file_extension(".txt")
            .save_text("hello", |_| {});
        let calls = ffi::test::take_calls();
        assert!(calls.iter().any(|call| matches!(call, Call::FileSaveText { request_id, suggested_name, mime_type, file_extension, text } if *request_id == 1 && suggested_name == "report" && mime_type == "text/plain" && file_extension == ".txt" && text == "hello")));
    }

    #[test]
    fn writer_created_and_finish_callbacks_decode_payload() {
        reset_file_runtime();
        ffi::test::reset();
        let writer = Rc::new(RefCell::new(None::<BrowserFileWriter>));
        let writer_clone = writer.clone();
        let _guard = File::save()
            .suggested_name("report")
            .create_writer(move |value| {
                writer_clone.replace(Some(value));
            });
        let payload = writer_payload(
            FileSaveMode::NativePicker as u32,
            "writer-1",
            Some("report.txt"),
        );
        unsafe {
            super::__fui_on_file_writer_created(
                1,
                FILE_STATUS_SUCCESS,
                payload.as_ptr(),
                payload.len() as u32,
            );
        }
        let writer = writer.borrow().clone().expect("writer");
        assert_eq!(writer.file_name, "report.txt");
        assert_eq!(writer.mode, FileSaveMode::NativePicker);

        ffi::test::reset();
        let finished = Rc::new(RefCell::new(None::<FileSaveResult>));
        let finished_clone = finished.clone();
        let _guard = writer.finish(move |value| {
            finished_clone.replace(Some(value));
        });
        let calls = ffi::test::take_calls();
        assert!(calls.iter().any(|call| matches!(call, Call::FileWriterFinish { request_id, writer_id } if *request_id == 2 && writer_id == "writer-1")));
        let payload = writer_payload(FileSaveMode::NativePicker as u32, "report.txt", None);
        unsafe {
            super::__fui_on_file_finish_result(
                2,
                FILE_STATUS_SUCCESS,
                44,
                payload.as_ptr(),
                payload.len() as u32,
            );
        }
        let finished = finished.borrow().clone().expect("finish");
        assert_eq!(finished.file_name, "report.txt");
        assert_eq!(finished.written_bytes, 44);
    }

    #[test]
    fn worker_process_requires_worker_and_sink() {
        reset_file_runtime();
        ffi::test::reset();
        let file =
            register_browser_file("picked-1".to_string(), "demo.bin".to_string(), None, 100, 0);
        let error = Rc::new(RefCell::new(String::new()));
        let error_clone = error.clone();
        let _request = File::process_file_in_worker(file)
            .on_error(move |event| {
                error_clone.replace(event.message);
            })
            .start();
        assert!(error
            .borrow()
            .contains("worker(wasm_path, entry_name) is required"));
    }

    #[test]
    fn worker_process_start_emits_host_call_and_callbacks_receive_payloads() {
        reset_file_runtime();
        ffi::test::reset();
        let file =
            register_browser_file("picked-1".to_string(), "demo.bin".to_string(), None, 100, 0);
        let progress = Rc::new(RefCell::new(None::<FileWorkerProcessProgress>));
        let result = Rc::new(RefCell::new(None::<FileWorkerProcessResult>));
        let progress_clone = progress.clone();
        let result_clone = result.clone();
        let _request = File::process_file_in_worker(file)
            .worker("./workers/file_worker.wasm", "entry")
            .save_to_picked_file("copy.bin")
            .on_progress(move |value| {
                progress_clone.replace(Some(value));
            })
            .on_complete(move |value| {
                result_clone.replace(Some(value));
            })
            .start();
        let calls = ffi::test::take_calls();
        assert!(calls.iter().any(|call| matches!(call, Call::FileProcessWorkerStart { request_id, worker_wasm_path, worker_entry_name, file_id, suggested_name, chunk_bytes, save_to_picked_file } if *request_id == 1 && worker_wasm_path == "./workers/file_worker.wasm" && worker_entry_name == "entry" && file_id == "picked-1" && suggested_name == "copy.bin" && *chunk_bytes == 64 * 1024 && *save_to_picked_file)));

        unsafe {
            super::__fui_on_file_worker_process_progress(1, 20, 100, b"copy.bin".as_ptr(), 8);
        }
        let progress = progress.borrow().clone().expect("progress");
        assert_eq!(progress.processed_bytes, 20);
        assert_eq!(progress.output_file_name, Some("copy.bin".to_string()));

        let payload = b"copy.bin\0sha256".to_vec();
        unsafe {
            super::__fui_on_file_worker_process_complete(
                1,
                100,
                payload.as_ptr(),
                payload.len() as u32,
            );
        }
        let result = result.borrow().clone().expect("result");
        assert_eq!(result.processed_bytes, 100);
        assert_eq!(result.output_file_name, Some("copy.bin".to_string()));
        assert_eq!(result.worker_result, Some("sha256".to_string()));
    }
}
