use crate::ffi;
use crate::logger::warn;
use crate::signal::{Callback, Signal, SubscriptionGuard};
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::thread::LocalKey;

const FIRST_DYNAMIC_SVG_ID: u32 = 0x1000_0000;
const FIRST_DYNAMIC_TEXTURE_ID: u32 = 0x2000_0000;
const MAX_DYNAMIC_ASSET_ID: u32 = 0xffff_ffff;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetLoadState {
    Idle = 0,
    Loading = 1,
    Ready = 2,
    Failed = 3,
}

#[derive(Clone)]
pub struct AssetStateSignal {
    inner: Rc<RefCell<Signal<AssetLoadState>>>,
}

impl AssetStateSignal {
    fn new(initial: AssetLoadState) -> Self {
        Self {
            inner: Rc::new(RefCell::new(Signal::new(initial))),
        }
    }

    pub fn get(&self) -> AssetLoadState {
        self.inner.borrow().get()
    }

    pub fn subscribe(&self, callback: Callback) -> SubscriptionGuard {
        self.inner.borrow_mut().subscribe(callback)
    }

    fn set(&self, next: AssetLoadState) {
        let callbacks = self.inner.borrow_mut().set(next);
        if let Some(callbacks) = callbacks {
            for callback in callbacks {
                callback();
            }
        }
    }
}

#[derive(Clone)]
struct AssetRecord {
    state: AssetStateSignal,
    error: String,
    width: f32,
    height: f32,
    url: String,
}

impl Default for AssetRecord {
    fn default() -> Self {
        Self {
            state: AssetStateSignal::new(AssetLoadState::Idle),
            error: String::new(),
            width: 0.0,
            height: 0.0,
            url: String::new(),
        }
    }
}

thread_local! {
    static SVG_ASSETS: RefCell<HashMap<u32, Rc<RefCell<AssetRecord>>>> = RefCell::new(HashMap::new());
    static TEXTURE_ASSETS: RefCell<HashMap<u32, Rc<RefCell<AssetRecord>>>> = RefCell::new(HashMap::new());
    static LOADED_FONT_IDS: RefCell<HashSet<u32>> = RefCell::new(default_loaded_font_ids());
    static SVG_IDS_BY_URL: RefCell<HashMap<String, u32>> = RefCell::new(HashMap::new());
    static TEXTURE_IDS_BY_URL: RefCell<HashMap<String, u32>> = RefCell::new(HashMap::new());
    static SVG_REF_COUNTS: RefCell<HashMap<u32, i32>> = RefCell::new(HashMap::new());
    static TEXTURE_REF_COUNTS: RefCell<HashMap<u32, i32>> = RefCell::new(HashMap::new());
    static PINNED_SVG_IDS: RefCell<HashSet<u32>> = RefCell::new(HashSet::new());
    static PINNED_TEXTURE_IDS: RefCell<HashSet<u32>> = RefCell::new(HashSet::new());
    static NEXT_DYNAMIC_SVG_ID: Cell<u32> = const { Cell::new(FIRST_DYNAMIC_SVG_ID) };
    static NEXT_DYNAMIC_TEXTURE_ID: Cell<u32> = const { Cell::new(FIRST_DYNAMIC_TEXTURE_ID) };
}

