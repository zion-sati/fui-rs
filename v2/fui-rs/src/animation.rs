use crate::frame_scheduler::mark_needs_commit;
use std::cell::RefCell;
use std::rc::Rc;

const MAX_FRAME_DELTA_MS: f64 = 100.0;

fn clamp_unit(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

fn clamp_frame_delta(delta_ms: f64) -> f64 {
    delta_ms.clamp(0.0, MAX_FRAME_DELTA_MS)
}

fn mix_float(from: f32, to: f32, amount: f32) -> f32 {
    from + ((to - from) * clamp_unit(amount))
}

fn mix_color_channel(from: u32, to: u32, amount: f32) -> u32 {
    let mixed = (from as f32) + (((to as f32) - (from as f32)) * clamp_unit(amount));
    mixed.round().clamp(0.0, 255.0) as u32
}

fn mix_color(from: u32, to: u32, amount: f32) -> u32 {
    let fr = (from >> 24) & 0xFF;
    let fg = (from >> 16) & 0xFF;
    let fb = (from >> 8) & 0xFF;
    let fa = from & 0xFF;
    let tr = (to >> 24) & 0xFF;
    let tg = (to >> 16) & 0xFF;
    let tb = (to >> 8) & 0xFF;
    let ta = to & 0xFF;
    (mix_color_channel(fr, tr, amount) << 24)
        | (mix_color_channel(fg, tg, amount) << 16)
        | (mix_color_channel(fb, tb, amount) << 8)
        | mix_color_channel(fa, ta, amount)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Easing {
    Linear,
    CubicIn,
    CubicOut,
    CubicInOut,
    QuadOut,
}

impl Easing {
    pub fn sample(self, progress: f32) -> f32 {
        let t = clamp_unit(progress);
        match self {
            Self::Linear => t,
            Self::CubicIn => t * t * t,
            Self::CubicOut => {
                let offset = t - 1.0;
                (offset * offset * offset) + 1.0
            }
            Self::CubicInOut => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    let offset = (-2.0 * t) + 2.0;
                    1.0 - ((offset * offset * offset) * 0.5)
                }
            }
            Self::QuadOut => {
                let offset = 1.0 - t;
                1.0 - (offset * offset)
            }
        }
    }
}

pub struct Easings;

impl Easings {
    pub fn linear() -> Easing {
        Easing::Linear
    }

    pub fn cubic_in() -> Easing {
        Easing::CubicIn
    }

    pub fn cubic_out() -> Easing {
        Easing::CubicOut
    }

    pub fn cubic_in_out() -> Easing {
        Easing::CubicInOut
    }

