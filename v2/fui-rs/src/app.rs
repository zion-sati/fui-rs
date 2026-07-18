use crate::bindings::ui;
use crate::context_menu_manager;
use crate::ffi::HandleValue;
use crate::frame_scheduler;
use crate::mobile_text_selection_toolbar;
use crate::node::{flex_box, FlexBox, Node, NodeRef, ThemeBindable};
use crate::panic_hook;
use crate::selection_handle_adorner;
use crate::theme;
use crate::timers;
use crate::tool_tip_manager;
use crate::Unit;
use crate::{focus_adorner, focus_visibility};
use std::cell::RefCell;
use std::rc::Rc;

type BuildPageFn<TPage> = Rc<dyn Fn() -> TPage>;
type RootFn<TPage> = Rc<dyn Fn(&TPage) -> NodeRef>;
type PageCallback<TPage> = Rc<dyn Fn(&TPage)>;
type PostCommitCallback = Box<dyn FnOnce()>;

thread_local! {
    static MOUNTED_ROOT: RefCell<Option<NodeRef>> = const { RefCell::new(None) };
    static MOUNTED_SHELL: RefCell<Option<FlexBox>> = const { RefCell::new(None) };
    static POST_COMMIT_CALLBACKS: RefCell<Vec<PostCommitCallback>> = const { RefCell::new(Vec::new()) };
}

fn create_empty_page() -> FlexBox {
    let root = flex_box();
    root.width(100.0, Unit::Percent)
        .height(100.0, Unit::Percent);
    root
}

fn clear_mount_state() {
    clear_mount_state_with_loaded_reset(true);
}

fn clear_mount_state_with_loaded_reset(clear_loaded_callbacks: bool) {
    clear_post_commit_callbacks();
    if clear_loaded_callbacks {
        frame_scheduler::reset_commit_state();
    } else {
        frame_scheduler::reset_commit_state_preserving_loaded_callbacks();
    }
    crate::event::reset();
    timers::cancel_all_timers();
    crate::fetch::dispose_all_fetch_requests();
    crate::file::reset_file_runtime();
    selection_handle_adorner::reset();
    mobile_text_selection_toolbar::reset();
    focus_adorner::clear();
    tool_tip_manager::ToolTipManager::clear();
    focus_visibility::reset_keyboard_focus_visibility();
    let disposed_shell = MOUNTED_SHELL.with(|slot| {
        if let Some(shell) = slot.borrow_mut().take() {
            shell.dispose();
            true
        } else {
            false
        }
    });
    MOUNTED_ROOT.with(|slot| {
        if let Some(root) = slot.borrow_mut().take() {
            if !disposed_shell {
                root.dispose();
            }
        }
    });
}

pub(crate) fn after_next_commit(callback: impl FnOnce() + 'static) {
    POST_COMMIT_CALLBACKS.with(|slot| slot.borrow_mut().push(Box::new(callback)));
}

fn clear_post_commit_callbacks() {
    POST_COMMIT_CALLBACKS.with(|slot| slot.borrow_mut().clear());
}

fn run_post_commit_callbacks() {
    let callbacks = POST_COMMIT_CALLBACKS.with(|slot| std::mem::take(&mut *slot.borrow_mut()));
    for callback in callbacks {
        callback();
    }
}

fn application_shell<T: Node>(root: &T) -> FlexBox {
    let shell = flex_box();
    shell
        .width(100.0, Unit::Percent)
        .height(100.0, Unit::Percent)
        .child(root)
        .child(&selection_handle_adorner::create_default_host())
        .child(&mobile_text_selection_toolbar::create_default_host())
        .child(&focus_adorner::create_default_host())
        .child(&tool_tip_manager::ToolTipManager::create_default_host())
        .child(&context_menu_manager::create_default_menu());
    shell.bind_theme(|shell, theme| {
        shell.bg_color(theme.colors.background);
    });
    shell
}

pub struct ApplicationRegistration {
    build_page_fn: Rc<dyn Fn() -> FlexBox>,
}

impl Default for ApplicationRegistration {
    fn default() -> Self {
        Self::new()
    }
}

impl ApplicationRegistration {
    pub fn new() -> Self {
        Self {
            build_page_fn: Rc::new(create_empty_page),
        }
    }

    pub fn page<TNode: Node + 'static>(mut self, build_page: impl Fn() -> TNode + 'static) -> Self {
        self.build_page_fn = Rc::new(move || {
            let node = build_page();
            let shell = flex_box();
            shell.child(&node);
            shell
        });
        self
    }

    pub fn register(self) -> ManagedApplication<FlexBox> {
        ManagedApplication::new(move || (self.build_page_fn)(), |page| page.clone())
    }
}

pub struct ManagedApplication<TPage: 'static> {
    build_page: BuildPageFn<TPage>,
    get_root: RootFn<TPage>,
    mount_page: Option<PageCallback<TPage>>,
    dispose_page: Option<PageCallback<TPage>>,
    active_page: RefCell<Option<Rc<TPage>>>,
}

