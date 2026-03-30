# Layout Animation

Automatic FLIP-style layout transitions without manual capture/diff.

## Overview

The `layout` module provides `LayoutAnimator` — a system for automatically
animating element positions when a layout changes. It implements the
[FLIP technique](https://aerotwist.com/blog/flip-your-animations/)
(First, Last, Invert, Play).

| Type | Purpose |
|------|---------|
| `Rect` | Captured element bounding rect |
| `LayoutAnimator` | Tracks elements and produces FLIP animations |
| `LayoutAnimation` | Translate + scale animation for a single element |
| `LayoutTransition` | Element ID + its layout animation |
| `SharedElementTransition` | Cross-view hero transitions |

## Quick Start

```rust
use spanda::layout::{LayoutAnimator, Rect};
use spanda::easing::Easing;

let mut layout = LayoutAnimator::new();

// Step 1: Track elements with their current rects
layout.track("card-1", Rect::new(0.0, 0.0, 200.0, 100.0));
layout.track("card-2", Rect::new(0.0, 120.0, 200.0, 100.0));

// Step 2: After layout change, compute transitions
let transitions = layout.compute_transitions(
    &[
        ("card-1", Rect::new(0.0, 120.0, 200.0, 100.0)),  // swapped
        ("card-2", Rect::new(0.0, 0.0, 200.0, 100.0)),
    ],
    0.4,
    Easing::EaseOutCubic,
);

// Step 3: Tick each frame
layout.update(dt);

// Step 4: Read CSS transforms
if let Some(css) = layout.css_transform("card-1") {
    // Apply: "translate(Xpx, Ypx) scale(SX, SY)"
    element.style().set_property("transform", &css);
}
```

## Batch List Reorder

Animate all elements in a list simultaneously:

```rust
let old = &[
    ("item-a", Rect::new(0.0, 0.0, 300.0, 60.0)),
    ("item-b", Rect::new(0.0, 70.0, 300.0, 60.0)),
    ("item-c", Rect::new(0.0, 140.0, 300.0, 60.0)),
];

let new = &[
    ("item-c", Rect::new(0.0, 0.0, 300.0, 60.0)),
    ("item-a", Rect::new(0.0, 70.0, 300.0, 60.0)),
    ("item-b", Rect::new(0.0, 140.0, 300.0, 60.0)),
];

let transitions = layout.animate_reorder(old, new, 0.4, Easing::EaseOutCubic);
```

## Enter / Exit Animations

### Enter (scale from 0 to full size)

```rust
let anim = layout.animate_enter(
    "new-card",
    Rect::new(50.0, 50.0, 200.0, 100.0),
    0.3,
    Easing::EaseOutCubic,
);
```

### Exit (scale from full size to 0)

```rust
if let Some(mut anim) = layout.animate_exit("leaving-card", 0.3, Easing::EaseInCubic) {
    // Tick the animation and apply transform until complete
}
```

## Shared Element Transitions

Animate an element from one view's position to another:

```rust
use spanda::layout::{SharedElementTransition, Rect};

let source = Rect::new(10.0, 10.0, 50.0, 50.0);   // thumbnail
let target = Rect::new(0.0, 0.0, 400.0, 300.0);    // full image

let mut transition = SharedElementTransition::new(
    source, target, 0.5, Easing::EaseOutCubic,
);

// In animation loop:
transition.update(dt);
let css = transition.css_transform();
element.style().set_property("transform", &css);
```

## DOM Integration (wasm-dom feature)

With the `wasm-dom` feature, capture rects directly from DOM elements:

```rust
let rect = Rect::from_element(&element);
layout.track_element(&element, "my-id");
```

## How FLIP Works

1. **First**: Capture the element's rect *before* the layout change
2. **Last**: Capture the rect *after* the layout change
3. **Invert**: Apply a CSS transform that moves the element from its *new*
   position back to its *old* position
4. **Play**: Animate the transform to identity (`translate(0, 0) scale(1, 1)`)

The `LayoutAnimator` handles all of this automatically — you just provide
the before and after rects.
