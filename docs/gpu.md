# GPU Compute Shaders

Offload large-scale animation workloads to the GPU via wgpu compute shaders.

## Overview

The `gpu` module (feature = `"gpu"`) provides `GpuAnimationBatch` — a batch
processor that evaluates thousands of homogeneous `f32` tweens in a single
GPU compute dispatch.

| Type | Purpose |
|------|---------|
| `GpuContext` | Shared wgpu `Device` and `Queue` |
| `GpuAnimationBatch` | Batch of f32 tweens, evaluated on GPU or CPU |
| `try_create_gpu_context()` | Auto-detect and create GPU context |

## Quick Start

```rust
use spanda::gpu::GpuAnimationBatch;
use spanda::{Tween, Easing};

// Auto-detect GPU, fallback to CPU
let mut batch = GpuAnimationBatch::new_auto();

// Push 10,000 tweens
for i in 0..10_000 {
    batch.push(
        Tween::new(0.0_f32, (i as f32) / 100.0)
            .duration(1.0)
            .easing(Easing::EaseOutCubic)
            .build(),
    );
}

// Single dispatch evaluates all tweens
batch.tick(1.0 / 60.0);

// Read back results
let positions: &[f32] = batch.read_back();
println!("Evaluated {} tweens", positions.len());
```

## Feature Flag

Add to your `Cargo.toml`:

```toml
[dependencies]
spanda = { version = "0.9", features = ["gpu"] }
```

This enables `wgpu`, `pollster`, and `bytemuck` as dependencies.

## Backend Selection

| Constructor | Backend | Notes |
|-------------|---------|-------|
| `new_auto()` | GPU, fallback CPU | Tries GPU first |
| `new_gpu(ctx)` | GPU | Requires explicit context |
| `new_cpu_fallback()` | CPU only | Same API, no GPU needed |

Check the backend:

```rust
if batch.is_gpu() {
    println!("Using GPU acceleration");
} else {
    println!("Using CPU fallback");
}
```

## Supported Easings on GPU

The compute shader implements 10 core easing functions:

| GPU ID | Easing |
|--------|--------|
| 0 | `Linear` |
| 1 | `EaseInQuad` |
| 2 | `EaseOutQuad` |
| 3 | `EaseInOutQuad` |
| 4 | `EaseInCubic` |
| 5 | `EaseOutCubic` |
| 6 | `EaseInOutCubic` |
| 7 | `EaseInQuart` |
| 8 | `EaseOutQuart` |
| 9 | `EaseInOutQuart` |

Unsupported easings (e.g. `RoughEase`, `Wiggle`) are mapped to `Linear` on
the GPU. Use `new_cpu_fallback()` if you need full easing support.

## Architecture

```
┌──────────────┐     ┌───────────────┐     ┌──────────────┐
│  CPU-side    │     │  GPU Buffers  │     │ Compute      │
│  TweenData[] │ ──→ │  params[]     │ ──→ │ Shader       │
│              │     │  globals      │     │ (WGSL)       │
│              │     │               │     │              │
│  results[]   │ ←── │  results[]    │ ←── │ 256 threads/ │
│              │     │  readback[]   │     │ workgroup    │
└──────────────┘     └───────────────┘     └──────────────┘
```

1. CPU pushes `GpuTweenData` structs (32 bytes each) into the param buffer
2. Global elapsed time is uploaded as a uniform
3. A single compute dispatch runs one thread per tween
4. Results are copied to a readback buffer and mapped to CPU memory

## Performance Notes

- **Workgroup size**: 256 threads (optimal for most GPUs)
- **Buffer resizing**: Automatic power-of-2 growth
- **Overhead**: GPU has fixed dispatch overhead (~0.1ms). Benefits appear
  at 1,000+ tweens
- **Best for**: Particle systems, large UI lists, batch color animations
- **Not ideal for**: Small numbers of tweens (<100) where CPU overhead is lower

## Integration with Bevy

For Bevy render pipelines, the GPU result buffer can be used directly without
CPU read-back. Access the internal buffer for integration:

```rust
// Future API (when Bevy integration is ready):
// let buffer = batch.result_buffer();
// Pass buffer handle to a Bevy render pipeline
```
