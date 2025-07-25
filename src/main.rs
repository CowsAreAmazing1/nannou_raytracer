

use nannou::prelude::*;
use bytemuck::{Pod, Zeroable};
use std::{collections::HashSet};

mod scene;
use scene::SceneData;

mod cpu_raytracer;
use cpu_raytracer::{DebugRay, shoot_debug_ray, check_camera_portal_teleport};


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
    _padding2: f32,
    camera_dir: [f32; 3],
    fov: f32,
}

struct Camera {
    position: Vec3,
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
            yaw: -PI/2.0,
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
            (self.yaw - PI/2.0).cos(),
            0.0,
            (self.yaw - PI/2.0).sin(),
        )
    }

    fn up(&self) -> Vec3 {
        vec3(0.0, 1.0, 0.0)
    }

    fn shader_camera_right(&self) -> Vec3 {
        let camera_forward = self.forward();
        let world_up = Vec3::Y;
        camera_forward.cross(world_up).normalize()
    }

    fn shader_camera_up(&self) -> Vec3 {
        let camera_right = self.shader_camera_right();
        let camera_forward = self.forward();
        camera_right.cross(camera_forward)
    }
    
    fn world_to_screen(&self, world_pos: Vec3, screen_size: Vec2) -> Option<Vec2> {
        // Transform to camera space
        let relative_pos = world_pos - self.position;
        
        let camera_forward = self.forward();
        let camera_right = self.shader_camera_right();
        let camera_up = self.shader_camera_up();
        
        // Project onto camera plane
        let forward_dist = relative_pos.dot(camera_forward);
        
        // Check if behind camera
        if forward_dist <= 0.1 {
            return None;
        }
        
        // Project to camera's right/up plane
        let right_offset = relative_pos.dot(camera_right);
        let up_offset = relative_pos.dot(camera_up);
        
        // Perspective
        let aspect_ratio = screen_size.x / screen_size.y;
        
        // Convert to UV coordinates like the shader does
        let fov_radians = 2.0 * self.fov_multiplier.atan();
        let uv_x = (right_offset / forward_dist) / fov_radians;
        let uv_y = (up_offset    / forward_dist) / fov_radians;
        
        // Apply aspect ratio correction like shader
        let corrected_uv_x = uv_x / aspect_ratio;
        
        // Convert to screen coordinates
        let screen_x = corrected_uv_x * screen_size.x * 0.5;
        let screen_y = uv_y * screen_size.y * 0.5; // Flip Y for Nannou
        
        // Check bounds
        if screen_x.abs() > screen_size.x * 0.5 || screen_y.abs() > screen_size.y * 0.5 {
            return None;
        }
        
        Some(vec2(screen_x, screen_y))
    }

    fn world_to_screen_unbounded(&self, world_pos: Vec3, screen_size: Vec2) -> Option<Vec2> {
        let relative_pos = world_pos - self.position;
        
        let camera_forward = self.forward();
        let camera_right = self.shader_camera_right();
        let camera_up = self.shader_camera_up();
        
        let forward_dist = relative_pos.dot(camera_forward);
        
        // Check if behind camera
        if forward_dist <= 0.1 {
            return None;
        }
        
        let right_offset = relative_pos.dot(camera_right);
        let up_offset = relative_pos.dot(camera_up);
        
        let aspect_ratio = screen_size.x / screen_size.y;
        let fov_radians = 2.0 * self.fov_multiplier.atan();
        let uv_x = (right_offset / forward_dist) / fov_radians;
        let uv_y = (up_offset / forward_dist) / fov_radians;
        
        let corrected_uv_x = uv_x / aspect_ratio;
        let screen_x = corrected_uv_x * screen_size.x * 0.5;
        let screen_y = uv_y * screen_size.y * 0.5;
        
        Some(vec2(screen_x, screen_y))
    }

    fn clip_ray_to_screen(visible_point: Vec3, invisible_point: Vec3, camera: &Camera, screen_size: Vec2) -> Option<Vec2> {
        let ray_dir = (invisible_point - visible_point).normalize();
        let screen_bounds = vec2(screen_size.x * 0.5, screen_size.y * 0.5);
        
        // Sample points along the ray to find screen intersection
        for i in 1..100 {
            let t = i as f32 * 0.1;
            let test_point = visible_point + ray_dir * t;
            
            if let Some(screen_pos) = camera.world_to_screen_unbounded(test_point, screen_size) {
                // Check if we've reached screen bounds
                if screen_pos.x.abs() >= screen_bounds.x || screen_pos.y.abs() >= screen_bounds.y {
                    // Clamp to screen bounds
                    let clamped_x = screen_pos.x.clamp(-screen_bounds.x, screen_bounds.x);
                    let clamped_y = screen_pos.y.clamp(-screen_bounds.y, screen_bounds.y);
                    return Some(vec2(clamped_x, clamped_y));
                }
            }
        }
        None
    }

    fn clip_line_segment_to_screen(start: Vec3, end: Vec3, camera: &Camera, screen_size: Vec2) -> Option<(Vec2, Vec2)> {
        let screen_bounds = vec2(screen_size.x * 0.5, screen_size.y * 0.5);
        let mut clipped_points = Vec::new();
        
        // Sample points along the line segment
        for i in 0..=50 {
            let t = i as f32 / 50.0;
            let test_point = start + t * (end - start);
            
            if let Some(screen_pos) = camera.world_to_screen_unbounded(test_point, screen_size) {
                // Check if point is within screen bounds
                if screen_pos.x.abs() <= screen_bounds.x && screen_pos.y.abs() <= screen_bounds.y {
                    clipped_points.push(screen_pos);
                }
            }
        }
        
        if clipped_points.len() >= 2 {
            Some((clipped_points[0], clipped_points[clipped_points.len() - 1]))
        } else {
            None
        }
    }
}


