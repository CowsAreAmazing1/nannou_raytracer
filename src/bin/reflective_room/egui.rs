use std::f32::consts::PI;

use nannou::glam::{Quat, Vec3};
use nannou_egui::egui::{self, Align2, CollapsingHeader, Slider, Ui};

use crate::{
    Model,
    camera::Camera,
    scene::primitive::{ellipse::Ellipse, plane::Plane},
};

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
        a /= PI;
        b /= PI;
        c /= PI;

        ui.collapsing("Normal", |ui| {
            ui.horizontal(|ui| {
                ui.label("a");
                ui.add(Slider::new(&mut a, -1.0..=1.0));
            });
            ui.horizontal(|ui| {
                ui.label("b");
                ui.add(Slider::new(&mut b, -1.0..=1.0));
            });
            ui.horizontal(|ui| {
                ui.label("c");
                ui.add(Slider::new(&mut c, -1.0..=1.0));
            });

            self.quat = Quat::from_euler(nannou::glam::EulerRot::XYZ, PI * a, PI * b, PI * c);
        });
    }
}

impl Ellipse {
    fn add_ui(&mut self, ui: &mut Ui) {
        ui.collapsing("Point", |ui| {
            ui.horizontal(|ui| {
                ui.label("x");
                ui.add(Slider::new(&mut self.center.x, -10.0..=10.0));
            });
            ui.horizontal(|ui| {
                ui.label("y");
                ui.add(Slider::new(&mut self.center.y, -10.0..=10.0));
            });
            ui.horizontal(|ui| {
                ui.label("z");
                ui.add(Slider::new(&mut self.center.z, -10.0..=10.0));
            });
        });

        // let (mut a, mut b, mut c) = self.rots;
        // a /= PI;
        // b /= PI;
        // c /= PI;

        // ui.collapsing("Normal", |ui| {
        //     ui.horizontal(|ui| {
        //         ui.label("a");
        //         ui.add(Slider::new(&mut a, -1.0..=1.0));
        //     });
        //     ui.horizontal(|ui| {
        //         ui.label("b");
        //         ui.add(Slider::new(&mut b, -1.0..=1.0));
        //     });
        //     ui.horizontal(|ui| {
        //         ui.label("c");
        //         ui.add(Slider::new(&mut c, -1.0..=1.0));
        //     });

        //     self.rots = (PI * a, PI * b, PI * c);
        // });

        ui.horizontal(|ui| {
            ui.label("Radius a");
            ui.add(Slider::new(&mut self.radius_a, 0.0..=1.0));
        });
        ui.horizontal(|ui| {
            ui.label("Radius b");
            ui.add(Slider::new(&mut self.radius_b, 0.0..=1.0));
        });
    }
}

impl Model {
    pub fn update_ui(&mut self) {
        let ctx = self.ui.begin_frame();
        let text = self.scenes[self.current_scene].name.to_string();
        egui::Window::new(text)
            .anchor(Align2::LEFT_TOP, [5.0, 5.0])
            .show(&ctx, |ui| {
                ui.collapsing("Camera", |ui| {
                    self.camera.add_ui(ui);
                });

                let scene_label = format!("Scene {}", self.current_scene + 1);

                CollapsingHeader::new(&scene_label)
                    .default_open(true)
                    .show(ui, |ui| {
                        // Plane UI
                        if !self.scenes[self.current_scene].data.planes.is_empty() {
                            ui.collapsing("Planes", |ui| {
                                let planes = &mut self.scenes[self.current_scene].data.planes;
                                for (plane_idx, plane) in planes.iter_mut().enumerate() {
                                    let plane_label = format!("Plane {}", plane_idx + 1);
                                    ui.collapsing(&plane_label, |ui| {
                                        plane.add_ui(ui);
                                    });
                                }
                            });
                        }

                        // Ellipse UI
                        if !self.scenes[self.current_scene].data.ellipses.is_empty() {
                            ui.collapsing("Ellipses", |ui| {
                                let ellipses = &mut self.scenes[self.current_scene].data.ellipses;
                                for (ellipse_idx, ellipse) in ellipses.iter_mut().enumerate() {
                                    let ellipse_label = format!("Ellipse {}", ellipse_idx + 1);
                                    ui.collapsing(&ellipse_label, |ui| {
                                        ellipse.add_ui(ui);
                                    });
                                }
                            });
                        }
                    });

                ui.separator();

                if ui.add(egui::Button::new("camera to origin")).clicked() {
                    self.camera.position = Vec3::ZERO;
                }
            });
    }
}
