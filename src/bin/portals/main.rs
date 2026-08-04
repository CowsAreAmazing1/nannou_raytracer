use std::f32::consts::FRAC_PI_2;

use nannou::prelude::*;
use nannou_egui::Egui;

use crate::{
    camera::Camera,
    cpu_raytracer::{DebugRayEmitter, check_camera_portal_teleport},
    gpu::{GpuState, Uniform},
    scene::Scene,
    util::quat_to,
};

mod camera;
mod cpu_raytracer;
mod gpu;
mod scene;
mod ui;
mod util;

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

    debug_ray_emitters: Vec<DebugRayEmitter>,

    ui: Egui,

    bp_u: f32,
    bp_v: f32,
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
        .raw_event(raw_event)
        .build()
        .unwrap();

    let window = app.window(window_id).unwrap();
    let device = window.device();

    let scenes = scene::data::create_scenes();

    let state = GpuState::new(device);

    let ui = Egui::from_window(&window);

    Model {
        window_id,
        state,
        current_scene: 1,
        scenes,
        camera: Camera::new(),
        mouse_locked: false,

        debug_ray_emitters: Vec::new(),

        ui,

        bp_u: 0.0,
        bp_v: 0.0,
    }
}

fn key_pressed(app: &App, model: &mut Model, key: Key) {
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
            // If mouse_locked is true, show cursor is false
            app.window(model.window_id)
                .unwrap()
                .set_cursor_visible(model.mouse_locked);

            model.mouse_locked = !model.mouse_locked;
            println!(
                "Mouse lock: {}",
                if model.mouse_locked { "ON" } else { "OFF" }
            );
        }
        Key::R => {
            model.add_debug_ray_emitter();
        }
        Key::C => {
            model.debug_ray_emitters = Vec::new();
        }
        _ => {}
    }
}

fn mouse_pressed(app: &App, model: &mut Model, _button: MouseButton) {
    if model.mouse_locked {
        // model.mouse_locked = true;

        let window = app.window(model.window_id).unwrap();
        let _ = window.set_cursor_grab(true);
        window.set_cursor_visible(false);

        println!("Mouse locked");
    }
}

fn mouse_moved(app: &App, model: &mut Model, pos: Point2) {
    if model.mouse_locked {
        // Update camera immediately when mouse moves
        // `pos` is mouse position relative to the center of the screen. (documented where?)
        // This is exactly how much the mouse has moved since the previous frame,
        // as it was reset to the center then

        let window = app.window(model.window_id).unwrap();
        let res = window.rect().wh() * 0.5;

        model.camera.yaw += pos.x * model.camera.sensitivity;
        model.camera.pitch = (model.camera.pitch + pos.y * model.camera.sensitivity)
            .clamp(-FRAC_PI_2 + 0.01, FRAC_PI_2 - 0.01);

        window.set_cursor_position_points(res.x, res.y).unwrap();
    }
}

fn raw_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    model.ui.handle_raw_event(event);
}

fn update(app: &App, model: &mut Model, update: Update) {
    model.update_ui();

    let dt = update.since_last.as_secs_f32();

    let old_position = model.camera.position;

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
        // println!("FOV: {:.2}", model.camera.fov_multiplier);
    }
    if app.keys.down.contains(&Key::Minus) {
        model.camera.fov_multiplier = (model.camera.fov_multiplier - 0.01).max(0.1);
        // println!("FOV: {:.2}", model.camera.fov_multiplier);
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
    if model.current_scene == 4 {
        let scene = &mut model.scenes[model.current_scene];

        if scene.data.portal_pair_count > 0 {
            // Oscillating portals
            let base_pos_a = scene.data.portal_pairs[0].portal_a.position();
            let base_pos_b = scene.data.portal_pairs[0].portal_b.position();

            let rot_a = (0.0, (time * 0.2).sin(), FRAC_PI_2);
            // let rot_b = Quat::from_rotation_y((-time * 0.3).sin())
            // * quat_to(-Vec3::X);

            scene.data.portal_pairs[0].animate_both(base_pos_a, rot_a, base_pos_b, (0.0, 0.0, 0.0));
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

            let rot_a = (FRAC_PI_2, rotation_speed, 0.0);
            let rot_b = (0.0, -rotation_speed * 0.7, FRAC_PI_2);

            scene.data.portal_pairs[1].animate_both(pos_a, rot_a, pos_b, rot_b);
        }
    } else if model.current_scene == 5 {
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

            let rot_a = quat_to(vel_a_norm).to_euler(nannou::glam::EulerRot::XYZ);
            let rot_b = quat_to(Vec3::X).to_euler(nannou::glam::EulerRot::XYZ);

            // scene.data.portal_pairs[0].animate_both(pos_a, rot_a, pos_b, rot_b);
            scene.data.portal_pairs[0].animate_both(pos_a, rot_a, a, rot_b);
        }
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let window = app.window(model.window_id).unwrap();
    let device = window.device();
    let queue = window.queue();

    let (w, h) = window.inner_size_pixels();

    // Prepare the current scene for the GPU. This is probably pretty expensive
    let scene_data = &model.scenes[model.current_scene];
    let raw_data = Scene::to_raw(scene_data);

    let u = model.bp_u;
    let v = model.bp_v;
    let base_perp = vec3(u.sin() * v.cos(), u.sin() * v.sin(), u.cos());
    let uniform = Uniform::build(
        w as f32,
        h as f32,
        app.time,
        model.current_scene,
        &model.camera,
        base_perp,
    );

    // Upload to the GPU
    model.state.write_uniform(queue, uniform);
    model.state.write_scene_data(queue, raw_data);

    model.state.render(device, queue, &frame);

    // // Draw debug rays
    let draw = app.draw();

    // Include the scale factor in the screen size
    let screen_size = vec2(w as f32, h as f32) * window.scale_factor();
    model.draw_debug_ray(&draw, screen_size);
    draw.to_frame(app, &frame).unwrap();

    // Draw ui on top of everything else
    model.ui.draw_to_frame(&frame).unwrap();
}
