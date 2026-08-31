use bytemuck::{Pod, Zeroable};
use nannou::{
    color::{GREEN, Srgb},
    glam::Vec3,
};

use crate::{
    scene::primitive::plane::{Plane, PlaneRaw},
    util::{color_convert, quat_to},
};

#[derive(Debug, Clone, Copy)]
pub struct Cube {
    /// Position of the center of the cube
    pub center: Vec3,
    /// Edge length
    pub size: f32,
    pub color: Srgb,
    pub reflectivity: f32,
}

impl Default for Cube {
    fn default() -> Self {
        Self {
            center: Vec3::ZERO,
            size: 1.0,
            color: color_convert(GREEN),
            reflectivity: 0.0,
        }
    }
}

impl Cube {
    pub fn new<P: Into<Vec3>, C: Into<Srgb>>(center: P, size: f32, color: C) -> Self {
        Self {
            center: center.into(),
            size,
            color: color.into(),
            ..Default::default()
        }
    }

    pub fn make_reflective(&mut self, reflectivity: f32) -> Self {
        self.reflectivity = reflectivity;
        *self
    }

    pub fn planes(&self) -> [Plane; 6] {
        let axes = [Vec3::X, Vec3::Y, Vec3::Z, -Vec3::X, -Vec3::Y, -Vec3::Z];

        std::array::from_fn(|i| {
            let axis = axes[i];
            let point = self.center + axis * (0.5 * self.size);
            let quat = quat_to(axis);

            Plane::new(point, quat, self.color)
                .make_finite(self.size, self.size)
                .make_reflective(self.reflectivity)
        })
    }

    pub fn planes_raw(&self) -> [PlaneRaw; 6] {
        let axes = [Vec3::X, Vec3::Y, Vec3::Z, -Vec3::X, -Vec3::Y, -Vec3::Z];

        std::array::from_fn(|i| {
            let axis = axes[i];
            let point = self.center + axis * (0.5 * self.size);
            let quat = quat_to(axis);

            Plane::new(point, quat, self.color)
                .make_finite(self.size, self.size)
                .make_reflective(self.reflectivity)
                .into()
        })
    }
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, Pod, Zeroable)]
pub struct CubeRaw {
    planes: [PlaneRaw; 6],
}

impl From<Cube> for CubeRaw {
    fn from(cube: Cube) -> Self {
        let planes = cube.planes_raw();

        Self { planes }
    }
}

impl From<&Cube> for CubeRaw {
    fn from(cube: &Cube) -> Self {
        let planes = cube.planes_raw();

        Self { planes }
    }
}
