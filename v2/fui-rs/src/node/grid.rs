use super::core::*;
use super::*;

#[derive(Clone)]
pub struct Grid {
    base: FlexBox,
    props: Rc<RefCell<GridProps>>,
    placements: Rc<RefCell<Vec<(NodeRef, GridPlacement)>>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GridTrack {
    pub value: f32,
    pub unit: GridUnit,
}

impl GridTrack {
    pub const fn px(value: f32) -> Self {
        Self {
            value,
            unit: GridUnit::Pixel,
        }
    }

    pub const fn star(value: f32) -> Self {
        Self {
            value,
            unit: GridUnit::Star,
        }
    }

    pub const fn auto() -> Self {
        Self {
            value: 0.0,
            unit: GridUnit::Auto,
        }
    }
}

impl Default for Grid {
    fn default() -> Self {
        let base = FlexBox::default();
        base.core.borrow_mut().kind = NodeKind::Grid;
        Self {
            base,
            props: Rc::new(RefCell::new(GridProps::default())),
            placements: Rc::new(RefCell::new(Vec::new())),
        }
    }
}

impl Node for Grid {
    fn retained_node_ref(&self) -> NodeRef {
        NodeRef::from_node(self.base.core.clone(), self.clone())
    }

    fn build_self(&self) {
        self.base.build_self();
        apply_grid_props(self.handle(), &self.props.borrow());
    }

    fn build_children(&self) {
        self.base.build_children();
        self.apply_grid_placements();
    }
}

impl Grid {
    pub fn shared_size_scope<T: Node>(target: &T, enabled: bool) {
        let node = target.retained_node_ref();
        node.set_shared_size_scope(enabled);
        if target.has_built_handle() {
            target.notify_retained_layout_mutation();
        }
    }

    pub(crate) fn downgrade(&self) -> WeakFlexBox {
        self.base.downgrade()
    }

    fn apply_grid_placements(&self) {
        for (child, placement) in self.placements.borrow().iter() {
            let handle = child.handle();
            if handle != NodeHandle::INVALID {
                ui::set_grid_placement(
                    handle.raw(),
                    placement.row,
                    placement.col,
                    placement.row_span,
                    placement.col_span,
                );
            }
        }
    }

    pub fn columns<I>(&self, tracks: I) -> &Self
    where
        I: IntoIterator<Item = GridTrack>,
    {
        let tracks: Vec<GridTrack> = tracks.into_iter().collect();
        let mut props = self.props.borrow_mut();
        props.columns = tracks.iter().map(|track| track.value).collect();
        props.column_types = tracks.iter().map(|track| track.unit).collect();
        drop(props);
        if self.has_built_handle() {
            self.build_self();
            self.notify_retained_layout_mutation();
        }
        self
    }

    pub fn rows<I>(&self, tracks: I) -> &Self
    where
        I: IntoIterator<Item = GridTrack>,
    {
        let tracks: Vec<GridTrack> = tracks.into_iter().collect();
        let mut props = self.props.borrow_mut();
        props.rows = tracks.iter().map(|track| track.value).collect();
        props.row_types = tracks.iter().map(|track| track.unit).collect();
        drop(props);
        if self.has_built_handle() {
            self.build_self();
            self.notify_retained_layout_mutation();
        }
        self
    }

    pub fn column_shared_size_group(&self, index: u32, group: impl Into<String>) -> &Self {
        let group = group.into();
        let mut props = self.props.borrow_mut();
        if let Some((_, existing)) = props
            .column_shared_size_groups
            .iter_mut()
            .find(|(existing_index, _)| *existing_index == index)
        {
            *existing = group;
        } else {
            props.column_shared_size_groups.push((index, group));
        }
        drop(props);
        if self.has_built_handle() {
            self.build_self();
            self.notify_retained_layout_mutation();
        }
        self
    }

    pub fn clear_column_shared_size_group(&self, index: u32) -> &Self {
        self.column_shared_size_group(index, "")
    }

    pub fn row_shared_size_group(&self, index: u32, group: impl Into<String>) -> &Self {
        let group = group.into();
        let mut props = self.props.borrow_mut();
        if let Some((_, existing)) = props
            .row_shared_size_groups
            .iter_mut()
            .find(|(existing_index, _)| *existing_index == index)
        {
            *existing = group;
        } else {
            props.row_shared_size_groups.push((index, group));
        }
        drop(props);
        if self.has_built_handle() {
            self.build_self();
            self.notify_retained_layout_mutation();
        }
        self
    }

    pub fn clear_row_shared_size_group(&self, index: u32) -> &Self {
        self.row_shared_size_group(index, "")
    }

    pub fn place_child<T: Node>(
        &self,
        child: &T,
        row: u32,
        col: u32,
        row_span: u32,
        col_span: u32,
    ) -> &Self {
        self.append_child(child);
        self.placements.borrow_mut().push((
            child.node_ref(),
            GridPlacement {
                row,
                col,
                row_span,
                col_span,
            },
        ));
        if self.has_built_handle() {
            self.apply_grid_placements();
        }
        self
    }
}

impl HasFlexBoxRoot for Grid {
    fn flex_box_root(&self) -> &FlexBox {
        &self.base
    }
}

impl ThemeBindable for Grid {
    fn theme_binding_node(&self) -> NodeRef {
        self.retained_node_ref()
    }

    fn weak_theme_target(&self) -> Box<dyn Fn() -> Option<Self>> {
        let base = self.base.downgrade();
        let props = Rc::downgrade(&self.props);
        let placements = Rc::downgrade(&self.placements);
        Box::new(move || {
            Some(Self {
                base: base.upgrade()?,
                props: props.upgrade()?,
                placements: placements.upgrade()?,
            })
        })
    }
}
