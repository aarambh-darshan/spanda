//! GPU batch animation demo — benchmarks CPU fallback path.
//!
//! Run: `cargo run --example gpu_batch_demo --features gpu`

#[cfg(feature = "gpu")]
fn main() {
    use spanda::gpu::GpuAnimationBatch;
    use spanda::{Easing, Tween};

    println!("╔═══════════════════════════════════════╗");
    println!("║   spanda — GPU Batch Animation Demo    ║");
    println!("╚═══════════════════════════════════════╝");
    println!();

    let count = 10_000;
    let frames = 60;

    // ── CPU Fallback ─────────────────────────────────────────────────────
    {
        let mut batch = GpuAnimationBatch::new_cpu_fallback();
        for i in 0..count {
            let end = (i as f32 / count as f32) * 100.0;
            batch.push(
                Tween::new(0.0_f32, end)
                    .duration(1.0)
                    .easing(Easing::EaseOutCubic)
                    .build(),
            );
        }

        let start = std::time::Instant::now();
        for _ in 0..frames {
            batch.tick(1.0 / 60.0);
        }
        let elapsed = start.elapsed();

        let results = batch.read_back();
        let min = results.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = results.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let avg = results.iter().sum::<f32>() / results.len() as f32;

        println!("CPU Fallback ({count} tweens × {frames} frames):");
        println!("  Time:  {:?}", elapsed);
        println!("  Min:   {min:.4}");
        println!("  Max:   {max:.4}");
        println!("  Avg:   {avg:.4}");
        println!();
    }

    // ── GPU (Auto Detection) ─────────────────────────────────────────────
    {
        let mut batch = GpuAnimationBatch::new_auto();
        let backend = if batch.is_gpu() { "GPU" } else { "CPU (no GPU found)" };

        for i in 0..count {
            let end = (i as f32 / count as f32) * 100.0;
            batch.push(
                Tween::new(0.0_f32, end)
                    .duration(1.0)
                    .easing(Easing::EaseOutCubic)
                    .build(),
            );
        }

        let start = std::time::Instant::now();
        for _ in 0..frames {
            batch.tick(1.0 / 60.0);
        }
        let elapsed = start.elapsed();

        let results = batch.read_back();
        let min = results.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = results.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let avg = results.iter().sum::<f32>() / results.len() as f32;

        println!("Auto Backend [{backend}] ({count} tweens × {frames} frames):");
        println!("  Time:  {:?}", elapsed);
        println!("  Min:   {min:.4}");
        println!("  Max:   {max:.4}");
        println!("  Avg:   {avg:.4}");
    }

    println!();
    println!("Done ✓");
}

#[cfg(not(feature = "gpu"))]
fn main() {
    eprintln!("This example requires the `gpu` feature:");
    eprintln!("  cargo run --example gpu_batch_demo --features gpu");
}
