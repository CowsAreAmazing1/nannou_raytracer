

use nannou::prelude::*;
use bytemuck::{Pod, Zeroable};


fn main() {
    nannou::app(model).update(update).run();
}



// Scene Objects
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct Plane {
    point: [f32; 3],
    _padding1: f32,
    normal: [f32; 3],
    _padding2: f32,
    color: [f32; 3],
    _padding3: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct Ellipse {
    center: [f32; 3],
    _padding1: f32,
    normal: [f32; 3],
    _padding2: f32,
    radius_a: f32,
    radius_b: f32,
    inner_radius_a: f32,
    inner_radius_b: f32,
    color: [f32; 3],
    _padding3: f32,
}

impl Default for Plane {
    fn default() -> Self {
        Self {
            point: [0.0; 3],
            _padding1: 0.0,
            normal: [0.0, 1.0, 0.0],
            _padding2: 0.0,
            color: [0.0; 3],
            _padding3: 0.0,
        }
    }
}

impl Default for Ellipse {
    fn default() -> Self {
        Self {
            center: [0.0; 3],
            _padding1: 0.0,
            normal: [0.0, 1.0, 0.0],
            _padding2: 0.0,
            radius_a: 0.0,
            radius_b: 0.0,
            inner_radius_a: 0.0,
            inner_radius_b: 0.0,
            color: [0.0; 3],
            _padding3: 0.0,
        }
    }
}


const MAX_PLANES: usize = 4;
const MAX_ELLIPSES: usize = 8;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct SceneData {
    plane_count: u32,
    ellipse_count: u32,
    _padding1: u32,
    _padding2: u32,
    planes: [Plane; MAX_PLANES],
    ellipses: [Ellipse; MAX_ELLIPSES],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct Uniforms {
    resolution: [f32; 2],
    time: f32,
    scene_id: u32,
    camera_pos: [f32; 3],
    _padding2: f32,
    // camera_dir: [f32; 3],
    // _padding3: f32,
}





struct Model {
    #[allow(dead_code)]
    window_id: WindowId,
    state: GpuState,
    current_scene: u32,
    scenes: Vec<SceneData>,
}

struct GpuState {
    render_pipeline: wgpu::RenderPipeline,

    uniform_buffer: wgpu::Buffer,
    scene_buffer: wgpu::Buffer,

    uniform_bind_group: wgpu::BindGroup,
}

impl Model {
    fn create_scenes() -> Vec<SceneData> {
        let mut scenes = Vec::new();

        let mut scene1 = SceneData {
            plane_count: 1,
            ellipse_count: 4,
            _padding1: 0,
            _padding2: 0,
            planes: [Plane::default(); MAX_PLANES],
            ellipses: [Ellipse::default(); MAX_ELLIPSES],
        };

        scene1.planes[0] = Plane {
            point: [0.0, -2.0, 0.0],
            _padding1: 0.0,
            normal: [0.0, 1.0, 0.0],
            _padding2: 0.0,
            color: [0.2, 0.0, 0.0],
            _padding3: 0.0,
        };

        let e_a = 2.0;
        let e_b = 2.5;
        let rim_thickness = 0.2;

        scene1.ellipses[0] = Ellipse {
            center: [0.0, 1.8, -4.0],
            _padding1: 0.0,
            normal: [0.0, -0.5, 1.0],
            _padding2: 0.0,
            radius_a: e_a,
            radius_b: e_b,
            inner_radius_a: e_a - rim_thickness,
            inner_radius_b: e_b - rim_thickness,
            color: [0.7, 0.4, 0.0],
            _padding3: 0.0,
        };
        scene1.ellipses[1] = Ellipse {
            center: scene1.ellipses[0].center,
            _padding1: 0.0,
            normal: scene1.ellipses[0].normal,
            _padding2: 0.0,
            radius_a: scene1.ellipses[0].inner_radius_a,
            radius_b: scene1.ellipses[0].inner_radius_b,
            inner_radius_a: 0.0,
            inner_radius_b: 0.0,
            color: [
                scene1.ellipses[0].color[0] - 0.2,
                scene1.ellipses[0].color[1] - 0.2,
                scene1.ellipses[0].color[2] - 0.2,
            ],
            _padding3: 0.0,
        };

        scenes.push(scene1);

        scenes
    }

    fn switch_scene(&mut self, scene_id: u32) {
        if scene_id < self.scenes.len() as u32 {
            self.current_scene = scene_id;
        }
    }

    fn animate_scene(&mut self, time: f32) {
        match self.current_scene {
            0 => {
                // Animate ellipse scene
                if self.scenes[0].ellipse_count > 0 {
                    let rotation = time * 0.5;
                    self.scenes[0].ellipses[0].normal = [
                        rotation.sin() * 0.5,
                        -0.5,
                        rotation.cos(),
                    ];
                    self.scenes[0].ellipses[1].normal = [
                        rotation.sin() * 0.5,
                        -0.5,
                        rotation.cos(),
                    ];
                }
            }
            _ => {}
        }
    }
}




fn model(app: &App) -> Model {
    let window_id = app.new_window().view(view).key_pressed(key_pressed).build().unwrap();
    let window = app.window(window_id).unwrap();
    let device = window.device();

    let scenes = Model::create_scenes();

    // Create uniform buffer
    let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Uniform Buffer"),
        size: std::mem::size_of::<Uniforms>() as u64,
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
            scene_buffer,
            uniform_bind_group,
        },
        current_scene: 0,
        scenes,
    }
}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    match key {
        Key::Key1 => {
            model.switch_scene(0);
            println!("Switched to Scene 1: Ellipse Showcase");
        },
        _ => {}
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    // Animate the current scene
    model.animate_scene(app.time);
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
        scene_id: model.current_scene,
        camera_pos,
        _padding2: 0.0,
        // camera_dir: [0.0, 0.0, -1.0],
        // _padding3: 0.0,
    };

    let scene_data = model.scenes[model.current_scene as usize];

    queue.write_buffer(&model.state.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    queue.write_buffer(&model.state.scene_buffer, 0, bytemuck::cast_slice(&[scene_data]));

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