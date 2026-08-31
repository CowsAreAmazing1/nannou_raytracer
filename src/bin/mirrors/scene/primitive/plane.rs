use bytemuck::{Pod, Zeroable};
use nannou::{
    color::{DARKGRAY, Srgb},
    glam::{Quat, Vec3},
};

use crate::util::{color_convert, vec_to};

#[derive(Debug, Clone, Copy)]
pub struct Plane {
    pub point: Vec3,
    pub quat: Quat,
    pub color: Srgb,
    pub width: f32,
    pub height: f32,
    pub is_infinite: bool,
    pub reflectivity: f32,
}

impl Default for Plane {
    fn default() -> Self {
        Self {
            point: Vec3::ZERO,
            quat: Quat::IDENTITY,
            color: color_convert(DARKGRAY),
            width: 1.0,
            height: 1.0,
            is_infinite: true,
            reflectivity: 0.0,
        }
    }
}

impl Plane {
    /// Creates a new infinite plane through the `point`, with the given `quat` rotation and `color`.
    pub fn new<P: Into<Vec3>, C: Into<Srgb>>(point: P, quat: Quat, color: C) -> Self {
        Self {
            point: point.into(),
            quat,
            color: color.into(),
            ..Default::default()
        }
    }

    // Sets the plane to be finite with the given `width` and `height`
    pub fn make_finite(&mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self.is_infinite = false;
        *self
    }

    pub fn make_reflective(&mut self, reflectivity: f32) -> Self {
        self.reflectivity = reflectivity;
        *self
    }

    pub fn normal(&self) -> Vec3 {
        vec_to(self.quat)
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct PlaneRaw {
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

impl Default for PlaneRaw {
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

impl From<Plane> for PlaneRaw {
    fn from(plane: Plane) -> Self {
        Self {
            point: plane.point.to_array(),
            _padding1: 0.0,
            normal: plane.normal().to_array(),
            _padding2: 0.0,
            color: plane.color.into_components().into(),
            _padding3: 0.0,
            width: plane.width,
            height: plane.height,
            is_infinite: if plane.is_infinite { 1.0 } else { 0.0 },
            reflectivity: plane.reflectivity,
        }
    }
}

impl From<&Plane> for PlaneRaw {
    fn from(plane: &Plane) -> Self {
        Self {
            point: plane.point.to_array(),
            _padding1: 0.0,
            normal: plane.normal().to_array(),
            _padding2: 0.0,
            color: plane.color.into_components().into(),
            _padding3: 0.0,
            width: plane.width,
            height: plane.height,
            is_infinite: if plane.is_infinite { 1.0 } else { 0.0 },
            reflectivity: plane.reflectivity,
        }
    }
}
