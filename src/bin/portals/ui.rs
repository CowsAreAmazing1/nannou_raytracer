use nannou_egui::egui::{self, Slider, Ui};

use crate::{Model, camera::Camera};

impl Camera {
    fn add_ui(&mut self, ui: &mut Ui) {
        let position = &mut self.position;
        ui.collapsing("Camera Position", |ui| {
            ui.add(Slider::new(&mut position.x, -10.0..=10.0));
            ui.add(Slider::new(&mut position.y, -10.0..=10.0));
            ui.add(Slider::new(&mut position.z, -10.0..=10.0));
        });

        ui.collapsing("Camera Rotation", |ui| {
            ui.add(Slider::new(&mut self.pitch, -10.0..=10.0));
            ui.add(Slider::new(&mut self.yaw, -10.0..=10.0));
        });
    }
}

impl Model {
    pub fn update_ui(&mut self) {
        let ctx = self.ui.begin_frame();
        egui::Window::new("Hi").show(&ctx, |ui| {
            if ui.add(egui::Button::new("Test")).clicked() {
                println!("hi");
            }

            ui.collapsing("Camera", |ui| {
                self.camera.add_ui(ui);
            })
        });
    }
}
