use std::f32::consts::PI;

use nannou::glam::{EulerRot::XYZ, Quat, Vec3};
use nannou_egui::egui::{self, Align2, CollapsingHeader, Slider, Ui};

use crate::{
    Model,
    camera::Camera,
    scene::{
        portal::{Portal, PortalPair},
        primitive::{cube::Cube, ellipse::Ellipse, plane::Plane},
    },
};

impl Camera {
    fn add_ui(&mut self, ui: &mut Ui) {
        let roll_text = if self.use_free_roll_camera {
            "free"
        } else {
            "clamped"
        };

        ui.horizontal(|ui| {
            ui.label("Use ");
            if ui.button(roll_text).clicked() {
                self.use_free_roll_camera = !self.use_free_roll_camera;
            }
            ui.label(" roll camera")
        });

        let position = &mut self.position;
        ui.collapsing("Camera Position", |ui| {
            ui.add(Slider::new(&mut position.x, -10.0..=10.0));
            ui.add(Slider::new(&mut position.y, -10.0..=10.0));
            ui.add(Slider::new(&mut position.z, -10.0..=10.0));
        });

        ui.collapsing("Camera Rotation", |ui| {
            let (x, y, z) = self.rotation.to_euler(XYZ);
            ui.label(format!("X: {}", x));
            ui.label(format!("Y: {}", y));
            ui.label(format!("Z: {}", z));
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

impl Cube {
    fn add_ui(&mut self, ui: &mut Ui) {
        ui.collapsing("Center", |ui| {
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

        ui.horizontal(|ui| {
            ui.label("Size");
            ui.add(Slider::new(&mut self.size, 0.0..=2.0));
        });
    }
}

impl Portal {
    fn add_ui(&mut self, ui: &mut Ui) {
        self.ellipse.add_ui(ui);

        self.transform_from_self();
    }
}

impl PortalPair {
    fn add_ui(&mut self, ui: &mut Ui) {
        ui.collapsing("Portal A", |ui| {
            self.portal_a.add_ui(ui);
        });
        ui.collapsing("Portal B", |ui| {
            self.portal_b.add_ui(ui);
        });

        ui.add(egui::Slider::new(&mut self.doorification, 0.0..=1.0));
        self.doorify_b_to_a();
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

                        // Cube UI
                        if !self.scenes[self.current_scene].data.cubes.is_empty() {
                            ui.collapsing("Cubes", |ui| {
                                let cubes = &mut self.scenes[self.current_scene].data.cubes;
                                for (cube_idx, cube) in cubes.iter_mut().enumerate() {
                                    let cube_label = format!("Cube {}", cube_idx + 1);
                                    ui.collapsing(&cube_label, |ui| {
                                        cube.add_ui(ui);
                                    });
                                }
                            });
                        }

                        // Portal Pair UI
                        if !self.scenes[self.current_scene].data.portal_pairs.is_empty() {
                            ui.collapsing("Portals", |ui| {
                                ui.add(egui::Checkbox::new(
                                    &mut self.show_portal_normals,
                                    "Show portal normals",
                                ));

                                let pairs = &mut self.scenes[self.current_scene].data.portal_pairs;
                                for (pair_idx, pair) in pairs.iter_mut().enumerate() {
                                    let pair_label = format!("Portal Pair {}", pair_idx + 1);
                                    ui.collapsing(&pair_label, |ui| {
                                        pair.add_ui(ui);
                                    });
                                }
                            });
                        }
                    });

                // test portal transforms
                if self.current_scene == 0 {
                    let scene_data = &mut self.scenes[0].data;
                    scene_data.cubes[0].center = self.camera.position;

                    // let portal_pair = &self.scenes[0].data.portal_pairs[0];
                    let portal_pair = &scene_data.portal_pairs[0];

                    // portal a
                    if ui
                        .add(egui::Button::new("Apply portal A transform"))
                        .clicked()
                    {
                        let transform = portal_pair.portal_a.transformation_matrix;

                        let cube = &mut scene_data.cubes[0];
                        cube.center = transform.transform_point3(cube.center);
                    }
                    if ui
                        .add(egui::Button::new("Apply portal A untransform"))
                        .clicked()
                    {
                        let transform = portal_pair.portal_a.inverse_transformation_matrix;

                        let cube = &mut scene_data.cubes[0];
                        cube.center = transform.transform_point3(cube.center);
                    }

                    // portal b
                    if ui
                        .add(egui::Button::new("Apply portal B transform"))
                        .clicked()
                    {
                        let transform = portal_pair.portal_b.transformation_matrix;

                        let cube = &mut scene_data.cubes[0];
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

                if ui.add(egui::Button::new("camera to origin")).clicked() {
                    self.camera.position = Vec3::ZERO;
                }
            });
    }
}
