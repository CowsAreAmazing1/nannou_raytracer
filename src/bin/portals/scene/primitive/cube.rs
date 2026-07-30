use bytemuck::{Pod, Zeroable};
use nannou::{color::Srgb, glam::Vec3};

use crate::{
    scene::primitive::plane::{Plane, PlaneRaw},
    util::quat_to,
};

#[derive(Default, Debug, Clone, Copy)]
pub struct Cube {
    pub center: Vec3,
    pub size: f32,
    pub color: Srgb,
}

impl Cube {
    pub fn new<P: Into<Vec3>, C: Into<Srgb>>(center: P, size: f32, color: C) -> Self {
        Self {
            center: center.into(),
            size,
            color: color.into(),
        }
    }
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, Pod, Zeroable)]
pub struct CubeRaw {
    planes: [PlaneRaw; 6],
}

impl From<Cube> for CubeRaw {
    fn from(cube: Cube) -> Self {
        let axes = [Vec3::X, Vec3::Y, Vec3::Z, -Vec3::X, -Vec3::Y, -Vec3::Z];

        let planes = std::array::from_fn(|i| {
            let axis = axes[i];
            let point = cube.center + axis * (0.5 * cube.size);
            let quat = quat_to(axis);
            let color = cube.color;
            Plane::new_finite(point, quat, color, cube.size, cube.size).into()
        });

        Self { planes }
    }
}
