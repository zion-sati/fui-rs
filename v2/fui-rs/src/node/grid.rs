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
        apply_grid_props(
            self.handle(),
            &self.props.borrow(),
            self.base.core.borrow().behavior.clone(),
        );
    }

    fn build_children(&self) {
        self.base.build_children();
        self.apply_grid_placements();
    }
}

impl Grid {
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

    pub fn width(&self, width: f32, unit: Unit) -> &Self {
        self.props.borrow_mut().width = Some((width, unit));
        if self.has_built_handle() {
            ui::set_width(self.handle().raw(), width, unit as u32);
            self.notify_retained_layout_mutation();
        }
        self
    }

    pub fn width_len(&self, length: Length) -> &Self {
        let (width, unit) = length;
        self.width(width, unit)
    }

    pub fn height(&self, height: f32, unit: Unit) -> &Self {
        self.props.borrow_mut().height = Some((height, unit));
        if self.has_built_handle() {
            ui::set_height(self.handle().raw(), height, unit as u32);
            self.notify_retained_layout_mutation();
        }
        self
    }

    pub fn height_len(&self, length: Length) -> &Self {
        let (height, unit) = length;
        self.height(height, unit)
    }

    pub fn bg_color(&self, color: u32) -> &Self {
        self.props.borrow_mut().bg_color = Some(color);
        if self.has_built_handle() {
            ui::set_bg_color(self.handle().raw(), color);
            self.notify_retained_mutation();
        }
        self
    }

    pub fn padding(&self, left: f32, top: f32, right: f32, bottom: f32) -> &Self {
        self.props.borrow_mut().padding = Some((left, top, right, bottom));
        if self.has_built_handle() {
            ui::set_padding(self.handle().raw(), left, top, right, bottom);
            self.notify_retained_layout_mutation();
        }
        self
    }

    pub fn corner_radius(&self, radius: f32) -> &Self {
        self.base.corner_radius(radius);
        self
    }

    pub fn corners(&self, tl: f32, tr: f32, br: f32, bl: f32) -> &Self {
        self.base.corners(tl, tr, br, bl);
        self
    }

    pub fn cursor(&self, style: CursorStyle) -> &Self {
        self.base.cursor(style);
        self
    }

    pub fn interactive(&self, interactive: bool) -> &Self {
        self.base.interactive(interactive);
        self
    }

    pub fn semantic_role(&self, role: SemanticRole) -> &Self {
        self.base.semantic_role(role);
        self
    }

    pub fn semantic_label(&self, label: impl Into<String>) -> &Self {
        self.base.semantic_label(label);
        self
    }

    pub(crate) fn semantic_disabled(&self, disabled: bool) -> &Self {
        self.base.semantic_disabled(disabled);
        self
    }

    pub fn on_pointer_down(&self, handler: impl Fn(&mut PointerEventArgs) + 'static) -> &Self {
        self.base.on_pointer_down(handler);
        self
    }

    pub fn on_pointer_up(&self, handler: impl Fn(&mut PointerEventArgs) + 'static) -> &Self {
        self.base.on_pointer_up(handler);
        self
    }

    pub fn on_pointer_enter(&self, handler: impl Fn(&mut PointerEventArgs) + 'static) -> &Self {
        self.base.on_pointer_enter(handler);
        self
    }

    pub fn on_pointer_leave(&self, handler: impl Fn(&mut PointerEventArgs) + 'static) -> &Self {
        self.base.on_pointer_leave(handler);
        self
    }

    pub fn on_pointer_cancel(&self, handler: impl Fn(&mut PointerEventArgs) + 'static) -> &Self {
        self.base.on_pointer_cancel(handler);
        self
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

    pub fn child<T: Node>(&self, child: &T) -> &Self {
        self.append_child(child);
        self
    }

    pub fn children<I, C>(&self, children: I) -> &Self
    where
        I: IntoIterator<Item = C>,
        C: Into<Child>,
    {
        for child in children {
            self.retained_node_ref()
                .append_child_ref(&child.into().node_ref);
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
