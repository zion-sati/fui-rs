use crate::assets;
use crate::bindings::ui;
use crate::logger::warn;
use std::cell::RefCell;
use std::rc::Rc;

const STYLE_MISMATCH_PENALTY: i32 = 1000;
const MAX_FONT_SCORE: i32 = i32::MAX;
const FIRST_DYNAMIC_FONT_ID: u32 = 1024;

thread_local! {
    static NEXT_DYNAMIC_FONT_ID: RefCell<u32> = const { RefCell::new(FIRST_DYNAMIC_FONT_ID) };
    static FONT_LOADED_CALLBACKS: RefCell<Vec<FontLoadedRegistration>> = const { RefCell::new(Vec::new()) };
    static FONT_READY_CALLBACKS: RefCell<Vec<FontReadyRegistration>> = const { RefCell::new(Vec::new()) };
    static REGISTERED_FONT_FALLBACKS: RefCell<Vec<(u32, u32)>> = const { RefCell::new(Vec::new()) };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FontStyle {
    #[default]
    Normal = 0,
    Italic = 1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FontWeight {
    #[default]
    Regular = 400,
    Medium = 500,
    Semibold = 600,
    Bold = 700,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontFaceLoadedEventArgs {
    pub font: FontFace,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontsLoadedEventArgs;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontStackLoadedEventArgs {
    pub stack: FontStack,
}

struct FontLoadedRegistration {
    font_id: u32,
    callback: Rc<dyn Fn(FontFaceLoadedEventArgs)>,
}

struct FontReadyRegistration {
    font_ids: Vec<u32>,
    callback: Rc<dyn Fn()>,
}

fn abs_i32(value: i32) -> i32 {
    if value < 0 {
        -value
    } else {
        value
    }
}

fn allocate_dynamic_font_id() -> u32 {
    NEXT_DYNAMIC_FONT_ID.with(|slot| {
        let mut next = slot.borrow_mut();
        let allocated = *next;
        *next += 1;
        allocated
    })
}

fn push_unique_font_id(font_ids: &mut Vec<u32>, font_id: u32) {
    if font_id == 0 || font_ids.contains(&font_id) {
        return;
    }
    font_ids.push(font_id);
}

fn normalize_font_ids(font_ids: &[u32]) -> Vec<u32> {
    let mut unique = Vec::with_capacity(font_ids.len());
    for font_id in font_ids {
        push_unique_font_id(&mut unique, *font_id);
    }
    unique
}

fn register_font_fallback_once(font_id: u32, fallback_font_id: u32) {
    if font_id == 0 || fallback_font_id == 0 {
        return;
    }
    let already_registered = REGISTERED_FONT_FALLBACKS.with(|pairs| {
        pairs
            .borrow()
            .iter()
            .any(|(registered_font_id, registered_fallback_id)| {
                *registered_font_id == font_id && *registered_fallback_id == fallback_font_id
            })
    });
    if already_registered {
        return;
    }
    ui::register_font_fallback(font_id, fallback_font_id);
    REGISTERED_FONT_FALLBACKS.with(|pairs| {
        pairs.borrow_mut().push((font_id, fallback_font_id));
    });
}

fn are_font_ids_loaded(font_ids: &[u32]) -> bool {
    font_ids
        .iter()
        .all(|font_id| FontFace::is_font_loaded(*font_id))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontFace {
    id: u32,
}

impl FontFace {
    pub(crate) fn new(id: u32) -> Self {
        Self { id }
    }

    pub fn load(url: &str) -> Self {
        Self::load_with_id(url, allocate_dynamic_font_id())
    }

    pub(crate) fn load_with_id(url: &str, id: u32) -> Self {
        Self::new(id).load_into(url)
    }

    pub(crate) fn id(&self) -> u32 {
        self.id
    }

    fn load_into(self, url: &str) -> Self {
        if url.is_empty() {
            warn("Typography", "FontFace.load() received an empty font URL.");
        }
        assets::load_font(self.id, url);
        self
    }

    pub fn is_loaded(&self) -> bool {
        Self::is_font_loaded(self.id)
    }

    pub(crate) fn is_font_loaded(font_id: u32) -> bool {
        font_id == 0 || (1..=6).contains(&font_id) || assets::is_font_loaded(font_id)
    }

    pub fn on_loaded(&self, callback: impl Fn(FontFaceLoadedEventArgs) + 'static) -> Self {
        Self::when_loaded(self.id, callback);
        self.clone()
    }

    pub(crate) fn when_loaded(font_id: u32, callback: impl Fn(FontFaceLoadedEventArgs) + 'static) {
        if Self::is_font_loaded(font_id) {
            callback(FontFaceLoadedEventArgs {
                font: FontFace::new(font_id),
            });
            return;
        }
        FONT_LOADED_CALLBACKS.with(|registrations| {
            registrations.borrow_mut().push(FontLoadedRegistration {
                font_id,
                callback: Rc::new(callback),
            });
        });
    }

    pub(crate) fn when_fonts_loaded(
        font_ids: &[u32],
        callback: impl Fn(FontsLoadedEventArgs) + 'static,
    ) {
        let unique_font_ids = normalize_font_ids(font_ids);
        if are_font_ids_loaded(&unique_font_ids) {
            callback(FontsLoadedEventArgs);
            return;
        }
        FONT_READY_CALLBACKS.with(|registrations| {
            registrations.borrow_mut().push(FontReadyRegistration {
                font_ids: unique_font_ids,
                callback: Rc::new(move || callback(FontsLoadedEventArgs)),
            });
        });
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontStack {
    id: u32,
    fallback_ids: Vec<u32>,
}

impl FontStack {
    pub fn new(face: FontFace) -> Self {
        Self {
            id: face.id(),
            fallback_ids: Vec::new(),
        }
    }

    pub(crate) fn from_id(id: u32) -> Self {
        Self::new(FontFace::new(id))
    }

    pub fn load(url: &str) -> Self {
        Self::new(FontFace::load(url))
    }

    pub(crate) fn id(&self) -> u32 {
        self.id
    }

    pub fn fallback_face(mut self, face: FontFace) -> Self {
        self = self.fallback_id(face.id());
        self
    }

    pub fn fallback_stack(mut self, stack: FontStack) -> Self {
        self = self.fallback_id(stack.id());
        self
    }

    pub fn fallback_loaded(self, url: &str) -> Self {
        self.fallback_loaded_with_id(url, allocate_dynamic_font_id())
    }

    pub(crate) fn fallback_loaded_with_id(mut self, url: &str, font_id: u32) -> Self {
        if url.is_empty() {
            warn(
                "Typography",
                "FontStack.fallback_loaded() received an empty font URL.",
            );
        }
        assets::load_font(font_id, url);
        self = self.fallback_id(font_id);
        self
    }

    pub(crate) fn required_font_ids(&self) -> Vec<u32> {
        let mut font_ids = Vec::with_capacity(1 + self.fallback_ids.len());
        push_unique_font_id(&mut font_ids, self.id);
        for fallback_id in &self.fallback_ids {
            push_unique_font_id(&mut font_ids, *fallback_id);
        }
        font_ids
    }

    pub fn is_loaded(&self) -> bool {
        are_font_ids_loaded(&self.required_font_ids())
    }

    pub fn on_loaded(&self, callback: impl Fn(FontStackLoadedEventArgs) + 'static) -> Self {
        let stack = self.clone();
        let required = self.required_font_ids();
        FontFace::when_fonts_loaded(&required, move |_| {
            callback(FontStackLoadedEventArgs {
                stack: stack.clone(),
            });
        });
        self.clone()
    }

    fn fallback_id(mut self, font_id: u32) -> Self {
        if font_id == 0 || font_id == self.id {
            warn(
                "Typography",
                &format!(
                    "FontStack.fallback() ignored font id {} for stack {}.",
                    font_id, self.id
                ),
            );
            return self;
        }
        if self.fallback_ids.contains(&font_id) {
            return self;
        }
        register_font_fallback_once(self.id, font_id);
        self.fallback_ids.push(font_id);
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontFamily {
    pub regular_stack: FontStack,
    pub bold_stack: Option<FontStack>,
    pub italic_stack: Option<FontStack>,
    pub bold_italic_stack: Option<FontStack>,
    pub medium_stack: Option<FontStack>,
    pub medium_italic_stack: Option<FontStack>,
    pub semibold_stack: Option<FontStack>,
    pub semibold_italic_stack: Option<FontStack>,
}

impl FontFamily {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        regular_stack: FontStack,
        bold_stack: Option<FontStack>,
        italic_stack: Option<FontStack>,
        bold_italic_stack: Option<FontStack>,
        medium_stack: Option<FontStack>,
        medium_italic_stack: Option<FontStack>,
        semibold_stack: Option<FontStack>,
        semibold_italic_stack: Option<FontStack>,
    ) -> Self {
        Self {
            regular_stack,
            bold_stack,
            italic_stack,
            bold_italic_stack,
            medium_stack,
            medium_italic_stack,
            semibold_stack,
            semibold_italic_stack,
        }
    }

    #[cfg(test)]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn from_ids(
        regular: u32,
        bold: u32,
        italic: u32,
        bold_italic: u32,
        medium: u32,
        medium_italic: u32,
        semibold: u32,
        semibold_italic: u32,
    ) -> Self {
        Self::new(
            FontStack::from_id(regular),
            (bold != 0).then(|| FontStack::from_id(bold)),
            (italic != 0).then(|| FontStack::from_id(italic)),
            (bold_italic != 0).then(|| FontStack::from_id(bold_italic)),
            (medium != 0).then(|| FontStack::from_id(medium)),
            (medium_italic != 0).then(|| FontStack::from_id(medium_italic)),
            (semibold != 0).then(|| FontStack::from_id(semibold)),
            (semibold_italic != 0).then(|| FontStack::from_id(semibold_italic)),
        )
    }

    pub fn with_regular_stack(regular: FontStack) -> Self {
        Self::new(regular, None, None, None, None, None, None, None)
    }

    pub fn with_regular_face(regular: FontFace) -> Self {
        Self::with_regular_stack(FontStack::new(regular))
    }

    pub fn regular_bold_stacks(regular: FontStack, bold: FontStack) -> Self {
        Self::new(regular, Some(bold), None, None, None, None, None, None)
    }

    pub(crate) fn italic_stack(mut self, stack: FontStack) -> Self {
        self.italic_stack = Some(stack);
        self
    }

    pub(crate) fn bold_italic_stack(mut self, stack: FontStack) -> Self {
        self.bold_italic_stack = Some(stack);
        self
    }

    pub(crate) fn resolve(&self, weight: FontWeight, style: FontStyle) -> u32 {
        let target_weight = weight as i32;
        let candidates = [
            Some((&self.regular_stack, FontWeight::Regular, FontStyle::Normal)),
            self.bold_stack
                .as_ref()
                .map(|stack| (stack, FontWeight::Bold, FontStyle::Normal)),
            self.italic_stack
                .as_ref()
                .map(|stack| (stack, FontWeight::Regular, FontStyle::Italic)),
            self.bold_italic_stack
                .as_ref()
                .map(|stack| (stack, FontWeight::Bold, FontStyle::Italic)),
            self.medium_stack
                .as_ref()
                .map(|stack| (stack, FontWeight::Medium, FontStyle::Normal)),
            self.medium_italic_stack
                .as_ref()
                .map(|stack| (stack, FontWeight::Medium, FontStyle::Italic)),
            self.semibold_stack
                .as_ref()
                .map(|stack| (stack, FontWeight::Semibold, FontStyle::Normal)),
            self.semibold_italic_stack
                .as_ref()
                .map(|stack| (stack, FontWeight::Semibold, FontStyle::Italic)),
        ];

        let mut best_id = 0;
        let mut best_score = MAX_FONT_SCORE;
        for (stack, candidate_weight, candidate_style) in candidates.into_iter().flatten() {
            let score = Self::score_candidate(
                stack.id(),
                candidate_weight,
                candidate_style,
                target_weight,
                style,
            );
            if score < best_score {
                best_score = score;
                best_id = stack.id();
            }
        }
        if best_id == 0 {
            warn(
                "Typography",
                &format!(
                    "FontFamily.resolve() could not resolve a font face for weight {} and style {}; the text will use font id 0.",
                    weight as i32,
                    style as u32,
                ),
            );
        }
        best_id
    }

    fn score_candidate(
        font_id: u32,
        weight: FontWeight,
        style: FontStyle,
        target_weight: i32,
        target_style: FontStyle,
    ) -> i32 {
        if font_id == 0 {
            return MAX_FONT_SCORE;
        }
        let mut score = abs_i32(weight as i32 - target_weight);
        if style != target_style {
            score += STYLE_MISMATCH_PENALTY;
        }
        score
    }
}

pub(crate) fn notify_font_loaded(font_id: u32) {
    FONT_LOADED_CALLBACKS.with(|registrations| {
        let callbacks: Vec<Rc<dyn Fn(FontFaceLoadedEventArgs)>> = registrations
            .borrow()
            .iter()
            .filter(|registration| registration.font_id == font_id)
            .map(|registration| registration.callback.clone())
            .collect();
        for callback in callbacks {
            callback(FontFaceLoadedEventArgs {
                font: FontFace::new(font_id),
            });
        }
    });

    FONT_READY_CALLBACKS.with(|registrations| {
        let mut registrations = registrations.borrow_mut();
        let mut ready_callbacks = Vec::new();
        registrations.retain(|registration| {
            if are_font_ids_loaded(&registration.font_ids) {
                ready_callbacks.push(registration.callback.clone());
                false
            } else {
                true
            }
        });
        drop(registrations);
        for callback in ready_callbacks {
            callback();
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{notify_font_loaded, FontFace, FontFamily, FontStack, FontStyle, FontWeight};
    use crate::assets;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn font_face_on_loaded_waits_for_bridge_callback() {
        assets::test_reset();
        let fired = Rc::new(Cell::new(0));
        FontFace::new(1024).on_loaded({
            let fired = fired.clone();
            move |_| fired.set(fired.get() + 1)
        });
        assert_eq!(fired.get(), 0);
        assets::on_font_loaded(1024);
        notify_font_loaded(1024);
        assert_eq!(fired.get(), 1);
    }

    #[test]
    fn built_in_font_faces_are_preloaded_like_fui_as() {
        assets::test_reset();
        for font_id in 1..=6 {
            assert!(FontFace::new(font_id).is_loaded());
        }
    }

    #[test]
    fn font_stack_reports_required_font_ids() {
        let stack = FontStack::from_id(1)
            .fallback_face(FontFace::new(3))
            .fallback_stack(FontStack::from_id(4));
        assert_eq!(stack.required_font_ids(), vec![1, 3, 4]);
    }

    #[test]
    fn font_family_resolves_best_matching_face() {
        let family = FontFamily::from_ids(1, 2, 5, 6, 9, 10, 11, 12);
        assert_eq!(family.resolve(FontWeight::Bold, FontStyle::Italic), 6);
        assert_eq!(family.resolve(FontWeight::Semibold, FontStyle::Normal), 11);
        assert_eq!(family.resolve(FontWeight::Medium, FontStyle::Italic), 10);
    }
}
