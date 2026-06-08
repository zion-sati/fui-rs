use crate::bindings::ui;
use crate::component::{flush_render_scheduler, install_active_app, Component, ComponentInstance};
use std::rc::Rc;

pub struct Application;

impl Application {
    pub fn run<C: Component + 'static>(factory: impl FnOnce() -> C) {
        ui::reset();
        ui::resize_window(ui::get_viewport_width(), ui::get_viewport_height());

        let instance = ComponentInstance::new(factory);
        install_active_app(instance.clone() as Rc<dyn crate::component::MountedApp>);
        instance.mount();
        flush_render_scheduler();
    }

    pub fn flush_renders() {
        flush_render_scheduler();
    }
}
