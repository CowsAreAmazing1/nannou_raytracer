
#![allow(dead_code)]

use bytemuck::{Pod, Zeroable};
use nannou::color::*;



// Scene Objects
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct Sphere {
    center: [f32; 3],
    radius: f32,
    color: [f32; 3],
    reflectivity: f32,
}

impl Default for Sphere {
    fn default() -> Self {
        Self {
            center: [0.0; 3],
            radius: 1.0,
            color: [0.0; 3],
            reflectivity: 0.0,
        }
    }
}

impl Sphere {
    pub fn new(center: [f32; 3], radius: f32, color: [f32; 3], reflectivity: f32) -> Self {
        Self {
            center,
            radius,
            color,
            reflectivity,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct Plane {
    point: [f32; 3],
    _padding1: f32,
    normal: [f32; 3],
    _padding2: f32,
    color: [f32; 3],
    _padding3: f32,
    width: f32,
    height: f32,
    is_infinite: f32, // 1.0 for infinite, 0.0 for finite
    reflectivity: f32,
}

impl Default for Plane {
    fn default() -> Self {
        Self {
            point: [0.0; 3],
            _padding1: 0.0,
            normal: [0.0, 1.0, 0.0],
            _padding2: 0.0,
            color: [0.0; 3],
            _padding3: 0.0,
            width: 1.0,
            height: 1.0,
            is_infinite: 1.0,
            reflectivity: 0.0,
        }
    }
}

impl Plane {
    pub fn new(point: [f32; 3], normal: [f32; 3], color: [f32; 3], reflectivity: f32) -> Self {
        Self {
            point,
            _padding1: 0.0,
            normal,
            _padding2: 0.0,
            color,
            _padding3: 0.0,
            width: 0.0,
            height: 0.0,
            is_infinite: 1.0,
            reflectivity,
        }
    }

    pub fn new_finite(point: [f32; 3], normal: [f32; 3], color: [f32; 3], width: f32, height: f32) -> Self {
        Self {
            point,
            _padding1: 0.0,
            normal,
            _padding2: 0.0,
            color,
            _padding3: 0.0,
            width,
            height,
            is_infinite: 0.0, // Mark as finite
            reflectivity: 0.0,
        }
    }
}




#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct Ellipse {
    center: [f32; 3],
    _padding1: f32,
    normal: [f32; 3],
    _padding2: f32,
    radius_a: f32,
    radius_b: f32,
    border_thickness: f32,
    _padding3: f32,
    color: [f32; 3],
    _padding4: f32,
    border_color: [f32; 3],
    reflectivity: f32,
}

impl Default for Ellipse {
    fn default() -> Self {
        Self {
            center: [0.0; 3],
            _padding1: 0.0,
            normal: [0.0, 1.0, 0.0],
            _padding2: 0.0,
            radius_a: 0.0,
            radius_b: 0.0,
            border_thickness: 0.0,
            _padding3: 0.0,
            color: [0.0; 3],
            _padding4: 0.0,
            border_color: [0.0; 3],
            reflectivity: 0.0,
        }
    }
}

impl Ellipse {
    pub fn new(center: [f32; 3], normal: [f32; 3], radius_a: f32, radius_b: f32, border_thickness: f32, color: [f32; 3], border_color: [f32; 3], reflectivity: f32) -> Self {
        Self {
            center,
            _padding1: 0.0,
            normal,
            _padding2: 0.0,
            radius_a,
            radius_b,
            border_thickness,
            _padding3: 0.0,
            color,
            _padding4: 0.0,
            border_color,
            reflectivity,
        }
    }
}


const MAX_SPHERES: usize = 10;
const MAX_PLANES: usize = 10;
const MAX_ELLIPSES: usize = 10;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct SceneData {
    pub sphere_count: u32,
    pub plane_count: u32,
    pub ellipse_count: u32,
    pub _padding: u32,
    pub spheres: [Sphere; MAX_SPHERES],
    pub planes: [Plane; MAX_PLANES],
    pub ellipses: [Ellipse; MAX_ELLIPSES],
}

impl Default for SceneData {
    fn default() -> Self {
        Self {
            plane_count: 0,
            sphere_count: 0,
            ellipse_count: 0,
            _padding: 0,
            spheres: [Sphere::default(); MAX_SPHERES],
            planes: [Plane::default(); MAX_PLANES],
            ellipses: [Ellipse::default(); MAX_ELLIPSES],
        }
    }
}

impl SceneData {
    pub fn new(sphere_count: u32, plane_count: u32, ellipse_count: u32) -> Self {
        Self {
            sphere_count,
            plane_count,
            ellipse_count,
            _padding: 0,
            spheres: [Sphere::default(); MAX_SPHERES],
            planes: [Plane::default(); MAX_PLANES],
            ellipses: [Ellipse::default(); MAX_ELLIPSES],
        }
    }

    pub fn add_sphere(&mut self, sphere: Sphere) {
        if self.sphere_count < MAX_SPHERES as u32 {
            self.spheres[self.sphere_count as usize] = sphere;
            self.sphere_count += 1;
        } else {
            println!("Max sphere count reached: {}", MAX_SPHERES);
        }
    }

    pub fn add_plane(&mut self, plane: Plane) {
        if self.plane_count < MAX_PLANES as u32 {
            self.planes[self.plane_count as usize] = plane;
            self.plane_count += 1;
        } else {
            println!("Max plane count reached: {}", MAX_PLANES);
        }
    }

    pub fn add_ellipse(&mut self, ellipse: Ellipse) {
        if self.ellipse_count < MAX_ELLIPSES as u32 {
            self.ellipses[self.ellipse_count as usize] = ellipse;
            self.ellipse_count += 1;
        } else {
            println!("Max ellipse count reached: {}", MAX_ELLIPSES);
        }
    }

    pub fn create_scenes() -> Vec<SceneData> {
        use crate::scene_builder::SceneBuilder;
        
        let mut scenes = Vec::new();

        // Scene 1: Portal scene (your original)
        let scene1 = SceneBuilder::portal_scene().build();
        scenes.push(scene1);

        // Scene 2: Cornell box with spheres
        let scene2 = SceneBuilder::new()
            .cornell_box().reflectivity(0.9).build()
            .sphere().at(-1.0, -1.0, -3.0).radius(0.8).red().reflectivity(0.0).build()
            .sphere().at(1.0, -1.0, -3.0).radius(0.8).blue().reflectivity(0.0).build()
            .sphere().at(0.0, 0.0, -2.0).radius(0.5).white().reflectivity(0.0).build()
            .build();
        scenes.push(scene2);

        // Scene 3: Grid of spheres
        let scene3 = SceneBuilder::new()
            .ground_plane(-2.0, [0.1, 0.1, 0.1], 0.0)
            .spheres_grid(3, 3, 2.0, 0.5, [0.8, 0.2, 0.2], 0.0)
            .build();
        scenes.push(scene3);

        // Scene 4: Mixed objects demo
        let scene4 = SceneBuilder::new()
            .ground_plane(-3.0, [0.3, 0.3, 0.3], 0.0)
            .sphere().at(-2.0, 0.0, -5.0).radius(1.0).color(1.0, 0.3, 0.3).build()
            .sphere().at(2.0, 0.0, -5.0).radius(1.0).color(0.3, 1.0, 0.3).build()
            .ellipse()
                .at(0.0, 1.0, -4.0)
                .normal(0.0, 0.0, 1.0)
                .radii(1.5, 0.8)
                .color(1.0, 1.0, 0.3)
                .border(0.1, 0.1, 0.1, 0.1)
                .build()
            .plane()
                .at(0.0, 0.0, -7.0)
                .normal(0.0, 0.0, 1.0)
                .color(0.2, 0.2, 0.5)
                .infinite()
                .build()
            .build();
        scenes.push(scene4);

        // Scene 5: Color tester
        let scene5 = SceneBuilder::new()
            .sphere()
                .at(0.0, 0.0, 0.0)
                .color_word(RED)
                .build()
            .sphere()
                .at(1.0, 0.0, 0.0)
                .color_word(CYAN)
                .build()
            .sphere()
                .at(2.0, 0.0, 0.0)
                .color_word(DARKSALMON)
                .build()
            .sphere()
                .at(3.0, 0.0, 0.0)
                .color_word(LIGHTSTEELBLUE)
                .build()
            .sphere()
                .at(4.0, 0.0, 0.0)
                .color_word(BLUEVIOLET)
                .build()
            .sphere()
                .at(5.0, 0.0, 0.0)
                .color_word(MEDIUMAQUAMARINE)
                .build()
            .build();
        scenes.push(scene5);

        scenes
    }
}