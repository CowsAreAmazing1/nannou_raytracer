use bytemuck::{Pod, Zeroable};
use nannou::{
    color::{BLACK, ORANGE, Srgb},
    glam::{Quat, Vec3},
};

use crate::util::{color_convert, quat_to, vec_to};

#[derive(Debug, Clone, Copy)]
pub struct Ellipse {
    /// Position of the center of the ellipse
    pub center: Vec3,
    /// Quaternion, representing the orientation of the ellipse normal w.r.t. the up (Y) axis
    pub quat: Quat,
    /// Innder radius
    pub radius_a: f32,
    /// Outer radius
    pub radius_b: f32,
    /// Thickness of the border
    border_thickness: f32,
    /// Inside color
    pub color: Srgb,
    /// Outer color
    border_color: Srgb,
    /// Reflectivity
    reflectivity: f32,
}

impl Default for Ellipse {
    fn default() -> Self {
        Self {
            center: Vec3::ZERO,
            quat: quat_to(Vec3::X),
            radius_a: 0.6,
            radius_b: 1.0,
            border_thickness: 0.15,
            color: color_convert(ORANGE),
            border_color: color_convert(BLACK),
            reflectivity: 0.0,
        }
    }
}

impl Ellipse {
    pub fn new<P: Into<Vec3>, C: Into<Srgb>>(center: P, quat: Quat, color: C) -> Self {
        Self {
            center: center.into(),
            quat,
            color: color.into(),
            ..Default::default()
        }
    }

    pub fn set_radii(&mut self, radius_a: f32, radius_b: f32) -> Self {
        self.radius_a = radius_a;
        self.radius_b = radius_b;
        *self
    }

    pub fn set_border<C: Into<Srgb>>(&mut self, border_thickness: f32, border_color: C) -> Self {
        self.border_thickness = border_thickness;
        self.border_color = border_color.into();
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
pub struct EllipseRaw {
    pub(crate) center: [f32; 3],
    _padding1: f32,
    pub(crate) normal: [f32; 3],
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

impl Default for EllipseRaw {
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

impl From<Ellipse> for EllipseRaw {
    fn from(value: Ellipse) -> Self {
        Self {
            center: value.center.to_array(),
            _padding1: 0.0,
            normal: value.normal().to_array(),
            _padding2: 0.0,
            radius_a: value.radius_a,
            radius_b: value.radius_b,
            border_thickness: value.border_thickness,
            _padding3: 0.0,
            color: value.color.into_components().into(),
            _padding4: 0.0,
            border_color: value.border_color.into_components().into(),
            reflectivity: value.reflectivity,
        }
    }
}

impl From<&Ellipse> for EllipseRaw {
    fn from(value: &Ellipse) -> Self {
        Self {
            center: value.center.to_array(),
            _padding1: 0.0,
            normal: value.normal().to_array(),
            _padding2: 0.0,
            radius_a: value.radius_a,
            radius_b: value.radius_b,
            border_thickness: value.border_thickness,
            _padding3: 0.0,
            color: value.color.into_components().into(),
            _padding4: 0.0,
            border_color: value.border_color.into_components().into(),
            reflectivity: value.reflectivity,
        }
    }
}
