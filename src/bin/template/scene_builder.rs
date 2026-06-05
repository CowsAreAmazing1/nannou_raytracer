#![allow(dead_code)]

use crate::scene::{Ellipse, Plane, SceneData, Sphere};
use nannou::color::Rgb;

pub struct SceneBuilder {
    spheres: Vec<Sphere>,
    planes: Vec<Plane>,
    ellipses: Vec<Ellipse>,
}

impl SceneBuilder {
    pub fn new() -> Self {
        Self {
            spheres: Vec::new(),
            planes: Vec::new(),
            ellipses: Vec::new(),
        }
    }

    // Sphere builder
    pub fn sphere(self) -> SphereBuilder {
        SphereBuilder::new(self)
    }

    // Plane builder
    pub fn plane(self) -> PlaneBuilder {
        PlaneBuilder::new(self)
    }

    // Ellipse builder
    pub fn ellipse(self) -> EllipseBuilder {
        EllipseBuilder::new(self)
    }

    // Convenience methods for common objects
    pub fn ground_plane(mut self, y: f32, color: [f32; 3]) -> Self {
        let plane = Plane::new([0.0, y, 0.0], [0.0, 1.0, 0.0], color);
        self.planes.push(plane);
        self
    }

    pub fn spheres_grid(
        mut self,
        rows: usize,
        cols: usize,
        spacing: f32,
        radius: f32,
        color: [f32; 3],
    ) -> Self {
        let start_x = -(cols as f32 - 1.0) * spacing / 2.0;
        let start_z = -(rows as f32 - 1.0) * spacing / 2.0;

        for row in 0..rows {
            for col in 0..cols {
                let x = start_x + col as f32 * spacing;
                let z = start_z + row as f32 * spacing;
                let sphere = Sphere::new([x, radius, z], radius, color);
                self.spheres.push(sphere);
            }
        }
        self
    }

    // Predefined scene templates
    pub fn portal_scene() -> Self {
        let e_a = 0.6;
        let e_b = 1.0;
        let rim_thickness = 0.1;

        Self::new()
            .ground_plane(-5.0, [0.2, 0.0, 0.0])
            .ellipse()
            .at(1.5, 1.0, -4.0)
            .normal(0.0, -0.5, 1.0)
            .radii(e_a, e_b)
            .color(0.7, 0.4, 0.0)
            .border(rim_thickness, 0.0, 0.0, 0.0)
            .build()
            .ellipse()
            .at(-1.5, 1.0, -4.0)
            .normal(0.0, -0.5, 1.0)
            .radii(e_a, e_b)
            .color(0.0, 0.4, 0.7)
            .border(rim_thickness, 0.0, 0.0, 0.0)
            .build()
    }

    pub fn cornell_box() -> Self {
        Self::new()
            // Floor
            .plane()
            .at(0.0, -2.0, 0.0)
            .normal(0.0, 1.0, 0.0)
            .color(0.8, 0.8, 0.8)
            .infinite()
            .build()
            // Ceiling
            .plane()
            .at(0.0, 2.0, 0.0)
            .normal(0.0, -1.0, 0.0)
            .color(0.8, 0.8, 0.8)
            .infinite()
            .build()
            // Back wall
            .plane()
            .at(0.0, 0.0, -5.0)
            .normal(0.0, 0.0, 1.0)
            .color(0.8, 0.8, 0.8)
            .infinite()
            .build()
            // Left wall (red)
            .plane()
            .at(-2.0, 0.0, 0.0)
            .normal(1.0, 0.0, 0.0)
            .color(0.8, 0.2, 0.2)
            .infinite()
            .build()
            // Right wall (green)
            .plane()
            .at(2.0, 0.0, 0.0)
            .normal(-1.0, 0.0, 0.0)
            .color(0.2, 0.8, 0.2)
            .infinite()
            .build()
    }

    pub fn build(self) -> SceneData {
        let mut scene_data = SceneData::default();

        // Add spheres
        for sphere in self.spheres {
            scene_data.add_sphere(sphere);
        }

        // Add planes
        for plane in self.planes {
            scene_data.add_plane(plane);
        }

        // Add ellipses
        for ellipse in self.ellipses {
            scene_data.add_ellipse(ellipse);
        }

        scene_data
    }
}

// Sphere builder
pub struct SphereBuilder {
    scene_builder: SceneBuilder,
    center: [f32; 3],
    radius: f32,
    color: [f32; 3],
}

