# Bezier Paths & Motion Paths

While [Tweens](tween.md) animate along a straight line from A to B, **motion paths** let you animate along arbitrary curves — arcs, S-curves, loops, and complex multi-segment routes.

Spanda provides three types:

- **BezierPath** — a single linear, quadratic, or cubic Bezier curve segment
- **MotionPath** — a multi-segment path composed of Bezier curves
- **MotionPathTween** — an `Update`-implementing wrapper that animates a value along a `MotionPath`

---

## BezierPath

A `BezierPath<T>` represents a single curve segment. It supports three variants:

### Linear (Straight Line)

```rust
use spanda::path::{BezierPath, PathEvaluate};

let line = BezierPath::linear([0.0_f32, 0.0], [100.0, 100.0]);

let mid = line.evaluate(0.5); // [50.0, 50.0]
```

### Quadratic Bezier (One Control Point)

A quadratic Bezier creates a smooth arc through one control point:

```rust
use spanda::path::{BezierPath, PathEvaluate};

let arc = BezierPath::quadratic(
    [0.0_f32, 0.0],    // start
    [50.0, 100.0],      // control point (pulls the curve upward)
    [100.0, 0.0],       // end
);

let peak = arc.evaluate(0.5); // somewhere near [50, 50]
```

### Cubic Bezier (Two Control Points)

A cubic Bezier gives the most control — S-curves, loops, and complex shapes:

```rust
use spanda::path::{BezierPath, PathEvaluate};

let s_curve = BezierPath::cubic(
    [0.0_f32, 0.0],    // start
    [0.0, 100.0],       // control 1 — pulls upward
    [100.0, 100.0],     // control 2 — pulls across
    [100.0, 0.0],       // end
);

let point = s_curve.evaluate(0.5); // midpoint of the S-curve
```

### The PathEvaluate Trait

All path types implement `PathEvaluate<T>`:

```rust
pub trait PathEvaluate<T> {
    fn evaluate(&self, t: f32) -> T;
}
```

- `t = 0.0` returns the start point
- `t = 1.0` returns the end point
- `t` is clamped to `[0.0, 1.0]`

### Algorithm

Bezier curves are evaluated using **De Casteljau's algorithm** — a numerically stable recursive interpolation that works with any type implementing `Interpolate`.

---

## MotionPath

A `MotionPath<T>` composes multiple `BezierPath` segments into one continuous path. Each segment has a **weight** that determines what fraction of the overall `t` range it occupies.

### Building a Path

```rust
use spanda::path::{MotionPath, PathEvaluate};

let path = MotionPath::new()
    .line([0.0_f32, 0.0], [100.0, 0.0])       // straight right
    .cubic(                                      // curve upward
        [100.0, 0.0],
        [100.0, 50.0],
        [150.0, 100.0],
        [200.0, 100.0],
    )
    .line([200.0, 100.0], [300.0, 100.0]);      // straight right again
```

### How Segments Share Time

By default, all segments have weight 1.0, meaning they share the `t` range equally:

```
3 segments, equal weight:
Segment 0: t = 0.000 → 0.333
Segment 1: t = 0.333 → 0.667
Segment 2: t = 0.667 → 1.000
```

### Weighted Segments

Use `_weighted` variants to give segments proportionally more or less time:

```rust
let path = MotionPath::new()
    .line_weighted([0.0_f32, 0.0], [300.0, 0.0], 3.0)  // 3x weight
    .line_weighted([300.0, 0.0], [400.0, 0.0], 1.0);    // 1x weight

// First segment gets 75% of the t range (3/4)
// Second segment gets 25% (1/4)
```

This is useful when segments have different visual lengths — a short connector segment shouldn't take as much time as a long straight section.

### Builder Methods

