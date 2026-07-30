use std::f32::consts::{FRAC_PI_2, PI};

use nannou::glam::Quat;
use nannou_egui::egui::{self, Slider, Ui};

use crate::{
    Model,
    camera::Camera,
    scene::{
        portal::Portal,
        primitive::{ellipse::Ellipse, plane::Plane},
    },
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

        let (mut a, mut b, mut c) = self.rots;
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

            self.rots = (PI * a, PI * b, PI * c);
        });

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

impl Portal {
    fn add_ui(&mut self, ui: &mut Ui) {
        // ui.collapsing("Point", |ui| {
        //     let position = &mut self.ellipse.center;
        //     ui.horizontal(|ui| {
        //         ui.label("x");
        //         ui.add(Slider::new(&mut position.x, -10.0..=10.0));
        //     });
        //     ui.horizontal(|ui| {
        //         ui.label("y");
        //         ui.add(Slider::new(&mut position.y, -10.0..=10.0));
        //     });
        //     ui.horizontal(|ui| {
        //         ui.label("z");
        //         ui.add(Slider::new(&mut position.z, -10.0..=10.0));
        //     });
        // });

        // let (mut a, mut b, mut c) = self.ellipse.quat.to_euler(nannou::glam::EulerRot::XYZ);
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

        //     self.ellipse.quat =
        //         Quat::from_euler(nannou::glam::EulerRot::XYZ, PI * a, PI * b, PI * c);
        // });

        self.ellipse.add_ui(ui);

        self.transform_from_self();
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
                // Plane UI
                ui.collapsing("Planes", |ui| {
                    let planes = &mut self.scenes[self.current_scene].data.planes;
                    for (plane_idx, plane) in planes.iter_mut().enumerate() {
                        let plane_label = format!("Plane {}", plane_idx + 1);
                        ui.collapsing(&plane_label, |ui| {
                            plane.add_ui(ui);
                        });
                    }
                });

                // Ellipse UI
                ui.collapsing("Ellipses", |ui| {
                    let ellipses = &mut self.scenes[self.current_scene].data.ellipses;
                    for (ellipse_idx, ellipse) in ellipses.iter_mut().enumerate() {
                        let ellipse_label = format!("Ellipse {}", ellipse_idx + 1);
                        ui.collapsing(&ellipse_label, |ui| {
                            ellipse.add_ui(ui);
                        });
                    }
                });

                // Portal Pair UI
                ui.collapsing("Portals", |ui| {
                    let pairs = &mut self.scenes[self.current_scene].data.portal_pairs;
                    for (pair_idx, pair) in pairs.iter_mut().enumerate() {
                        let pair_label = format!("Portal Pair {}", pair_idx + 1);
                        ui.collapsing(&pair_label, |ui| {
                            ui.collapsing("Portal A", |ui| {
                                pair.portal_a.add_ui(ui);
                            });
                            ui.collapsing("Portal B", |ui| {
                                pair.portal_b.add_ui(ui);
                            });
                        });
                    }
                });
            });

            // test portal transforms
            if self.current_scene == 0 {
                let scene = &mut self.scenes[0].data;
                scene.cubes[0].center = self.camera.position;

                let portal_pair = self.scenes[0].data.portal_pairs[0];

                // portal a
                if ui
                    .add(egui::Button::new("Apply portal A transform"))
                    .clicked()
                {
                    let transform = portal_pair.portal_a.transformation_matrix;

                    let cube = &mut self.scenes[0].data.cubes[0];
                    cube.center = transform.transform_point3(cube.center);
                }
                if ui
                    .add(egui::Button::new("Apply portal A untransform"))
                    .clicked()
                {
                    let transform = portal_pair.portal_a.inverse_transformation_matrix;

                    let cube = &mut self.scenes[0].data.cubes[0];
                    cube.center = transform.transform_point3(cube.center);
                }

                // portal b
                if ui
                    .add(egui::Button::new("Apply portal B transform"))
                    .clicked()
                {
                    let transform = portal_pair.portal_b.transformation_matrix;

                    let cube = &mut self.scenes[0].data.cubes[0];
                    cube.center = transform.transform_point3(cube.center);
                }
                if ui
                    .add(egui::Button::new("Apply portal B untransform"))
                    .clicked()
                {
                    let transform = portal_pair.portal_b.inverse_transformation_matrix;

                    let cube = &mut self.scenes[0].data.cubes[0];
                    cube.center = transform.transform_point3(cube.center);
                }
            }

            ui.separator();

            ui.add(egui::Slider::new(&mut self.bp_u, -FRAC_PI_2..=FRAC_PI_2));
            ui.add(egui::Slider::new(&mut self.bp_v, -PI..=PI));
        });
    }
}
