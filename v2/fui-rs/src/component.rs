use crate::bindings::ui;
use crate::node::{Node, BuiltNode};
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

pub trait Component {
    fn render(&self) -> Box<dyn Node>;
}

pub(crate) struct OwnerHandle {
    on_dirty: RefCell<Option<Rc<dyn Fn()>>>,
}

impl OwnerHandle {
    fn new() -> Self {
        Self {
            on_dirty: RefCell::new(None),
        }
    }

    fn install(&self, callback: Rc<dyn Fn()>) {
        self.on_dirty.replace(Some(callback));
    }

    pub(crate) fn notify_dirty(&self) {
        if let Some(callback) = self.on_dirty.borrow().as_ref() {
            callback();
        }
    }
}

thread_local! {
    static CURRENT_OWNER: RefCell<Option<Rc<OwnerHandle>>> = RefCell::new(None);
    static ACTIVE_APP: RefCell<Option<Rc<dyn MountedApp>>> = RefCell::new(None);
    static RENDER_SCHEDULER: RenderScheduler = RenderScheduler::new();
}

pub(crate) fn current_owner() -> Rc<OwnerHandle> {
    CURRENT_OWNER.with(|slot| {
        slot.borrow()
            .as_ref()
            .cloned()
            .expect("state() must be created while a component is being constructed or rendered")
    })
}

pub(crate) fn with_current_owner<T>(owner: Rc<OwnerHandle>, render: impl FnOnce() -> T) -> T {
    CURRENT_OWNER.with(|slot| {
        let previous = slot.replace(Some(owner));
        let result = render();
        slot.replace(previous);
        result
    })
}

pub(crate) trait MountedApp {
    fn unmount(&self);
}

trait RenderJob {
    fn is_render_queued(&self) -> bool;
    fn set_render_queued(&self, queued: bool);
    fn perform_render(&self);
}

pub struct RenderScheduler {
    queue: RefCell<Vec<Rc<dyn RenderJob>>>,
    flushing: Cell<bool>,
}

impl RenderScheduler {
    fn new() -> Self {
        Self {
            queue: RefCell::new(Vec::new()),
            flushing: Cell::new(false),
        }
    }

    fn enqueue(&self, job: Rc<dyn RenderJob>) {
        if job.is_render_queued() {
            return;
        }

        job.set_render_queued(true);
        self.queue.borrow_mut().push(job);
        ui::request_render();

        if !self.flushing.get() {
            self.flush();
        }
    }

    fn flush(&self) {
        if self.flushing.get() {
            return;
        }

        self.flushing.set(true);
        while !self.queue.borrow().is_empty() {
            let current = std::mem::take(&mut *self.queue.borrow_mut());
            for job in current {
                job.set_render_queued(false);
                job.perform_render();
            }
        }
        self.flushing.set(false);
    }
}

struct Reconciler;

impl Reconciler {
    fn reconcile(root: &RefCell<Option<BuiltNode>>, next_tree: Box<dyn Node>) -> BuiltNode {
        if let Some(mut current_root) = root.borrow_mut().take() {
            current_root.destroy();
        }

        next_tree.build()
    }
}

pub struct ComponentInstance<C: Component + 'static> {
    component: Rc<RefCell<C>>,
    owner: Rc<OwnerHandle>,
    dirty: Cell<bool>,
    render_queued: Cell<bool>,
    mounted_root: RefCell<Option<BuiltNode>>,
}

impl<C: Component + 'static> ComponentInstance<C> {
    pub fn new(factory: impl FnOnce() -> C) -> Rc<Self> {
        let owner = Rc::new(OwnerHandle::new());
        let component = with_current_owner(owner.clone(), || Rc::new(RefCell::new(factory())));
        let instance = Rc::new(Self {
            component,
            owner: owner.clone(),
            dirty: Cell::new(false),
            render_queued: Cell::new(false),
            mounted_root: RefCell::new(None),
        });

        let weak_instance: Weak<Self> = Rc::downgrade(&instance);
        owner.install(Rc::new(move || {
            if let Some(instance) = weak_instance.upgrade() {
                instance.mark_dirty();
            }
        }));

        instance
    }

    pub fn mount(self: &Rc<Self>) {
        self.mark_dirty();
    }

    fn mark_dirty(self: &Rc<Self>) {
        self.dirty.set(true);
        enqueue_render_job(self.clone());
    }

    fn rebuild(&self) {
        let next_tree = with_current_owner(self.owner.clone(), || self.component.borrow().render());
        let next_root = Reconciler::reconcile(&self.mounted_root, next_tree);
        ui::set_root(next_root.handle());
        ui::commit_frame();
        self.mounted_root.replace(Some(next_root));
        self.dirty.set(false);
    }

    fn unmount_impl(&self) {
        if let Some(mut root) = self.mounted_root.borrow_mut().take() {
            root.destroy();
        }
        self.dirty.set(false);
        self.render_queued.set(false);
    }
}

impl<C: Component + 'static> MountedApp for ComponentInstance<C> {
    fn unmount(&self) {
        self.unmount_impl();
    }
}

impl<C: Component + 'static> RenderJob for ComponentInstance<C> {
    fn is_render_queued(&self) -> bool {
        self.render_queued.get()
    }

    fn set_render_queued(&self, queued: bool) {
        self.render_queued.set(queued);
    }

    fn perform_render(&self) {
        if self.dirty.get() {
            self.rebuild();
        }
    }
}

fn enqueue_render_job<C: Component + 'static>(instance: Rc<ComponentInstance<C>>) {
    RENDER_SCHEDULER.with(|scheduler| scheduler.enqueue(instance as Rc<dyn RenderJob>));
}

pub(crate) fn install_active_app(app: Rc<dyn MountedApp>) {
    ACTIVE_APP.with(|slot| {
        if let Some(previous) = slot.borrow_mut().replace(app) {
            previous.unmount();
        }
    });
}

pub fn flush_render_scheduler() {
    RENDER_SCHEDULER.with(|scheduler| scheduler.flush());
}
