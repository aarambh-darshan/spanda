# Gesture Recognition

Higher-level gesture detection built on top of `PointerData` and `Observer`.

## Overview

The `gesture` module provides `GestureRecognizer`, a platform-agnostic gesture
detector that recognizes:

| Gesture | Detection Method |
|---------|-----------------|
| **Tap** | Quick down+up within distance and time thresholds |
| **Long Press** | Held pointer beyond threshold without movement |
| **Swipe** | Fast directional movement with sufficient velocity |
| **Pinch** | Two-finger distance change (scale) |
| **Rotate** | Two-finger angle change |

## Quick Start

```rust
use spanda::gesture::{GestureRecognizer, Gesture, GestureConfig};
use spanda::drag::PointerData;

let mut recognizer = GestureRecognizer::new();

// Feed pointer events from your event system
recognizer.on_pointer_down(PointerData {
    x: 100.0, y: 100.0,
    pressure: 0.5,
    pointer_id: 0,
});

// Tick each frame for long press detection
recognizer.update(dt);

// Check for gestures on pointer up
if let Some(gesture) = recognizer.on_pointer_up(data) {
    match gesture {
        Gesture::Tap { position } => handle_tap(position),
        Gesture::Swipe { direction, velocity, .. } => handle_swipe(direction, velocity),
        _ => {}
    }
}
```

## Configuration

All thresholds are configurable via `GestureConfig`:

```rust
use spanda::gesture::GestureConfig;

let config = GestureConfig {
    tap_max_distance: 10.0,       // px — max movement for tap
    tap_max_duration: 0.3,        // seconds — max hold for tap
    long_press_threshold: 0.5,    // seconds to trigger long press
    swipe_min_velocity: 300.0,    // px/s — minimum swipe speed
    swipe_min_distance: 50.0,     // px — minimum swipe distance
    pinch_min_scale_delta: 0.05,  // minimum scale change for pinch
    rotation_min_angle: 0.1,      // radians — minimum rotation angle
};

let recognizer = GestureRecognizer::with_config(config);
```

## Integration with Observer (WASM)

For DOM-backed gesture recognition, combine with `Observer`:

```rust
use spanda::integrations::observer::{Observer, ObserverCallbacks};
use spanda::gesture::GestureRecognizer;

let recognizer = Rc::new(RefCell::new(GestureRecognizer::new()));

let r = recognizer.clone();
let observer = Observer::bind(&element, ObserverCallbacks {
    on_press: Some(Box::new(move |data| {
        r.borrow_mut().on_pointer_down(data);
    })),
    on_move: Some(Box::new(move |data| {
        if let Some(gesture) = r.borrow_mut().on_pointer_move(data) {
            handle_multi_touch(gesture);
        }
    })),
    on_release: Some(Box::new(move |data| {
        if let Some(gesture) = r.borrow_mut().on_pointer_up(data) {
            handle_gesture(gesture);
        }
    })),
    on_wheel: None,
});
```

## Gesture Types

### Swipe Direction

```rust
match gesture {
    Gesture::Swipe { direction, velocity, delta } => {
        match direction {
            SwipeDirection::Up    => scroll_up(),
            SwipeDirection::Down  => scroll_down(),
            SwipeDirection::Left  => go_back(),
            SwipeDirection::Right => go_forward(),
        }
    }
    _ => {}
}
```

### Pinch Zoom

```rust
Gesture::Pinch { scale, center } => {
    // scale > 1.0 = zoom in, < 1.0 = zoom out
    set_zoom(current_zoom * scale);
    set_zoom_center(center);
}
```

### Rotation

```rust
Gesture::Rotate { angle, center } => {
    // angle in radians, positive = clockwise
    set_rotation(current_rotation + angle);
}
```

## Callbacks

With the `std` feature, register a gesture callback:

```rust
recognizer.on_gesture(|gesture| {
    println!("Detected: {:?}", gesture);
});
```
