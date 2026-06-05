use bytemuck::{Pod, Zeroable};
use nannou::prelude::*;
use std::collections::HashSet;

mod scene;
mod scene_builder;
use scene::SceneData;

fn main() {
    nannou::app(model).update(update).run();
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct Uniforms {
    resolution: [f32; 2],
    time: f32,
    scene_id: u32,
    camera_pos: [f32; 3],
    fov: f32,
    camera_dir: [f32; 3],
    _padding: f32,
    camera_velocity: [f32; 3],
    _padding2: f32,
}

struct Camera {
    position: Vec3,
    velocity: Vec3,
    yaw: f32,
    pitch: f32,
    speed: f32,
    sensitivity: f32,
    fov_multiplier: f32,
}

impl Camera {
    fn new() -> Self {
        Self {
            position: vec3(0.0, 1.0, 0.0),
            velocity: Vec3::ZERO,
            yaw: -PI / 2.0,
            pitch: 0.0,
            speed: 5.0,
            sensitivity: 0.003,
            fov_multiplier: 1.0,
        }
    }

    fn forward(&self) -> Vec3 {
        vec3(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
    }

    fn right(&self) -> Vec3 {
        vec3(
            (self.yaw - PI / 2.0).cos(),
            0.0,
            (self.yaw - PI / 2.0).sin(),
        )
    }

    fn up(&self) -> Vec3 {
        vec3(0.0, 1.0, 0.0)
    }
}

struct Model {
    #[allow(dead_code)]
    window_id: WindowId,
    state: GpuState,
    current_scene: u32,
    scenes: Vec<SceneData>,
    camera: Camera,
    keys_pressed: HashSet<Key>,
    mouse_locked: bool,
    last_mouse_pos: Option<Vec2>,
}

struct GpuState {
    render_pipeline: wgpu::RenderPipeline,

    uniform_buffer: wgpu::Buffer,
    scene_buffer: wgpu::Buffer,

    uniform_bind_group: wgpu::BindGroup,
}

impl Model {
    fn switch_scene(&mut self, scene_id: u32) {
        if scene_id < self.scenes.len() as u32 {
            self.current_scene = scene_id;
        }
    }
}

fn model(app: &App) -> Model {
    let window_id = app
        .new_window()
        .view(view)
        .key_pressed(key_pressed)
        .key_released(key_released)
        .mouse_pressed(mouse_pressed)
        .mouse_moved(mouse_moved)
        .build()
        .unwrap();
    let window = app.window(window_id).unwrap();
    let device = window.device();

    let scenes = SceneData::create_scenes();

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
        camera: Camera::new(),
        keys_pressed: HashSet::new(),
        mouse_locked: false,
        last_mouse_pos: None,
    }
}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    model.keys_pressed.insert(key);

    match key {
        Key::Key1 => {
            model.switch_scene(0);
            println!("Switched to Scene 1");
        }
        Key::Key2 => {
            model.switch_scene(1);
            println!("Switched to Scene 2");
        }
        Key::Key3 => {
            model.switch_scene(2);
            println!("Switched to Scene 3");
        }
        Key::Key4 => {
            model.switch_scene(3);
            println!("Switched to Scene 4");
        }
        Key::Key5 => {
            model.switch_scene(4);
            println!("Switched to Scene 5");
        }
        Key::Key6 => {
            model.switch_scene(5);
            println!("Switched to Scene 6");
        }
        Key::Tab => {
            model.mouse_locked = !model.mouse_locked;
            model.last_mouse_pos = None;
            println!(
                "Mouse lock {}",
                if model.mouse_locked { "ON" } else { "OFF" }
            );
        }
        _ => {}
    }
}

fn key_released(_app: &App, model: &mut Model, key: Key) {
    model.keys_pressed.remove(&key);
}

fn mouse_pressed(app: &App, model: &mut Model, _button: MouseButton) {
    if !model.mouse_locked {
        model.mouse_locked = true;
        model.last_mouse_pos = None;

        let window = app.window(model.window_id).unwrap();
        let _ = window.set_cursor_grab(true);
        window.set_cursor_visible(false);

        println!("Mouse locked");
    }
}

fn mouse_moved(_app: &App, model: &mut Model, pos: Point2) {
    if model.mouse_locked {
        if let Some(last_pos) = model.last_mouse_pos {
            let mouse_delta = vec2(pos.x, pos.y) - last_pos;
            model.camera.yaw += mouse_delta.x * model.camera.sensitivity;
            model.camera.pitch += mouse_delta.y * model.camera.sensitivity;

            model.camera.pitch = model.camera.pitch.clamp(-PI / 2.0 + 0.1, PI / 2.0 - 0.1);
        }
        model.last_mouse_pos = Some(vec2(pos.x, pos.y));
    }
}

fn update(_app: &App, model: &mut Model, update: Update) {
    let dt = update.since_last.as_secs_f32();

    let mut movement = Vec3::ZERO;

    if model.keys_pressed.contains(&Key::W) {
        movement += model.camera.forward();
    }
    if model.keys_pressed.contains(&Key::A) {
        movement += model.camera.right();
    }
    if model.keys_pressed.contains(&Key::S) {
        movement -= model.camera.forward();
    }
    if model.keys_pressed.contains(&Key::D) {
        movement -= model.camera.right();
    }
    if model.keys_pressed.contains(&Key::Space) {
        movement += model.camera.up();
    }
    if model.keys_pressed.contains(&Key::LShift) {
        movement -= model.camera.up();
    }

    if movement.length() > 0.0 {
        movement = movement.normalize() * model.camera.speed * dt;
        model.camera.velocity += movement;
    }

    model.camera.velocity *= 0.98;
    model.camera.position += model.camera.velocity * dt;
}

fn view(app: &App, model: &Model, frame: Frame) {
    let window = app.window(model.window_id).unwrap();
    let device = window.device();
    let queue = window.queue();

    let camera_pos = [
        model.camera.position.x,
        model.camera.position.y,
        model.camera.position.z,
    ];

    let camera_forward = model.camera.forward();
    let camera_dir = [camera_forward.x, camera_forward.y, camera_forward.z];

    // Update uniforms
    let (w, h) = window.inner_size_pixels();
    let uniforms = Uniforms {
        resolution: [w as f32, h as f32],
        time: app.time,
        scene_id: model.current_scene,
        camera_pos,
        fov: model.camera.fov_multiplier,
        camera_dir,
        _padding: 0.0,
        camera_velocity: [
            model.camera.velocity.x,
            model.camera.velocity.y,
            model.camera.velocity.z,
        ],
        _padding2: 0.0,
    };

    let scene_data = model.scenes[model.current_scene as usize];

    queue.write_buffer(
        &model.state.uniform_buffer,
        0,
        bytemuck::cast_slice(&[uniforms]),
    );
    queue.write_buffer(
        &model.state.scene_buffer,
        0,
        bytemuck::cast_slice(&[scene_data]),
    );

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

    // let draw = app.draw();
    // draw.to_frame(app, &frame).unwrap();
}
