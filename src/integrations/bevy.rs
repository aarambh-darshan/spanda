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
use bevy_ecs::prelude::*;
use bevy_ecs::event::Event;
use bevy_ecs::component::Component;
use bevy_time::Time;

use crate::spring::Spring;
use crate::traits::Update as SpandaUpdate;
use crate::tween::Tween;
use crate::traits::Animatable;

// ── TweenCompleted event ─────────────────────────────────────────────────────

/// Event fired when a [`Tween`] component completes its animation.
#[derive(Event)]
pub struct TweenCompleted {
    /// The entity whose tween just completed.
    pub entity: Entity,
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
/// - `TweenCompleted` event
pub struct SpandaPlugin;

impl Plugin for SpandaPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TweenCompleted>()
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
    mut events: EventWriter<TweenCompleted>,
) {
    let dt = time.delta_seconds();
    for (entity, mut tween) in query.iter_mut() {
        let was_complete = tween.is_complete();
        tween.update(dt);
        if !was_complete && tween.is_complete() {
            events.send(TweenCompleted { entity });
        }
    }
}

/// Tick all `Spring` components.
fn spanda_tick_spring(time: Res<Time>, mut query: Query<&mut Spring>) {
    let dt = time.delta_seconds();
    for mut spring in query.iter_mut() {
        spring.update(dt);
    }
}
