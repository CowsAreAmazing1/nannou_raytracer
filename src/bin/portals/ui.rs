use std::f32::consts::PI;

use nannou::glam::{Quat, Vec3};
use nannou_egui::egui::{self, Slider, Ui};

use crate::{Model, camera::Camera, scene::primitive::Plane};

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
        let labels = ["x".to_string(), "y".to_string(), "z".to_string()];

        ui.collapsing("Point", |ui| {
            for (label, p) in labels.iter().zip(self.point.iter_mut()) {
                ui.horizontal(|ui| {
                    ui.label(label);
                    ui.add(Slider::new(p, -10.0..=10.0).text(label));
                });
            }
        });
        ui.collapsing("Normal", |ui| {
            // // TODO: hard to keep the normal vec normalized. Idealy use fancy 3D rotation
            // for (label, n) in labels.iter().zip(self.normal.iter_mut()) {
            //     ui.horizontal(|ui| {
            //         ui.label(label);
            //         ui.add(Slider::new(n, -10.0..=10.0).text(label));
            //     });
            // }

            // Use a quaternion maybe?
            let mut a = 0.0;
            let mut b = 0.0;
            let mut c = 0.0;

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

            let quat = Quat::from_euler(nannou::glam::EulerRot::XYZ, a, b, c);
            self.normal = (quat * Vec3::Y).to_array();
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
