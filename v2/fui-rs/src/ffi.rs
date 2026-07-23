#![allow(clippy::too_many_arguments, clippy::type_complexity)]

pub use crate::generated::ffi::*;

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
#[derive(Debug, Clone, PartialEq)]
pub enum Call {
    Reset,
    CreateNode {
        node_type: u32,
        handle: u64,
    },
    DeleteNode {
        handle: u64,
    },
    NodeAddChild {
        parent: u64,
        child: u64,
    },
    NodeRemoveChild {
        parent: u64,
        child: u64,
    },
    SetRoot {
        handle: u64,
    },
    SetNodeId {
        handle: u64,
        node_id: String,
    },
    SetSemanticRole {
        handle: u64,
        role_enum: u32,
    },
    SetSemanticExpanded {
        handle: u64,
        has_expanded: bool,
        is_expanded: bool,
    },
    SetSemanticLabel {
        handle: u64,
        label: String,
    },
    SetSemanticChecked {
        handle: u64,
        checked_state_enum: u32,
    },
    SetSemanticSelected {
        handle: u64,
        has_selected: bool,
        selected: bool,
    },
    SetSemanticDisabled {
        handle: u64,
        has_disabled: bool,
        disabled: bool,
    },
    SetSemanticValueRange {
        handle: u64,
        has_value_range: bool,
        value_now: f32,
        value_min: f32,
        value_max: f32,
    },
    SetSemanticOrientation {
        handle: u64,
        orientation_enum: u32,
    },
    RequestSemanticAnnouncement {
        handle: u64,
    },
    PushSemanticScope {
        handle: u64,
        token: u32,
    },
    RemoveSemanticScope {
        token: u32,
    },
    SetIsPortal {
        handle: u64,
        is_portal: bool,
    },
    SetVisibility {
        handle: u64,
        visibility_enum: u32,
    },
    SetWidth {
        handle: u64,
        value: f32,
        unit_enum: u32,
    },
    SetHeight {
        handle: u64,
        value: f32,
        unit_enum: u32,
    },
    SetFillWidth {
        handle: u64,
        fill: bool,
    },
    SetFillHeight {
        handle: u64,
        fill: bool,
    },
    SetFillWidthPercent {
        handle: u64,
        percent: f32,
    },
    SetFillHeightPercent {
        handle: u64,
        percent: f32,
    },
    SetMinWidth {
        handle: u64,
        value: f32,
        unit_enum: u32,
    },
    SetMaxWidth {
        handle: u64,
        value: f32,
        unit_enum: u32,
    },
    SetMinHeight {
        handle: u64,
        value: f32,
        unit_enum: u32,
    },
    SetMaxHeight {
        handle: u64,
        value: f32,
        unit_enum: u32,
    },
    SetBgColor {
        handle: u64,
        color: u32,
    },
    SetBoxStyle {
        handle: u64,
        bg_color: u32,
        radius_tl: f32,
        radius_tr: f32,
        radius_br: f32,
        radius_bl: f32,
        border_width: f32,
        border_color: u32,
        border_style_enum: u32,
        border_dash_on: f32,
        border_dash_off: f32,
    },
    SetLinearGradient {
        handle: u64,
        sx: f32,
        sy: f32,
        ex: f32,
        ey: f32,
        offsets: Vec<f32>,
        colors: Vec<u32>,
    },
    SetDropShadow {
        handle: u64,
        color: u32,
        offset_x: f32,
        offset_y: f32,
        blur_sigma: f32,
        spread: f32,
    },
    SetLayerEffect {
        handle: u64,
        opacity: f32,
        blur_sigma: f32,
        blend_mode_enum: u32,
    },
    SetBackgroundBlur {
        handle: u64,
        blur_sigma: f32,
    },
    SetText {
        handle: u64,
        text: String,
    },
    SetTextStyleRuns {
        handle: u64,
        run_count: u32,
        words: Vec<u32>,
    },
    PrepareNode {
        handle: u64,
    },
    SetDynamicTextCharset {
        handle: u64,
        charset: String,
    },
    GetTextMetrics {
        handle: u64,
    },
    SetFont {
        handle: u64,
        font_id: u32,
        size: f32,
    },
    RegisterFontFallback {
        font_id: u32,
        fallback_font_id: u32,
    },
    SetLineHeight {
        handle: u64,
        line_height: f32,
    },
    SetTextColor {
        handle: u64,
        color: u32,
    },
    SetTextAlign {
        handle: u64,
        align_enum: u32,
    },
    SetTextVerticalAlign {
        handle: u64,
        align_enum: u32,
    },
    SetTextLimits {
        handle: u64,
        max_chars: i32,
        max_lines: i32,
    },
    SetTextWrapping {
        handle: u64,
        wrap: bool,
    },
    SetTextOverflow {
        handle: u64,
        overflow_enum: u32,
    },
    SetTextOverflowFade {
        handle: u64,
        horizontal: bool,
        vertical: bool,
    },
    SetSelectable {
        handle: u64,
        selectable: bool,
        selection_color: u32,
    },
    SetTextSelectionRange {
        handle: u64,
        start: u32,
        end: u32,
    },
    SetEditable {
        handle: u64,
        editable: bool,
    },
    SetEditorCommandKeys {
        handle: u64,
        enabled: bool,
    },
    SetEditorAcceptsTab {
        handle: u64,
        enabled: bool,
    },
    ReplaceTextRange {
        handle: u64,
        start: u32,
        end: u32,
        text: String,
        caret: u32,
    },
    SetTextObscured {
        handle: u64,
        obscured: bool,
    },
    SetCaretColor {
        handle: u64,
        color: u32,
    },
    RegisterTextInputMetadata {
        handle: u64,
        is_password: bool,
        hint: String,
    },
    SetPreserveSelectionOnPointerDown {
        handle: u64,
        preserve: bool,
    },
    SetInteractive {
        handle: u64,
        interactive: bool,
    },
    SetFocusable {
        handle: u64,
        focusable: bool,
        tab_index: i32,
    },
    RequestFocus {
        handle: u64,
    },
    SetPadding {
        handle: u64,
        left: f32,
        top: f32,
        right: f32,
        bottom: f32,
    },
    SetFlexDirection {
        handle: u64,
        dir_enum: u32,
    },
    SetFlexBasis {
        handle: u64,
        basis: f32,
    },
    SetJustifyContent {
        handle: u64,
        justify_enum: u32,
    },
    SetAlignItems {
        handle: u64,
        align_enum: u32,
    },
    SetAlignSelf {
        handle: u64,
        align_enum: u32,
    },
    SetMargin {
        handle: u64,
        left: f32,
        top: f32,
        right: f32,
        bottom: f32,
    },
    SetPositionType {
        handle: u64,
        pos_enum: u32,
    },
    SetPosition {
        handle: u64,
        left: f32,
        top: f32,
        right: f32,
        bottom: f32,
    },
    SetIsSharedSizeScope {
        handle: u64,
        is_scope: bool,
    },
    SetCustomDrawable {
        handle: u64,
        is_custom_drawable: bool,
    },
    SetFlexWrap {
        handle: u64,
        wrap_enum: u32,
    },
    SetClipToBounds {
        handle: u64,
        clip: bool,
    },
    SetSelectionArea {
        handle: u64,
        is_area: bool,
    },
    SetSelectionAreaBarrier {
        handle: u64,
        is_barrier: bool,
    },
    ClearSelection {
        text_node_handle: u64,
    },
    RetargetSelection {
        from_text_node_handle: u64,
        to_text_node_handle: u64,
    },
    GridSetColumns {
        handle: u64,
        values: Vec<f32>,
        type_enums: Vec<u8>,
    },
    GridSetRows {
        handle: u64,
        values: Vec<f32>,
        type_enums: Vec<u8>,
    },
    GridSetColumnSharedSizeGroup {
        handle: u64,
        index: u32,
        group: String,
    },
    GridSetRowSharedSizeGroup {
        handle: u64,
        index: u32,
        group: String,
    },
    NodeSetGridPlacement {
        handle: u64,
        row: u32,
        col: u32,
        row_span: u32,
        col_span: u32,
    },
    SetImage {
        handle: u64,
        texture_id: u32,
        object_fit_enum: u32,
        sampling_kind: u32,
        max_aniso: u32,
    },
    SetImageNine {
        handle: u64,
        texture_id: u32,
        inset_left: f32,
        inset_top: f32,
        inset_right: f32,
        inset_bottom: f32,
        sampling_kind: u32,
        max_aniso: u32,
    },
    SetSvg {
        handle: u64,
        svg_id: u32,
        tint_color: u32,
        sampling_kind: u32,
        max_aniso: u32,
    },
    SetScrollProxyTarget {
        handle: u64,
        scroll_handle: u64,
    },
    SetScrollEnabled {
        handle: u64,
        enabled_x: bool,
        enabled_y: bool,
    },
    SetScrollFriction {
        handle: u64,
        friction: f32,
    },
    SetSmoothScrolling {
        handle: u64,
        smooth_scrolling: bool,
    },
    SetScrollOffset {
        handle: u64,
        offset_x: f32,
        offset_y: f32,
    },
    ClearMomentumScroll,
    SetScrollContentSize {
        handle: u64,
        content_width: f32,
        content_height: f32,
    },
    CommitFrame,
    ResizeWindow {
        logical_w: f32,
        logical_h: f32,
    },
    RequestRender,
    GetViewportWidth,
    GetViewportHeight,
    GetDevicePixelRatio,
    SetPointerCapture {
        handle: u64,
    },
    ReleasePointerCapture,
    CanNavigateBack,
    CanNavigateForward,
    NavigateBack,
    NavigateForward,
    ReloadPage,
    CopyText {
        text: String,
    },
    SetApplicationCaption {
        caption: String,
    },
    HasTextSelectionSnapshot {
        handle: u64,
    },
    CopyTextSelectionSnapshot {
        handle: u64,
    },
    CutFocusedTextSelection,
    CutTextSelectionSnapshot {
        handle: u64,
    },
    CutTextRangeSnapshot {
        handle: u64,
        start: u32,
        end: u32,
    },
    DeleteFocusedTextRange {
        start: u32,
        end: u32,
    },
    CommitTextActionFocus {
        handle: u64,
    },
    CopyCurrentSelection,
    HasTextSelection {
        handle: u64,
    },
    UndoTextEdit {
        handle: u64,
    },
    RedoTextEdit {
        handle: u64,
    },
    CopyTextSelection {
        handle: u64,
    },
    CutTextSelection {
        handle: u64,
    },
    PasteText {
        handle: u64,
    },
    SelectAllText {
        handle: u64,
    },
    SelectWordAt {
        handle: u64,
        x: f32,
        y: f32,
    },
    ClearCurrentSelection,
    IsPointInSelection {
        x: f32,
        y: f32,
    },
    GetTextRangeRectCount {
        handle: u64,
        start: u32,
        end: u32,
    },
    CopyTextRangeRects {
        handle: u64,
        start: u32,
        end: u32,
        max_rect_count: u32,
    },
    CopyCrossSelectionEndpointRects {
        handle: u64,
    },
    BeginSelectionEndpointDrag {
        handle: u64,
        endpoint: u32,
    },
    StartTimer {
        timer_id: u32,
        delay_ms: i32,
    },
    CancelTimer {
        timer_id: u32,
    },
    SetCursor {
        style: u32,
    },
    NavigateTo {
        target: String,
        open_in_new_tab: bool,
    },
    ShowUrlPreview {
        url: String,
    },
    HideUrlPreview,
    Log {
        category: String,
        message: String,
    },
    LogsEnabled,
    IsDarkMode,
    GetAccentColor,
    GetPlatformFamily,
    GetHostEnvironment,
    GetHostCapabilities,
    IsCoarsePointer,
    LoadSvg {
        svg_id: u32,
        url: String,
    },
    ReleaseSvg {
        svg_id: u32,
    },
    LoadTexture {
        texture_id: u32,
        url: String,
    },
    ReleaseTexture {
        texture_id: u32,
    },
    LoadFont {
        font_id: u32,
        url: String,
    },
    BitmapCommit {
        texture_id: u32,
        bytes: Vec<u8>,
        width: u32,
        height: u32,
    },
    BitmapCommitDirty {
        texture_id: u32,
        bytes: Vec<u8>,
        full_width: u32,
        full_height: u32,
        sub_x: u32,
        sub_y: u32,
        sub_w: u32,
        sub_h: u32,
    },
    BitmapRelease {
        texture_id: u32,
    },
    RenderNodeToRgba {
        handle: u64,
        width: u32,
        height: u32,
        out_capacity: u32,
        scale: f32,
        x: f32,
        y: f32,
    },
    FetchStart {
        request_id: u32,
        method: String,
        url: String,
        headers: Vec<String>,
        body: Vec<u8>,
    },
    FetchCancel {
        request_id: u32,
    },
    SetPersistedScrollOffset {
        node_id: String,
        x: f32,
        y: f32,
    },
    TryGetPersistedScrollOffset {
        node_id: String,
    },
    SetPersistedState {
        node_id: String,
        kind: String,
        version: u32,
        payload: String,
    },
    CopyPersistedState {
        node_id: String,
        kind: String,
    },
    WorkerStartString {
        worker_id: u32,
        wasm_path: String,
        entry: String,
        input: String,
    },
    WorkerCancel {
        worker_id: u32,
    },
    FileCapabilities,
    FilePick {
        request_id: u32,
        accept: String,
        multiple: bool,
    },
    FileReadChunk {
        request_id: u32,
        file_id: String,
        offset_bytes: u64,
        max_bytes: u32,
    },
    FileSaveText {
        request_id: u32,
        suggested_name: String,
        mime_type: String,
        file_extension: String,
        text: String,
    },
    FileSaveBytes {
        request_id: u32,
        suggested_name: String,
        mime_type: String,
        file_extension: String,
        bytes: Vec<u8>,
    },
    FileCreateWriter {
        request_id: u32,
        suggested_name: String,
        mime_type: String,
        file_extension: String,
    },
    FileWriterWriteText {
        request_id: u32,
        writer_id: String,
        text: String,
    },
    FileWriterWriteBytes {
        request_id: u32,
        writer_id: String,
        bytes: Vec<u8>,
    },
    FileWriterFinish {
        request_id: u32,
        writer_id: String,
    },
    FileProcessWorkerStart {
        request_id: u32,
        worker_wasm_path: String,
        worker_entry_name: String,
        file_id: String,
        suggested_name: String,
        chunk_bytes: u32,
        save_to_picked_file: bool,
    },
    FileProcessWorkerCancel {
        request_id: u32,
    },
    PathCreate {
        path_id: u32,
    },
    PathDestroy {
        path_id: u32,
    },
    PathMoveTo {
        path_id: u32,
        x: f32,
        y: f32,
    },
    PathLineTo {
        path_id: u32,
        x: f32,
        y: f32,
    },
    PathQuadTo {
        path_id: u32,
        cx: f32,
        cy: f32,
        x: f32,
        y: f32,
    },
    PathCubicTo {
        path_id: u32,
        cx1: f32,
        cy1: f32,
        cx2: f32,
        cy2: f32,
        x: f32,
        y: f32,
    },
    PathClose {
        path_id: u32,
    },
    PathAddRect {
        path_id: u32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    },
    PathAddCircle {
        path_id: u32,
        cx: f32,
        cy: f32,
        r: f32,
    },
    CanvasDrawRect {
        canvas_ptr: usize,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        fill_color: u32,
        stroke_color: u32,
        stroke_width: f32,
    },
    CanvasDrawCircle {
        canvas_ptr: usize,
        cx: f32,
        cy: f32,
        radius: f32,
        fill_color: u32,
        stroke_color: u32,
        stroke_width: f32,
    },
    CanvasDrawLine {
        canvas_ptr: usize,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        color: u32,
        stroke_width: f32,
    },
    CanvasDrawRoundRect {
        canvas_ptr: usize,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        rx: f32,
        ry: f32,
        fill_color: u32,
        stroke_color: u32,
        stroke_width: f32,
    },
    CanvasDrawPath {
        canvas_ptr: usize,
        path_id: u32,
        fill_color: u32,
        stroke_color: u32,
        stroke_width: f32,
    },
    CanvasDrawTextNode {
        canvas_ptr: usize,
        handle_lo: u32,
        handle_hi: u32,
        x: f32,
        y: f32,
    },
    CanvasDrawImage {
        canvas_ptr: usize,
        texture_id: u32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        sampling_kind: u32,
        max_aniso: u32,
    },
    CanvasDrawSvg {
        canvas_ptr: usize,
        svg_id: u32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    },
    CanvasDrawBatch {
        canvas_ptr: usize,
        words: Vec<u32>,
    },
    CanvasCreateOffscreen {
        width: u32,
        height: u32,
        offscreen_id: u32,
    },
    CanvasGetOffscreenPtr {
        offscreen_id: u32,
    },
    CanvasReadOffscreenPixels {
        offscreen_id: u32,
        width: u32,
        height: u32,
    },
    CanvasDestroyOffscreen {
        offscreen_id: u32,
    },
    CanvasSave {
        canvas_ptr: usize,
    },
    CanvasRestore {
        canvas_ptr: usize,
    },
    CanvasTranslate {
        canvas_ptr: usize,
        x: f32,
        y: f32,
    },
    CanvasScale {
        canvas_ptr: usize,
        sx: f32,
        sy: f32,
    },
    CanvasRotate {
        canvas_ptr: usize,
        degrees: f32,
    },
    CanvasClipRect {
        canvas_ptr: usize,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    },
    CanvasClipRoundRect {
        canvas_ptr: usize,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        tl: f32,
        tr: f32,
        br: f32,
        bl: f32,
    },
    GetBounds {
        handle: u64,
    },
    GetVisibleBounds {
        handle: u64,
    },
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
thread_local! {
    static CALLS: std::cell::RefCell<Vec<Call>> = const { std::cell::RefCell::new(Vec::new()) };
    static NEXT_HANDLE: std::cell::Cell<u64> = const { std::cell::Cell::new(1) };
    static VIEWPORT: std::cell::Cell<(f32, f32)> = const { std::cell::Cell::new((320.0, 220.0)) };
    static DEVICE_PIXEL_RATIO: std::cell::Cell<f32> = const { std::cell::Cell::new(1.0) };
    static CAN_NAVIGATE_BACK: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static CAN_NAVIGATE_FORWARD: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static LOGS_ENABLED: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
    static HOST_NOW_MS: std::cell::Cell<f64> = const { std::cell::Cell::new(0.0) };
    static SYSTEM_DARK_MODE: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
    static SYSTEM_ACCENT_COLOR: std::cell::Cell<u32> = const { std::cell::Cell::new(0x2563EBFF) };
    static PLATFORM_FAMILY: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    static HOST_ENVIRONMENT: std::cell::Cell<u32> = const { std::cell::Cell::new(3) };
    static HOST_CAPABILITIES: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    static COARSE_POINTER: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static HAS_TEXT_SELECTION_SNAPSHOT: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static HAS_TEXT_SELECTION: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static IS_POINT_IN_SELECTION: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static TEXT_RANGE_RECTS: std::cell::RefCell<Vec<(f32, f32, f32, f32)>> = const { std::cell::RefCell::new(Vec::new()) };
    static CROSS_SELECTION_ENDPOINT_RECTS: std::cell::RefCell<Option<[(f32, f32, f32, f32); 2]>> = const { std::cell::RefCell::new(None) };
    static BEGIN_SELECTION_ENDPOINT_DRAG_RESULT: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
    static CAN_UNDO_TEXT_EDIT: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static CAN_REDO_TEXT_EDIT: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static NEXT_PATH_ID: std::cell::Cell<u32> = const { std::cell::Cell::new(1) };
    static NEXT_OFFSCREEN_ID: std::cell::Cell<u32> = const { std::cell::Cell::new(1) };
    static TEXT_METRICS: std::cell::Cell<(f32, f32, f32, u32, f32)> = const { std::cell::Cell::new((100.0, 20.0, 15.0, 1, 100.0)) };
    static PERSISTED_SCROLL: std::cell::RefCell<std::collections::HashMap<String, (f32, f32)>> = std::cell::RefCell::new(std::collections::HashMap::new());
    static PERSISTED_TEXT: std::cell::RefCell<std::collections::HashMap<(String, String), (u32, String)>> = std::cell::RefCell::new(std::collections::HashMap::new());
    static DEBUG_TREE_WORDS: std::cell::RefCell<Vec<u32>> = const { std::cell::RefCell::new(Vec::new()) };
    static NEXT_SEMANTIC_SCOPE_TOKEN: std::cell::Cell<u32> = const { std::cell::Cell::new(1) };
    static VISIBLE_BOUNDS: std::cell::RefCell<Option<(f32, f32, f32, f32)>> = const { std::cell::RefCell::new(None) };
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
fn push_call(call: Call) {
    CALLS.with(|calls| calls.borrow_mut().push(call));
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub mod test {
    use super::{
        Call, BEGIN_SELECTION_ENDPOINT_DRAG_RESULT, CALLS, CAN_NAVIGATE_BACK, CAN_NAVIGATE_FORWARD,
        CAN_REDO_TEXT_EDIT, CAN_UNDO_TEXT_EDIT, COARSE_POINTER, CROSS_SELECTION_ENDPOINT_RECTS,
        DEBUG_TREE_WORDS, DEVICE_PIXEL_RATIO, HAS_TEXT_SELECTION, HAS_TEXT_SELECTION_SNAPSHOT,
        HOST_CAPABILITIES, HOST_ENVIRONMENT, HOST_NOW_MS, IS_POINT_IN_SELECTION, LOGS_ENABLED,
        NEXT_HANDLE, NEXT_OFFSCREEN_ID, NEXT_PATH_ID, NEXT_SEMANTIC_SCOPE_TOKEN, PERSISTED_SCROLL,
        PERSISTED_TEXT, PLATFORM_FAMILY, SYSTEM_ACCENT_COLOR, SYSTEM_DARK_MODE, TEXT_METRICS,
        TEXT_RANGE_RECTS, VIEWPORT, VISIBLE_BOUNDS,
    };

    pub fn reset() {
        crate::theme::current_theme();
        CALLS.with(|calls| calls.borrow_mut().clear());
        NEXT_HANDLE.with(|next| next.set(1));
        VIEWPORT.with(|viewport| viewport.set((320.0, 220.0)));
        DEVICE_PIXEL_RATIO.with(|value| value.set(1.0));
        CAN_NAVIGATE_BACK.with(|value| value.set(false));
        CAN_NAVIGATE_FORWARD.with(|value| value.set(false));
        LOGS_ENABLED.with(|value| value.set(true));
        HOST_NOW_MS.with(|value| value.set(0.0));
        SYSTEM_DARK_MODE.with(|value| value.set(true));
        SYSTEM_ACCENT_COLOR.with(|value| value.set(0x2563EBFF));
        PLATFORM_FAMILY.with(|value| value.set(0));
        HOST_ENVIRONMENT.with(|value| value.set(3));
        HOST_CAPABILITIES.with(|value| value.set(0));
        COARSE_POINTER.with(|value| value.set(false));
        HAS_TEXT_SELECTION_SNAPSHOT.with(|value| value.set(false));
        HAS_TEXT_SELECTION.with(|value| value.set(false));
        IS_POINT_IN_SELECTION.with(|value| value.set(false));
        TEXT_RANGE_RECTS.with(|value| value.borrow_mut().clear());
        CROSS_SELECTION_ENDPOINT_RECTS.with(|value| value.borrow_mut().take());
        BEGIN_SELECTION_ENDPOINT_DRAG_RESULT.with(|value| value.set(true));
        CAN_UNDO_TEXT_EDIT.with(|value| value.set(false));
        CAN_REDO_TEXT_EDIT.with(|value| value.set(false));
        NEXT_PATH_ID.with(|next| next.set(1));
        NEXT_OFFSCREEN_ID.with(|next| next.set(1));
        TEXT_METRICS.with(|value| value.set((100.0, 20.0, 15.0, 1, 100.0)));
        PERSISTED_SCROLL.with(|map| map.borrow_mut().clear());
        PERSISTED_TEXT.with(|map| map.borrow_mut().clear());
        DEBUG_TREE_WORDS.with(|words| words.borrow_mut().clear());
        NEXT_SEMANTIC_SCOPE_TOKEN.with(|next| next.set(1));
        VISIBLE_BOUNDS.with(|bounds| bounds.borrow_mut().take());
    }

    pub fn take_calls() -> Vec<Call> {
        CALLS.with(|calls| std::mem::take(&mut *calls.borrow_mut()))
    }

    pub fn set_viewport(width: f32, height: f32) {
        VIEWPORT.with(|viewport| viewport.set((width, height)));
    }

    pub fn set_device_pixel_ratio(value: f32) {
        DEVICE_PIXEL_RATIO.with(|dpr| dpr.set(value));
    }

    pub fn set_can_navigate_back(value: bool) {
        CAN_NAVIGATE_BACK.with(|flag| flag.set(value));
    }

    pub fn set_can_navigate_forward(value: bool) {
        CAN_NAVIGATE_FORWARD.with(|flag| flag.set(value));
    }

    pub fn set_logs_enabled(value: bool) {
        LOGS_ENABLED.with(|flag| flag.set(value));
    }

    pub fn set_host_now_ms(value: f64) {
        HOST_NOW_MS.with(|slot| slot.set(value));
    }

    pub fn host_now_ms() -> f64 {
        HOST_NOW_MS.with(|slot| slot.get())
    }

    pub fn set_system_dark_mode(value: bool) {
        SYSTEM_DARK_MODE.with(|flag| flag.set(value));
    }

    pub fn set_system_accent_color(value: u32) {
        SYSTEM_ACCENT_COLOR.with(|color| color.set(value));
    }

    pub fn set_platform_family(value: u32) {
        PLATFORM_FAMILY.with(|family| family.set(value));
    }

    pub fn set_host_environment(value: u32) {
        HOST_ENVIRONMENT.with(|environment| environment.set(value));
    }

    pub fn set_host_capabilities(value: u32) {
        HOST_CAPABILITIES.with(|capabilities| capabilities.set(value));
    }

    pub fn set_coarse_pointer(value: bool) {
        COARSE_POINTER.with(|flag| flag.set(value));
    }

    pub fn set_has_text_selection_snapshot(value: bool) {
        HAS_TEXT_SELECTION_SNAPSHOT.with(|flag| flag.set(value));
    }

    pub fn set_has_text_selection(value: bool) {
        HAS_TEXT_SELECTION.with(|flag| flag.set(value));
    }

    pub fn set_is_point_in_selection(value: bool) {
        IS_POINT_IN_SELECTION.with(|flag| flag.set(value));
    }

    pub fn set_text_range_rects(rects: &[(f32, f32, f32, f32)]) {
        TEXT_RANGE_RECTS.with(|slot| {
            let mut buffer = slot.borrow_mut();
            buffer.clear();
            buffer.extend_from_slice(rects);
        });
    }

    pub fn set_cross_selection_endpoint_rects(value: Option<[(f32, f32, f32, f32); 2]>) {
        CROSS_SELECTION_ENDPOINT_RECTS.with(|slot| {
            *slot.borrow_mut() = value;
        });
    }

    pub fn set_begin_selection_endpoint_drag_result(value: bool) {
        BEGIN_SELECTION_ENDPOINT_DRAG_RESULT.with(|slot| slot.set(value));
    }

    pub fn set_can_undo_text_edit(value: bool) {
        CAN_UNDO_TEXT_EDIT.with(|flag| flag.set(value));
    }

    pub fn set_can_redo_text_edit(value: bool) {
        CAN_REDO_TEXT_EDIT.with(|flag| flag.set(value));
    }

    pub fn set_text_metrics(
        width: f32,
        height: f32,
        baseline: f32,
        line_count: u32,
        max_line_width: f32,
    ) {
        TEXT_METRICS.with(|value| value.set((width, height, baseline, line_count, max_line_width)));
    }

    pub fn set_debug_tree_words(words: &[u32]) {
        DEBUG_TREE_WORDS.with(|slot| {
            let mut buffer = slot.borrow_mut();
            buffer.clear();
            buffer.extend_from_slice(words);
        });
    }

    pub fn set_visible_bounds(value: Option<(f32, f32, f32, f32)>) {
        VISIBLE_BOUNDS.with(|slot| {
            *slot.borrow_mut() = value;
        });
    }
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_reset() {
    push_call(Call::Reset);
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_create_node(type_: u32) -> u64 {
    let handle = NEXT_HANDLE.with(|next| {
        let handle = next.get();
        next.set(handle + 1);
        handle
    });
    push_call(Call::CreateNode {
        node_type: type_,
        handle,
    });
    handle
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_delete_node(handle: u64) {
    push_call(Call::DeleteNode { handle });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_node_add_child(parent: u64, child: u64) {
    push_call(Call::NodeAddChild { parent, child });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_node_remove_child(parent: u64, child: u64) {
    push_call(Call::NodeRemoveChild { parent, child });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_root(handle: u64) {
    push_call(Call::SetRoot { handle });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_node_id(handle: u64, utf8_id: *const u8, len: u32) {
    let node_id = if utf8_id.is_null() || len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(utf8_id, len as usize)).into_owned()
    };
    push_call(Call::SetNodeId { handle, node_id });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_semantic_role(handle: u64, role_enum: u32) {
    push_call(Call::SetSemanticRole { handle, role_enum });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_semantic_expanded(handle: u64, has_expanded: bool, is_expanded: bool) {
    push_call(Call::SetSemanticExpanded {
        handle,
        has_expanded,
        is_expanded,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_semantic_label(handle: u64, utf8_label: *const u8, len: u32) {
    let label = if utf8_label.is_null() || len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(utf8_label, len as usize)).into_owned()
    };
    push_call(Call::SetSemanticLabel { handle, label });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_semantic_checked(handle: u64, checked_state_enum: u32) {
    push_call(Call::SetSemanticChecked {
        handle,
        checked_state_enum,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_semantic_selected(handle: u64, has_selected: bool, is_selected: bool) {
    push_call(Call::SetSemanticSelected {
        handle,
        has_selected,
        selected: is_selected,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_semantic_disabled(handle: u64, has_disabled: bool, disabled: bool) {
    push_call(Call::SetSemanticDisabled {
        handle,
        has_disabled,
        disabled,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_semantic_value_range(
    handle: u64,
    has_value_range: bool,
    value_now: f32,
    value_min: f32,
    value_max: f32,
) {
    push_call(Call::SetSemanticValueRange {
        handle,
        has_value_range,
        value_now,
        value_min,
        value_max,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_semantic_orientation(handle: u64, orientation_enum: u32) {
    push_call(Call::SetSemanticOrientation {
        handle,
        orientation_enum,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_request_semantic_announcement(handle: u64) {
    push_call(Call::RequestSemanticAnnouncement { handle });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_push_semantic_scope(handle: u64) -> u32 {
    let token = NEXT_SEMANTIC_SCOPE_TOKEN.with(|next| {
        let token = next.get();
        next.set(token + 1);
        token
    });
    push_call(Call::PushSemanticScope { handle, token });
    token
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_remove_semantic_scope(token: u32) {
    push_call(Call::RemoveSemanticScope { token });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_is_portal(handle: u64, is_portal: bool) {
    push_call(Call::SetIsPortal { handle, is_portal });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_visibility(handle: u64, visibility_enum: u32) {
    push_call(Call::SetVisibility {
        handle,
        visibility_enum,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_width(handle: u64, value: f32, unit_enum: u32) {
    push_call(Call::SetWidth {
        handle,
        value,
        unit_enum,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_height(handle: u64, value: f32, unit_enum: u32) {
    push_call(Call::SetHeight {
        handle,
        value,
        unit_enum,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_fill_width(handle: u64, fill: bool) {
    push_call(Call::SetFillWidth { handle, fill });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_fill_height(handle: u64, fill: bool) {
    push_call(Call::SetFillHeight { handle, fill });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_fill_width_percent(handle: u64, percent: f32) {
    push_call(Call::SetFillWidthPercent { handle, percent });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_fill_height_percent(handle: u64, percent: f32) {
    push_call(Call::SetFillHeightPercent { handle, percent });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_min_width(handle: u64, value: f32, unit_enum: u32) {
    push_call(Call::SetMinWidth {
        handle,
        value,
        unit_enum,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_max_width(handle: u64, value: f32, unit_enum: u32) {
    push_call(Call::SetMaxWidth {
        handle,
        value,
        unit_enum,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_min_height(handle: u64, value: f32, unit_enum: u32) {
    push_call(Call::SetMinHeight {
        handle,
        value,
        unit_enum,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_max_height(handle: u64, value: f32, unit_enum: u32) {
    push_call(Call::SetMaxHeight {
        handle,
        value,
        unit_enum,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_bg_color(handle: u64, color: u32) {
    push_call(Call::SetBgColor { handle, color });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_box_style(
    handle: u64,
    bg_color: u32,
    radius_tl: f32,
    radius_tr: f32,
    radius_br: f32,
    radius_bl: f32,
    border_width: f32,
    border_color: u32,
    border_style_enum: u32,
    border_dash_on: f32,
    border_dash_off: f32,
) {
    push_call(Call::SetBoxStyle {
        handle,
        bg_color,
        radius_tl,
        radius_tr,
        radius_br,
        radius_bl,
        border_width,
        border_color,
        border_style_enum,
        border_dash_on,
        border_dash_off,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_linear_gradient(
    handle: u64,
    sx: f32,
    sy: f32,
    ex: f32,
    ey: f32,
    stop_count: u32,
    offsets: *const f32,
    colors: *const u32,
) {
    let len = stop_count as usize;
    let copied_offsets = if offsets.is_null() || len == 0 {
        Vec::new()
    } else {
        std::slice::from_raw_parts(offsets, len).to_vec()
    };
    let copied_colors = if colors.is_null() || len == 0 {
        Vec::new()
    } else {
        std::slice::from_raw_parts(colors, len).to_vec()
    };
    push_call(Call::SetLinearGradient {
        handle,
        sx,
        sy,
        ex,
        ey,
        offsets: copied_offsets,
        colors: copied_colors,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_drop_shadow(
    handle: u64,
    color: u32,
    offset_x: f32,
    offset_y: f32,
    blur_sigma: f32,
    spread: f32,
) {
    push_call(Call::SetDropShadow {
        handle,
        color,
        offset_x,
        offset_y,
        blur_sigma,
        spread,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_layer_effect(
    handle: u64,
    opacity: f32,
    blur_sigma: f32,
    blend_mode_enum: u32,
) {
    push_call(Call::SetLayerEffect {
        handle,
        opacity,
        blur_sigma,
        blend_mode_enum,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_background_blur(handle: u64, blur_sigma: f32) {
    push_call(Call::SetBackgroundBlur { handle, blur_sigma });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_text(handle: u64, utf8_str: *const u8, len: u32) {
    let text = if utf8_str.is_null() || len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(utf8_str, len as usize)).into_owned()
    };
    push_call(Call::SetText { handle, text });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_text_style_runs(handle: u64, run_count: u32, runs_words: *const u32) {
    let len = (run_count as usize).saturating_mul(7);
    let words = if runs_words.is_null() || len == 0 {
        Vec::new()
    } else {
        std::slice::from_raw_parts(runs_words, len).to_vec()
    };
    push_call(Call::SetTextStyleRuns {
        handle,
        run_count,
        words,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_prepare_node(handle: u64) -> u32 {
    push_call(Call::PrepareNode { handle });
    handle as u32
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_dynamic_text_charset(handle: u64, utf8_charset: *const u8, len: u32) {
    let charset = if utf8_charset.is_null() || len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(utf8_charset, len as usize)).into_owned()
    };
    push_call(Call::SetDynamicTextCharset { handle, charset });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_get_text_metrics(
    handle: u64,
    out_width: *mut f32,
    out_height: *mut f32,
    out_baseline: *mut f32,
    out_line_count: *mut u32,
    out_max_line_width: *mut f32,
) -> bool {
    push_call(Call::GetTextMetrics { handle });
    let (width, height, baseline, line_count, max_line_width) =
        TEXT_METRICS.with(|value| value.get());
    if !out_width.is_null() {
        *out_width = width;
    }
    if !out_height.is_null() {
        *out_height = height;
    }
    if !out_baseline.is_null() {
        *out_baseline = baseline;
    }
    if !out_line_count.is_null() {
        *out_line_count = line_count;
    }
    if !out_max_line_width.is_null() {
        *out_max_line_width = max_line_width;
    }
    true
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_font(handle: u64, font_id: u32, size: f32) {
    push_call(Call::SetFont {
        handle,
        font_id,
        size,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_register_font_fallback(font_id: u32, fallback_font_id: u32) {
    push_call(Call::RegisterFontFallback {
        font_id,
        fallback_font_id,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_line_height(handle: u64, line_height: f32) {
    push_call(Call::SetLineHeight {
        handle,
        line_height,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_text_color(handle: u64, color: u32) {
    push_call(Call::SetTextColor { handle, color });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_text_align(handle: u64, align_enum: u32) {
    push_call(Call::SetTextAlign { handle, align_enum });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_text_vertical_align(handle: u64, align_enum: u32) {
    push_call(Call::SetTextVerticalAlign { handle, align_enum });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_text_limits(handle: u64, max_chars: i32, max_lines: i32) {
    push_call(Call::SetTextLimits {
        handle,
        max_chars,
        max_lines,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_text_wrapping(handle: u64, wrap: bool) {
    push_call(Call::SetTextWrapping { handle, wrap });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_text_obscured(handle: u64, obscured: bool) {
    push_call(Call::SetTextObscured { handle, obscured });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_text_overflow(handle: u64, overflow_enum: u32) {
    push_call(Call::SetTextOverflow {
        handle,
        overflow_enum,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_text_overflow_fade(handle: u64, horizontal: bool, vertical: bool) {
    push_call(Call::SetTextOverflowFade {
        handle,
        horizontal,
        vertical,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_selectable(handle: u64, selectable: bool, selection_color: u32) {
    push_call(Call::SetSelectable {
        handle,
        selectable,
        selection_color,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_editable(handle: u64, editable: bool) {
    push_call(Call::SetEditable { handle, editable });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_editor_command_keys(handle: u64, enabled: bool) {
    push_call(Call::SetEditorCommandKeys { handle, enabled });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_editor_accepts_tab(handle: u64, enabled: bool) {
    push_call(Call::SetEditorAcceptsTab { handle, enabled });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_replace_text_range(
    handle: u64,
    start: u32,
    end: u32,
    text_ptr: *const u8,
    text_len: u32,
    caret: u32,
) {
    let text = if text_ptr.is_null() || text_len == 0 {
        String::new()
    } else {
        let bytes = std::slice::from_raw_parts(text_ptr, text_len as usize);
        String::from_utf8_lossy(bytes).into_owned()
    };
    push_call(Call::ReplaceTextRange {
        handle,
        start,
        end,
        text,
        caret,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_caret_color(handle: u64, color: u32) {
    push_call(Call::SetCaretColor { handle, color });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_text_selection_range(handle: u64, start: u32, end: u32) {
    push_call(Call::SetTextSelectionRange { handle, start, end });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_preserve_selection_on_pointer_down(handle: u64, preserve: bool) {
    push_call(Call::SetPreserveSelectionOnPointerDown { handle, preserve });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_register_text_input_metadata(
    handle: u64,
    is_password: bool,
    hint_ptr: usize,
    hint_len: u32,
) {
    let hint = if hint_ptr == 0 || hint_len == 0 {
        String::new()
    } else {
        let bytes = std::slice::from_raw_parts(hint_ptr as *const u8, hint_len as usize);
        String::from_utf8_lossy(bytes).into_owned()
    };
    push_call(Call::RegisterTextInputMetadata {
        handle,
        is_password,
        hint,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_interactive(handle: u64, interactive: bool) {
    push_call(Call::SetInteractive {
        handle,
        interactive,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_focusable(handle: u64, focusable: bool, tab_index: i32) {
    push_call(Call::SetFocusable {
        handle,
        focusable,
        tab_index,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_request_focus(handle: u64) {
    push_call(Call::RequestFocus { handle });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_padding(handle: u64, left: f32, top: f32, right: f32, bottom: f32) {
    push_call(Call::SetPadding {
        handle,
        left,
        top,
        right,
        bottom,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_flex_direction(handle: u64, dir_enum: u32) {
    push_call(Call::SetFlexDirection { handle, dir_enum });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_flex_basis(handle: u64, basis: f32) {
    push_call(Call::SetFlexBasis { handle, basis });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_justify_content(handle: u64, justify_enum: u32) {
    push_call(Call::SetJustifyContent {
        handle,
        justify_enum,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_align_items(handle: u64, align_enum: u32) {
    push_call(Call::SetAlignItems { handle, align_enum });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_align_self(handle: u64, align_enum: u32) {
    push_call(Call::SetAlignSelf { handle, align_enum });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_margin(handle: u64, left: f32, top: f32, right: f32, bottom: f32) {
    push_call(Call::SetMargin {
        handle,
        left,
        top,
        right,
        bottom,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_position_type(handle: u64, pos_enum: u32) {
    push_call(Call::SetPositionType { handle, pos_enum });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_position(handle: u64, left: f32, top: f32, right: f32, bottom: f32) {
    push_call(Call::SetPosition {
        handle,
        left,
        top,
        right,
        bottom,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_is_shared_size_scope(handle: u64, is_scope: bool) {
    push_call(Call::SetIsSharedSizeScope { handle, is_scope });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_custom_drawable(handle: u64, is_custom_drawable: bool) {
    push_call(Call::SetCustomDrawable {
        handle,
        is_custom_drawable,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_flex_wrap(handle: u64, wrap_enum: u32) {
    push_call(Call::SetFlexWrap { handle, wrap_enum });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_clip_to_bounds(handle: u64, clip: bool) {
    push_call(Call::SetClipToBounds { handle, clip });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_selection_area(handle: u64, is_area: bool) {
    push_call(Call::SetSelectionArea { handle, is_area });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_selection_area_barrier(handle: u64, is_barrier: bool) {
    push_call(Call::SetSelectionAreaBarrier { handle, is_barrier });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_clear_selection(text_node_handle: u64) {
    push_call(Call::ClearSelection { text_node_handle });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_retarget_selection(from_text_node_handle: u64, to_text_node_handle: u64) {
    push_call(Call::RetargetSelection {
        from_text_node_handle,
        to_text_node_handle,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_grid_set_columns(handle: u64, count: u32, values: *const f32, types: *const u8) {
    let len = count as usize;
    let copied_values = if values.is_null() || len == 0 {
        Vec::new()
    } else {
        std::slice::from_raw_parts(values, len).to_vec()
    };
    let copied_types = if types.is_null() || len == 0 {
        Vec::new()
    } else {
        std::slice::from_raw_parts(types, len).to_vec()
    };
    push_call(Call::GridSetColumns {
        handle,
        values: copied_values,
        type_enums: copied_types,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_grid_set_rows(handle: u64, count: u32, values: *const f32, types: *const u8) {
    let len = count as usize;
    let copied_values = if values.is_null() || len == 0 {
        Vec::new()
    } else {
        std::slice::from_raw_parts(values, len).to_vec()
    };
    let copied_types = if types.is_null() || len == 0 {
        Vec::new()
    } else {
        std::slice::from_raw_parts(types, len).to_vec()
    };
    push_call(Call::GridSetRows {
        handle,
        values: copied_values,
        type_enums: copied_types,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_grid_set_column_shared_size_group(
    handle: u64,
    index: u32,
    utf8_group: *const u8,
    len: u32,
) {
    let group = if utf8_group.is_null() || len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(utf8_group, len as usize)).into_owned()
    };
    push_call(Call::GridSetColumnSharedSizeGroup {
        handle,
        index,
        group,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_grid_set_row_shared_size_group(
    handle: u64,
    index: u32,
    utf8_group: *const u8,
    len: u32,
) {
    let group = if utf8_group.is_null() || len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(utf8_group, len as usize)).into_owned()
    };
    push_call(Call::GridSetRowSharedSizeGroup {
        handle,
        index,
        group,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_node_set_grid_placement(
    child: u64,
    row: u32,
    col: u32,
    row_span: u32,
    col_span: u32,
) {
    push_call(Call::NodeSetGridPlacement {
        handle: child,
        row,
        col,
        row_span,
        col_span,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_image(
    handle: u64,
    texture_id: u32,
    object_fit_enum: u32,
    sampling_kind: u32,
    max_aniso: u32,
) {
    push_call(Call::SetImage {
        handle,
        texture_id,
        object_fit_enum,
        sampling_kind,
        max_aniso,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_image_nine(
    handle: u64,
    texture_id: u32,
    inset_left: f32,
    inset_top: f32,
    inset_right: f32,
    inset_bottom: f32,
    sampling_kind: u32,
    max_aniso: u32,
) {
    push_call(Call::SetImageNine {
        handle,
        texture_id,
        inset_left,
        inset_top,
        inset_right,
        inset_bottom,
        sampling_kind,
        max_aniso,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_svg(
    handle: u64,
    svg_id: u32,
    tint_color: u32,
    sampling_kind: u32,
    max_aniso: u32,
) {
    push_call(Call::SetSvg {
        handle,
        svg_id,
        tint_color,
        sampling_kind,
        max_aniso,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_scroll_proxy_target(handle: u64, scroll_handle: u64) {
    push_call(Call::SetScrollProxyTarget {
        handle,
        scroll_handle,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_scroll_enabled(handle: u64, enabled_x: bool, enabled_y: bool) {
    push_call(Call::SetScrollEnabled {
        handle,
        enabled_x,
        enabled_y,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_scroll_friction(handle: u64, friction: f32) {
    push_call(Call::SetScrollFriction { handle, friction });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_smooth_scrolling(handle: u64, smooth_scrolling: bool) {
    push_call(Call::SetSmoothScrolling {
        handle,
        smooth_scrolling,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_scroll_offset(handle: u64, offset_x: f32, offset_y: f32) {
    push_call(Call::SetScrollOffset {
        handle,
        offset_x,
        offset_y,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_set_scroll_content_size(handle: u64, content_width: f32, content_height: f32) {
    push_call(Call::SetScrollContentSize {
        handle,
        content_width,
        content_height,
    });
}

#[cfg(target_arch = "wasm32")]
pub unsafe fn ui_clear_momentum_scroll() {
    crate::generated::ffi::ui_clear_momentum_scroll()
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_clear_momentum_scroll() {
    push_call(Call::ClearMomentumScroll);
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_commit_frame() {
    push_call(Call::CommitFrame);
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_resize_window(logical_w: f32, logical_h: f32) {
    VIEWPORT.with(|viewport| viewport.set((logical_w, logical_h)));
    push_call(Call::ResizeWindow {
        logical_w,
        logical_h,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn request_render() {
    push_call(Call::RequestRender);
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn get_viewport_width() -> f32 {
    push_call(Call::GetViewportWidth);
    VIEWPORT.with(|viewport| viewport.get().0)
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn get_viewport_height() -> f32 {
    push_call(Call::GetViewportHeight);
    VIEWPORT.with(|viewport| viewport.get().1)
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn get_device_pixel_ratio() -> f32 {
    push_call(Call::GetDevicePixelRatio);
    DEVICE_PIXEL_RATIO.with(|value| value.get())
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_set_pointer_capture(handle: u64) {
    push_call(Call::SetPointerCapture { handle });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_release_pointer_capture() {
    push_call(Call::ReleasePointerCapture);
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_can_navigate_back() -> bool {
    push_call(Call::CanNavigateBack);
    CAN_NAVIGATE_BACK.with(|value| value.get())
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_can_navigate_forward() -> bool {
    push_call(Call::CanNavigateForward);
    CAN_NAVIGATE_FORWARD.with(|value| value.get())
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_navigate_back() {
    push_call(Call::NavigateBack);
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_navigate_forward() {
    push_call(Call::NavigateForward);
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_reload_page() {
    push_call(Call::ReloadPage);
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_copy_text(ptr: usize, len: u32) {
    let text = if ptr == 0 || len == 0 {
        String::new()
    } else {
        let bytes = std::slice::from_raw_parts(ptr as *const u8, len as usize);
        String::from_utf8_lossy(bytes).into_owned()
    };
    push_call(Call::CopyText { text });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_set_application_caption(ptr: usize, len: u32) {
    let caption = if ptr == 0 || len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(ptr as *const u8, len as usize))
            .into_owned()
    };
    push_call(Call::SetApplicationCaption { caption });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_has_text_selection_snapshot(handle: u64) -> bool {
    push_call(Call::HasTextSelectionSnapshot { handle });
    HAS_TEXT_SELECTION_SNAPSHOT.with(|value| value.get())
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_freeze_text_selection_snapshot(_handle: u64) {}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_copy_text_selection_snapshot(handle: u64) -> bool {
    push_call(Call::CopyTextSelectionSnapshot { handle });
    true
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_cut_focused_text_selection() -> bool {
    push_call(Call::CutFocusedTextSelection);
    true
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_cut_text_selection_snapshot(handle: u64) -> bool {
    push_call(Call::CutTextSelectionSnapshot { handle });
    true
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_cut_text_range_snapshot(handle: u64, start: u32, end: u32) -> bool {
    push_call(Call::CutTextRangeSnapshot { handle, start, end });
    true
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_delete_focused_text_range(start: u32, end: u32) -> bool {
    push_call(Call::DeleteFocusedTextRange { start, end });
    true
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_commit_text_action_focus(handle: u64) {
    push_call(Call::CommitTextActionFocus { handle });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_copy_current_selection() {
    push_call(Call::CopyCurrentSelection);
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_has_text_selection(handle: u64) -> bool {
    push_call(Call::HasTextSelection { handle });
    HAS_TEXT_SELECTION.with(|value| value.get())
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_can_undo_text_edit(_handle: u64) -> bool {
    CAN_UNDO_TEXT_EDIT.with(|value| value.get())
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_can_redo_text_edit(_handle: u64) -> bool {
    CAN_REDO_TEXT_EDIT.with(|value| value.get())
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_undo_text_edit(handle: u64) {
    push_call(Call::UndoTextEdit { handle });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_redo_text_edit(handle: u64) {
    push_call(Call::RedoTextEdit { handle });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_copy_text_selection(handle: u64) {
    push_call(Call::CopyTextSelection { handle });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_cut_text_selection(handle: u64) {
    push_call(Call::CutTextSelection { handle });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_paste_text(handle: u64) {
    push_call(Call::PasteText { handle });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_select_all_text(handle: u64) {
    push_call(Call::SelectAllText { handle });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_select_word_at(handle: u64, logical_x: f32, logical_y: f32) -> bool {
    push_call(Call::SelectWordAt {
        handle,
        x: logical_x,
        y: logical_y,
    });
    true
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_clear_current_selection() {
    push_call(Call::ClearCurrentSelection);
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_is_point_in_selection(logical_x: f32, logical_y: f32) -> bool {
    push_call(Call::IsPointInSelection {
        x: logical_x,
        y: logical_y,
    });
    IS_POINT_IN_SELECTION.with(|value| value.get())
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_get_text_range_rect_count(handle: u64, start: u32, end: u32) -> u32 {
    push_call(Call::GetTextRangeRectCount { handle, start, end });
    TEXT_RANGE_RECTS.with(|value| value.borrow().len() as u32)
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_copy_text_range_rects(
    handle: u64,
    start: u32,
    end: u32,
    out_rect_words: *mut f32,
    max_rect_count: u32,
) -> u32 {
    push_call(Call::CopyTextRangeRects {
        handle,
        start,
        end,
        max_rect_count,
    });
    let rects = TEXT_RANGE_RECTS.with(|value| value.borrow().clone());
    let copied = rects.len().min(max_rect_count as usize);
    if !out_rect_words.is_null() {
        for (index, (x, y, width, height)) in rects.iter().take(copied).enumerate() {
            let base = index * 4;
            *out_rect_words.add(base) = *x;
            *out_rect_words.add(base + 1) = *y;
            *out_rect_words.add(base + 2) = *width;
            *out_rect_words.add(base + 3) = *height;
        }
    }
    copied as u32
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_copy_cross_selection_endpoint_rects(
    area_handle: u64,
    out_rect_words: *mut f32,
) -> bool {
    push_call(Call::CopyCrossSelectionEndpointRects {
        handle: area_handle,
    });
    let Some(rects) = CROSS_SELECTION_ENDPOINT_RECTS.with(|value| *value.borrow()) else {
        return false;
    };
    if !out_rect_words.is_null() {
        let [(sx, sy, sw, sh), (ex, ey, ew, eh)] = rects;
        *out_rect_words.add(0) = sx;
        *out_rect_words.add(1) = sy;
        *out_rect_words.add(2) = sw;
        *out_rect_words.add(3) = sh;
        *out_rect_words.add(4) = ex;
        *out_rect_words.add(5) = ey;
        *out_rect_words.add(6) = ew;
        *out_rect_words.add(7) = eh;
    }
    true
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_begin_selection_endpoint_drag(handle: u64, endpoint: u32) -> bool {
    push_call(Call::BeginSelectionEndpointDrag { handle, endpoint });
    BEGIN_SELECTION_ENDPOINT_DRAG_RESULT.with(|value| value.get())
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_start_timer(timer_id: u32, delay_ms: i32) {
    push_call(Call::StartTimer { timer_id, delay_ms });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_cancel_timer(timer_id: u32) {
    push_call(Call::CancelTimer { timer_id });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_set_cursor(style: u32) {
    push_call(Call::SetCursor { style });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_navigate_to(ptr: usize, len: u32, open_in_new_tab: bool) {
    let target = if ptr == 0 || len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(ptr as *const u8, len as usize))
            .into_owned()
    };
    push_call(Call::NavigateTo {
        target,
        open_in_new_tab,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_show_url_preview(ptr: usize, len: u32) {
    let url = if ptr == 0 || len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(ptr as *const u8, len as usize))
            .into_owned()
    };
    push_call(Call::ShowUrlPreview { url });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_hide_url_preview() {
    push_call(Call::HideUrlPreview);
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_log(
    category_ptr: usize,
    category_len: u32,
    message_ptr: usize,
    message_len: u32,
) {
    let category = if category_ptr == 0 || category_len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(
            category_ptr as *const u8,
            category_len as usize,
        ))
        .into_owned()
    };
    let message = if message_ptr == 0 || message_len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(
            message_ptr as *const u8,
            message_len as usize,
        ))
        .into_owned()
    };
    push_call(Call::Log { category, message });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_logs_enabled() -> bool {
    push_call(Call::LogsEnabled);
    LOGS_ENABLED.with(|value| value.get())
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_is_dark_mode() -> bool {
    push_call(Call::IsDarkMode);
    SYSTEM_DARK_MODE.with(|value| value.get())
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_get_accent_color() -> u32 {
    push_call(Call::GetAccentColor);
    SYSTEM_ACCENT_COLOR.with(|value| value.get())
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_get_platform_family() -> u32 {
    push_call(Call::GetPlatformFamily);
    PLATFORM_FAMILY.with(|value| value.get())
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_get_host_environment() -> u32 {
    push_call(Call::GetHostEnvironment);
    HOST_ENVIRONMENT.with(|value| value.get())
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_get_host_capabilities() -> u32 {
    push_call(Call::GetHostCapabilities);
    HOST_CAPABILITIES.with(|value| value.get())
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_is_coarse_pointer() -> bool {
    push_call(Call::IsCoarsePointer);
    COARSE_POINTER.with(|value| value.get())
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_load_svg(svg_id: u32, ptr: usize, len: u32) {
    let url = if ptr == 0 || len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(ptr as *const u8, len as usize))
            .into_owned()
    };
    push_call(Call::LoadSvg { svg_id, url });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_release_svg(svg_id: u32) {
    push_call(Call::ReleaseSvg { svg_id });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_load_texture(texture_id: u32, ptr: usize, len: u32) {
    let url = if ptr == 0 || len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(ptr as *const u8, len as usize))
            .into_owned()
    };
    push_call(Call::LoadTexture { texture_id, url });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_release_texture(texture_id: u32) {
    push_call(Call::ReleaseTexture { texture_id });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_load_font(font_id: u32, ptr: usize, len: u32) {
    let url = if ptr == 0 || len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(ptr as *const u8, len as usize))
            .into_owned()
    };
    push_call(Call::LoadFont { font_id, url });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_bitmap_commit(
    texture_id: u32,
    bytes_ptr: usize,
    bytes_len: u32,
    width: u32,
    height: u32,
) {
    let bytes = if bytes_ptr == 0 || bytes_len == 0 {
        Vec::new()
    } else {
        std::slice::from_raw_parts(bytes_ptr as *const u8, bytes_len as usize).to_vec()
    };
    push_call(Call::BitmapCommit {
        texture_id,
        bytes,
        width,
        height,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_bitmap_commit_dirty(
    texture_id: u32,
    bytes_ptr: usize,
    bytes_len: u32,
    full_width: u32,
    full_height: u32,
    sub_x: u32,
    sub_y: u32,
    sub_w: u32,
    sub_h: u32,
) {
    let bytes = if bytes_ptr == 0 || bytes_len == 0 {
        Vec::new()
    } else {
        std::slice::from_raw_parts(bytes_ptr as *const u8, bytes_len as usize).to_vec()
    };
    push_call(Call::BitmapCommitDirty {
        texture_id,
        bytes,
        full_width,
        full_height,
        sub_x,
        sub_y,
        sub_w,
        sub_h,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_bitmap_release(texture_id: u32) {
    push_call(Call::BitmapRelease { texture_id });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_render_node_to_rgba(
    handle: u64,
    width: u32,
    height: u32,
    _out_ptr: usize,
    out_capacity: u32,
    scale: f32,
    x: f32,
    y: f32,
) -> u32 {
    push_call(Call::RenderNodeToRgba {
        handle,
        width,
        height,
        out_capacity,
        scale,
        x,
        y,
    });
    out_capacity
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_fetch_start(
    request_id: u32,
    method_ptr: usize,
    method_len: u32,
    url_ptr: usize,
    url_len: u32,
    headers_ptr: usize,
    headers_len: u32,
    body_ptr: usize,
    body_len: u32,
) {
    let method = if method_ptr == 0 || method_len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(
            method_ptr as *const u8,
            method_len as usize,
        ))
        .into_owned()
    };
    let url = if url_ptr == 0 || url_len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(
            url_ptr as *const u8,
            url_len as usize,
        ))
        .into_owned()
    };
    let headers = if headers_ptr == 0 || headers_len < 4 {
        Vec::new()
    } else {
        let bytes = std::slice::from_raw_parts(headers_ptr as *const u8, headers_len as usize);
        let mut cursor = 0usize;
        let count = u32::from_le_bytes(bytes[cursor..cursor + 4].try_into().unwrap_or([0, 0, 0, 0]))
            as usize;
        cursor += 4;
        let mut parts = Vec::with_capacity(count);
        for _ in 0..count {
            if cursor + 4 > bytes.len() {
                break;
            }
            let len =
                u32::from_le_bytes(bytes[cursor..cursor + 4].try_into().unwrap_or([0, 0, 0, 0]))
                    as usize;
            cursor += 4;
            if cursor + len > bytes.len() {
                break;
            }
            parts.push(String::from_utf8_lossy(&bytes[cursor..cursor + len]).into_owned());
            cursor += len;
        }
        parts
    };
    let body = if body_ptr == 0 || body_len == 0 {
        Vec::new()
    } else {
        std::slice::from_raw_parts(body_ptr as *const u8, body_len as usize).to_vec()
    };
    push_call(Call::FetchStart {
        request_id,
        method,
        url,
        headers,
        body,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_fetch_cancel(request_id: u32) {
    push_call(Call::FetchCancel { request_id });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_set_persisted_scroll_offset(
    node_id_ptr: usize,
    node_id_len: u32,
    x: f32,
    y: f32,
) {
    let node_id = if node_id_ptr == 0 || node_id_len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(
            node_id_ptr as *const u8,
            node_id_len as usize,
        ))
        .into_owned()
    };
    push_call(Call::SetPersistedScrollOffset {
        node_id: node_id.clone(),
        x,
        y,
    });
    if !node_id.is_empty() {
        PERSISTED_SCROLL.with(|map| {
            map.borrow_mut().insert(node_id, (x, y));
        });
    }
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_try_get_persisted_scroll_offset(
    node_id_ptr: usize,
    node_id_len: u32,
    out_x: usize,
    out_y: usize,
) -> bool {
    let node_id = if node_id_ptr == 0 || node_id_len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(
            node_id_ptr as *const u8,
            node_id_len as usize,
        ))
        .into_owned()
    };
    push_call(Call::TryGetPersistedScrollOffset {
        node_id: node_id.clone(),
    });
    let entry = PERSISTED_SCROLL.with(|map| map.borrow().get(&node_id).copied());
    let Some((x, y)) = entry else {
        return false;
    };
    if out_x != 0 {
        *(out_x as *mut f32) = x;
    }
    if out_y != 0 {
        *(out_y as *mut f32) = y;
    }
    true
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_set_persisted_state(
    node_id_ptr: usize,
    node_id_len: u32,
    kind_ptr: usize,
    kind_len: u32,
    version: u32,
    payload_ptr: usize,
    payload_len: u32,
) {
    let node_id = if node_id_ptr == 0 || node_id_len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(
            node_id_ptr as *const u8,
            node_id_len as usize,
        ))
        .into_owned()
    };
    let kind = if kind_ptr == 0 || kind_len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(
            kind_ptr as *const u8,
            kind_len as usize,
        ))
        .into_owned()
    };
    let payload = if payload_ptr == 0 || payload_len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(
            payload_ptr as *const u8,
            payload_len as usize,
        ))
        .into_owned()
    };
    push_call(Call::SetPersistedState {
        node_id: node_id.clone(),
        kind: kind.clone(),
        version,
        payload: payload.clone(),
    });
    if !node_id.is_empty() && !kind.is_empty() {
        PERSISTED_TEXT.with(|map| {
            map.borrow_mut().insert((node_id, kind), (version, payload));
        });
    }
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_copy_persisted_state(
    node_id_ptr: usize,
    node_id_len: u32,
    kind_ptr: usize,
    kind_len: u32,
    out_version_ptr: usize,
    payload_ptr: usize,
    payload_capacity: u32,
) -> i32 {
    let node_id = if node_id_ptr == 0 || node_id_len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(
            node_id_ptr as *const u8,
            node_id_len as usize,
        ))
        .into_owned()
    };
    let kind = if kind_ptr == 0 || kind_len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(
            kind_ptr as *const u8,
            kind_len as usize,
        ))
        .into_owned()
    };
    push_call(Call::CopyPersistedState {
        node_id: node_id.clone(),
        kind: kind.clone(),
    });
    let Some((version, payload)) =
        PERSISTED_TEXT.with(|map| map.borrow().get(&(node_id, kind)).cloned())
    else {
        return -1;
    };
    if out_version_ptr != 0 {
        *(out_version_ptr as *mut u32) = version;
    }
    let bytes = payload.as_bytes();
    if bytes.len() > payload_capacity as usize {
        return bytes.len() as i32;
    }
    if payload_ptr != 0 && !bytes.is_empty() {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), payload_ptr as *mut u8, bytes.len());
    }
    bytes.len() as i32
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_worker_start_string(
    worker_id: u32,
    wasm_path_ptr: usize,
    wasm_path_len: u32,
    entry_ptr: usize,
    entry_len: u32,
    input_ptr: usize,
    input_len: u32,
) {
    let wasm_path = if wasm_path_ptr == 0 || wasm_path_len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(
            wasm_path_ptr as *const u8,
            wasm_path_len as usize,
        ))
        .into_owned()
    };
    let entry = if entry_ptr == 0 || entry_len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(
            entry_ptr as *const u8,
            entry_len as usize,
        ))
        .into_owned()
    };
    let input = if input_ptr == 0 || input_len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(
            input_ptr as *const u8,
            input_len as usize,
        ))
        .into_owned()
    };
    push_call(Call::WorkerStartString {
        worker_id,
        wasm_path,
        entry,
        input,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_worker_cancel(worker_id: u32) {
    push_call(Call::WorkerCancel { worker_id });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_file_capabilities() -> u32 {
    push_call(Call::FileCapabilities);
    (1 << 0) | (1 << 1) | (1 << 2) | (1 << 3) | (1 << 4) | (1 << 5) | (1 << 6)
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_file_pick(request_id: u32, accept_ptr: usize, accept_len: u32, multiple: bool) {
    let accept = if accept_ptr == 0 || accept_len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(
            accept_ptr as *const u8,
            accept_len as usize,
        ))
        .into_owned()
    };
    push_call(Call::FilePick {
        request_id,
        accept,
        multiple,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_file_read_chunk(
    request_id: u32,
    file_id_ptr: usize,
    file_id_len: u32,
    offset_bytes: u64,
    max_bytes: u32,
) {
    let file_id = if file_id_ptr == 0 || file_id_len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(
            file_id_ptr as *const u8,
            file_id_len as usize,
        ))
        .into_owned()
    };
    push_call(Call::FileReadChunk {
        request_id,
        file_id,
        offset_bytes,
        max_bytes,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_file_save_text(
    request_id: u32,
    suggested_name_ptr: usize,
    suggested_name_len: u32,
    mime_type_ptr: usize,
    mime_type_len: u32,
    file_extension_ptr: usize,
    file_extension_len: u32,
    text_ptr: usize,
    text_len: u32,
) {
    let suggested_name = if suggested_name_ptr == 0 || suggested_name_len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(
            suggested_name_ptr as *const u8,
            suggested_name_len as usize,
        ))
        .into_owned()
    };
    let mime_type = if mime_type_ptr == 0 || mime_type_len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(
            mime_type_ptr as *const u8,
            mime_type_len as usize,
        ))
        .into_owned()
    };
    let file_extension = if file_extension_ptr == 0 || file_extension_len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(
            file_extension_ptr as *const u8,
            file_extension_len as usize,
        ))
        .into_owned()
    };
    let text = if text_ptr == 0 || text_len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(
            text_ptr as *const u8,
            text_len as usize,
        ))
        .into_owned()
    };
    push_call(Call::FileSaveText {
        request_id,
        suggested_name,
        mime_type,
        file_extension,
        text,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_file_save_bytes(
    request_id: u32,
    suggested_name_ptr: usize,
    suggested_name_len: u32,
    mime_type_ptr: usize,
    mime_type_len: u32,
    file_extension_ptr: usize,
    file_extension_len: u32,
    bytes_ptr: usize,
    bytes_len: u32,
) {
    let suggested_name = if suggested_name_ptr == 0 || suggested_name_len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(
            suggested_name_ptr as *const u8,
            suggested_name_len as usize,
        ))
        .into_owned()
    };
    let mime_type = if mime_type_ptr == 0 || mime_type_len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(
            mime_type_ptr as *const u8,
            mime_type_len as usize,
        ))
        .into_owned()
    };
    let file_extension = if file_extension_ptr == 0 || file_extension_len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(
            file_extension_ptr as *const u8,
            file_extension_len as usize,
        ))
        .into_owned()
    };
    let bytes = if bytes_ptr == 0 || bytes_len == 0 {
        Vec::new()
    } else {
        std::slice::from_raw_parts(bytes_ptr as *const u8, bytes_len as usize).to_vec()
    };
    push_call(Call::FileSaveBytes {
        request_id,
        suggested_name,
        mime_type,
        file_extension,
        bytes,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_file_create_writer(
    request_id: u32,
    suggested_name_ptr: usize,
    suggested_name_len: u32,
    mime_type_ptr: usize,
    mime_type_len: u32,
    file_extension_ptr: usize,
    file_extension_len: u32,
) {
    let suggested_name = if suggested_name_ptr == 0 || suggested_name_len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(
            suggested_name_ptr as *const u8,
            suggested_name_len as usize,
        ))
        .into_owned()
    };
    let mime_type = if mime_type_ptr == 0 || mime_type_len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(
            mime_type_ptr as *const u8,
            mime_type_len as usize,
        ))
        .into_owned()
    };
    let file_extension = if file_extension_ptr == 0 || file_extension_len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(
            file_extension_ptr as *const u8,
            file_extension_len as usize,
        ))
        .into_owned()
    };
    push_call(Call::FileCreateWriter {
        request_id,
        suggested_name,
        mime_type,
        file_extension,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_file_writer_write_text(
    request_id: u32,
    writer_id_ptr: usize,
    writer_id_len: u32,
    text_ptr: usize,
    text_len: u32,
) {
    let writer_id = if writer_id_ptr == 0 || writer_id_len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(
            writer_id_ptr as *const u8,
            writer_id_len as usize,
        ))
        .into_owned()
    };
    let text = if text_ptr == 0 || text_len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(
            text_ptr as *const u8,
            text_len as usize,
        ))
        .into_owned()
    };
    push_call(Call::FileWriterWriteText {
        request_id,
        writer_id,
        text,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_file_writer_write_bytes(
    request_id: u32,
    writer_id_ptr: usize,
    writer_id_len: u32,
    bytes_ptr: usize,
    bytes_len: u32,
) {
    let writer_id = if writer_id_ptr == 0 || writer_id_len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(
            writer_id_ptr as *const u8,
            writer_id_len as usize,
        ))
        .into_owned()
    };
    let bytes = if bytes_ptr == 0 || bytes_len == 0 {
        Vec::new()
    } else {
        std::slice::from_raw_parts(bytes_ptr as *const u8, bytes_len as usize).to_vec()
    };
    push_call(Call::FileWriterWriteBytes {
        request_id,
        writer_id,
        bytes,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_file_writer_finish(request_id: u32, writer_id_ptr: usize, writer_id_len: u32) {
    let writer_id = if writer_id_ptr == 0 || writer_id_len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(
            writer_id_ptr as *const u8,
            writer_id_len as usize,
        ))
        .into_owned()
    };
    push_call(Call::FileWriterFinish {
        request_id,
        writer_id,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_file_process_worker_start(
    request_id: u32,
    worker_wasm_path_ptr: usize,
    worker_wasm_path_len: u32,
    worker_entry_ptr: usize,
    worker_entry_len: u32,
    file_id_ptr: usize,
    file_id_len: u32,
    suggested_name_ptr: usize,
    suggested_name_len: u32,
    chunk_bytes: u32,
    save_to_picked_file: bool,
) {
    let worker_wasm_path = if worker_wasm_path_ptr == 0 || worker_wasm_path_len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(
            worker_wasm_path_ptr as *const u8,
            worker_wasm_path_len as usize,
        ))
        .into_owned()
    };
    let worker_entry_name = if worker_entry_ptr == 0 || worker_entry_len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(
            worker_entry_ptr as *const u8,
            worker_entry_len as usize,
        ))
        .into_owned()
    };
    let file_id = if file_id_ptr == 0 || file_id_len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(
            file_id_ptr as *const u8,
            file_id_len as usize,
        ))
        .into_owned()
    };
    let suggested_name = if suggested_name_ptr == 0 || suggested_name_len == 0 {
        String::new()
    } else {
        String::from_utf8_lossy(std::slice::from_raw_parts(
            suggested_name_ptr as *const u8,
            suggested_name_len as usize,
        ))
        .into_owned()
    };
    push_call(Call::FileProcessWorkerStart {
        request_id,
        worker_wasm_path,
        worker_entry_name,
        file_id,
        suggested_name,
        chunk_bytes,
        save_to_picked_file,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_file_process_worker_cancel(request_id: u32) {
    push_call(Call::FileProcessWorkerCancel { request_id });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_path_create() -> u32 {
    let path_id = NEXT_PATH_ID.with(|next| {
        let current = next.get();
        next.set(current + 1);
        current
    });
    push_call(Call::PathCreate { path_id });
    path_id
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_path_destroy(path_id: u32) {
    push_call(Call::PathDestroy { path_id });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_path_move_to(path_id: u32, x: f32, y: f32) {
    push_call(Call::PathMoveTo { path_id, x, y });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_path_line_to(path_id: u32, x: f32, y: f32) {
    push_call(Call::PathLineTo { path_id, x, y });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_path_quad_to(path_id: u32, cx: f32, cy: f32, x: f32, y: f32) {
    push_call(Call::PathQuadTo {
        path_id,
        cx,
        cy,
        x,
        y,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_path_cubic_to(
    path_id: u32,
    cx1: f32,
    cy1: f32,
    cx2: f32,
    cy2: f32,
    x: f32,
    y: f32,
) {
    push_call(Call::PathCubicTo {
        path_id,
        cx1,
        cy1,
        cx2,
        cy2,
        x,
        y,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_path_close(path_id: u32) {
    push_call(Call::PathClose { path_id });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_path_add_rect(path_id: u32, x: f32, y: f32, w: f32, h: f32) {
    push_call(Call::PathAddRect {
        path_id,
        x,
        y,
        w,
        h,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_path_add_circle(path_id: u32, cx: f32, cy: f32, r: f32) {
    push_call(Call::PathAddCircle { path_id, cx, cy, r });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_canvas_draw_rect(
    canvas_ptr: usize,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    fill_color: u32,
    stroke_color: u32,
    stroke_width: f32,
) {
    push_call(Call::CanvasDrawRect {
        canvas_ptr,
        x,
        y,
        w,
        h,
        fill_color,
        stroke_color,
        stroke_width,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_canvas_draw_circle(
    canvas_ptr: usize,
    cx: f32,
    cy: f32,
    radius: f32,
    fill_color: u32,
    stroke_color: u32,
    stroke_width: f32,
) {
    push_call(Call::CanvasDrawCircle {
        canvas_ptr,
        cx,
        cy,
        radius,
        fill_color,
        stroke_color,
        stroke_width,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_canvas_draw_line(
    canvas_ptr: usize,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    color: u32,
    stroke_width: f32,
) {
    push_call(Call::CanvasDrawLine {
        canvas_ptr,
        x1,
        y1,
        x2,
        y2,
        color,
        stroke_width,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_canvas_draw_round_rect(
    canvas_ptr: usize,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    rx: f32,
    ry: f32,
    fill_color: u32,
    stroke_color: u32,
    stroke_width: f32,
) {
    push_call(Call::CanvasDrawRoundRect {
        canvas_ptr,
        x,
        y,
        w,
        h,
        rx,
        ry,
        fill_color,
        stroke_color,
        stroke_width,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_canvas_draw_path(
    canvas_ptr: usize,
    path_id: u32,
    fill_color: u32,
    stroke_color: u32,
    stroke_width: f32,
) {
    push_call(Call::CanvasDrawPath {
        canvas_ptr,
        path_id,
        fill_color,
        stroke_color,
        stroke_width,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_canvas_draw_text_node(
    canvas_ptr: usize,
    handle_lo: u32,
    handle_hi: u32,
    x: f32,
    y: f32,
) {
    push_call(Call::CanvasDrawTextNode {
        canvas_ptr,
        handle_lo,
        handle_hi,
        x,
        y,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_canvas_draw_image(
    canvas_ptr: usize,
    texture_id: u32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    sampling_kind: u32,
    max_aniso: u32,
) {
    push_call(Call::CanvasDrawImage {
        canvas_ptr,
        texture_id,
        x,
        y,
        w,
        h,
        sampling_kind,
        max_aniso,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_canvas_draw_svg(canvas_ptr: usize, svg_id: u32, x: f32, y: f32, w: f32, h: f32) {
    push_call(Call::CanvasDrawSvg {
        canvas_ptr,
        svg_id,
        x,
        y,
        w,
        h,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_canvas_draw_batch(canvas_ptr: usize, words_ptr: usize, word_count: u32) {
    let words = if words_ptr == 0 || word_count == 0 {
        Vec::new()
    } else {
        std::slice::from_raw_parts(words_ptr as *const u32, word_count as usize).to_vec()
    };
    push_call(Call::CanvasDrawBatch { canvas_ptr, words });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_canvas_save(canvas_ptr: usize) {
    push_call(Call::CanvasSave { canvas_ptr });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_canvas_restore(canvas_ptr: usize) {
    push_call(Call::CanvasRestore { canvas_ptr });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_canvas_translate(canvas_ptr: usize, x: f32, y: f32) {
    push_call(Call::CanvasTranslate { canvas_ptr, x, y });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_canvas_scale(canvas_ptr: usize, sx: f32, sy: f32) {
    push_call(Call::CanvasScale { canvas_ptr, sx, sy });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_canvas_rotate(canvas_ptr: usize, degrees: f32) {
    push_call(Call::CanvasRotate {
        canvas_ptr,
        degrees,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_canvas_clip_rect(canvas_ptr: usize, x: f32, y: f32, w: f32, h: f32) {
    push_call(Call::CanvasClipRect {
        canvas_ptr,
        x,
        y,
        w,
        h,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_canvas_clip_round_rect(
    canvas_ptr: usize,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    tl: f32,
    tr: f32,
    br: f32,
    bl: f32,
) {
    push_call(Call::CanvasClipRoundRect {
        canvas_ptr,
        x,
        y,
        w,
        h,
        tl,
        tr,
        br,
        bl,
    });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_canvas_create_offscreen(width: u32, height: u32) -> u32 {
    let offscreen_id = NEXT_OFFSCREEN_ID.with(|next| {
        let value = next.get();
        next.set(value + 1);
        value
    });
    push_call(Call::CanvasCreateOffscreen {
        width,
        height,
        offscreen_id,
    });
    offscreen_id
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_canvas_get_offscreen_ptr(offscreen_id: u32) -> usize {
    push_call(Call::CanvasGetOffscreenPtr { offscreen_id });
    0x1000 + offscreen_id as usize
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_canvas_read_offscreen_pixels(
    offscreen_id: u32,
    out_ptr: usize,
    width: u32,
    height: u32,
) {
    push_call(Call::CanvasReadOffscreenPixels {
        offscreen_id,
        width,
        height,
    });
    if out_ptr != 0 {
        std::ptr::write_bytes(
            out_ptr as *mut u8,
            0,
            (width as usize) * (height as usize) * 4,
        );
    }
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn fui_canvas_destroy_offscreen(offscreen_id: u32) {
    push_call(Call::CanvasDestroyOffscreen { offscreen_id });
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_get_bounds(
    handle: u64,
    out_x: *mut f32,
    out_y: *mut f32,
    out_width: *mut f32,
    out_height: *mut f32,
) -> bool {
    push_call(Call::GetBounds { handle });
    if !out_x.is_null() {
        *out_x = 0.0;
    }
    if !out_y.is_null() {
        *out_y = 0.0;
    }
    if !out_width.is_null() {
        *out_width = 100.0;
    }
    if !out_height.is_null() {
        *out_height = 100.0;
    }
    true
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_get_visible_bounds(
    handle: u64,
    out_x: *mut f32,
    out_y: *mut f32,
    out_width: *mut f32,
    out_height: *mut f32,
) -> bool {
    push_call(Call::GetVisibleBounds { handle });
    let bounds = VISIBLE_BOUNDS.with(|slot| *slot.borrow());
    let Some((x, y, width, height)) = bounds else {
        if !out_x.is_null() {
            *out_x = 0.0;
        }
        if !out_y.is_null() {
            *out_y = 0.0;
        }
        if !out_width.is_null() {
            *out_width = 100.0;
        }
        if !out_height.is_null() {
            *out_height = 100.0;
        }
        return true;
    };
    if width <= 0.0 || height <= 0.0 {
        return false;
    }
    if !out_x.is_null() {
        *out_x = x;
    }
    if !out_y.is_null() {
        *out_y = y;
    }
    if !out_width.is_null() {
        *out_width = width;
    }
    if !out_height.is_null() {
        *out_height = height;
    }
    true
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-runtime")))]
pub unsafe fn ui_get_debug_tree_buffer(out_length: *mut u32) -> *mut u32 {
    let ptr = DEBUG_TREE_WORDS.with(|slot| {
        let words = slot.borrow();
        if !out_length.is_null() {
            *out_length = words.len() as u32;
        }
        words.as_ptr() as *mut u32
    });
    ptr
}