struct GpuState {
    render_pipeline: wgpu::RenderPipeline,

    uniform_buffer: wgpu::Buffer,
    scene_buffer: wgpu::Buffer,

    uniform_bind_group: wgpu::BindGroup,
}

struct Model {
    window_id: WindowId,
    state: GpuState,
    current_scene: u32,
    scenes: Vec<SceneData>,
    camera: Camera,
    keys_pressed: HashSet<Key>,
    mouse_locked: bool,
    last_mouse_pos: Option<Vec2>,

    debug_rays: Vec<DebugRay>,
}



impl Model {
    fn switch_scene(&mut self, scene_id: u32) {
        if scene_id < self.scenes.len() as u32 {
            self.current_scene = scene_id;
        }
    }
}




fn model(app: &App) -> Model {
    let window_id = app.new_window()
        .view(view)
        .key_pressed(key_pressed)
        .key_released(key_released)
        .mouse_pressed(mouse_pressed)
        .mouse_moved(mouse_moved)
        .build().unwrap();
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
        current_scene: 3,
        scenes,
        camera: Camera::new(),
        keys_pressed: HashSet::new(),
        mouse_locked: false,
        last_mouse_pos: None,

        debug_rays: Vec::new(),
    }
}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    model.keys_pressed.insert(key);

    match key {
        Key::Key1 => {
            model.switch_scene(0);
            println!("Switched to Scene {}: {}", 1, "Ellipse Showcase");
        },
        Key::Key2 => {
            model.switch_scene(1);
            println!("Switched to Scene {}: {}", 2, "Portal Pair Setup");
        },
        Key::Key3 => {
            model.switch_scene(2);
            println!("Switched to Scene {}: {}", 3, "Single Portal Pair");
        },
        Key::Key4 => {
            model.switch_scene(3);
            println!("Switched to Scene {}: {}", 4, "Rooms");
        },
        Key::Key5 => {
            model.switch_scene(4);
            println!("Switched to Scene {}: {}", 5, "Infinite Portal");
        },
        Key::Key6 => {
            model.switch_scene(5);
            println!("Switched to Scene {}: {}", 6, "Infinite Portal");
        },
        Key::Tab => {
            model.mouse_locked = !model.mouse_locked;
            model.last_mouse_pos = None;
            println!("Mouse lock: {}", if model.mouse_locked { "ON" } else { "OFF" });
        }
        Key::R => {
            shoot_debug_ray(model);
        }
        Key::C => {
            model.debug_rays = Vec::new();
        }
        Key::Equals => {
            model.camera.fov_multiplier = (model.camera.fov_multiplier + 0.01).min(3.0);
            println!("FOV: {:.2}", model.camera.fov_multiplier);
        }
        Key::Minus => {
            model.camera.fov_multiplier = (model.camera.fov_multiplier - 0.01).max(0.1);
            println!("FOV: {:.2}", model.camera.fov_multiplier);
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
        // Update camera immediately when mouse moves
        if let Some(last_pos) = model.last_mouse_pos {
            let mouse_delta = vec2(pos.x, pos.y) - last_pos;
            model.camera.yaw += mouse_delta.x * model.camera.sensitivity;
            model.camera.pitch += mouse_delta.y * model.camera.sensitivity;
            
            model.camera.pitch = model.camera.pitch.clamp(-PI / 2.0 + 0.1, PI / 2.0 - 0.1);
        }
        model.last_mouse_pos = Some(vec2(pos.x, pos.y));
    }
}

fn update(app: &App, model: &mut Model, update: Update) {
    let dt = update.since_last.as_secs_f32();
    
    let old_position = model.camera.position;
    
    // Only handle WASD movement in update_camera
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
        let new_position = model.camera.position + movement;

        if let Some(teleported_pos) = check_camera_portal_teleport(
            &model.scenes[model.current_scene as usize],
            old_position,
            new_position,
        ) {
            model.camera.position = teleported_pos;
        } else {
            model.camera.position = new_position;
        }
    }

    animate_portals(model, app.time);
}

