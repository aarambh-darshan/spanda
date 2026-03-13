# Integrations

`spanda` is designed as a pure data-transformer. It knows nothing about the screen, pixels, or your windowing library. This makes integrating it trivial.

## TUI / CLI

In a CLI or terminal UI (like `ratatui`), you usually run a frame loop. You can use standard Rust clocks to compute `dt`:

```rust
use spanda::clock::{Clock, WallClock};

let mut clock = WallClock::new();

loop {
    let dt = clock.delta();
    tween.update(dt);
    
    // Render your TUI
    
    std::thread::sleep(std::time::Duration::from_millis(16));
}
```

## Bevy Plugin

If you use [Bevy](https://bevyengine.org), activate the `bevy` feature in `Cargo.toml`:

```toml
[dependencies]
spanda = { version = "0.1", features = ["bevy"] }
```

This adds `SpandaPlugin`, which automatically registers `Tween` and `Spring` as ECS Components and ticks them in the `Update` schedule using Bevy's `Time` resource.

```rust
use bevy::prelude::*;
use spanda::integrations::bevy::{SpandaPlugin, TweenCompleted};
use spanda::{Tween, Easing};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Add the integration plugin
        .add_plugins(SpandaPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, listen)
        .run();
}

fn setup(mut commands: Commands) {
    // Spawns an entity with a Tween component.
    // The plugin will tick it for you automatically every frame!
    commands.spawn((
        // Transform, SpriteBundle, etc...
        Tween::new(0.0_f32, 100.0).duration(1.0).easing(Easing::EaseInOut).build(),
    ));
}

fn listen(mut events: EventReader<TweenCompleted>) {
    // SpandaPlugin fires this when a Tween finishes
    for event in events.read() {
        println!("Entity {:?} finished its tween!", event.entity);
    }
}
```

## WASM / Web

If you're building a WebAssembly app (e.g. `wasm-bindgen`, `leptos`, `yew`), standard `std::time` doesn't work for smooth visuals. You need to bind to the browser's `requestAnimationFrame`.

Activate the `wasm` feature:

```toml
[dependencies]
spanda = { version = "0.1", features = ["wasm"] }
```

Use `RafDriver`. Pass it the high-res milliseconds provided by Javascript:

```rust
use spanda::integrations::wasm::RafDriver;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct App {
    driver: RafDriver,
}

#[wasm_bindgen]
impl App {
    pub fn new() -> Self {
        let mut driver = RafDriver::new();
        // driver.add(Tween::new(...).build());
        Self { driver }
    }

    // Call this from JS requestAnimationFrame(timestamp => app.tick(timestamp))
    pub fn tick(&mut self, timestamp_ms: f64) {
        // Automatically computes dt and ticks all internal animations
        self.driver.tick(timestamp_ms);
    }
}
```
