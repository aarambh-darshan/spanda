//! GPU compute shader backend for batch tween evaluation.
//!
//! `GpuAnimationBatch` offloads large-scale animation workloads to the GPU
//! using a wgpu compute shader.  Thousands of homogeneous `f32` tweens are
//! evaluated in a single dispatch.
//!
//! Requires the `gpu` feature.  Gracefully falls back to CPU evaluation when
//! no GPU adapter is available.
//!
//! # Example
//!
//! ```rust,ignore
//! use spanda::gpu::GpuAnimationBatch;
//! use spanda::{Tween, Easing};
//!
//! // CPU fallback — works everywhere
//! let mut batch = GpuAnimationBatch::new_cpu_fallback();
//! for i in 0..10_000 {
//!     batch.push(Tween::new(0.0_f32, 1.0).duration(1.0).build());
//! }
//! batch.tick(0.5);
//! let positions = batch.read_back();
//! assert_eq!(positions.len(), 10_000);
//! ```

use crate::easing::Easing;
use crate::traits::Update;
use crate::tween::Tween;

use std::sync::Arc;

// ── GPU Tween Data (matches WGSL struct layout) ─────────────────────────────

/// Per-tween parameters uploaded to the GPU.
///
/// Must match the WGSL `TweenParams` struct layout exactly (32 bytes).
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck_derive::Pod, bytemuck_derive::Zeroable)]
struct GpuTweenData {
    start: f32,
    end_val: f32,
    duration: f32,
    delay: f32,
    easing_id: u32,
    _pad1: u32,
    _pad2: u32,
    _pad3: u32,
}

/// Global uniforms uploaded to the GPU.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck_derive::Pod, bytemuck_derive::Zeroable)]
struct GpuGlobals {
    elapsed: f32,
    count: u32,
}

// ── Easing ID mapping ───────────────────────────────────────────────────────

fn easing_to_gpu_id(easing: &Easing) -> u32 {
    match easing {
        Easing::Linear => 0,
        Easing::EaseInQuad => 1,
        Easing::EaseOutQuad => 2,
        Easing::EaseInOutQuad => 3,
        Easing::EaseInCubic => 4,
        Easing::EaseOutCubic => 5,
        Easing::EaseInOutCubic => 6,
        Easing::EaseInQuart => 7,
        Easing::EaseOutQuart => 8,
        Easing::EaseInOutQuart => 9,
        _ => 0, // Unsupported easings fall back to Linear on GPU
    }
}

// ── GpuContext ──────────────────────────────────────────────────────────────

/// Shared GPU device and queue.
#[derive(Debug)]
pub struct GpuContext {
    /// The wgpu device.
    pub device: wgpu::Device,
    /// The wgpu queue.
    pub queue: wgpu::Queue,
}

/// Try to create a GPU context.  Returns `None` if no adapter is available.
pub fn try_create_gpu_context() -> Option<GpuContext> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
    }))?;

    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("spanda-gpu"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::downlevel_defaults(),
            memory_hints: wgpu::MemoryHints::Performance,
        },
        None,
    ))
    .ok()?;

    Some(GpuContext { device, queue })
}

// ── Backend enum ────────────────────────────────────────────────────────────

enum BatchBackend {
    Gpu {
        ctx: Arc<GpuContext>,
        pipeline: wgpu::ComputePipeline,
        param_buffer: wgpu::Buffer,
        result_buffer: wgpu::Buffer,
        readback_buffer: wgpu::Buffer,
        globals_buffer: wgpu::Buffer,
        bind_group: wgpu::BindGroup,
        capacity: usize,
    },
    Cpu {
        tweens: Vec<Tween<f32>>,
    },
}

impl core::fmt::Debug for BatchBackend {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            BatchBackend::Gpu { capacity, .. } => f.debug_struct("Gpu").field("capacity", capacity).finish(),
            BatchBackend::Cpu { tweens } => f.debug_struct("Cpu").field("count", &tweens.len()).finish(),
        }
    }
}

