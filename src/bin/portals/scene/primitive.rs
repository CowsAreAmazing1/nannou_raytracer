use bytemuck::{Pod, Zeroable};
use nannou::{
    color::Srgb,
    glam::{Quat, Vec3},
};

#[derive(Debug, Clone, Copy)]
pub struct Plane {
    pub point: Vec3,
    pub quat: Quat,
    pub color: Srgb,
    pub width: f32,
    pub height: f32,
    pub is_infinite: bool,
}

impl Default for Plane {
    fn default() -> Self {
        Self {
            point: Vec3::ZERO,
            quat: Quat::IDENTITY,
            color: Srgb::default(),
            width: 1.0,
            height: 1.0,
            is_infinite: true,
        }
    }
}

impl Plane {
    pub fn new<P: Into<Vec3>, C: Into<Srgb>>(point: P, quat: Quat, color: C) -> Self {
        Self {
            point: point.into(),
            quat,
            color: color.into(),
            width: 0.0,
            height: 0.0,
            is_infinite: true,
        }
    }

    pub fn new_finite<P: Into<Vec3>, C: Into<Srgb>>(
        point: P,
        quat: Quat,
        color: C,
        width: f32,
        height: f32,
    ) -> Self {
        Self {
            point: point.into(),
            quat,
            color: color.into(),
            width,
            height,
            is_infinite: false,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct PlaneRaw {
    pub point: [f32; 3],
    _padding1: f32,
    pub normal: [f32; 3],
    _padding2: f32,
    pub color: [f32; 3],
    _padding3: f32,
    pub width: f32,
    pub height: f32,
    pub is_infinite: f32, // 1.0 for infinite, 0.0 for finite
    _padding4: f32,
}

impl From<Plane> for PlaneRaw {
    fn from(plane: Plane) -> Self {
        Self {
            point: plane.point.to_array(),
            _padding1: 0.0,
            normal: (plane.quat * Vec3::Y).to_array(),
            _padding2: 0.0,
            color: plane.color.into_components().into(),
            _padding3: 0.0,
            width: plane.width,
            height: plane.height,
            is_infinite: if plane.is_infinite { 1.0 } else { 0.01 },
            _padding4: 0.0,
        }
    }
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
            _padding4: 0.0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct Ellipse {
    pub center: [f32; 3],
    _padding1: f32,
    pub normal: [f32; 3],
    _padding2: f32,
    pub radius_a: f32,
    pub radius_b: f32,
    pub border_thickness: f32,
    _padding3: f32,
    pub color: [f32; 3],
    _padding4: f32,
    pub border_color: [f32; 3],
    _padding5: f32,
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
            _padding5: 0.0,
        }
    }
}

impl Ellipse {
    pub fn new(
        center: [f32; 3],
        normal: [f32; 3],
        radius_a: f32,
        radius_b: f32,
        border_thickness: f32,
        color: [f32; 3],
        border_color: [f32; 3],
    ) -> Self {
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
            _padding5: 0.0,
        }
    }
}
