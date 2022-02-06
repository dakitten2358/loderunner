use crate::BevyState;
use bevy::asset::{HandleId, LoadState};
use bevy::ecs::schedule::ParallelSystemDescriptor;
use bevy::prelude::*;
use bevy::utils::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemLabel)]
pub enum LoadingLabel {
    Pre,
    Post,
}

pub struct LoadingPlugin<S: BevyState> {
    /// The loading state during which progress will be tracked
    pub loading_state: S,
    /// The next state to transition to, when all progress completes
    pub next_state: S,
}

impl<S: BevyState> Plugin for LoadingPlugin<S> {
    fn build(&self, app: &mut App) {
        app.init_resource::<AssetsLoading>();
        app.add_system_set(SystemSet::on_enter(self.loading_state.clone()).with_system(loadstate_enter));
        app.add_system_set(
            SystemSet::on_update(self.loading_state.clone())
                .with_system(clear_progress.label(LoadingLabel::Pre))
                .with_system(
                    check_progress::<S>
                        .config(|(s, _, _)| {
                            *s = Some(Some(self.next_state.clone()));
                        })
                        .label(LoadingLabel::Post),
                )
                .with_system(track(assets_progress)),
        );
        app.add_system_set(
            SystemSet::on_exit(self.loading_state.clone())
                .with_system(loadstate_exit)
                .with_system(assets_loading_reset),
        );
    }
}

fn loadstate_enter(mut commands: Commands) {
    commands.insert_resource(ProgressCounter::default());
}

fn loadstate_exit(mut commands: Commands) {
    commands.remove_resource::<ProgressCounter>();
}

fn assets_loading_reset(mut loading: ResMut<AssetsLoading>) {
    *loading = AssetsLoading::default();
}

fn assets_progress(mut loading: ResMut<AssetsLoading>, server: Res<AssetServer>) -> Progress {
    // TODO: avoid this temporary vec (HashSet::drain_filter is in Rust nightly)
    let mut done = vec![];
    for handle in loading.handles.iter() {
        if server.get_load_state(*handle) != LoadState::Loading {
            done.push(*handle);
        }
    }
    for handle in done {
        loading.handles.remove(&handle);
    }

    Progress {
        done: loading.total - loading.handles.len() as u32,
        total: loading.total,
    }
}

pub fn track<Params, S: IntoSystem<(), Progress, Params>>(s: S) -> ParallelSystemDescriptor {
    s.chain(tracker).before(LoadingLabel::Post).after(LoadingLabel::Pre).into()
}

fn tracker(In(progress): In<Progress>, counter: Res<ProgressCounter>) {
    counter.manually_tick(progress);
}

fn check_progress<S: BevyState>(next_state: Local<Option<S>>, mut counter: ResMut<ProgressCounter>, mut state: ResMut<State<S>>) {
    let total = counter.total.load(Ordering::Acquire);
    let done = counter.done.load(Ordering::Acquire);

    let progress = Progress { done, total };

    // Update total progress to report to user
    counter.last_progress = progress;

    if progress.is_ready() {
        if let Some(next_state) = &*next_state {
            state.set(next_state.clone()).ok();
        }
    }
}

fn clear_progress(counter: ResMut<ProgressCounter>) {
    counter.done.store(0, Ordering::Release);
    counter.total.store(0, Ordering::Release);
}

#[derive(Default)]
pub struct AssetsLoading {
    handles: HashSet<HandleId>,
    total: u32,
}

impl AssetsLoading {
    /// Add an asset to be tracked
    pub fn add<T: Into<HandleId>>(&mut self, handle: T) {
        self.handles.insert(handle.into());
        self.total += 1;
    }
}

/// Resource for tracking overall progress
///
/// This resource is automatically created when entering the load state and removed when exiting it.
#[derive(Default)]
pub struct ProgressCounter {
    // use atomics to track overall progress,
    // so that we can avoid mut access in tracked systems,
    // allowing them to run in parallel
    done: AtomicU32,
    total: AtomicU32,
    last_progress: Progress,
}

impl ProgressCounter {
    /// Get the latest overall progress information
    ///
    /// This is the combined total of all systems.
    ///
    /// It is updated during `ReadyLabel::Post`.
    /// If your system runs after that label, you will get the value from the current frame update.
    /// If your system runs before that label, you will get the value from the previous frame update.
    pub fn progress(&self) -> Progress {
        self.last_progress
    }

    /// Add some amount of progress to the running total for the current frame.
    ///
    /// You typically don't need to call this function yourself.
    ///
    /// It may be useful for advanced use cases, like from exclusive systems.
    pub fn manually_tick(&self, progress: Progress) {
        self.total.fetch_add(progress.total, Ordering::Release);
        // use `min` to clamp in case a bad user provides `done > total`
        self.done.fetch_add(progress.done.min(progress.total), Ordering::Release);
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Progress {
    /// Units of work completed
    pub done: u32,
    /// Total units of work expected
    pub total: u32,
}

impl Progress {
    fn is_ready(self) -> bool {
        self.done >= self.total
    }
}

impl From<Progress> for f32 {
    fn from(p: Progress) -> f32 {
        p.done as f32 / p.total as f32
    }
}
