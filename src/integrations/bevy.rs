//! Bevy integration — `SpandaPlugin` for automatic animation ticking.
//!
//! Activate with `features = ["bevy"]` in your `Cargo.toml`.
//!
//! # Usage
//!
//! ```rust,ignore
//! use bevy::prelude::*;
//! use spanda::integrations::bevy::SpandaPlugin;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(SpandaPlugin)
//!         .run();
//! }
//! ```

use bevy_app::{App, Plugin, Update};
use bevy_ecs::component::Component;
use bevy_ecs::message::{Message, MessageWriter};
use bevy_ecs::prelude::*;
use bevy_time::Time;

use crate::spring::Spring;
use crate::traits::Animatable;
use crate::traits::Update as SpandaUpdate;
use crate::tween::Tween;

// ── TweenCompleted event ─────────────────────────────────────────────────────

/// Event fired when a [`Tween`] component completes its animation.
///
/// Listen for this event in your systems to trigger follow-up logic:
///
/// ```rust,ignore
/// fn on_tween_done(mut events: EventReader<TweenCompleted>) {
///     for ev in events.read() {
///         println!("Entity {:?} finished tweening!", ev.entity);
///     }
/// }
/// ```
#[derive(Debug, Message)]
pub struct TweenCompleted {
    /// The entity whose tween just completed.
    pub entity: Entity,
}

// ── SpringSettled event ──────────────────────────────────────────────────────

/// Event fired when a [`Spring`] component settles to its target.
///
/// Listen for this event to trigger logic once a spring-driven animation
/// reaches its resting state:
///
/// ```rust,ignore
/// fn on_spring_rest(mut events: EventReader<SpringSettled>) {
///     for ev in events.read() {
///         println!("Entity {:?} spring settled!", ev.entity);
///     }
/// }
/// ```
#[derive(Debug, Message)]
pub struct SpringSettled {
    /// The entity whose spring just settled.
    pub entity: Entity,
}

// ── AnimationLabel component ─────────────────────────────────────────────────

/// Optional label component to identify animations by name in event handlers.
///
/// ```rust,ignore
/// commands.spawn((
///     Tween::new(0.0_f32, 1.0).duration(0.5).build(),
///     AnimationLabel::new("fade_in"),
/// ));
/// ```
#[derive(Component, Clone, Debug)]
pub struct AnimationLabel {
    /// The string label for this animation.
    pub label: &'static str,
}

impl AnimationLabel {
    /// Create a new animation label.
    pub fn new(label: &'static str) -> Self {
        Self { label }
    }
}

// ── SpandaPlugin ─────────────────────────────────────────────────────────────

/// Bevy plugin that automatically ticks all `Tween<T>` and `Spring` components
/// each frame using `Time::delta_seconds()`.
///
/// # What it registers
///
/// - `spanda_tick_tween_f32` system — ticks all `Tween<f32>` components
/// - `spanda_tick_tween_vec2` system — ticks all `Tween<[f32; 2]>` components
/// - `spanda_tick_tween_vec3` system — ticks all `Tween<[f32; 3]>` components
/// - `spanda_tick_tween_vec4` system — ticks all `Tween<[f32; 4]>` components
/// - `spanda_tick_spring` system — ticks all `Spring` components
/// - `TweenCompleted` event — fires when a tween finishes
/// - `SpringSettled` event — fires when a spring reaches its target
#[derive(Debug)]
pub struct SpandaPlugin;

impl Plugin for SpandaPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<TweenCompleted>()
            .add_message::<SpringSettled>()
            .add_systems(
                Update,
                (
                    spanda_tick_tween::<f32>,
                    spanda_tick_tween::<[f32; 2]>,
                    spanda_tick_tween::<[f32; 3]>,
                    spanda_tick_tween::<[f32; 4]>,
                    spanda_tick_spring,
                ),
            );
    }
}

// ── Systems ──────────────────────────────────────────────────────────────────

/// Tick all `Tween<T>` components and fire `TweenCompleted` when done.
fn spanda_tick_tween<T: Animatable + Send + Sync>(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Tween<T>)>,
    mut events: MessageWriter<TweenCompleted>,
) {
    let dt = time.delta_secs();
    for (entity, mut tween) in query.iter_mut() {
        let was_complete = tween.is_complete();
        tween.update(dt);
        if !was_complete && tween.is_complete() {
            events.write(TweenCompleted { entity });
        }
    }
}

/// Tick all `Spring` components and fire `SpringSettled` when they rest.
fn spanda_tick_spring(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Spring)>,
    mut events: MessageWriter<SpringSettled>,
) {
    let dt = time.delta_secs();
    for (entity, mut spring) in query.iter_mut() {
        let was_settled = spring.is_settled();
        spring.update(dt);
        if !was_settled && spring.is_settled() {
            events.write(SpringSettled { entity });
        }
    }
}
