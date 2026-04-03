//! Bevy bounce example — demonstrates SpandaPlugin with springs and tweens.
//!
//! This example shows how to use spanda with Bevy ECS for automatic
//! animation ticking, `TweenCompleted` events, and `SpringSettled` events.
//!
//! Run with: `cargo run --example bevy_bounce --features bevy`
//!
//! **Note**: Requires `features = ["bevy"]` in your Cargo.toml.
//!
//! ```rust,ignore
//! // This example requires the bevy feature and bevy dependency.
//! // Add to your Cargo.toml:
//! // [dependencies]
//! // bevy = "0.18"
//! // spanda = { version = "0.9.2", features = ["bevy"] }
//!
//! use bevy::prelude::*;
//! use spanda::integrations::bevy::{SpandaPlugin, TweenCompleted, SpringSettled};
//! use spanda::{Tween, Easing, Spring, SpringConfig};
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(SpandaPlugin)
//!         .add_systems(Startup, setup)
//!         .add_systems(Update, (read_springs, listen_tween_complete, listen_spring_settled))
//!         .run();
//! }
//!
//! #[derive(Component)]
//! struct BounceMarker;
//!
//! fn setup(mut commands: Commands) {
//!     commands.spawn(Camera2dBundle::default());
//!
//!     // Spawn a tween-animated entity
//!     commands.spawn((
//!         SpriteBundle {
//!             sprite: Sprite {
//!                 color: Color::rgb(0.2, 0.7, 1.0),
//!                 custom_size: Some(Vec2::new(50.0, 50.0)),
//!                 ..default()
//!             },
//!             transform: Transform::from_xyz(-200.0, 0.0, 0.0),
//!             ..default()
//!         },
//!         Tween::new(0.0_f32, 400.0)
//!             .duration(2.0)
//!             .easing(Easing::EaseOutBounce)
//!             .build(),
//!     ));
//!
//!     // Spawn a spring-animated entity
//!     let mut spring = Spring::new(SpringConfig::wobbly());
//!     spring.set_target(200.0);
//!     commands.spawn((
//!         SpriteBundle {
//!             sprite: Sprite {
//!                 color: Color::rgb(1.0, 0.4, 0.2),
//!                 custom_size: Some(Vec2::new(50.0, 50.0)),
//!                 ..default()
//!             },
//!             transform: Transform::from_xyz(0.0, -100.0, 0.0),
//!             ..default()
//!         },
//!         spring,
//!         BounceMarker,
//!     ));
//! }
//!
//! fn read_springs(
//!     mut query: Query<(&mut Transform, &Spring), With<BounceMarker>>,
//! ) {
//!     for (mut transform, spring) in query.iter_mut() {
//!         transform.translation.y = spring.position() - 100.0;
//!     }
//! }
//!
//! fn listen_tween_complete(mut events: MessageReader<TweenCompleted>) {
//!     for ev in events.read() {
//!         println!("Tween completed on entity {:?}", ev.entity);
//!     }
//! }
//!
//! fn listen_spring_settled(mut events: MessageReader<SpringSettled>) {
//!     for ev in events.read() {
//!         println!("Spring settled on entity {:?}", ev.entity);
//!     }
//! }
//! ```

fn main() {
    println!("  ══════════════════════════════════════════════");
    println!("  Bevy Bounce Example — SpandaPlugin Demo");
    println!("  ══════════════════════════════════════════════\n");
    println!("  This example demonstrates spanda's Bevy integration.");
    println!("  To run the full Bevy example, use:\n");
    println!("    cargo run --example bevy_bounce --features bevy\n");
    println!("  The SpandaPlugin provides:");
    println!("    - Auto-ticking of Tween<f32/[f32;2]/[f32;3]/[f32;4]> components");
    println!("    - Auto-ticking of Spring components");
    println!("    - TweenCompleted event when tweens finish");
    println!("    - SpringSettled event when springs reach their target\n");

    // Demonstrate the Spring and SpringN APIs (no Bevy needed)
    use spanda::spring::{Spring, SpringN, SpringConfig};
    use spanda::traits::Update;

    println!("  ── Spring (f32) ──\n");
    let mut spring = Spring::new(SpringConfig::wobbly()).with_position(0.0);
    spring.set_target(100.0);

    println!("  {:>6}  {:>10}  {:>10}  {:>8}", "frame", "position", "velocity", "settled");
    for frame in 0..30 {
        spring.update(1.0 / 60.0);
        if frame % 3 == 0 {
            println!(
                "  {:>6}  {:>10.2}  {:>10.2}  {:>8}",
                frame,
                spring.position(),
                spring.velocity(),
                spring.is_settled(),
            );
        }
    }

    println!("\n  ── SpringN<[f32; 2]> (2D) ──\n");
    let mut spring2d = SpringN::new(SpringConfig::wobbly(), [0.0_f32, 0.0]);
    spring2d.set_target([100.0, 200.0]);

    println!("  {:>6}  {:>10}  {:>10}  {:>8}", "frame", "x", "y", "settled");
    for frame in 0..60 {
        spring2d.update(1.0 / 60.0);
        if frame % 6 == 0 {
            let pos = spring2d.position();
            println!(
                "  {:>6}  {:>10.2}  {:>10.2}  {:>8}",
                frame, pos[0], pos[1],
                spring2d.is_settled(),
            );
        }
    }

    println!("\n  ── SpringN<[f32; 3]> (3D) ──\n");
    let mut spring3d = SpringN::new(SpringConfig::stiff(), [0.0_f32, 0.0, 0.0]);
    spring3d.set_target([50.0, 100.0, 150.0]);

    for _ in 0..1000 {
        spring3d.update(1.0 / 60.0);
    }

    let pos = spring3d.position();
    println!("  Final 3D position: ({:.2}, {:.2}, {:.2})", pos[0], pos[1], pos[2]);
    println!("  Settled: {}\n", spring3d.is_settled());
}
