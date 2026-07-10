use super::core::*;
use super::*;
use crate::assets::{
    acquire_texture_asset, get_texture_asset_error, get_texture_asset_height,
    get_texture_asset_state, get_texture_asset_url, get_texture_asset_width, release_texture_asset,
    AssetLoadState,
};
use crate::image_sampling::ImageSampling;
use crate::signal::{Callback, SubscriptionGuard};
use std::rc::Weak;

struct ImageState {
    source_url: String,
    owned_texture_asset_id: u32,
    requested_width_value: f32,
    requested_width_unit: Unit,
    has_requested_width: bool,
    requested_height_value: f32,
    requested_height_unit: Unit,
    has_requested_height: bool,
    tracked_texture_asset_id: u32,
    asset_state_subscription: Option<SubscriptionGuard>,
}

impl Default for ImageState {
    fn default() -> Self {
        Self {
            source_url: String::new(),
            owned_texture_asset_id: 0,
            requested_width_value: 0.0,
            requested_width_unit: Unit::Auto,
            has_requested_width: false,
            requested_height_value: 0.0,
            requested_height_unit: Unit::Auto,
            has_requested_height: false,
            tracked_texture_asset_id: 0,
            asset_state_subscription: None,
        }
    }
}

#[derive(Clone)]
pub struct ImageNode {
    core: Rc<RefCell<NodeCore>>,
    props: Rc<RefCell<ImageProps>>,
    state: Rc<RefCell<ImageState>>,
}