    pub fn quad_out() -> Easing {
        Easing::QuadOut
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnimationTiming {
    pub duration_ms: f64,
    pub easing: Easing,
}

impl AnimationTiming {
    pub fn new(duration_ms: f64) -> Self {
        Self {
            duration_ms: duration_ms.max(0.0),
            easing: Easing::Linear,
        }
    }

    pub fn with_easing(duration_ms: f64, easing: Easing) -> Self {
        Self {
            duration_ms: duration_ms.max(0.0),
            easing,
        }
    }
}

trait AnimationDriver {
    fn on_start(&mut self, _timestamp_ms: f64) {}
    fn on_sample(&mut self, eased_progress: f32, linear_progress: f32);
    fn on_stop(&mut self, _finished: bool) {}
}

struct AnimationState {
    timing: AnimationTiming,
    running: bool,
    started: bool,
    last_timestamp_ms: f64,
    elapsed_ms: f64,
    driver: Box<dyn AnimationDriver>,
}

#[derive(Clone)]
pub struct Animation {
    inner: Rc<RefCell<AnimationState>>,
}

impl Animation {
    fn new(timing: AnimationTiming, driver: Box<dyn AnimationDriver>) -> Self {
        Self {
            inner: Rc::new(RefCell::new(AnimationState {
                timing,
                running: false,
                started: false,
                last_timestamp_ms: 0.0,
                elapsed_ms: 0.0,
                driver,
            })),
        }
    }

    pub fn is_running(&self) -> bool {
        self.inner.borrow().running
    }

    pub fn cancel(&self) {
        get_animation_manager().cancel(self);
    }

    pub fn finish(&self) {
        get_animation_manager().finish(self);
    }

    pub fn dispose(&self) {
        self.cancel();
    }

    fn attach(&self, known_timestamp_ms: f64, has_known_timestamp: bool) {
        {
            let mut state = self.inner.borrow_mut();
            state.running = true;
            state.started = false;
            state.last_timestamp_ms = 0.0;
            state.elapsed_ms = 0.0;
        }
        if has_known_timestamp {
            self.start_internal(known_timestamp_ms);
        }
    }

    fn tick(&self, timestamp_ms: f64) {
        if !self.is_running() {
            return;
        }
        if !self.inner.borrow().started {
            self.start_internal(timestamp_ms);
            return;
        }
        let (duration_ms, easing, last_timestamp_ms) = {
            let state = self.inner.borrow();
            (
                state.timing.duration_ms,
                state.timing.easing,
                state.last_timestamp_ms,
            )
        };
        let delta_ms = clamp_frame_delta(timestamp_ms - last_timestamp_ms);
        let should_finish = {
            let mut state = self.inner.borrow_mut();
            state.last_timestamp_ms = timestamp_ms;
            state.elapsed_ms += delta_ms;
            let progress = if duration_ms <= 0.0 {
                1.0
            } else {
                clamp_unit((state.elapsed_ms / duration_ms) as f32)
            };
            state.driver.on_sample(easing.sample(progress), progress);
            progress >= 1.0
        };
        if should_finish {
            self.stop(true);
        }
    }

    fn cancel_internal(&self) {
        if self.is_running() {
            self.stop(false);
        }
    }

    fn finish_internal(&self) {
        if !self.is_running() {
            return;
        }
        if !self.inner.borrow().started {
            let mut state = self.inner.borrow_mut();
            state.started = true;
            state.last_timestamp_ms = 0.0;
            state.elapsed_ms = state.timing.duration_ms;
            state.driver.on_start(0.0);
        }
        self.inner.borrow_mut().driver.on_sample(1.0, 1.0);
        self.stop(true);
    }

    fn start_internal(&self, timestamp_ms: f64) {
        let duration_ms = {
            let mut state = self.inner.borrow_mut();
            state.started = true;
            state.last_timestamp_ms = timestamp_ms;
            state.elapsed_ms = 0.0;
            state.driver.on_start(timestamp_ms);
            state.timing.duration_ms
        };
        if duration_ms <= 0.0 {
            self.inner.borrow_mut().driver.on_sample(1.0, 1.0);
            self.stop(true);
            return;
        }
        let easing = self.inner.borrow().timing.easing;
        self.inner
            .borrow_mut()
            .driver
            .on_sample(easing.sample(0.0), 0.0);
    }

    fn stop(&self, finished: bool) {
        let mut state = self.inner.borrow_mut();
        state.running = false;
        state.driver.on_stop(finished);
    }
}

#[derive(Default)]
struct AnimationManagerState {
    active_animations: Vec<Animation>,
    last_timestamp_ms: f64,
    has_last_timestamp: bool,
}

thread_local! {
    static ANIMATION_MANAGER: RefCell<AnimationManagerState> = const { RefCell::new(AnimationManagerState {
        active_animations: Vec::new(),
        last_timestamp_ms: 0.0,
        has_last_timestamp: false,
    }) };
}

#[derive(Clone, Copy, Debug, Default)]
pub struct AnimationManager;

impl AnimationManager {
    pub fn start(&self, animation: Animation) -> Animation {
        self.cancel(&animation);
        ANIMATION_MANAGER.with(|slot| {
            let mut state = slot.borrow_mut();
            animation.attach(state.last_timestamp_ms, state.has_last_timestamp);
            if animation.is_running() {
                state.active_animations.push(animation.clone());
            }
        });
        mark_needs_commit();
        animation
    }

    pub fn cancel(&self, animation: &Animation) {
        ANIMATION_MANAGER.with(|slot| {
            let mut state = slot.borrow_mut();
            if let Some(index) = state
                .active_animations
                .iter()
                .position(|candidate| Rc::ptr_eq(&candidate.inner, &animation.inner))
            {
                state.active_animations.swap_remove(index);
                animation.cancel_internal();
            }
        });
    }

    pub fn finish(&self, animation: &Animation) {
        ANIMATION_MANAGER.with(|slot| {
            let mut state = slot.borrow_mut();
            if let Some(index) = state
                .active_animations
                .iter()
                .position(|candidate| Rc::ptr_eq(&candidate.inner, &animation.inner))
            {
                state.active_animations.swap_remove(index);
                animation.finish_internal();
            }
        });
    }

    pub fn tick(&self, timestamp_ms: f64) {
        let active_animations = ANIMATION_MANAGER.with(|slot| {
            let mut state = slot.borrow_mut();
            state.last_timestamp_ms = timestamp_ms;
            state.has_last_timestamp = true;
            state.active_animations.clone()
        });
        let mut completed = Vec::new();
        let mut has_active = false;
        for animation in active_animations {
            animation.tick(timestamp_ms);
            if animation.is_running() {
                has_active = true;
            } else {
                completed.push(animation);
            }
        }
        if !completed.is_empty() {
            ANIMATION_MANAGER.with(|slot| {
                let mut state = slot.borrow_mut();
                state.active_animations.retain(|candidate| {
                    !completed.iter().any(|completed_animation| {
                        Rc::ptr_eq(&candidate.inner, &completed_animation.inner)
                    })
                });
            });
        }
        if has_active {
            mark_needs_commit();
        }
    }

    pub fn has_active_animations(&self) -> bool {
        ANIMATION_MANAGER.with(|slot| !slot.borrow().active_animations.is_empty())
    }

    pub fn reset(&self) {
        ANIMATION_MANAGER.with(|slot| {
            let mut state = slot.borrow_mut();
            for animation in state.active_animations.drain(..) {
                animation.cancel_internal();
            }
            state.last_timestamp_ms = 0.0;
            state.has_last_timestamp = false;
        });
    }
}

pub fn get_animation_manager() -> AnimationManager {
    AnimationManager
}

pub fn tick_animations(timestamp_ms: f64) {
    get_animation_manager().tick(timestamp_ms);
}

pub fn reset_animations() {
    get_animation_manager().reset();
}

struct FloatAnimationDriver {
    from: f32,
    to: f32,
    handler: Box<dyn Fn(f32)>,
}

impl AnimationDriver for FloatAnimationDriver {
    fn on_sample(&mut self, eased_progress: f32, _linear_progress: f32) {
        (self.handler)(mix_float(self.from, self.to, eased_progress));
    }
}

struct ColorAnimationDriver {
    from: u32,
    to: u32,
    handler: Box<dyn Fn(u32)>,
}

impl AnimationDriver for ColorAnimationDriver {
    fn on_sample(&mut self, eased_progress: f32, _linear_progress: f32) {
        (self.handler)(mix_color(self.from, self.to, eased_progress));
    }
}

pub fn animate_float(
    from_value: f32,
    to_value: f32,
    timing: AnimationTiming,
    handler: impl Fn(f32) + 'static,
) -> Animation {
    get_animation_manager().start(Animation::new(
        timing,
        Box::new(FloatAnimationDriver {
            from: from_value,
            to: to_value,
            handler: Box::new(handler),
        }),
    ))
}

pub fn animate_float_with<Owner: 'static>(
    owner: Owner,
    from_value: f32,
    to_value: f32,
    timing: AnimationTiming,
    handler: impl Fn(&Owner, f32) + 'static,
) -> Animation {
    animate_float(from_value, to_value, timing, move |value| {
        handler(&owner, value);
    })
}

pub fn animate_color(
    from_value: u32,
    to_value: u32,
    timing: AnimationTiming,
    handler: impl Fn(u32) + 'static,
) -> Animation {
    get_animation_manager().start(Animation::new(
        timing,
        Box::new(ColorAnimationDriver {
            from: from_value,
            to: to_value,
            handler: Box::new(handler),
        }),
    ))
}

pub fn animate_color_with<Owner: 'static>(
    owner: Owner,
    from_value: u32,
    to_value: u32,
    timing: AnimationTiming,
    handler: impl Fn(&Owner, u32) + 'static,
) -> Animation {
    animate_color(from_value, to_value, timing, move |value| {
        handler(&owner, value);
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge_callbacks::__fui_on_frame;
    use crate::ffi::{self, Call};
    use crate::frame_scheduler::reset_commit_state;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn advances_float_animations_from_frame_timestamps() {
        reset_animations();
        reset_commit_state();
        let value = Rc::new(Cell::new(-1.0));
        let value_for_handler = value.clone();
        let animation = animate_float(10.0, 30.0, AnimationTiming::new(200.0), move |next| {
            value_for_handler.set(next);
        });
        let manager = get_animation_manager();

        manager.tick(1000.0);
        assert_eq!(value.get(), 10.0);
        assert!(animation.is_running());

        manager.tick(1050.0);
        assert_eq!(value.get(), 15.0);

        manager.tick(1150.0);
        assert_eq!(value.get(), 25.0);

        manager.tick(1250.0);
        assert_eq!(value.get(), 30.0);
        assert!(!animation.is_running());
        assert!(!manager.has_active_animations());
    }

    #[test]
    fn clamps_stale_frame_gaps_instead_of_jumping_to_completion() {
        reset_animations();
        reset_commit_state();
        let value = Rc::new(Cell::new(-1.0));
        let value_for_handler = value.clone();
        let manager = get_animation_manager();
        animate_float(0.0, 100.0, AnimationTiming::new(200.0), move |next| {
            value_for_handler.set(next);
        });

        manager.tick(1000.0);
        manager.tick(1800.0);

        assert_eq!(value.get(), 50.0);
        assert!(manager.has_active_animations());
    }

    #[test]
    fn supports_owner_bound_float_and_color_helpers() {
        reset_animations();
        reset_commit_state();
        let float_owner = Rc::new(Cell::new(-1.0));
        let color_owner = Rc::new(Cell::new(0u32));
        let manager = get_animation_manager();

        animate_float_with(
            float_owner.clone(),
            4.0,
            12.0,
            AnimationTiming::new(80.0),
            |owner, value| owner.set(value),
        );
        animate_color_with(
            color_owner.clone(),
            0x000000FF,
            0xFF0000FF,
            AnimationTiming::new(80.0),
            |owner, value| owner.set(value),
        );

        manager.tick(500.0);
        assert_eq!(float_owner.get(), 4.0);
        assert_eq!(color_owner.get(), 0x000000FF);

        manager.tick(540.0);
        assert_eq!(float_owner.get(), 8.0);
        assert_eq!(color_owner.get(), 0x800000FF);

        manager.tick(580.0);
        assert_eq!(float_owner.get(), 12.0);
        assert_eq!(color_owner.get(), 0xFF0000FF);
    }

    #[test]
    fn cancels_and_finishes_animations_through_the_animation_object() {
        reset_animations();
        reset_commit_state();
        let value = Rc::new(Cell::new(-1.0));
        let first_value = value.clone();
        let animation = animate_float(0.0, 100.0, AnimationTiming::new(100.0), move |next| {
            first_value.set(next);
        });
        let manager = get_animation_manager();

        manager.tick(100.0);
        manager.tick(140.0);
        assert_eq!(value.get(), 40.0);

        animation.cancel();
        manager.tick(200.0);
        assert_eq!(value.get(), 40.0);
        assert!(!animation.is_running());

        let second_value = value.clone();
        let second = animate_float(0.0, 100.0, AnimationTiming::new(100.0), move |next| {
            second_value.set(next);
        });
        manager.tick(300.0);
        second.finish();
        assert_eq!(value.get(), 100.0);
        assert!(!second.is_running());
        assert!(!manager.has_active_animations());
    }

    #[test]
    fn uses_the_shared_frame_hook_to_tick_active_animations() {
        reset_animations();
        reset_commit_state();
        let owner = Rc::new(Cell::new(-1.0));
        animate_float_with(
            owner.clone(),
            2.0,
            6.0,
            AnimationTiming::new(100.0),
            |slot, value| {
                slot.set(value);
            },
        );

        __fui_on_frame(1000.0);
        assert_eq!(crate::frame_signal::frame_time_signal().value(), 1000.0);
        assert_eq!(owner.get(), 2.0);

        __fui_on_frame(1050.0);
        assert_eq!(owner.get(), 4.0);
    }

    #[test]
    fn requests_render_when_animation_starts() {
        reset_animations();
        reset_commit_state();
        ffi::test::reset();
        animate_float(0.0, 1.0, AnimationTiming::new(50.0), |_| {});
        let calls = ffi::test::take_calls();
        assert_eq!(
            calls
                .iter()
                .filter(|call| matches!(call, Call::RequestRender))
                .count(),
            1
        );
    }
}
