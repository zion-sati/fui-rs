use crate::DemoEnvironment;
use fui::prelude::*;
use std::any::Any;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DemoPageId {
    Dashboard,
    BasicControls,
    TextAndFonts,
    Advanced,
    ImmediateDrawing,
    Platform,
}

impl DemoPageId {
    pub const ALL: [Self; 6] = [
        Self::Dashboard,
        Self::BasicControls,
        Self::TextAndFonts,
        Self::Advanced,
        Self::ImmediateDrawing,
        Self::Platform,
    ];

    pub const fn metadata(self) -> DemoPageMetadata {
        match self {
            Self::Dashboard => DemoPageMetadata::new(
                self,
                "FUI-RS Demo",
                "Dashboard",
                "EffinDOM and FUI-RS ecosystem overview",
                "pages/dashboard.rs",
                "getting-started",
            ),
            Self::BasicControls => DemoPageMetadata::new(
                self,
                "FUI-RS Demo - Basic controls",
                "Basic controls",
                "Common retained controls and accessibility",
                "pages/basic_controls.rs",
                "controls-and-nodes",
            ),
            Self::TextAndFonts => DemoPageMetadata::new(
                self,
                "FUI-RS Demo - Text and fonts",
                "Text and fonts",
                "Typography, selection, editing, IME, and font fallback",
                "pages/text_and_fonts.rs",
                "text-and-fonts",
            ),
            Self::Advanced => DemoPageMetadata::new(
                self,
                "FUI-RS Demo - Advanced",
                "Advanced",
                "Advanced controls, workers, virtualization, and drag and drop",
                "pages/advanced.rs",
                "advanced-controls",
            ),
            Self::ImmediateDrawing => DemoPageMetadata::new(
                self,
                "FUI-RS Demo - Immediate drawing",
                "Immediate drawing",
                "Immediate-mode, bitmap, text, image, SVG, and animated drawing",
                "pages/immediate_drawing.rs",
                "custom-drawing",
            ),
            Self::Platform => DemoPageMetadata::new(
                self,
                "FUI-RS Demo - Platform",
                "Platform",
                "Portable capabilities and host-specific diagnostics",
                "pages/platform.rs",
                "platform-and-hosts",
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DemoPageMetadata {
    pub id: DemoPageId,
    pub title: &'static str,
    pub tab_label: &'static str,
    pub semantic_summary: &'static str,
    pub source_path: &'static str,
    pub documentation_path: &'static str,
}

impl DemoPageMetadata {
    pub const fn new(
        id: DemoPageId,
        title: &'static str,
        tab_label: &'static str,
        semantic_summary: &'static str,
        source_path: &'static str,
        documentation_path: &'static str,
    ) -> Self {
        Self {
            id,
            title,
            tab_label,
            semantic_summary,
            source_path,
            documentation_path,
        }
    }
}

pub struct UniversalDemoPage {
    metadata: DemoPageMetadata,
    view: RetainedView,
    state: Option<Rc<dyn Any>>,
}

impl UniversalDemoPage {
    pub fn new(metadata: DemoPageMetadata, view: RetainedView) -> Self {
        Self {
            metadata,
            view,
            state: None,
        }
    }

    pub fn with_state<T: 'static>(mut self, state: T) -> Self {
        let state: Rc<dyn Any> = Rc::new(state);
        self.view = self.view.clone().keep_alive(state.clone());
        self.state = Some(state);
        self
    }

    pub fn state<T: 'static>(&self) -> Option<&T> {
        self.state.as_deref()?.downcast_ref::<T>()
    }

    pub fn metadata(&self) -> DemoPageMetadata {
        self.metadata
    }

    pub fn view(&self) -> RetainedView {
        self.view.clone()
    }

    pub fn source_url(&self, environment: &DemoEnvironment) -> String {
        environment.links().source(self.metadata.source_path)
    }

    pub fn documentation_url(&self, environment: &DemoEnvironment) -> String {
        environment
            .links()
            .documentation(self.metadata.documentation_path)
    }
}

pub type DemoPageFactory = fn(&DemoEnvironment) -> UniversalDemoPage;
