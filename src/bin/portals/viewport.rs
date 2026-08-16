use nannou::prelude::*;
use nannou_egui::Egui;

use crate::{
    Model,
    camera::Camera,
    gpu::{GpuState, Uniform},
    scene::{Scene, SceneDataRaw},
    ui::DebugRayEmitter,
};

pub struct Viewport {
    pub window_id: WindowId,
    pub state: GpuState,
    pub camera: Camera,
    pub mouse_locked: bool,
    pub show_portal_normals: bool,

    pub ui: Egui,
}

impl Viewport {
    pub fn new(app: &App) -> Self {
        let window_id = app
            .new_window()
            .view(crate::view)
            .key_pressed(crate::key_pressed)
            .mouse_pressed(crate::mouse_pressed)
            .mouse_moved(crate::mouse_moved)
            .raw_event(crate::raw_event)
            .build()
            .unwrap();

        let window = app.window(window_id).unwrap();
        let device = window.device();
        let state = GpuState::new(device);

        let ui = Egui::from_window(&window);

        Self {
            window_id,
            state,
            camera: Camera::new(),
            mouse_locked: false,
            show_portal_normals: false,

            ui,
        }
    }

    pub fn debug_ray_emitter(&mut self) -> DebugRayEmitter {
        let camera = &self.camera;
        DebugRayEmitter::new(camera.position, camera.directions())
    }

    pub fn view(
        &self,
        app: &App,
        model: &Model,
        frame: Frame,
        scene: &Scene,
        raw_data: SceneDataRaw,
    ) {
        let window = app.window(self.window_id).unwrap();
        let device = window.device();
        let queue = window.queue();

        let (w, h) = window.inner_size_pixels();

        let uniform = Uniform::build(
            w as f32,
            h as f32,
            app.time,
            model.current_scene,
            &self.camera,
        );

        // Upload to the GPU
        self.state.write_uniform(queue, uniform);
        self.state.write_scene_data(queue, raw_data);

        self.state.render(device, queue, &frame);

        // // Draw debug rays
        let draw = app.draw();

        // Include the scale factor in the screen size
        let screen_size = vec2(w as f32, h as f32) * window.scale_factor();

        self.draw_debug_ray(&model.debug_ray_emitters, scene, &draw, screen_size);
        self.draw_portal_normals(scene, &draw, screen_size);
        self.draw_look_ellipse(scene, &draw, screen_size);
        self.camera.draw_ring(&draw, screen_size);

        draw.to_frame(app, &frame).unwrap();

        // Add UI last
        self.ui.draw_to_frame(&frame).unwrap();
    }
}
