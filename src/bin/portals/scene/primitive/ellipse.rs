use bytemuck::{Pod, Zeroable};
use nannou::{
    color::Srgb,
    glam::{Quat, Vec3},
};

use crate::util::{WORLD_FORWARDS, quat_to};

#[derive(Debug, Clone, Copy)]
pub struct Ellipse {
    /// Position of the center of the ellipse
    pub center: Vec3,
    /// Quaternion, representing the orientation of the ellipse normal w.r.t. the up (Y) axis
    pub quat: Quat,
    /// Innder radius
    pub(crate) radius_a: f32,
    /// Outer radius
    pub(crate) radius_b: f32,
    border_thickness: f32,
    /// Inside color
    pub color: Srgb,
    /// Outer color
    border_color: Srgb,
}

impl Default for Ellipse {
    fn default() -> Self {
        Self {
            center: Vec3::ZERO,
            quat: quat_to(Vec3::X),
            radius_a: 0.0,
            radius_b: 1.0,
            border_thickness: 0.1,
            color: Srgb::default(),
            border_color: Srgb::default(),
        }
    }
}

impl Ellipse {
    pub fn new<P: Into<Vec3>, C: Into<Srgb>>(
        center: P,
        quat: Quat,
        radius_a: f32,
        radius_b: f32,
        border_thickness: f32,
        color: C,
        border_color: C,
    ) -> Self {
        Self {
            center: center.into(),
            quat,
            radius_a,
            radius_b,
            border_thickness,
            color: color.into(),
            border_color: border_color.into(),
        }
    }

    pub fn quat(&self) -> Quat {
        self.quat
    }

    pub fn normal(&self) -> Vec3 {
        (self.quat() * WORLD_FORWARDS).normalize()
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
    _padding5: f32,
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
            _padding5: 0.0,
        }
    }
}

impl From<Ellipse> for EllipseRaw {
    fn from(value: Ellipse) -> Self {
        Self {
            center: value.center.to_array(),
            _padding1: 0.0,
            normal: (value.quat() * Vec3::Y).normalize().to_array(),
            _padding2: 0.0,
            radius_a: value.radius_a,
            radius_b: value.radius_b,
            border_thickness: value.border_thickness,
            _padding3: 0.0,
            color: value.color.into_components().into(),
            _padding4: 0.0,
            border_color: value.border_color.into_components().into(),
            _padding5: 0.0,
        }
    }
}

impl From<&Ellipse> for EllipseRaw {
    fn from(value: &Ellipse) -> Self {
        Self {
            center: value.center.to_array(),
            _padding1: 0.0,
            normal: (value.quat() * Vec3::Y).normalize().to_array(),
            _padding2: 0.0,
            radius_a: value.radius_a,
            radius_b: value.radius_b,
            border_thickness: value.border_thickness,
            _padding3: 0.0,
            color: value.color.into_components().into(),
            _padding4: 0.0,
            border_color: value.border_color.into_components().into(),
            _padding5: 0.0,
        }
    }
}
