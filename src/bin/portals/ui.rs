use std::f32::consts::PI;

use nannou::glam::Quat;
use nannou_egui::egui::{self, Slider, Ui};

use crate::{Model, camera::Camera, scene::primitive::plane::Plane};

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

impl Plane {
    fn add_ui(&mut self, ui: &mut Ui) {
        ui.collapsing("Point", |ui| {
            ui.horizontal(|ui| {
                ui.label("x");
                ui.add(Slider::new(&mut self.point.x, -10.0..=10.0));
            });
            ui.horizontal(|ui| {
                ui.label("y");
                ui.add(Slider::new(&mut self.point.y, -10.0..=10.0));
            });
            ui.horizontal(|ui| {
                ui.label("z");
                ui.add(Slider::new(&mut self.point.z, -10.0..=10.0));
            });
        });

        let (mut a, mut b, mut c) = self.quat.to_euler(nannou::glam::EulerRot::XYZ);

        ui.collapsing("Normal", |ui| {
            ui.horizontal(|ui| {
                ui.label("a");
                ui.add(Slider::new(&mut a, -PI..=PI));
            });
            ui.horizontal(|ui| {
                ui.label("b");
                ui.add(Slider::new(&mut b, -PI..=PI));
            });
            ui.horizontal(|ui| {
                ui.label("c");
                ui.add(Slider::new(&mut c, -PI..=PI));
            });

            self.quat = Quat::from_euler(nannou::glam::EulerRot::XYZ, a, b, c);
        });
    }
}

impl Model {
    pub fn update_ui(&mut self) {
        let ctx = self.ui.begin_frame();
        egui::Window::new("Hi").show(&ctx, |ui| {
            ui.collapsing("Camera", |ui| {
                self.camera.add_ui(ui);
            });

            let scene_label = format!("Scene {}", self.current_scene + 1);
            ui.collapsing(&scene_label, |ui| {
                let planes = &mut self.scenes[self.current_scene].data.planes;
                for (plane_idx, plane) in planes.iter_mut().enumerate() {
                    let plane_label = format!("Plane {}", plane_idx + 1);
                    ui.collapsing(&plane_label, |ui| {
                        plane.add_ui(ui);
                    });
                }
            });
        });
    }
}