// ── GpuAnimationBatch ───────────────────────────────────────────────────────

/// A batch of f32 tweens evaluated in parallel.
///
/// If created with [`new_gpu`], evaluation happens on the GPU via a wgpu
/// compute shader.  If created with [`new_cpu_fallback`], the same API
/// evaluates tweens on the CPU.
#[derive(Debug)]
pub struct GpuAnimationBatch {
    backend: BatchBackend,
    tween_data: Vec<GpuTweenData>,
    results: Vec<f32>,
    elapsed: f32,
}

const INITIAL_CAPACITY: usize = 4096;
const WORKGROUP_SIZE: u32 = 256;

impl GpuAnimationBatch {
    /// Create a GPU-accelerated batch.
    ///
    /// If no GPU is available, falls back to CPU automatically.
    pub fn new_auto() -> Self {
        match try_create_gpu_context() {
            Some(ctx) => Self::new_gpu(Arc::new(ctx)),
            None => Self::new_cpu_fallback(),
        }
    }

    /// Create a GPU-accelerated batch with an explicit GPU context.
    pub fn new_gpu(ctx: Arc<GpuContext>) -> Self {
        let (pipeline, param_buffer, result_buffer, readback_buffer, globals_buffer, bind_group) =
            create_gpu_resources(&ctx.device, INITIAL_CAPACITY);

        Self {
            backend: BatchBackend::Gpu {
                ctx,
                pipeline,
                param_buffer,
                result_buffer,
                readback_buffer,
                globals_buffer,
                bind_group,
                capacity: INITIAL_CAPACITY,
            },
            tween_data: Vec::new(),
            results: Vec::new(),
            elapsed: 0.0,
        }
    }

    /// Create a CPU-only fallback batch.
    ///
    /// Uses the same API as the GPU batch but evaluates on the CPU.
    /// Useful for testing or when no GPU is available.
    pub fn new_cpu_fallback() -> Self {
        Self {
            backend: BatchBackend::Cpu {
                tweens: Vec::new(),
            },
            tween_data: Vec::new(),
            results: Vec::new(),
            elapsed: 0.0,
        }
    }

    /// Push a tween into the batch.  Returns the index.
    pub fn push(&mut self, tween: Tween<f32>) -> usize {
        let idx = self.tween_data.len();

        let data = GpuTweenData {
            start: tween.start,
            end_val: tween.end,
            duration: tween.duration,
            delay: tween.delay,
            easing_id: easing_to_gpu_id(&tween.easing),
            _pad1: 0,
            _pad2: 0,
            _pad3: 0,
        };

        self.tween_data.push(data);
        self.results.push(tween.start);

        if let BatchBackend::Cpu { tweens } = &mut self.backend {
            tweens.push(tween);
        }

        idx
    }

