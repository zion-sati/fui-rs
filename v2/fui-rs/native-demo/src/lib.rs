use fui::prelude::*;
use fui_rs_demo_page_registry::NativeDemoShell;
use fui_rs_demo_universal::{DemoEnvironment, DemoLinks, DemoPageId};

struct NativeApplication {
    root: FlexBox,
    _shell: NativeDemoShell,
    _worker_host_services: fui::worker_host_services::NativeWorkerHostServiceRegistration,
}

fn build_application() -> NativeApplication {
    use_system_theme();
    let worker_host_services = fui_rs_demo_worker::register_native_worker_host_services()
        .expect("native demo Worker host services must register");
    Application::caption(DemoPageId::Dashboard.metadata().title);
    let shell = NativeDemoShell::new(DemoEnvironment::native(
        fui::platform::platform_family(),
        DemoLinks::new(
            "https://github.com/zion-sati/fui-rs/tree/main/demo",
            "https://docs.rs/fui-rs/latest/fui",
        ),
    ));
    NativeApplication {
        root: shell.root(),
        _shell: shell,
        _worker_host_services: worker_host_services,
    }
}

fui_managed_app!(
    NativeApplication,
    build_application,
    |application: &NativeApplication| application.root.clone()
);
