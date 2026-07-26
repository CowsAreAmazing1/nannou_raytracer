use bytemuck::{NoUninit, Pod, Zeroable, cast_slice};
use nannou::{
    Frame,
    wgpu::{self, Device, Queue},
};

use crate::{Camera, scene::SceneData};

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct Uniform {
    resolution: [f32; 2],
    time: f32,
    scene_id: u32,
    camera_pos: [f32; 3],
    _padding2: f32,
    camera_dir: [f32; 3],
    fov: f32,
}

impl Uniform {
    pub fn build(w: f32, h: f32, time: f32, current_scene: usize, camera: &Camera) -> Self {
        let camera_pos = camera.position.to_array();
        let camera_dir = camera.forward().to_array();

        Self {
            resolution: [w, h],
            time,
            scene_id: current_scene as u32,
            camera_pos,
            _padding2: 0.0,
            camera_dir,
            fov: camera.fov_multiplier,
        }
    }
}

pub struct GpuState {
    render_pipeline: wgpu::RenderPipeline,

    uniform_buffer: wgpu::Buffer,
    scene_buffer: wgpu::Buffer,

    uniform_bind_group: wgpu::BindGroup,
}

impl GpuState {
    pub fn new(device: &Device) -> Self {
        // Create uniform buffer
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: std::mem::size_of::<Uniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let scene_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Scene Buffer"),
            size: std::mem::size_of::<SceneData>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Uniform Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // Create bind group
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniform Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: scene_buffer.as_entire_binding(),
                },
            ],
        });

        let render_shader = include_str!("render.wgsl");
        let render_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Render Shader"),
            source: wgpu::ShaderSource::Wgsl(render_shader.into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &render_module,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &render_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: Frame::TEXTURE_FORMAT,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 4,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self {
            render_pipeline,
            uniform_buffer,
            scene_buffer,
            uniform_bind_group,
        }
    }

    pub fn write_uniform<U: NoUninit>(&self, queue: &Queue, uniform: U) {
        queue.write_buffer(&self.uniform_buffer, 0, cast_slice(&[uniform]));
    }

    pub fn write_scene_data<S: NoUninit>(&self, queue: &Queue, scene_data: S) {
        queue.write_buffer(&self.scene_buffer, 0, cast_slice(&[scene_data]));
    }

    pub fn render(&self, device: &Device, queue: &Queue, frame: &Frame) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let render_pass_desc = wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: frame.texture_view(),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        };

        {
            let mut render_pass = encoder.begin_render_pass(&render_pass_desc);
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }
        queue.submit(Some(encoder.finish()));
    }
}
