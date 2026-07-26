use nannou::prelude::*;

mod gpu;
use gpu::GpuState;

mod scene;
use scene::Scene;

mod cpu_raytracer;
use cpu_raytracer::{DebugRay, check_camera_portal_teleport};

use crate::gpu::Uniform;

fn main() {
    nannou::app(model).update(update).run();
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
        Vec3::Y
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

    fn world_to_screen_unbounded(&self, world_pos: Vec3, screen_size: Vec2) -> Option<Vec2> {
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
        let uv_x = right_offset / forward_dist / fov_radians;
        let uv_y = up_offset / forward_dist / fov_radians;

        // Apply aspect ratio correction like shader
        let corrected_uv_x = uv_x / aspect_ratio;

        // Convert to screen coordinates
        let screen_x = corrected_uv_x * screen_size.x * 0.5;
        let screen_y = uv_y * screen_size.y * 0.5; // Flip Y for Nannou

        Some(vec2(screen_x, screen_y))
    }

    fn world_to_screen(&self, world_pos: Vec3, screen_size: Vec2) -> Option<Vec2> {
        self.world_to_screen_unbounded(world_pos, screen_size)
            .and_then(|screen_pos| {
                // Check bounds
                if screen_pos.x.abs() > screen_size.x * 0.5
                    || screen_pos.y.abs() > screen_size.y * 0.5
                {
                    return None;
                }

                Some(screen_pos)
            })
    }

    fn clip_ray_to_screen(
        visible_point: Vec3,
        invisible_point: Vec3,
        camera: &Camera,
        screen_size: Vec2,
    ) -> Option<Vec2> {
        let ray_dir = (invisible_point - visible_point).normalize();
        let screen_bounds = screen_size * 0.5;

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

    fn clip_line_segment_to_screen(
        start: Vec3,
        end: Vec3,
        camera: &Camera,
        screen_size: Vec2,
    ) -> Option<(Vec2, Vec2)> {
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

struct Model {
    window_id: WindowId,
    state: GpuState,
    current_scene: usize,
    scenes: Vec<Scene>,
    camera: Camera,
    mouse_locked: bool,
    last_mouse_pos: Option<Vec2>,

    debug_rays: Vec<DebugRay>,
}

impl Model {
    fn switch_scene(&mut self, scene_id: usize) {
        if scene_id < self.scenes.len() {
            println!(
                "Switched to Scene {}: {}",
                scene_id + 1,
                self.scenes[scene_id].name
            );
            self.current_scene = scene_id;
        } else {
            println!("Scene ID {} is out of bounds", scene_id);
        }
    }
}

fn model(app: &App) -> Model {
    let window_id = app
        .new_window()
        .view(view)
        .key_pressed(key_pressed)
        .mouse_pressed(mouse_pressed)
        .mouse_moved(mouse_moved)
        .build()
        .unwrap();

    let window = app.window(window_id).unwrap();
    let device = window.device();

    let scenes = scene::data::create_scenes();

    let state = GpuState::new(device);

    Model {
        window_id,
        state,
        current_scene: 3,
        scenes,
        camera: Camera::new(),
        mouse_locked: false,
        last_mouse_pos: None,

        debug_rays: Vec::new(),
    }
}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    match key {
        Key::Right => {
            if model.current_scene + 1 < model.scenes.len() {
                model.switch_scene(model.current_scene + 1);
            }
        }
        Key::Left => {
            if model.current_scene > 0 {
                model.switch_scene(model.current_scene - 1);
            }
        }
        Key::Tab => {
            model.mouse_locked = !model.mouse_locked;
            model.last_mouse_pos = None;
            println!(
                "Mouse lock: {}",
                if model.mouse_locked { "ON" } else { "OFF" }
            );
        }
        Key::R => {
            model.shoot_debug_ray();
        }
        Key::C => {
            model.debug_rays = Vec::new();
        }
        _ => {}
    }
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

    if app.keys.down.contains(&Key::W) {
        movement += model.camera.forward();
    }
    if app.keys.down.contains(&Key::A) {
        movement += model.camera.right();
    }
    if app.keys.down.contains(&Key::S) {
        movement -= model.camera.forward();
    }
    if app.keys.down.contains(&Key::D) {
        movement -= model.camera.right();
    }
    if app.keys.down.contains(&Key::Space) {
        movement += model.camera.up();
    }
    if app.keys.down.contains(&Key::LShift) {
        movement -= model.camera.up();
    }
    if app.keys.down.contains(&Key::Equals) {
        model.camera.fov_multiplier = (model.camera.fov_multiplier + 0.01).min(3.0);
        println!("FOV: {:.2}", model.camera.fov_multiplier);
    }
    if app.keys.down.contains(&Key::Minus) {
        model.camera.fov_multiplier = (model.camera.fov_multiplier - 0.01).max(0.1);
        println!("FOV: {:.2}", model.camera.fov_multiplier);
    }

    if movement.length() > 0.0 {
        movement = movement.normalize() * model.camera.speed * dt;
        let new_position = model.camera.position + movement;

        if let Some(teleported_pos) = check_camera_portal_teleport(
            &model.scenes[model.current_scene].data,
            old_position,
            new_position,
        ) {
            model.camera.position = teleported_pos;
        } else {
            model.camera.position = new_position;
        }
    }

    // let lerp = app.time % 1.0;
    // let pos = lerp * (1.0 - lerp) * Vec3::ZERO + lerp * Vec3::new(0.0, 1.0, 5.0);

    // model.camera.position = pos;

    animate_portals(model, app.time);
}

fn animate_portals(model: &mut Model, time: f32) {
    if model.current_scene == 5 {
        let scene = &mut model.scenes[model.current_scene];

        if scene.data.portal_pair_count > 0 {
            // Oscillating portals
            let base_pos_a = scene.data.portal_pairs[0].portal_a.position();
            let base_pos_b = scene.data.portal_pairs[0].portal_b.position();

            let rot_a = Quat::from_rotation_y((time * 0.2).sin())
                * Quat::from_rotation_arc(Vec3::Y, Vec3::X);
            // let rot_b = Quat::from_rotation_y((-time * 0.3).sin())
            // * Quat::from_rotation_arc(Vec3::Y, -Vec3::X);

            scene.data.portal_pairs[0].animate_both(base_pos_a, rot_a, base_pos_b, Quat::IDENTITY);
        }

        if scene.data.portal_pair_count > 1 {
            // Rotating second portal pair
            let rotation_speed = time * 0.8;
            let pos_a = Vec3::new(0.0, 1.0, -6.3);
            let pos_b = Vec3::new(
                1.4 + (rotation_speed * 2.0).cos() * 0.5,
                1.0 + (rotation_speed).sin() * 0.3,
                -1.0 + (rotation_speed * 1.5).sin() * 0.4,
            );

            let rot_a =
                Quat::from_rotation_y(rotation_speed) * Quat::from_rotation_arc(Vec3::Y, Vec3::Z);
            let rot_b = Quat::from_rotation_y(-rotation_speed * 0.7)
                * Quat::from_rotation_z(PI / 2.0)
                * Quat::from_rotation_y(-PI / 2.0);

            scene.data.portal_pairs[1].animate_both(pos_a, rot_a, pos_b, rot_b);
        }
    } else if model.current_scene == 6 {
        let scene = &mut model.scenes[model.current_scene];

        if scene.data.portal_pair_count > 0 {
            let sign = (time % 2.0 - 1.0).signum();
            let time = sign * (1.0 - time % 2.0) + 1.0;
            let time = 3.0 * time * time - 2.0 * time * time * time;

            let a = vec3(0.0, 1.0, 0.0);
            let b = vec3(-2.0, 1.0, 0.0);
            let c = vec3(-2.0, 1.0, 2.0);

            let f3 = time * time;
            let f1 = 1.0 - 2.0 * time + f3;
            let f2 = 2.0 * time - 2.0 * f3;

            let pos_a = f1 * a + f2 * b + f3 * c;
            // let pos_b = vec3(-pos_a.x, pos_a.y, pos_a.z);

            let f4 = -2.0 + 2.0 * time;
            let f5 = 2.0 - 4.0 * time;
            let f6 = 2.0 * time;

            let vel_a = f4 * a + f5 * b + f6 * c;
            // let vel_b = vec3(-vel_a.x, vel_a.y, vel_a.z);
            let vel_a_norm = vel_a.normalize();
            // let vel_b_norm = vel_b.normalize();

            let rot_a = Quat::from_rotation_arc(Vec3::Y, vel_a_norm);
            // let rot_b = Quat::from_rotation_arc(Vec3::Y, vel_b_norm);

            // scene.data.portal_pairs[0].animate_both(pos_a, rot_a, pos_b, rot_b);
            scene.data.portal_pairs[0].animate_both(
                pos_a,
                rot_a,
                a,
                Quat::from_rotation_arc(Vec3::Y, Vec3::X),
            );
        }
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let window = app.window(model.window_id).unwrap();
    let device = window.device();
    let queue = window.queue();

    // Update uniforms
    let (w, h) = window.inner_size_points();
    let screen_size = vec2(w, h);

    let scene_data = model.scenes[model.current_scene].data;

    let uniform = Uniform::build(w, h, app.time, model.current_scene, &model.camera);
    model.state.write_uniform(queue, uniform);
    model.state.write_scene_data(queue, scene_data);

    model.state.render(device, queue, &frame);

    let draw = app.draw();
    model.draw_debug_ray(&draw, screen_size);

    draw.to_frame(app, &frame).unwrap();
}
