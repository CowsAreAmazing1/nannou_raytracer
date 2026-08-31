use std::f32::consts::FRAC_PI_2;

use nannou::prelude::*;
use nannou_egui::Egui;

use crate::{
    camera::Camera,
    gpu::{GpuState, Uniform},
    scene::Scene,
    ui::DebugRayEmitter,
};

mod camera;
mod cpu_raytracer;
mod egui;
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
    max_bounces: u32,

    debug_ray_emitters: Vec<DebugRayEmitter>,

    ui: Egui,
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
        current_scene: 0,
        scenes,
        camera: Camera::new(),
        mouse_locked: false,
        max_bounces: 3,

        debug_ray_emitters: Vec::new(),

        ui,
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
            let window = app.window(model.window_id).unwrap();

            // If mouse_locked is true, show cursor is false
            window.set_cursor_visible(model.mouse_locked);

            model.mouse_locked = !model.mouse_locked;

            let res = window.rect().wh() * 0.5;
            window.set_cursor_position_points(res.x, res.y).unwrap();

            println!(
                "Mouse lock: {}",
                if model.mouse_locked { "ON" } else { "OFF" }
            );
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
    model.camera.movement(app, dt);

    if app.keys.down.contains(&Key::R) {
        model.add_debug_ray_emitter();
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let window = app.window(model.window_id).unwrap();
    let device = window.device();
    let queue = window.queue();

    let (w, h) = window.inner_size_points();

    // Prepare the current scene for the GPU. This is probably pretty expensive
    let scene_data = &model.scenes[model.current_scene];
    let raw_data = Scene::to_raw(scene_data);

    let uniform = Uniform::build(
        w,
        h,
        app.time,
        model.current_scene,
        &model.camera,
        model.max_bounces,
    );

    // Upload to the GPU
    model.state.write_uniform(queue, uniform);
    model.state.write_scene_data(queue, raw_data);

    model.state.render(device, queue, &frame);

    // // Draw debug rays
    let draw = app.draw();

    // Include the scale factor in the screen size
    let screen_size = vec2(w, h);

    model.draw_debug_ray(&draw, screen_size);
    model.draw_look_ellipse(&draw, screen_size);

    draw.to_frame(app, &frame).unwrap();

    // Draw ui on top of everything else
    model.ui.draw_to_frame(&frame).unwrap();
}
