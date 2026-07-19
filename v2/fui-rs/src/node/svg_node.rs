use super::core::*;
use super::*;
use crate::assets::{
    acquire_svg_asset, get_svg_asset_error, get_svg_asset_height, get_svg_asset_state,
    get_svg_asset_url, get_svg_asset_width, release_svg_asset, AssetLoadState,
};
use crate::image_sampling::ImageSampling;
use crate::signal::{Callback, SubscriptionGuard};
use std::rc::Weak;

struct SvgState {
    source_url: String,
    owned_svg_asset_id: u32,
    requested_width_value: f32,
    requested_width_unit: Unit,
    has_requested_width: bool,
    requested_height_value: f32,
    requested_height_unit: Unit,
    has_requested_height: bool,
    tracked_svg_asset_id: u32,
    asset_state_subscription: Option<SubscriptionGuard>,
}

impl Default for SvgState {
    fn default() -> Self {
        Self {
            source_url: String::new(),
            owned_svg_asset_id: 0,
            requested_width_value: 0.0,
            requested_width_unit: Unit::Auto,
            has_requested_width: false,
            requested_height_value: 0.0,
            requested_height_unit: Unit::Auto,
            has_requested_height: false,
            tracked_svg_asset_id: 0,
            asset_state_subscription: None,
        }
    }
}

#[derive(Clone)]
pub struct SvgNode {
    base: FlexBox,
    props: Rc<RefCell<SvgProps>>,
    state: Rc<RefCell<SvgState>>,
}

impl SvgNode {
    pub fn new(svg_id: u32) -> Self {
        let base = FlexBox::default();
        base.core.borrow_mut().kind = NodeKind::Svg;
        let node = Self {
            base,
            props: Rc::new(RefCell::new(SvgProps {
                svg_id,
                source_url: None,
                tint_color: 0,
                sampling_kind: ImageSamplingKind::Linear,
                max_aniso: 0,
            })),
            state: Rc::new(RefCell::new(SvgState {
                requested_width_value: 0.0,
                requested_width_unit: Unit::Auto,
                has_requested_width: true,
                requested_height_value: 0.0,
                requested_height_unit: Unit::Auto,
                has_requested_height: true,
                ..SvgState::default()
            })),
        };
        node.attach_asset_state_listener();
        node
    }

    pub fn width(&self, width: f32, unit: Unit) -> &Self {
        {
            let mut state = self.state.borrow_mut();
            state.requested_width_value = width;
            state.requested_width_unit = unit;
            state.has_requested_width = true;
        }
        self.apply_resolved_sizing();
        self
    }

    pub fn height(&self, height: f32, unit: Unit) -> &Self {
        {
            let mut state = self.state.borrow_mut();
            state.requested_height_value = height;
            state.requested_height_unit = unit;
            state.has_requested_height = true;
        }
        self.apply_resolved_sizing();
        self
    }

    pub fn svg(&self, svg_id: u32) -> &Self {
        self.release_owned_source_asset();
        self.state.borrow_mut().source_url.clear();
        self.props.borrow_mut().svg_id = svg_id;
        self.props.borrow_mut().source_url = None;
        self.retained_node_ref().set_image_url_for_routing(None);
        self.attach_asset_state_listener();
        self.apply_resolved_sizing();
        if self.has_built_handle() {
            self.apply_svg_source();
            self.notify_retained_mutation();
        }
        self
    }

    pub fn source(&self, url: impl Into<String>) -> &Self {
        let url = url.into();
        if url.is_empty() {
            return self.clear_source();
        }
        if self.state.borrow().owned_svg_asset_id != 0 && self.state.borrow().source_url == url {
            return self;
        }
        self.release_owned_source_asset();
        {
            let mut state = self.state.borrow_mut();
            state.source_url = url.clone();
        }
        let svg_id = acquire_svg_asset(&url);
        {
            let mut props = self.props.borrow_mut();
            props.svg_id = svg_id;
            props.source_url = Some(url.clone());
        }
        self.state.borrow_mut().owned_svg_asset_id = svg_id;
        self.retained_node_ref()
            .set_image_url_for_routing(Some(url.clone()));
        self.attach_asset_state_listener();
        self.apply_resolved_sizing();
        if self.has_built_handle() {
            self.apply_svg_source();
            self.notify_retained_mutation();
        }
        self
    }

    pub fn clear_source(&self) -> &Self {
        self.release_owned_source_asset();
        {
            let mut state = self.state.borrow_mut();
            state.source_url.clear();
        }
        {
            let mut props = self.props.borrow_mut();
            props.svg_id = 0;
            props.source_url = None;
        }
        self.retained_node_ref().set_image_url_for_routing(None);
        self.attach_asset_state_listener();
        self.apply_resolved_sizing();
        if self.has_built_handle() {
            self.apply_svg_source();
            self.notify_retained_mutation();
        }
        self
    }

