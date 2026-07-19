use super::*;

#[derive(Clone)]
pub struct AntiSelectionArea {
    root: FlexBox,
}

impl AntiSelectionArea {
    pub fn new() -> Self {
        let root = flex_box();
        root.selection_area_barrier(true);
        Self { root }
    }
}

impl Default for AntiSelectionArea {
    fn default() -> Self {
        Self::new()
    }
}

impl Node for AntiSelectionArea {
    fn retained_node_ref(&self) -> NodeRef {
        self.root.retained_node_ref()
    }

    fn build_self(&self) {
        self.root.build_self();
    }
}

impl HasFlexBoxRoot for AntiSelectionArea {
    fn flex_box_root(&self) -> &FlexBox {
        &self.root
    }
}

impl ThemeBindable for AntiSelectionArea {
    fn theme_binding_node(&self) -> NodeRef {
        self.root.retained_node_ref()
    }

    fn weak_theme_target(&self) -> Box<dyn Fn() -> Option<Self>> {
        let weak_root = self.root.downgrade();
        Box::new(move || {
            Some(AntiSelectionArea {
                root: weak_root.upgrade()?,
            })
        })
    }
}
