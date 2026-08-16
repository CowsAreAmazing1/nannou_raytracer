use std::f32::consts::FRAC_PI_2;

use nannou::prelude::*;

use crate::{scene::Scene, ui::DebugRayEmitter, viewport::Viewport};

mod camera;
mod cpu_raytracer;
mod egui;
mod gpu;
mod scene;
mod ui;
mod util;
mod viewport;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    viewports: Vec<Viewport>,

    current_scene: usize,
    scenes: Vec<Scene>,
    debug_ray_emitters: Vec<DebugRayEmitter>,
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

    fn get_focused_viewport(&self, window_id: WindowId) -> Option<&Viewport> {
        self.viewports.iter().find(|v| v.window_id == window_id)
    }

    fn get_focused_viewport_mut(&mut self, window_id: WindowId) -> Option<&mut Viewport> {
        self.viewports.iter_mut().find(|v| v.window_id == window_id)
    }
}

fn model(app: &App) -> Model {
    let scenes = scene::data::create_scenes();

    let viewports = vec![Viewport::new(app), Viewport::new(app)];

    Model {
        viewports,

        current_scene: 1,
        scenes,
        debug_ray_emitters: Vec::new(),
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
            let window = app.main_window();

            if let Some(viewport) = model
                .viewports
                .iter_mut()
                .find(|v| v.window_id == window.id())
            {
                // If mouse_locked is true, show cursor is false
                window.set_cursor_visible(viewport.mouse_locked);

                viewport.mouse_locked = !viewport.mouse_locked;

                let res = window.rect().wh() * 0.5;
                window.set_cursor_position_points(res.x, res.y).unwrap();

                println!(
                    "Mouse lock for {:?}: {}",
                    window.id(),
                    if viewport.mouse_locked { "ON" } else { "OFF" }
                );
            }
        }
        Key::C => {
            model.debug_ray_emitters = Vec::new();
        }
        _ => {}
    }
}

fn mouse_pressed(app: &App, model: &mut Model, _button: MouseButton) {
    let window = app.main_window();
    if let Some(viewport) = model.get_focused_viewport(window.id())
        && viewport.mouse_locked
    {
        // model.mouse_locked = true;

        let _ = window.set_cursor_grab(true);
        window.set_cursor_visible(false);

        println!("Mouse locked");
    }
}

fn mouse_moved(app: &App, model: &mut Model, pos: Point2) {
    let window = app.main_window();

    for viewport in &mut model.viewports {
        if viewport.window_id == window.id() && viewport.mouse_locked {
            // Update camera immediately when mouse moves
            // `pos` is mouse position relative to the center of the screen. (documented where?)
            // This is exactly how much the mouse has moved since the previous frame,
            // as it was reset to the center then

            let window = app.window(viewport.window_id).unwrap();
            let res = window.rect().wh() * 0.5;

            viewport.camera.yaw += pos.x * viewport.camera.sensitivity;
            viewport.camera.pitch = (viewport.camera.pitch + pos.y * viewport.camera.sensitivity)
                .clamp(-FRAC_PI_2 + 0.01, FRAC_PI_2 - 0.01);

            window.set_cursor_position_points(res.x, res.y).unwrap();
        } else {
            viewport.mouse_locked = false;
        }
    }
}

fn raw_event(app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    let window = app.main_window();
    if let Some(viewport) = model.get_focused_viewport_mut(window.id()) {
        viewport.ui.handle_raw_event(event);
    }
}

fn update(app: &App, model: &mut Model, update: Update) {
    let dt = update.since_last.as_secs_f32();

    let mut emitters_to_add = Vec::new();

    let window = app.main_window();
    let current_scene = model.current_scene;
    let (viewports, scenes) = (&mut model.viewports, &model.scenes);
    if let Some(viewport) = viewports.iter_mut().find(|v| v.window_id == window.id()) {
        let scene_data = &scenes[current_scene].data;
        viewport.camera.movement(app, dt, scene_data);

        if app.keys.down.contains(&Key::R) {
            emitters_to_add.push(viewport.debug_ray_emitter());
        }
    }

    model.debug_ray_emitters.extend(emitters_to_add);

    let mut viewports = std::mem::take(&mut model.viewports);
    for viewport in &mut viewports {
        viewport.update_ui(model);
    }
    model.viewports = viewports;
}

fn view(app: &App, model: &Model, frame: Frame) {
    let current_scene = model.current_scene;
    let scene = &model.scenes[current_scene];
    let scene_raw = scene.to_raw();

    let viewport = model
        .viewports
        .iter()
        .find(|v| v.window_id == frame.window_id())
        .unwrap();

    viewport.view(app, model, frame, scene, scene_raw);
}