    pub fn tint(&self, tint_color: u32) -> &Self {
        self.props.borrow_mut().tint_color = tint_color;
        if self.has_built_handle() {
            self.apply_svg_source();
            self.notify_retained_mutation();
        }
        self
    }

    pub fn alt_text(&self, value: impl Into<String>) -> &Self {
        let value = value.into();
        self.semantic_role(SemanticRole::Image);
        self.semantic_label(value);
        self
    }

    pub fn sampling(&self, sampling: ImageSampling) -> &Self {
        let mut props = self.props.borrow_mut();
        props.sampling_kind = sampling.ffi_kind();
        props.max_aniso = sampling.max_aniso();
        drop(props);
        if self.has_built_handle() {
            self.apply_svg_source();
            self.notify_retained_mutation();
        }
        self
    }

    pub fn asset_state_signal(&self) -> crate::assets::AssetStateSignal {
        get_svg_asset_state(self.props.borrow().svg_id)
    }

    pub fn asset_state(&self) -> AssetLoadState {
        self.asset_state_signal().get()
    }

    pub fn asset_error(&self) -> String {
        get_svg_asset_error(self.props.borrow().svg_id)
    }

    pub fn asset_url(&self) -> String {
        let source_url = self.state.borrow().source_url.clone();
        if source_url.is_empty() {
            get_svg_asset_url(self.props.borrow().svg_id)
        } else {
            source_url
        }
    }

    pub fn asset_width(&self) -> f32 {
        get_svg_asset_width(self.props.borrow().svg_id)
    }

    pub fn asset_height(&self) -> f32 {
        get_svg_asset_height(self.props.borrow().svg_id)
    }

    fn apply_svg_source(&self) {
        let props = self.props.borrow();
        ui::set_svg(
            self.handle().raw(),
            props.svg_id,
            props.tint_color,
            props.sampling_kind,
            props.max_aniso,
        );
    }

    fn apply_resolved_sizing(&self) {
        let (
            has_requested_width,
            requested_width_value,
            requested_width_unit,
            has_requested_height,
            requested_height_value,
            requested_height_unit,
        ) = {
            let state = self.state.borrow();
            (
                state.has_requested_width,
                state.requested_width_value,
                state.requested_width_unit,
                state.has_requested_height,
                state.requested_height_value,
                state.requested_height_unit,
            )
        };
        if !has_requested_width && !has_requested_height {
            return;
        }

        let asset_width = self.asset_width();
        let asset_height = self.asset_height();
        let has_intrinsic_size = asset_width > 0.0 && asset_height > 0.0;
        if has_requested_width {
            let mut resolved_width_value = requested_width_value;
            let mut resolved_width_unit = requested_width_unit;
            if requested_width_unit == Unit::Auto && has_intrinsic_size {
                if has_requested_height && requested_height_unit == Unit::Pixel {
                    resolved_width_value = requested_height_value * (asset_width / asset_height);
                } else {
                    resolved_width_value = asset_width;
                }
                resolved_width_unit = Unit::Pixel;
            }
            self.base.width(resolved_width_value, resolved_width_unit);
        }

        if has_requested_height {
            let mut resolved_height_value = requested_height_value;
            let mut resolved_height_unit = requested_height_unit;
            if requested_height_unit == Unit::Auto && has_intrinsic_size {
                if has_requested_width && requested_width_unit == Unit::Pixel {
                    resolved_height_value = requested_width_value * (asset_height / asset_width);
                } else {
                    resolved_height_value = asset_height;
                }
                resolved_height_unit = Unit::Pixel;
            }
            self.base
                .height(resolved_height_value, resolved_height_unit);
        }
    }

    fn attach_asset_state_listener(&self) {
        let svg_id = self.props.borrow().svg_id;
        {
            let mut state = self.state.borrow_mut();
            if state.tracked_svg_asset_id == svg_id {
                return;
            }
            state.asset_state_subscription = None;
            state.tracked_svg_asset_id = svg_id;
            if svg_id == 0 {
                return;
            }
        }

        let base_weak = self.base.downgrade();
        let props_weak: Weak<RefCell<SvgProps>> = Rc::downgrade(&self.props);
        let state_weak: Weak<RefCell<SvgState>> = Rc::downgrade(&self.state);
        let callback: Callback = Rc::new(move || {
            if let (Some(base), Some(props), Some(state)) = (
                base_weak.upgrade(),
                props_weak.upgrade(),
                state_weak.upgrade(),
            ) {
                let node = SvgNode { base, props, state };
                node.apply_resolved_sizing();
            }
        });

        let guard = get_svg_asset_state(svg_id).subscribe(callback);
        self.state.borrow_mut().asset_state_subscription = Some(guard);
    }