impl SphereBuilder {
    fn new(scene_builder: SceneBuilder) -> Self {
        Self {
            scene_builder,
            center: [0.0, 0.0, 0.0],
            radius: 1.0,
            color: [1.0, 1.0, 1.0],
        }
    }

    pub fn at(mut self, x: f32, y: f32, z: f32) -> Self {
        self.center = [x, y, z];
        self
    }

    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    pub fn color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.color = [r, g, b];
        self
    }

    pub fn color_word(mut self, color: Rgb<u8>) -> Self {
        self.color = [
            color.red as f32 / 255.0,
            color.green as f32 / 255.0,
            color.blue as f32 / 255.0,
        ];
        self
    }

    pub fn red(mut self) -> Self {
        self.color = [1.0, 0.0, 0.0];
        self
    }

    pub fn green(mut self) -> Self {
        self.color = [0.0, 1.0, 0.0];
        self
    }

    pub fn blue(mut self) -> Self {
        self.color = [0.0, 0.0, 1.0];
        self
    }

    pub fn white(mut self) -> Self {
        self.color = [1.0, 1.0, 1.0];
        self
    }

    pub fn black(mut self) -> Self {
        self.color = [0.0, 0.0, 0.0];
        self
    }

    pub fn build(mut self) -> SceneBuilder {
        let sphere = Sphere::new(self.center, self.radius, self.color);
        self.scene_builder.spheres.push(sphere);
        self.scene_builder
    }
}

// Plane builder
pub struct PlaneBuilder {
    scene_builder: SceneBuilder,
    point: [f32; 3],
    normal: [f32; 3],
    color: [f32; 3],
    width: f32,
    height: f32,
    is_infinite: bool,
}

impl PlaneBuilder {
    fn new(scene_builder: SceneBuilder) -> Self {
        Self {
            scene_builder,
            point: [0.0, 0.0, 0.0],
            normal: [0.0, 1.0, 0.0],
            color: [1.0, 1.0, 1.0],
            width: 1.0,
            height: 1.0,
            is_infinite: true,
        }
    }

    pub fn at(mut self, x: f32, y: f32, z: f32) -> Self {
        self.point = [x, y, z];
        self
    }

    pub fn normal(mut self, x: f32, y: f32, z: f32) -> Self {
        self.normal = [x, y, z];
        self
    }

    pub fn color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.color = [r, g, b];
        self
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self.is_infinite = false;
        self
    }

    pub fn infinite(mut self) -> Self {
        self.is_infinite = true;
        self
    }

    pub fn build(mut self) -> SceneBuilder {
        let plane = if self.is_infinite {
            Plane::new(self.point, self.normal, self.color)
        } else {
            Plane::new_finite(self.point, self.normal, self.color, self.width, self.height)
        };
        self.scene_builder.planes.push(plane);
        self.scene_builder
    }
}

// Ellipse builder
pub struct EllipseBuilder {
    scene_builder: SceneBuilder,
    center: [f32; 3],
    normal: [f32; 3],
    radius_a: f32,
    radius_b: f32,
    border_thickness: f32,
    color: [f32; 3],
    border_color: [f32; 3],
}

impl EllipseBuilder {
    fn new(scene_builder: SceneBuilder) -> Self {
        Self {
            scene_builder,
            center: [0.0, 0.0, 0.0],
            normal: [0.0, 1.0, 0.0],
            radius_a: 1.0,
            radius_b: 1.0,
            border_thickness: 0.0,
            color: [1.0, 1.0, 1.0],
            border_color: [0.0, 0.0, 0.0],
        }
    }

    pub fn at(mut self, x: f32, y: f32, z: f32) -> Self {
        self.center = [x, y, z];
        self
    }

    pub fn normal(mut self, x: f32, y: f32, z: f32) -> Self {
        self.normal = [x, y, z];
        self
    }

    pub fn radii(mut self, a: f32, b: f32) -> Self {
        self.radius_a = a;
        self.radius_b = b;
        self
    }

    pub fn color(mut self, r: f32, g: f32, b: f32) -> Self {
        self.color = [r, g, b];
        self
    }

    pub fn border(mut self, thickness: f32, r: f32, g: f32, b: f32) -> Self {
        self.border_thickness = thickness;
        self.border_color = [r, g, b];
        self
    }

    pub fn build(mut self) -> SceneBuilder {
        let ellipse = Ellipse::new(
            self.center,
            self.normal,
            self.radius_a,
            self.radius_b,
            self.border_thickness,
            self.color,
            self.border_color,
        );
        self.scene_builder.ellipses.push(ellipse);
        self.scene_builder
    }
}
