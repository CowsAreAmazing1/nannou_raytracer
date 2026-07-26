use nannou::prelude::*;

use crate::{
    camera::Camera,
    cpu_raytracer::{DebugRay, check_camera_portal_teleport},
    gpu::{GpuState, Uniform},
    scene::Scene,
};

mod camera;
mod cpu_raytracer;
mod gpu;
mod scene;

fn main() {
    nannou::app(model).update(update).run();
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