fn animate_portals(model: &mut Model, time: f32) {
    if model.current_scene == 7 {
        let scene = &mut model.scenes[model.current_scene as usize];
        
        if scene.portal_pair_count > 0 {
            // Oscillating portals
            let offset_a = Vec3::new((time * 0.5).sin() * 0.3, 0.0, 0.0);
            let offset_b = Vec3::new((time * 0.7).cos() * 0.2, (time * 0.3).sin() * 0.2, 0.0);
            
            let base_pos_a = Vec3::new(-1.4, 1.0, -5.0);
            let base_pos_b = Vec3::new(1.4, 1.0, -5.0);
            
            let rot_a = Quat::from_rotation_y(time * 0.2) * Quat::from_rotation_arc(Vec3::Y, Vec3::X);
            let rot_b = Quat::from_rotation_y(-time * 0.3) * Quat::from_rotation_arc(Vec3::Y, -Vec3::X);
            
            scene.portal_pairs[0].animate_both(
                base_pos_a + offset_a, rot_a,
                base_pos_b + offset_b, rot_b
            );
        }
        
        if scene.portal_pair_count > 1 {
            // Rotating second portal pair
            let rotation_speed = time * 0.8;
            let pos_a = Vec3::new(0.0, 1.0, -6.3);
            let pos_b = Vec3::new(
                1.4 + (rotation_speed * 2.0).cos() * 0.5,
                1.0 + (rotation_speed).sin() * 0.3,
                -1.0 + (rotation_speed * 1.5).sin() * 0.4
            );
            
            let rot_a = Quat::from_rotation_y(rotation_speed) * Quat::from_rotation_arc(Vec3::Y, Vec3::Z);
            let rot_b = Quat::from_rotation_y(-rotation_speed * 0.7) * 
                       Quat::from_rotation_z(PI/2.0) * 
                       Quat::from_rotation_y(-PI/2.0);
            
            scene.portal_pairs[1].animate_both(pos_a, rot_a, pos_b, rot_b);
        }
    }
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
    let screen_size = vec2(w as f32, h as f32);

    let uniforms = Uniforms {
        resolution: [w as f32, h as f32],
        time: app.time,
        scene_id: model.current_scene,
        camera_pos,
        _padding2: 0.0,
        camera_dir,
        fov: model.camera.fov_multiplier,
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

    for ray in model.debug_rays.iter() {
        for segment in &ray.segments {
            // Try to get screen positions for both points
            let start_2d = model.camera.world_to_screen(segment.start, screen_size);
            let end_2d = model.camera.world_to_screen(segment.end, screen_size);
            
            // Handle different visibility cases
            match (start_2d, end_2d) {
                // Both points visible - draw normally
                (Some(start), Some(end)) => {
                    draw.line()
                        .start(pt2(start.x, start.y))
                        .end(pt2(end.x, end.y))
                        .color(rgb(segment.color[0], segment.color[1], segment.color[2]))
                        .weight(3.0);
                }
                // Only start visible - clip to screen edge
                (Some(start), None) => {
                    if let Some(clipped_end) = Camera::clip_ray_to_screen(segment.start, segment.end, &model.camera, screen_size) {
                        draw.line()
                            .start(pt2(start.x, start.y))
                            .end(pt2(clipped_end.x, clipped_end.y))
                            .color(rgb(segment.color[0], segment.color[1], segment.color[2]))
                            .weight(3.0);
                    }
                }
                // Only end visible - clip from screen edge
                (None, Some(end)) => {
                    if let Some(clipped_start) = Camera::clip_ray_to_screen(segment.end, segment.start, &model.camera, screen_size) {
                        draw.line()
                            .start(pt2(clipped_start.x, clipped_start.y))
                            .end(pt2(end.x, end.y))
                            .color(rgb(segment.color[0], segment.color[1], segment.color[2]))
                            .weight(3.0);
                    }
                }
                // Neither visible - try to find screen intersection
                (None, None) => {
                    if let Some((clipped_start, clipped_end)) = Camera::clip_line_segment_to_screen(
                        segment.start, segment.end, &model.camera, screen_size
                    ) {
                        draw.line()
                            .start(pt2(clipped_start.x, clipped_start.y))
                            .end(pt2(clipped_end.x, clipped_end.y))
                            .color(rgb(segment.color[0], segment.color[1], segment.color[2]))
                            .weight(3.0);
                    }
                }
            }
        }

        if let Some(ref first_segment) = ray.segments.first() {
            if let Some(origin_2d) = model.camera.world_to_screen(first_segment.start, screen_size) {
                draw.ellipse()
                    .xy(pt2(origin_2d.x, origin_2d.y))
                    .radius(5.0)
                    .color(RED);
            }
        }
    }
    

    draw.to_frame(app, &frame).unwrap();
}