| Method | Description |
|--------|-------------|
| `.line(start, end)` | Append a linear segment (weight 1.0) |
| `.quadratic(start, control, end)` | Append a quadratic Bezier (weight 1.0) |
| `.cubic(start, c1, c2, end)` | Append a cubic Bezier (weight 1.0) |
| `.line_weighted(start, end, weight)` | Linear with custom weight |
| `.quadratic_weighted(start, control, end, weight)` | Quadratic with custom weight |
| `.cubic_weighted(start, c1, c2, end, weight)` | Cubic with custom weight |
| `.segment(bezier_path)` | Append a raw `BezierPath` (weight 1.0) |
| `.segment_weighted(bezier_path, weight)` | Raw path with custom weight |
| `.segment_count()` | Number of segments |

---

## MotionPathTween

`MotionPathTween` wraps a `MotionPath` and implements `Update`, making it usable everywhere a regular `Tween` works — timelines, sequences, drivers, and the animation loop.

```rust
use spanda::path::{MotionPath, MotionPathTween};
use spanda::easing::Easing;
use spanda::traits::Update;

let path = MotionPath::new()
    .line([0.0_f32, 0.0], [100.0, 0.0])
    .line([100.0, 0.0], [100.0, 100.0]);

let mut tween = MotionPathTween::new(path)
    .duration(2.0)
    .easing(Easing::EaseInOutCubic);

// In your render loop:
tween.update(dt);
let position = tween.value(); // current [x, y] on the path
```

### Methods

| Method | Description |
|--------|-------------|
| `MotionPathTween::new(path)` | Create a tween along the path (default: 1.0s, Linear) |
| `.duration(seconds)` | Set duration |
| `.easing(curve)` | Set easing curve |
| `.value()` | Current position on the path |
| `.progress()` | Raw progress `0.0..=1.0` |
| `.is_complete()` | Whether the animation has finished |
| `.reset()` | Reset to the beginning |

### With AnimationDriver

```rust
use spanda::driver::AnimationDriver;
use spanda::path::{MotionPath, MotionPathTween};

let mut driver = AnimationDriver::new();

let path = MotionPath::new()
    .cubic([0.0_f32, 0.0], [50.0, 100.0], [100.0, 100.0], [150.0, 0.0]);

driver.add(MotionPathTween::new(path).duration(1.0));
driver.tick(0.5); // advances the path tween
```

### With Timelines

```rust
use spanda::timeline::Timeline;
use spanda::path::{MotionPath, MotionPathTween};
use spanda::tween::Tween;
use spanda::easing::Easing;

let path = MotionPath::new()
    .cubic([0.0_f32, 0.0], [50.0, 100.0], [100.0, 100.0], [150.0, 0.0]);

let mut tl = Timeline::new()
    .add("move", MotionPathTween::new(path).duration(1.0), 0.0)
    .add("fade", Tween::new(0.0_f32, 1.0).duration(0.5).build(), 0.0);

tl.play();
```

---

## Supported Types

Bezier paths and motion paths work with any type that implements `Interpolate + Clone`:

- `f32`, `f64` — 1D curves (useful for non-linear value transitions)
- `[f32; 2]` — 2D paths (most common: x, y movement)
- `[f32; 3]` — 3D paths (x, y, z movement)
- `[f32; 4]` — 4D paths (3D + alpha, or RGBA colour paths)
- Custom types implementing `Interpolate`

---

## Edge Cases

| Scenario | Behaviour |
|----------|-----------|
| Empty `MotionPath` evaluated | Panics ("empty path") |
| Single segment | Evaluates the segment directly |
| `t` outside `[0.0, 1.0]` | Clamped to endpoints |
| Zero-weight segments | Treated as zero-length — skipped |
| `MotionPathTween` with `duration(0.0)` | Completes immediately, returns end position |

---

## Bezier Path Visualisation

### Linear Path
```
Start ──────────────── End
```

### Quadratic Bezier
```
        Control
        ╱    ╲
       ╱      ╲
Start ╱        ╲ End
```

### Cubic Bezier
```
     C1─────C2
    ╱          ╲
   ╱            ╲
Start            End
```

The control points "pull" the curve toward them without the curve necessarily passing through them.

---

## Full Motion Path System (0.4.0+)

The advanced motion path system provides GSAP MotionPathPlugin-equivalent functionality. All types output raw `[f32; 2]` values — you decide where to render them.