    /// Advance all tweens by `dt` seconds and evaluate.
    pub fn tick(&mut self, dt: f32) {
        self.elapsed += dt;
        let count = self.tween_data.len();

        if count == 0 {
            return;
        }

        match &mut self.backend {
            BatchBackend::Gpu {
                ctx,
                pipeline,
                param_buffer,
                result_buffer,
                readback_buffer,
                globals_buffer,
                bind_group,
                capacity,
            } => {
                // Resize GPU buffers if needed
                if count > *capacity {
                    let new_cap = count.next_power_of_two();
                    let (new_pipeline, new_param, new_result, new_readback, new_globals, new_bg) =
                        create_gpu_resources(&ctx.device, new_cap);
                    *pipeline = new_pipeline;
                    *param_buffer = new_param;
                    *result_buffer = new_result;
                    *readback_buffer = new_readback;
                    *globals_buffer = new_globals;
                    *bind_group = new_bg;
                    *capacity = new_cap;
                }

                // Upload tween params
                ctx.queue.write_buffer(
                    param_buffer,
                    0,
                    bytemuck::cast_slice(&self.tween_data),
                );

                // Upload globals
                let globals = GpuGlobals {
                    elapsed: self.elapsed,
                    count: count as u32,
                };
                ctx.queue.write_buffer(globals_buffer, 0, bytemuck::bytes_of(&globals));

                // Dispatch compute
                let mut encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("spanda-gpu-encoder"),
                });

                {
                    let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                        label: Some("spanda-gpu-pass"),
                        timestamp_writes: None,
                    });
                    pass.set_pipeline(pipeline);
                    pass.set_bind_group(0, Some(&*bind_group), &[]);
                    let workgroups = (count as u32).div_ceil(WORKGROUP_SIZE);
                    pass.dispatch_workgroups(workgroups, 1, 1);
                }

                // Copy results to readback buffer
                let result_size = (count * std::mem::size_of::<f32>()) as u64;
                encoder.copy_buffer_to_buffer(result_buffer, 0, readback_buffer, 0, result_size);

                ctx.queue.submit(std::iter::once(encoder.finish()));

                // Map and read back
                let buffer_slice = readback_buffer.slice(..result_size);
                let (sender, receiver) = std::sync::mpsc::channel();
                buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
                    let _ = sender.send(result);
                });
                ctx.device.poll(wgpu::Maintain::Wait);

                if let Ok(Ok(())) = receiver.recv() {
                    let data = buffer_slice.get_mapped_range();
                    let float_data: &[f32] = bytemuck::cast_slice(&data);
                    self.results.clear();
                    self.results.extend_from_slice(&float_data[..count]);
                    drop(data);
                    readback_buffer.unmap();
                }
            }
            BatchBackend::Cpu { tweens } => {
                self.results.clear();
                for tween in tweens.iter_mut() {
                    tween.update(dt);
                    self.results.push(tween.value());
                }
            }
        }
    }

    /// Read back the evaluated tween values.
    pub fn read_back(&self) -> &[f32] {
        &self.results
    }

    /// Number of tweens in the batch.
    pub fn len(&self) -> usize {
        self.tween_data.len()
    }

    /// Whether the batch is empty.
    pub fn is_empty(&self) -> bool {
        self.tween_data.is_empty()
    }

    /// Clear all tweens and results.
    pub fn clear(&mut self) {
        self.tween_data.clear();
        self.results.clear();
        self.elapsed = 0.0;
        if let BatchBackend::Cpu { tweens } = &mut self.backend {
            tweens.clear();
        }
    }

    /// Current elapsed time.
    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }

    /// Whether this batch uses GPU acceleration.
    pub fn is_gpu(&self) -> bool {
        matches!(&self.backend, BatchBackend::Gpu { .. })
    }
}

// ── GPU resource creation ───────────────────────────────────────────────────