impl ImageNode {
    pub fn new(texture_id: u32) -> Self {
        let node = Self {
            core: Rc::new(RefCell::new(NodeCore::new(NodeKind::Image))),
            props: Rc::new(RefCell::new(ImageProps {
                width: None,
                height: None,
                texture_id,
                source_url: None,
                object_fit: ObjectFit::Fill,
                sampling_kind: ImageSamplingKind::Linear,
                max_aniso: 0,
                image_nine: None,
            })),
            state: Rc::new(RefCell::new(ImageState {
                requested_width_value: 0.0,
                requested_width_unit: Unit::Auto,
                has_requested_width: true,
                requested_height_value: 0.0,
                requested_height_unit: Unit::Auto,
                has_requested_height: true,
                ..ImageState::default()
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

    pub fn texture(&self, texture_id: u32) -> &Self {
        self.release_owned_source_asset();
        self.state.borrow_mut().source_url.clear();
        self.props.borrow_mut().texture_id = texture_id;
        self.props.borrow_mut().source_url = None;
        self.retained_node_ref().set_image_url_for_routing(None);
        self.attach_asset_state_listener();
        self.apply_resolved_sizing();
        if self.has_built_handle() {
            self.apply_image_source();
            self.notify_retained_mutation();
        }
        self
    }

    pub fn source(&self, url: impl Into<String>) -> &Self {
        let url = url.into();
        if url.is_empty() {
            return self.clear_source();
        }
        if self.state.borrow().owned_texture_asset_id != 0 && self.state.borrow().source_url == url
        {
            return self;
        }
        self.release_owned_source_asset();
        {
            let mut state = self.state.borrow_mut();
            state.source_url = url.clone();
        }
        let texture_id = acquire_texture_asset(&url);
        {
            let mut props = self.props.borrow_mut();
            props.texture_id = texture_id;
            props.source_url = Some(url.clone());
        }
        self.state.borrow_mut().owned_texture_asset_id = texture_id;
        self.retained_node_ref()
            .set_image_url_for_routing(Some(url.clone()));
        self.attach_asset_state_listener();
        self.apply_resolved_sizing();
        if self.has_built_handle() {
            self.apply_image_source();
            self.notify_retained_mutation();
        }
        self
    }

    pub fn source_url(&self, url: impl Into<String>) -> &Self {
        self.source(url)
    }

    pub fn clear_source(&self) -> &Self {
        self.release_owned_source_asset();
        {
            let mut state = self.state.borrow_mut();
            state.source_url.clear();
        }
        {
            let mut props = self.props.borrow_mut();
            props.texture_id = 0;
            props.source_url = None;
            props.image_nine = None;
        }
        self.retained_node_ref().set_image_url_for_routing(None);
        self.attach_asset_state_listener();
        self.apply_resolved_sizing();
        if self.has_built_handle() {
            self.apply_image_source();
            self.notify_retained_mutation();
        }
        self
    }

    pub fn clear_source_url(&self) -> &Self {
        self.clear_source()
    }

    pub fn alt_text(&self, value: impl Into<String>) -> &Self {
        let value = value.into();
        self.semantic_role(SemanticRole::Image);
        self.semantic_label(value);
        self
    }

    pub fn object_fit(&self, object_fit: ObjectFit) -> &Self {
        self.props.borrow_mut().object_fit = object_fit;
        if self.has_built_handle() {
            self.apply_image_source();
            self.notify_retained_mutation();
        }
        self
    }

    pub fn sampling(&self, sampling: ImageSampling) -> &Self {
        let mut props = self.props.borrow_mut();
        props.sampling_kind = sampling.ffi_kind();
        props.max_aniso = sampling.max_aniso();
        drop(props);
        if self.has_built_handle() {
            self.apply_image_source();
            self.notify_retained_mutation();
        }
        self
    }

    pub fn image_nine(
        &self,
        inset_left: f32,
        inset_top: f32,
        inset_right: f32,
        inset_bottom: f32,
    ) -> &Self {
        self.props.borrow_mut().image_nine =
            Some((inset_left, inset_top, inset_right, inset_bottom));
        if self.has_built_handle() {
            self.apply_image_source();
            self.notify_retained_mutation();
        }
        self
    }

    pub fn clear_image_nine(&self) -> &Self {
        self.props.borrow_mut().image_nine = None;
        if self.has_built_handle() {
            self.apply_image_source();
            self.notify_retained_mutation();
        }
        self
    }

    pub fn interactive(&self, interactive: bool) -> &Self {
        self.core.borrow_mut().behavior.interactive = interactive;
        self
    }

    pub fn on_click(&self, handler: impl Fn(&mut PointerEventArgs) + 'static) -> &Self {
        self.core.borrow_mut().handlers.pointer_click = Some(Rc::new(handler));
        self.retained_node_ref().require_interactive();
        self
    }

    pub fn asset_state_signal(&self) -> crate::assets::AssetStateSignal {
        get_texture_asset_state(self.props.borrow().texture_id)
    }

    pub fn asset_state(&self) -> AssetLoadState {
        self.asset_state_signal().get()
    }

    pub fn asset_error(&self) -> String {
        get_texture_asset_error(self.props.borrow().texture_id)
    }

    pub fn asset_url(&self) -> String {
        let source_url = self.state.borrow().source_url.clone();
        if source_url.is_empty() {
            get_texture_asset_url(self.props.borrow().texture_id)
        } else {
            source_url
        }
    }

    pub fn asset_width(&self) -> f32 {
        get_texture_asset_width(self.props.borrow().texture_id)
    }

    pub fn asset_height(&self) -> f32 {
        get_texture_asset_height(self.props.borrow().texture_id)
    }

    fn apply_image_source(&self) {
        let props = self.props.borrow();
        if let Some((left, top, right, bottom)) = props.image_nine {
            ui::set_image_nine(
                self.handle().raw(),
                props.texture_id,
                left,
                top,
                right,
                bottom,
                props.sampling_kind,
                props.max_aniso,
            );
        } else {
            ui::set_image(
                self.handle().raw(),
                props.texture_id,
                props.object_fit as u32,
                props.sampling_kind,
                props.max_aniso,
            );
        }
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
        let mut props = self.props.borrow_mut();

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
            props.width = Some((resolved_width_value, resolved_width_unit));
            if self.has_built_handle() {
                ui::set_width(
                    self.handle().raw(),
                    resolved_width_value,
                    resolved_width_unit as u32,
                );
            }
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
            props.height = Some((resolved_height_value, resolved_height_unit));
            if self.has_built_handle() {
                ui::set_height(
                    self.handle().raw(),
                    resolved_height_value,
                    resolved_height_unit as u32,
                );
            }
        }

        drop(props);
        if self.has_built_handle() {
            self.notify_retained_layout_mutation();
        }
    }

    fn attach_asset_state_listener(&self) {
        let texture_id = self.props.borrow().texture_id;
        {
            let mut state = self.state.borrow_mut();
            if state.tracked_texture_asset_id == texture_id {
                return;
            }
            state.asset_state_subscription = None;
            state.tracked_texture_asset_id = texture_id;
            if texture_id == 0 {
                return;
            }
        }

        let core_weak: Weak<RefCell<NodeCore>> = Rc::downgrade(&self.core);
        let props_weak: Weak<RefCell<ImageProps>> = Rc::downgrade(&self.props);
        let state_weak: Weak<RefCell<ImageState>> = Rc::downgrade(&self.state);
        let callback: Callback = Rc::new(move || {
            if let (Some(core), Some(props), Some(state)) = (
                core_weak.upgrade(),
                props_weak.upgrade(),
                state_weak.upgrade(),
            ) {
                let node = ImageNode { core, props, state };
                node.apply_resolved_sizing();
            }
        });

        let guard = get_texture_asset_state(texture_id).subscribe(callback);
        self.state.borrow_mut().asset_state_subscription = Some(guard);
    }

    fn release_owned_source_asset(&self) {
        let owned_texture_asset_id = self.state.borrow().owned_texture_asset_id;
        if owned_texture_asset_id == 0 {
            return;
        }
        release_texture_asset(owned_texture_asset_id);
        self.state.borrow_mut().owned_texture_asset_id = 0;
    }

    #[cfg(test)]
    pub(crate) fn test_texture_id(&self) -> u32 {
        self.props.borrow().texture_id
    }
}

impl Node for ImageNode {
    fn retained_node_ref(&self) -> NodeRef {
        NodeRef::from_node(self.core.clone(), self.clone())
    }

    fn build_self(&self) {
        self.apply_resolved_sizing();
        apply_image_props(
            self.handle(),
            &self.props.borrow(),
            self.core.borrow().behavior.clone(),
        );
    }
}

impl Drop for ImageNode {
    fn drop(&mut self) {
        if Rc::strong_count(&self.state) == 1 {
            self.state.borrow_mut().asset_state_subscription = None;
            let owned_texture_asset_id = self.state.borrow().owned_texture_asset_id;
            if owned_texture_asset_id != 0 {
                release_texture_asset(owned_texture_asset_id);
                self.state.borrow_mut().owned_texture_asset_id = 0;
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

        let image = ImageNode::new(0);
        image.source("/img/sample.png");
        let texture_id = image.test_texture_id();
        assert!(texture_id >= 0x2000_0000);
        assert_eq!(image.asset_state(), AssetLoadState::Loading);

        Application::mount(image.clone());
        ffi::test::take_calls();

        bridge_callbacks::__fui_on_texture_loaded(texture_id, 80.0, 40.0);
        let calls = ffi::test::take_calls();
        assert!(calls.iter().any(|call| matches!(
            call,
            Call::SetWidth { value, unit_enum, .. } if *value == 80.0 && *unit_enum == Unit::Pixel as u32
        )));
        assert!(calls.iter().any(|call| matches!(
            call,
            Call::SetHeight { value, unit_enum, .. } if *value == 40.0 && *unit_enum == Unit::Pixel as u32
        )));
        assert_eq!(image.asset_width(), 80.0);
        assert_eq!(image.asset_height(), 40.0);
        assert_eq!(image.asset_state(), AssetLoadState::Ready);
    }

    #[test]
    fn explicit_texture_id_attaches_listener_and_resizes_after_ready() {
        assets::test_reset();
        ffi::test::reset();

        let image = ImageNode::new(77);
        assert_eq!(image.asset_state(), AssetLoadState::Idle);

        Application::mount(image.clone());
        ffi::test::take_calls();

        bridge_callbacks::__fui_on_texture_loaded(77, 96.0, 48.0);
        let calls = ffi::test::take_calls();
        assert!(calls.iter().any(|call| matches!(
            call,
            Call::SetWidth { value, unit_enum, .. } if *value == 96.0 && *unit_enum == Unit::Pixel as u32
        )));
        assert!(calls.iter().any(|call| matches!(
            call,
            Call::SetHeight { value, unit_enum, .. } if *value == 48.0 && *unit_enum == Unit::Pixel as u32
        )));
        assert_eq!(image.asset_width(), 96.0);
        assert_eq!(image.asset_height(), 48.0);
        assert_eq!(image.asset_state(), AssetLoadState::Ready);
    }
}