fn default_loaded_font_ids() -> HashSet<u32> {
    HashSet::from([1, 2, 7])
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

fn next_dynamic_svg_id() -> u32 {
    NEXT_DYNAMIC_SVG_ID.with(|slot| {
        let next = slot.get();
        assert!(
            next != MAX_DYNAMIC_ASSET_ID,
            "Dynamic SVG asset id space exhausted."
        );
        slot.set(next + 1);
        next
    })
}

fn next_dynamic_texture_id() -> u32 {
    NEXT_DYNAMIC_TEXTURE_ID.with(|slot| {
        let next = slot.get();
        assert!(
            next != MAX_DYNAMIC_ASSET_ID,
            "Dynamic texture asset id space exhausted."
        );
        slot.set(next + 1);
        next
    })
}

fn get_svg_record(svg_id: u32) -> Rc<RefCell<AssetRecord>> {
    SVG_ASSETS.with(|records| {
        let mut records = records.borrow_mut();
        records
            .entry(svg_id)
            .or_insert_with(|| Rc::new(RefCell::new(AssetRecord::default())))
            .clone()
    })
}

fn get_texture_record(texture_id: u32) -> Rc<RefCell<AssetRecord>> {
    TEXTURE_ASSETS.with(|records| {
        let mut records = records.borrow_mut();
        records
            .entry(texture_id)
            .or_insert_with(|| Rc::new(RefCell::new(AssetRecord::default())))
            .clone()
    })
}

fn begin_load(record: &Rc<RefCell<AssetRecord>>) {
    let state = {
        let mut record = record.borrow_mut();
        record.error.clear();
        record.width = 0.0;
        record.height = 0.0;
        record.state.clone()
    };
    state.set(AssetLoadState::Loading);
}

fn mark_loaded(record: &Rc<RefCell<AssetRecord>>, width: f32, height: f32) {
    let state = {
        let mut record = record.borrow_mut();
        record.error.clear();
        record.width = width;
        record.height = height;
        record.state.clone()
    };
    state.set(AssetLoadState::Ready);
}

fn mark_failed(record: &Rc<RefCell<AssetRecord>>, error: String) {
    let state = {
        let mut record = record.borrow_mut();
        record.error = error;
        record.width = 0.0;
        record.height = 0.0;
        record.state.clone()
    };
    state.set(AssetLoadState::Failed);
}

fn increment_ref_count(ref_counts: &RefCell<HashMap<u32, i32>>, asset_id: u32) {
    let mut ref_counts = ref_counts.borrow_mut();
    let next = ref_counts.get(&asset_id).copied().unwrap_or(0) + 1;
    ref_counts.insert(asset_id, next);
}

fn decrement_ref_count(ref_counts: &RefCell<HashMap<u32, i32>>, asset_id: u32) -> i32 {
    let mut ref_counts = ref_counts.borrow_mut();
    let Some(current) = ref_counts.get(&asset_id).copied() else {
        return -1;
    };
    let next = current - 1;
    if next <= 0 {
        ref_counts.remove(&asset_id);
        0
    } else {
        ref_counts.insert(asset_id, next);
        next
    }
}

fn remove_url_binding(
    url_to_id: &'static LocalKey<RefCell<HashMap<String, u32>>>,
    url: &str,
    asset_id: u32,
) {
    if url.is_empty() {
        return;
    }
    url_to_id.with(|url_to_id| {
        let mut url_to_id = url_to_id.borrow_mut();
        if url_to_id.get(url).copied() == Some(asset_id) {
            url_to_id.remove(url);
        }
    });
}

fn load_svg_internal(svg_id: u32, url: &str, pinned: bool) {
    let record = get_svg_record(svg_id);
    {
        let mut record_mut = record.borrow_mut();
        if !record_mut.url.is_empty() && record_mut.url != url {
            remove_url_binding(&SVG_IDS_BY_URL, &record_mut.url, svg_id);
        }
        record_mut.url = url.to_string();
    }
    SVG_IDS_BY_URL.with(|map| {
        map.borrow_mut().insert(url.to_string(), svg_id);
    });
    PINNED_SVG_IDS.with(|ids| {
        let mut ids = ids.borrow_mut();
        if pinned {
            ids.insert(svg_id);
        } else {
            ids.remove(&svg_id);
        }
    });
    begin_load(&record);
    with_utf8(url, |ptr, len| unsafe {
        ffi::fui_load_svg(svg_id, ptr, len);
    });
}

fn load_texture_internal(texture_id: u32, url: &str, pinned: bool) {
    let record = get_texture_record(texture_id);
    {
        let mut record_mut = record.borrow_mut();
        if !record_mut.url.is_empty() && record_mut.url != url {
            remove_url_binding(&TEXTURE_IDS_BY_URL, &record_mut.url, texture_id);
        }
        record_mut.url = url.to_string();
    }
    TEXTURE_IDS_BY_URL.with(|map| {
        map.borrow_mut().insert(url.to_string(), texture_id);
    });
    PINNED_TEXTURE_IDS.with(|ids| {
        let mut ids = ids.borrow_mut();
        if pinned {
            ids.insert(texture_id);
        } else {
            ids.remove(&texture_id);
        }
    });
    begin_load(&record);
    with_utf8(url, |ptr, len| unsafe {
        ffi::fui_load_texture(texture_id, ptr, len);
    });
}

pub(crate) fn load_font(font_id: u32, url: &str) {
    LOADED_FONT_IDS.with(|ids| {
        ids.borrow_mut().remove(&font_id);
    });
    with_utf8(url, |ptr, len| unsafe {
        ffi::fui_load_font(font_id, ptr, len);
    });
}

pub fn allocate_dynamic_texture_id() -> u32 {
    next_dynamic_texture_id()
}

pub(crate) fn is_font_loaded(font_id: u32) -> bool {
    LOADED_FONT_IDS.with(|ids| ids.borrow().contains(&font_id))
}

pub(crate) fn on_font_loaded(font_id: u32) {
    LOADED_FONT_IDS.with(|ids| {
        ids.borrow_mut().insert(font_id);
    });
}

pub fn load_svg(svg_id: u32, url: &str) {
    load_svg_internal(svg_id, url, true);
}

pub fn load_texture(texture_id: u32, url: &str) {
    load_texture_internal(texture_id, url, true);
}

pub fn acquire_svg_asset(url: &str) -> u32 {
    if url.is_empty() {
        return 0;
    }
    let svg_id = SVG_IDS_BY_URL
        .with(|map| map.borrow().get(url).copied())
        .unwrap_or_else(|| {
            let svg_id = next_dynamic_svg_id();
            load_svg_internal(svg_id, url, false);
            svg_id
        });
    SVG_REF_COUNTS.with(|ref_counts| increment_ref_count(ref_counts, svg_id));
    svg_id
}

pub fn acquire_texture_asset(url: &str) -> u32 {
    if url.is_empty() {
        return 0;
    }
    let texture_id = TEXTURE_IDS_BY_URL
        .with(|map| map.borrow().get(url).copied())
        .unwrap_or_else(|| {
            let texture_id = next_dynamic_texture_id();
            load_texture_internal(texture_id, url, false);
            texture_id
        });
    TEXTURE_REF_COUNTS.with(|ref_counts| increment_ref_count(ref_counts, texture_id));
    texture_id
}

pub fn release_svg_asset(svg_id: u32) {
    if svg_id == 0 {
        return;
    }
    let remaining = SVG_REF_COUNTS.with(|ref_counts| decrement_ref_count(ref_counts, svg_id));
    let pinned = PINNED_SVG_IDS.with(|ids| ids.borrow().contains(&svg_id));
    if remaining != 0 || pinned {
        return;
    }
    let url = get_svg_record(svg_id).borrow().url.clone();
    remove_url_binding(&SVG_IDS_BY_URL, &url, svg_id);
    SVG_ASSETS.with(|records| {
        records.borrow_mut().remove(&svg_id);
    });
    unsafe {
        ffi::fui_release_svg(svg_id);
    }
}

pub fn release_texture_asset(texture_id: u32) {
    if texture_id == 0 {
        return;
    }
    let remaining =
        TEXTURE_REF_COUNTS.with(|ref_counts| decrement_ref_count(ref_counts, texture_id));
    let pinned = PINNED_TEXTURE_IDS.with(|ids| ids.borrow().contains(&texture_id));
    if remaining != 0 || pinned {
        return;
    }
    let url = get_texture_record(texture_id).borrow().url.clone();
    remove_url_binding(&TEXTURE_IDS_BY_URL, &url, texture_id);
    TEXTURE_ASSETS.with(|records| {
        records.borrow_mut().remove(&texture_id);
    });
    unsafe {
        ffi::fui_release_texture(texture_id);
    }
}

pub fn ensure_svg_asset(url: &str) -> u32 {
    acquire_svg_asset(url)
}

pub fn ensure_texture_asset(url: &str) -> u32 {
    acquire_texture_asset(url)
}

pub fn get_svg_asset_state(svg_id: u32) -> AssetStateSignal {
    get_svg_record(svg_id).borrow().state.clone()
}

pub fn get_texture_asset_state(texture_id: u32) -> AssetStateSignal {
    get_texture_record(texture_id).borrow().state.clone()
}

pub fn get_svg_asset_state_value(svg_id: u32) -> AssetLoadState {
    get_svg_record(svg_id).borrow().state.get()
}

pub fn get_texture_asset_state_value(texture_id: u32) -> AssetLoadState {
    get_texture_record(texture_id).borrow().state.get()
}

pub fn get_svg_asset_error(svg_id: u32) -> String {
    get_svg_record(svg_id).borrow().error.clone()
}

pub fn get_svg_asset_url(svg_id: u32) -> String {
    get_svg_record(svg_id).borrow().url.clone()
}

pub fn get_svg_asset_width(svg_id: u32) -> f32 {
    get_svg_record(svg_id).borrow().width
}

pub fn get_svg_asset_height(svg_id: u32) -> f32 {
    get_svg_record(svg_id).borrow().height
}

pub fn get_texture_asset_error(texture_id: u32) -> String {
    get_texture_record(texture_id).borrow().error.clone()
}

pub fn get_texture_asset_url(texture_id: u32) -> String {
    get_texture_record(texture_id).borrow().url.clone()
}

pub fn get_texture_asset_width(texture_id: u32) -> f32 {
    get_texture_record(texture_id).borrow().width
}

pub fn get_texture_asset_height(texture_id: u32) -> f32 {
    get_texture_record(texture_id).borrow().height
}

pub fn mark_texture_asset_ready(texture_id: u32, width: f32, height: f32) {
    mark_loaded(&get_texture_record(texture_id), width, height);
}

pub fn on_svg_loaded(svg_id: u32, width: f32, height: f32) {
    mark_loaded(&get_svg_record(svg_id), width, height);
}

pub fn on_svg_failed(svg_id: u32, error: String) {
    let record = get_svg_record(svg_id);
    let url = record.borrow().url.clone();
    let message = if error.is_empty() {
        "unknown error".to_string()
    } else {
        error.clone()
    };
    mark_failed(&record, error);
    warn(
        "Assets",
        &format!("SVG load failed for \"{url}\": {message}"),
    );
}

pub fn on_texture_loaded(texture_id: u32, width: f32, height: f32) {
    mark_loaded(&get_texture_record(texture_id), width, height);
}

pub fn on_texture_failed(texture_id: u32, error: String) {
    let record = get_texture_record(texture_id);
    let url = record.borrow().url.clone();
    let message = if error.is_empty() {
        "unknown error".to_string()
    } else {
        error.clone()
    };
    mark_failed(&record, error);
    warn(
        "Assets",
        &format!("Texture load failed for \"{url}\": {message}"),
    );
}

#[cfg(test)]
pub(crate) fn test_reset() {
    SVG_ASSETS.with(|slot| slot.borrow_mut().clear());
    TEXTURE_ASSETS.with(|slot| slot.borrow_mut().clear());
    SVG_IDS_BY_URL.with(|slot| slot.borrow_mut().clear());
    TEXTURE_IDS_BY_URL.with(|slot| slot.borrow_mut().clear());
    SVG_REF_COUNTS.with(|slot| slot.borrow_mut().clear());
    TEXTURE_REF_COUNTS.with(|slot| slot.borrow_mut().clear());
    PINNED_SVG_IDS.with(|slot| slot.borrow_mut().clear());
    PINNED_TEXTURE_IDS.with(|slot| slot.borrow_mut().clear());
    LOADED_FONT_IDS.with(|ids| {
        *ids.borrow_mut() = default_loaded_font_ids();
    });
    NEXT_DYNAMIC_SVG_ID.with(|slot| slot.set(FIRST_DYNAMIC_SVG_ID));
    NEXT_DYNAMIC_TEXTURE_ID.with(|slot| slot.set(FIRST_DYNAMIC_TEXTURE_ID));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::{self, Call};
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn asset_loaders_emit_host_calls() {
        test_reset();
        ffi::test::reset();

        load_font(1, "/v2/fonts/NotoSans-Regular.ttf");
        load_svg(2, "data:image/svg+xml,%3Csvg/%3E");
        load_texture(3, "/v2/fui-as/demo/demo-texture.png");

        let calls = ffi::test::take_calls();
        assert!(calls.iter().any(|call| matches!(call, Call::LoadFont { font_id: 1, url } if url == "/v2/fonts/NotoSans-Regular.ttf")));
        assert!(calls.iter().any(|call| matches!(call, Call::LoadSvg { svg_id: 2, url } if url == "data:image/svg+xml,%3Csvg/%3E")));
        assert!(calls.iter().any(|call| matches!(call, Call::LoadTexture { texture_id: 3, url } if url == "/v2/fui-as/demo/demo-texture.png")));
    }

    #[test]
    fn acquired_assets_share_url_and_release_when_last_ref_drops() {
        test_reset();
        ffi::test::reset();

        let first = acquire_texture_asset("/img/a.png");
        let second = acquire_texture_asset("/img/a.png");
        assert_eq!(first, second);
        let calls = ffi::test::take_calls();
        let load_calls = calls
            .iter()
            .filter(|call| matches!(call, Call::LoadTexture { .. }))
            .count();
        assert_eq!(load_calls, 1);

        release_texture_asset(first);
        let calls = ffi::test::take_calls();
        assert!(!calls.iter().any(
            |call| matches!(call, Call::ReleaseTexture { texture_id } if *texture_id == first)
        ));

        release_texture_asset(second);
        let calls = ffi::test::take_calls();
        assert!(calls.iter().any(
            |call| matches!(call, Call::ReleaseTexture { texture_id } if *texture_id == first)
        ));
    }

    #[test]
    fn pinned_assets_are_not_released_by_ref_count_drop() {
        test_reset();
        ffi::test::reset();

        load_svg(55, "/img/pinned.svg");
        let acquired = acquire_svg_asset("/img/pinned.svg");
        assert_eq!(acquired, 55);
        ffi::test::take_calls();

        release_svg_asset(acquired);
        let calls = ffi::test::take_calls();
        assert!(!calls
            .iter()
            .any(|call| matches!(call, Call::ReleaseSvg { svg_id } if *svg_id == acquired)));
    }

    #[test]
    fn asset_state_signal_notifies_and_tracks_ready_and_failed_payloads() {
        test_reset();
        let texture_id = acquire_texture_asset("/img/ready.png");
        let state = get_texture_asset_state(texture_id);
        assert_eq!(state.get(), AssetLoadState::Loading);

        let hits = Rc::new(Cell::new(0));
        let hits_clone = hits.clone();
        let callback: Callback = Rc::new(move || hits_clone.set(hits_clone.get() + 1));
        let _guard = state.subscribe(callback);

        on_texture_loaded(texture_id, 64.0, 32.0);
        assert_eq!(state.get(), AssetLoadState::Ready);
        assert_eq!(get_texture_asset_width(texture_id), 64.0);
        assert_eq!(get_texture_asset_height(texture_id), 32.0);

        on_texture_failed(texture_id, "broken".to_string());
        assert_eq!(state.get(), AssetLoadState::Failed);
        assert_eq!(get_texture_asset_error(texture_id), "broken");
        assert_eq!(hits.get(), 2);
    }
}
