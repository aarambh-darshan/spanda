# Timelines & Sequences

When you have multiple animations, managing them individually becomes difficult. `spanda` provides two orchestrators:

- **Timeline**: Runs multiple animations *concurrently* (at the same time).
- **Sequence**: Runs multiple animations *sequentially* (one after another).

## Sequences

A `Sequence` is a builder for a Timeline that automatically calculates the correct start times to chain animations end-to-end. You can also inject specific time gaps between them.

```rust
use spanda::timeline::Sequence;
use spanda::tween::Tween;

let mut seq = Sequence::new()
    // Start immediately
    .then(Tween::new(0.0_f32, 100.0).duration(1.0).build(), 0.0)
    
    // Wait 0.5s after the first tween finishes
    .gap(0.5) 
    
    // Then run this tween 
    .then(Tween::new(100.0_f32, 0.0).duration(1.0).build(), 0.0)
    .build();

seq.play(); // Timelines must be played explicitly

// update(dt) returns true as long as ANY animation in the sequence is running
while seq.update(0.016) {
    // Both timeline entries are ticked over the total duration of 2.5s
}
```

## Timelines

If you need absolute control over exactly when an animation starts relative to the timeline's beginning (timestamp `0.0`), use a `Timeline`.

```rust
use spanda::timeline::Timeline;

let mut timeline = Timeline::new()
    // Starts immediately
    .add("fade_in", Tween::new(0.0_f32, 1.0).build(), 0.0)
    
    // Starts 0.4 seconds into the timeline, overlapping the fade
    .add("slide_up", Tween::new(50.0_f32, 0.0).build(), 0.4)
    
    // Starts 1.2 seconds in
    .add("scale", Tween::new(1.0_f32, 1.5).build(), 1.2);

timeline.play();
```

## Controls & Callbacks

Like active tweens, timelines can be controlled:

```rust
timeline.pause();
timeline.resume();
```

*(Note: `Timeline` requires the `std` feature for `Box<dyn Update>`, and `Sequence` returns a `Timeline`)*
