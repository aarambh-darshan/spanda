//! Integration tests for spanda v0.9.0 — GPU Batch (CPU fallback path).
//!
//! These tests exercise the CPU fallback path which works everywhere.
//! GPU-specific tests require a GPU adapter and are tested separately.

#![cfg(feature = "gpu")]

use spanda::gpu::GpuAnimationBatch;
use spanda::traits::Update;
use spanda::{Easing, Tween};

#[test]
fn gpu_batch_cpu_fallback_basic() {
    let mut batch = GpuAnimationBatch::new_cpu_fallback();
    assert!(batch.is_empty());
    assert!(!batch.is_gpu());

    batch.push(Tween::new(0.0_f32, 100.0).duration(1.0).build());
    batch.push(Tween::new(10.0_f32, 50.0).duration(1.0).build());
    assert_eq!(batch.len(), 2);

    batch.tick(0.5);
    let results = batch.read_back();
    assert_eq!(results.len(), 2);
    // Linear easing at t=0.5 → 50%
    assert!(
        (results[0] - 50.0).abs() < 1.0,
        "results[0] = {}",
        results[0]
    );
    assert!(
        (results[1] - 30.0).abs() < 1.0,
        "results[1] = {}",
        results[1]
    );
}

#[test]
fn gpu_batch_cpu_fallback_empty() {
    let mut batch = GpuAnimationBatch::new_cpu_fallback();
    batch.tick(1.0);
    assert!(batch.read_back().is_empty());
}

#[test]
fn gpu_batch_cpu_fallback_clear() {
    let mut batch = GpuAnimationBatch::new_cpu_fallback();
    batch.push(Tween::new(0.0_f32, 100.0).duration(1.0).build());
    batch.tick(0.5);
    assert!(!batch.is_empty());

    batch.clear();
    assert!(batch.is_empty());
    assert!(batch.read_back().is_empty());
}

#[test]
fn gpu_batch_cpu_matches_individual_tweens() {
    let easing = Easing::EaseOutCubic;
    let dt = 0.5_f32;

    // Individual tween
    let mut individual = Tween::new(0.0_f32, 100.0)
        .duration(1.0)
        .easing(easing.clone())
        .build();
    individual.update(dt);
    let expected = individual.value();

    // Batch
    let mut batch = GpuAnimationBatch::new_cpu_fallback();
    batch.push(
        Tween::new(0.0_f32, 100.0)
            .duration(1.0)
            .easing(easing)
            .build(),
    );
    batch.tick(dt);
    let actual = batch.read_back()[0];

    assert!(
        (actual - expected).abs() < 1e-4,
        "Batch ({actual}) should match individual ({expected})"
    );
}

#[test]
fn gpu_batch_cpu_large_batch() {
    let mut batch = GpuAnimationBatch::new_cpu_fallback();
    for i in 0..5000 {
        batch.push(Tween::new(0.0_f32, (i as f32) * 2.0).duration(1.0).build());
    }

    batch.tick(1.0);
    let results = batch.read_back();
    assert_eq!(results.len(), 5000);

    // Spot-check a few values
    assert!((results[0] - 0.0).abs() < 1e-2);
    assert!((results[100] - 200.0).abs() < 1e-2);
    assert!((results[4999] - 9998.0).abs() < 1e-2);
}