    fn release_owned_source_asset(&self) {
        let owned_svg_asset_id = self.state.borrow().owned_svg_asset_id;
        if owned_svg_asset_id == 0 {
            return;
        }
        release_svg_asset(owned_svg_asset_id);
        self.state.borrow_mut().owned_svg_asset_id = 0;
    }

    #[cfg(test)]
    pub(crate) fn test_svg_id(&self) -> u32 {
        self.props.borrow().svg_id
    }
}

impl Node for SvgNode {
    fn retained_node_ref(&self) -> NodeRef {
        NodeRef::from_node(self.base.core.clone(), self.clone())
    }

    fn build_self(&self) {
        self.apply_resolved_sizing();
        self.base.build_self();
        apply_svg_props(self.handle(), &self.props.borrow());
    }
}

impl HasFlexBoxRoot for SvgNode {
    fn flex_box_root(&self) -> &FlexBox {
        &self.base
    }

    fn set_flex_box_surface_width(&self, value: f32, unit: Unit) {
        self.width(value, unit);
    }

    fn set_flex_box_surface_height(&self, value: f32, unit: Unit) {
        self.height(value, unit);
    }
}

impl ThemeBindable for SvgNode {
    fn theme_binding_node(&self) -> NodeRef {
        self.retained_node_ref()
    }

    fn weak_theme_target(&self) -> Box<dyn Fn() -> Option<Self>> {
        let base = self.base.downgrade();
        let props = Rc::downgrade(&self.props);
        let state = Rc::downgrade(&self.state);
        Box::new(move || {
            Some(Self {
                base: base.upgrade()?,
                props: props.upgrade()?,
                state: state.upgrade()?,
            })
        })
    }
}

impl Drop for SvgNode {
    fn drop(&mut self) {
        if Rc::strong_count(&self.state) == 1 {
            self.state.borrow_mut().asset_state_subscription = None;
            let owned_svg_asset_id = self.state.borrow().owned_svg_asset_id;
            if owned_svg_asset_id != 0 {
                release_svg_asset(owned_svg_asset_id);
                self.state.borrow_mut().owned_svg_asset_id = 0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::{self, AssetLoadState};
    use crate::bridge_callbacks;
    use crate::ffi::{self, Call};
    use crate::Application;

    #[test]
    fn source_url_reuses_registry_asset_and_resizes_after_ready() {
        assets::test_reset();
        ffi::test::reset();

        let svg = SvgNode::new(0);
        svg.source("/img/icon.svg");
        let svg_id = svg.test_svg_id();
        assert!(svg_id >= 0x1000_0000);
        assert_eq!(svg.asset_state(), AssetLoadState::Loading);

        Application::mount(svg.clone());
        ffi::test::take_calls();

        bridge_callbacks::__fui_on_svg_loaded(svg_id, 30.0, 15.0);
        let calls = ffi::test::take_calls();
        assert!(calls.iter().any(|call| matches!(
            call,
            Call::SetWidth { value, unit_enum, .. } if *value == 30.0 && *unit_enum == Unit::Pixel as u32
        )));
        assert!(calls.iter().any(|call| matches!(
            call,
            Call::SetHeight { value, unit_enum, .. } if *value == 15.0 && *unit_enum == Unit::Pixel as u32
        )));
        assert_eq!(svg.asset_width(), 30.0);
        assert_eq!(svg.asset_height(), 15.0);
        assert_eq!(svg.asset_state(), AssetLoadState::Ready);
    }

    #[test]
    fn explicit_svg_id_attaches_listener_and_resizes_after_ready() {
        assets::test_reset();
        ffi::test::reset();

        let svg = SvgNode::new(88);
        assert_eq!(svg.asset_state(), AssetLoadState::Idle);

        Application::mount(svg.clone());
        ffi::test::take_calls();

        bridge_callbacks::__fui_on_svg_loaded(88, 40.0, 20.0);
        let calls = ffi::test::take_calls();
        assert!(calls.iter().any(|call| matches!(
            call,
            Call::SetWidth { value, unit_enum, .. } if *value == 40.0 && *unit_enum == Unit::Pixel as u32
        )));
        assert!(calls.iter().any(|call| matches!(
            call,
            Call::SetHeight { value, unit_enum, .. } if *value == 20.0 && *unit_enum == Unit::Pixel as u32
        )));
        assert_eq!(svg.asset_width(), 40.0);
        assert_eq!(svg.asset_height(), 20.0);
        assert_eq!(svg.asset_state(), AssetLoadState::Ready);
    }
}
