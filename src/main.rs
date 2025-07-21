

use nannou::prelude::*;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct Uniforms {
    resolution: [f32; 2],
    time: f32,
    _padding: f32,
    camera_pos: [f32; 3],
    _padding2: f32,
    camera_dir: [f32; 3],
    _padding3: f32,
}





fn main() {
    nannou::app(model).run();
}


struct Model {
    #[allow(dead_code)]
    window_id: WindowId,
    state: GpuState,
}

struct GpuState {
    render_pipeline: wgpu::RenderPipeline,

    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
}




fn model(app: &App) -> Model {
    let window_id = app.new_window().view(view).build().unwrap();
    let window = app.window(window_id).unwrap();

    let device = window.device();

    // Create uniform buffer
    let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Uniform Buffer"),
        size: std::mem::size_of::<Uniforms>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // Create bind group layout
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Uniform Bind Group Layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });

    // Create bind group
    let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Uniform Bind Group"),
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });

    let render_shader = include_str!("render.wgsl");
    let render_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Render Shader"),
        source: wgpu::ShaderSource::Wgsl(render_shader.into()),
    });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
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
            })]
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

    Model {
        window_id,
        state: GpuState {
            render_pipeline,
            uniform_buffer,
            uniform_bind_group,
        }
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let window = app.window(model.window_id).unwrap();
    let device = window.device();
    let queue = window.queue();

    // Mouse-controlled orbital camera
    let mouse = app.mouse.position();
    let window_size = window.inner_size_points();
    
    // Normalize mouse position to -1.0 to 1.0 range
    let mouse_x = (mouse.x / (window_size.0 * 0.5)) as f32;
    let mouse_y = (mouse.y / (window_size.1 * 0.5)) as f32;
    
    // Calculate orbital angles from mouse position
    let horizontal_angle = mouse_x * std::f32::consts::PI; // Full rotation left/right
    let vertical_angle = (mouse_y * 0.5 + 0.3) * std::f32::consts::PI * 0.3; // Limited vertical range
    
    // Calculate camera position in spherical coordinates
    let camera_radius = 8.0;
    let camera_pos = [
        camera_radius * horizontal_angle.cos() * vertical_angle.cos(),
        camera_radius * vertical_angle.sin() + 1.0, // Offset up slightly
        camera_radius * horizontal_angle.sin() * vertical_angle.cos(),
    ];

    // Update uniforms
    let (w, h) = window.inner_size_pixels();
    let uniforms = Uniforms {
        resolution: [w as f32, h as f32],
        time: app.time,
        _padding: 0.0,
        camera_pos,
        _padding2: 0.0,
        camera_dir: [0.0, 0.0, -1.0], // Not used in shader anymore
        _padding3: 0.0,
    };

    queue.write_buffer(&model.state.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

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
        render_pass.set_pipeline(&model.state.render_pipeline);
        render_pass.set_bind_group(0, &model.state.uniform_bind_group, &[]);
        render_pass.draw(0..3, 0..1);
    }
    queue.submit(Some(encoder.finish()));

    let draw = app.draw();
    draw.to_frame(app, &frame).unwrap();
}