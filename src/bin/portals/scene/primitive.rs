use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct Plane {
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
            _padding4: 0.0,
        }
    }
}

impl Plane {
    pub fn new(point: [f32; 3], normal: [f32; 3], color: [f32; 3]) -> Self {
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
            _padding4: 0.0,
        }
    }

    pub fn new_finite(
        point: [f32; 3],
        normal: [f32; 3],
        color: [f32; 3],
        width: f32,
        height: f32,
    ) -> Self {
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