impl<TPage: 'static> ManagedApplication<TPage> {
    pub fn new<TNode: Node + 'static>(
        build_page: impl Fn() -> TPage + 'static,
        get_root: impl Fn(&TPage) -> TNode + 'static,
    ) -> Self {
        Self {
            build_page: Rc::new(build_page),
            get_root: Rc::new(move |page| {
                let root = get_root(page);
                root.node_ref()
            }),
            mount_page: None,
            dispose_page: None,
            active_page: RefCell::new(None),
        }
    }

    pub fn mount_page(mut self, callback: impl Fn(&TPage) + 'static) -> Self {
        self.mount_page = Some(Rc::new(callback));
        self
    }

    pub fn dispose_page(mut self, callback: impl Fn(&TPage) + 'static) -> Self {
        self.dispose_page = Some(Rc::new(callback));
        self
    }

    pub fn run(&self) {
        panic_hook::install();
        self.dispose();
        ui::reset();
        ui::resize_window(ui::get_viewport_width(), ui::get_viewport_height());
        theme::use_system_theme();

        let page = Rc::new((self.build_page)());
        let root = (self.get_root)(&page);
        let shell = application_shell(&NodeRefMount(root.clone()));
        shell.build();
        ui::set_root(shell.handle().raw());

        MOUNTED_ROOT.with(|slot| slot.borrow_mut().replace(root));
        MOUNTED_SHELL.with(|slot| slot.borrow_mut().replace(shell));
        self.active_page.borrow_mut().replace(page.clone());
        frame_scheduler::fire_loaded_callbacks();
        frame_scheduler::mark_needs_commit();
        frame_scheduler::flush_commit();
        if focus_adorner::refresh_after_commit() {
            frame_scheduler::mark_needs_commit();
            frame_scheduler::flush_commit();
        }
        run_post_commit_callbacks();

        if let Some(callback) = self.mount_page.as_ref() {
            callback(&page);
        }
    }

    pub fn dispose(&self) {
        if let Some(page) = self.active_page.borrow_mut().take() {
            if let Some(callback) = self.dispose_page.as_ref() {
                callback(&page);
            }
        }
        clear_mount_state();
        ui::set_root(HandleValue::Invalid as u64);
    }

    pub fn get_active_page(&self) -> Option<Rc<TPage>> {
        self.active_page.borrow().as_ref().cloned()
    }

    pub fn use_system_theme(&self) -> theme::Theme {
        theme::use_system_theme()
    }

    pub fn use_custom_theme(&self, value: theme::Theme) -> theme::Theme {
        theme::use_custom_theme(value)
    }

    pub fn set_accent_color(&self, color: u32) -> theme::Theme {
        theme::set_accent_color(color)
    }

    pub fn is_dark_mode(&self) -> bool {
        theme::is_dark_mode()
    }

    pub fn is_using_system_theme(&self) -> bool {
        theme::is_using_system_theme()
    }

    pub fn get_theme(&self) -> theme::Theme {
        theme::current_theme()
    }

    pub fn flush_renders(&self) {
        Application::flush_renders();
    }

    pub fn capture_persisted_ui_state(&self) {
        Application::capture_persisted_ui_state();
    }

    pub fn restore_persisted_ui_state(&self) {
        Application::restore_persisted_ui_state();
    }
}

pub struct Application;

impl Application {
    pub fn mount<TNode: Node>(root: TNode) {
        panic_hook::install();
        clear_mount_state_with_loaded_reset(false);
        ui::reset();
        ui::resize_window(ui::get_viewport_width(), ui::get_viewport_height());
        let mounted_root = root.node_ref();
        let shell = application_shell(&root);
        shell.build();
        ui::set_root(shell.handle().raw());
        MOUNTED_ROOT.with(|slot| {
            *slot.borrow_mut() = Some(mounted_root);
        });
        MOUNTED_SHELL.with(|slot| slot.borrow_mut().replace(shell));
        frame_scheduler::fire_loaded_callbacks();
        frame_scheduler::mark_needs_commit();
        frame_scheduler::flush_commit();
        if focus_adorner::refresh_after_commit() {
            frame_scheduler::mark_needs_commit();
            frame_scheduler::flush_commit();
        }
        run_post_commit_callbacks();
    }

    pub fn unmount() {
        clear_mount_state();
        ui::set_root(HandleValue::Invalid as u64);
    }

    pub fn flush_renders() {
        if !frame_scheduler::flush_commit() {
            return;
        }
        if focus_adorner::refresh_after_commit() {
            frame_scheduler::mark_needs_commit();
            frame_scheduler::flush_commit();
        }
        run_post_commit_callbacks();
    }

    pub(crate) fn resolve_mounted_node(handle: crate::node::NodeHandle) -> Option<NodeRef> {
        crate::event::resolve_node(handle)
    }

    pub fn capture_persisted_ui_state() {
        MOUNTED_ROOT.with(|slot| {
            if let Some(root) = slot.borrow().as_ref() {
                root.capture_persisted_state_tree();
            }
        });
    }

    pub fn restore_persisted_ui_state() {
        MOUNTED_ROOT.with(|slot| {
            if let Some(root) = slot.borrow().as_ref() {
                root.restore_persisted_state_tree();
            }
        });
    }
}

#[derive(Clone)]
struct NodeRefMount(NodeRef);

impl Node for NodeRefMount {
    fn node_ref(&self) -> crate::node::NodeRef {
        self.0.clone()
    }

    fn retained_node_ref(&self) -> crate::node::NodeRef {
        self.0.clone()
    }

    fn build_self(&self) {}
}

#[cfg(not(feature = "worker-runtime"))]
#[no_mangle]
pub extern "C" fn __flushRenders() {
    Application::flush_renders();
}