fn create_gpu_resources(
    device: &wgpu::Device,
    capacity: usize,
) -> (
    wgpu::ComputePipeline,
    wgpu::Buffer,
    wgpu::Buffer,
    wgpu::Buffer,
    wgpu::Buffer,
    wgpu::BindGroup,
) {
    let shader_source = include_str!("gpu_tween.wgsl");
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("spanda-gpu-shader"),
        source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    });

    let param_size = (capacity * std::mem::size_of::<GpuTweenData>()) as u64;
    let result_size = (capacity * std::mem::size_of::<f32>()) as u64;
    let globals_size = std::mem::size_of::<GpuGlobals>() as u64;

    let param_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("spanda-params"),
        size: param_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let result_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("spanda-results"),
        size: result_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let readback_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("spanda-readback"),
        size: result_size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let globals_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("spanda-globals"),
        size: globals_size,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("spanda-bgl"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("spanda-pl"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("spanda-pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("spanda-bg"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: param_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: result_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: globals_buffer.as_entire_binding(),
            },
        ],
    });

    (pipeline, param_buffer, result_buffer, readback_buffer, globals_buffer, bind_group)
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_fallback_basic() {
        let mut batch = GpuAnimationBatch::new_cpu_fallback();
        assert!(batch.is_empty());
        assert!(!batch.is_gpu());

        batch.push(Tween::new(0.0_f32, 100.0).duration(1.0).build());
        batch.push(Tween::new(50.0_f32, 150.0).duration(1.0).build());
        assert_eq!(batch.len(), 2);

        batch.tick(0.5);
        let results = batch.read_back();
        assert_eq!(results.len(), 2);
        assert!((results[0] - 50.0).abs() < 1.0, "results[0]={}", results[0]);
        assert!((results[1] - 100.0).abs() < 1.0, "results[1]={}", results[1]);
    }

    #[test]
    fn cpu_fallback_completes() {
        let mut batch = GpuAnimationBatch::new_cpu_fallback();
        batch.push(Tween::new(0.0_f32, 100.0).duration(1.0).build());
        batch.tick(1.0);
        let results = batch.read_back();
        assert!((results[0] - 100.0).abs() < 1e-4);
    }

    #[test]
    fn cpu_fallback_empty() {
        let mut batch = GpuAnimationBatch::new_cpu_fallback();
        batch.tick(0.5);
        assert!(batch.read_back().is_empty());
    }

    #[test]
    fn cpu_fallback_clear() {
        let mut batch = GpuAnimationBatch::new_cpu_fallback();
        batch.push(Tween::new(0.0_f32, 100.0).duration(1.0).build());
        batch.tick(0.5);
        assert_eq!(batch.len(), 1);

        batch.clear();
        assert!(batch.is_empty());
        assert!(batch.read_back().is_empty());
        assert!((batch.elapsed()).abs() < 1e-6);
    }

    #[test]
    fn cpu_fallback_easing() {
        let mut batch = GpuAnimationBatch::new_cpu_fallback();
        batch.push(
            Tween::new(0.0_f32, 100.0)
                .duration(1.0)
                .easing(Easing::EaseInQuad)
                .build(),
        );
        batch.tick(0.5);
        let results = batch.read_back();
        // EaseInQuad at t=0.5 → 0.25, so value ≈ 25.0
        assert!((results[0] - 25.0).abs() < 1.0, "results[0]={}", results[0]);
    }

    #[test]
    fn cpu_fallback_many_tweens() {
        let mut batch = GpuAnimationBatch::new_cpu_fallback();
        for i in 0..1000 {
            batch.push(
                Tween::new(0.0_f32, i as f32)
                    .duration(1.0)
                    .build(),
            );
        }
        batch.tick(1.0);
        let results = batch.read_back();
        assert_eq!(results.len(), 1000);
        for (i, &val) in results.iter().enumerate() {
            assert!(
                (val - i as f32).abs() < 1e-2,
                "results[{i}] = {val}, expected {}",
                i
            );
        }
    }

    #[test]
    fn easing_id_mapping() {
        assert_eq!(easing_to_gpu_id(&Easing::Linear), 0);
        assert_eq!(easing_to_gpu_id(&Easing::EaseInQuad), 1);
        assert_eq!(easing_to_gpu_id(&Easing::EaseOutCubic), 5);
        assert_eq!(easing_to_gpu_id(&Easing::EaseInOutQuart), 9);
        // Unsupported easing falls back to Linear (0)
        assert_eq!(easing_to_gpu_id(&Easing::EaseInOutBounce), 0);
    }

    #[test]
    fn gpu_tween_data_layout() {
        // Verify struct is 32 bytes (WGSL alignment requirement)
        assert_eq!(std::mem::size_of::<GpuTweenData>(), 32);
    }

    #[test]
    fn gpu_globals_layout() {
        // Verify struct is 8 bytes
        assert_eq!(std::mem::size_of::<GpuGlobals>(), 8);
    }
}