### CatmullRomSpline

A smooth curve that passes **through** every given point (unlike raw Bezier where control points pull but don't lie on the curve).

```rust
use spanda::bezier::{CatmullRomSpline, PathEvaluate2D};

let spline = CatmullRomSpline::new(vec![
    [0.0, 0.0],
    [100.0, 50.0],
    [200.0, 0.0],
    [300.0, 50.0],
]);

let mid = spline.evaluate([0.0, 0.0], 0.5);
let tan = spline.tangent([0.0, 0.0], 0.5);
```

The `tension` parameter controls curviness:
- `0.0` = straight lines between points
- `0.5` = standard Catmull-Rom (default)
- `1.0` = maximum curvature
- `>1.0` = exaggerated curvature (similar to GSAP `curviness`)

```rust
let exaggerated = CatmullRomSpline::new(points).tension(1.5);
```

### PolyPath

A smooth path through `[f32; 2]` point arrays with arc-length parameterization for constant-speed motion:

```rust
use spanda::motion_path::PolyPath;

let path = PolyPath::from_points(vec![
    [0.0, 0.0],
    [100.0, 50.0],
    [200.0, 0.0],
]);

let pos = path.position(0.5);      // arc-length parameterized
let tan = path.tangent(0.5);       // tangent vector
let rot = path.rotation_deg(0.5);  // auto-rotate angle in degrees
let len = path.arc_length();       // total curve length
```

#### Start/End Offsets

Restrict animation to a portion of the path:

```rust
let path = PolyPath::from_points(points)
    .start_offset(0.2)   // begin at 20% of the path
    .end_offset(0.8);    // stop at 80%
```

#### Rotation Offset

Add a constant rotation offset to the auto-rotate angle:

```rust
let path = PolyPath::from_points(points)
    .rotation_offset(90.0);  // +90 degrees
```

#### Custom Tension

```rust
let path = PolyPath::from_points_with_tension(points, 1.5);
```

### CompoundPath

Multi-segment path from SVG-style commands:

```rust
use spanda::motion_path::{CompoundPath, PathCommand};

let path = CompoundPath::new(vec![
    PathCommand::MoveTo([0.0, 0.0]),
    PathCommand::CubicTo {
        control1: [50.0, 100.0],
        control2: [100.0, 100.0],
        end: [150.0, 0.0],
    },
    PathCommand::LineTo([200.0, 0.0]),
    PathCommand::Close,
]);

let pos = path.position(0.5);
let rot = path.rotation_deg(0.5);
```

Supports the same `start_offset`, `end_offset`, and `rotation_offset` as PolyPath.

| Command | Description |
|---------|-------------|
| `MoveTo(point)` | Start a new subpath |
| `LineTo(point)` | Straight line to point |
| `QuadTo { control, end }` | Quadratic Bezier |
| `CubicTo { control1, control2, end }` | Cubic Bezier |
| `Close` | Line back to the last MoveTo point |

### SvgPathParser

Parse SVG `d` attribute strings into `PathCommand` sequences:

```rust
use spanda::svg_path::SvgPathParser;
use spanda::motion_path::CompoundPath;

let commands = SvgPathParser::parse("M 0 0 C 50 100 100 100 150 0 L 200 0");
let path = CompoundPath::new(commands);
```

Supports: M, L, H, V, Q, C, Z (uppercase = absolute, lowercase = relative).

### CSS Cubic-Bezier Easing

```rust
use spanda::Easing;

// Equivalent to CSS: cubic-bezier(0.25, 0.1, 0.25, 1.0)
let ease = Easing::CubicBezier(0.25, 0.1, 0.25, 1.0);
```

### CSS Steps Easing

```rust
use spanda::Easing;

// Equivalent to CSS: steps(4)
let ease = Easing::Steps(4);
```

### Arc-Length Parameterization

Both `PolyPath` and `CompoundPath` use arc-length parameterization internally, meaning `position(0.5)` returns the point at 50% of the total curve length — not 50% of the parametric `t` value. This produces constant-speed motion along curves of any shape.
