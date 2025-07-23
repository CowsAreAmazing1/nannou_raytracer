
use bytemuck::{Pod, Zeroable};
use nannou::prelude::*;



#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct Plane {
    pub point: [f32; 3],
    _padding1: f32,
    pub normal: [f32; 3],
    _padding2: f32,
    pub color: [f32; 3],
    _padding3: f32,
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
    pub fn new(center: [f32; 3], normal: [f32; 3], radius_a: f32, radius_b: f32, border_thickness: f32, color: [f32; 3], border_color: [f32; 3]) -> Self {
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





#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct Portal {
    pub ellipse: Ellipse,
    pub transformation_matrix: [f32; 16],
    pub inverse_transformation_matrix: [f32; 16],
}

impl Default for Portal {
    fn default() -> Self {
        Self {
            ellipse: Ellipse::default(),
            transformation_matrix: Mat4::IDENTITY.to_cols_array(),
            inverse_transformation_matrix: Mat4::IDENTITY.to_cols_array(),
        }
    }
}

impl Portal {
    #[allow(dead_code)]
    pub fn new(position: Vec3, rotation: Quat, radius_a: f32, radius_b: f32) -> Self {
        let ellipse = Ellipse::new(
            position.to_array(),
            (rotation * Vec3::Y).to_array(),
            radius_a,
            radius_b,
            0.0,
            [1.0, 1.0, 1.0],
            [0.5, 0.5, 0.5],
        );
        
        let transform_mat4 = Mat4::from_rotation_translation(rotation, position);
        let inverse_mat4 = transform_mat4.inverse();
        
        Self {
            ellipse,
            transformation_matrix: transform_mat4.to_cols_array(),
            inverse_transformation_matrix: inverse_mat4.to_cols_array(),
        }
    }

    pub fn from_ellipse(ellipse: Ellipse) -> Self {
        let position = Vec3::from(ellipse.center);
        let rotation = Quat::from_rotation_arc(Vec3::X, Vec3::from(ellipse.normal));

        let transformation_matrix = Mat4::from_rotation_translation(rotation, position);
        let inverse_transformation_matrix = transformation_matrix.inverse();

        Self {
            ellipse,
            transformation_matrix: transformation_matrix.to_cols_array(),
            inverse_transformation_matrix: inverse_transformation_matrix.to_cols_array(),
        }
    }
}





#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct PortalPair {
    pub portal_a: Portal,
    pub portal_b: Portal,
}

impl Default for PortalPair {
    fn default() -> Self {
        Self {
            portal_a: Portal::default(),
            portal_b: Portal::default(),
        }
    }
}

impl PortalPair {
    pub fn new(portal_a: Portal, portal_b: Portal) -> Self {
        let mut flipped_b = portal_b;

        // Get the original transformation matrix
        let original_transform = Mat4::from_cols_array(&portal_b.transformation_matrix);
        
        // Create a 180-degree rotation around the portal's Y-axis (up vector)
        // This flips the Z-axis (forward/backward direction) while keeping position and up
        let flip_rotation = Mat4::from_rotation_y(std::f32::consts::PI);
        
        // Apply the flip to the transformation matrix
        let flipped_transform = original_transform * flip_rotation;
        let flipped_inverse = flipped_transform.inverse();
        
        // Update only the transformation matrices, keep the ellipse unchanged
        flipped_b.transformation_matrix = flipped_transform.to_cols_array();
        flipped_b.inverse_transformation_matrix = flipped_inverse.to_cols_array();

        Self { 
            portal_a, 
            portal_b: flipped_b, 
        }
    }
}




const MAX_PLANES: usize = 4;
const MAX_ELLIPSES: usize = 4;
const MAX_PORTAL_PAIRS: usize = 4;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct SceneData {
    pub plane_count: u32,
    pub ellipse_count: u32,
    pub portal_pair_count: u32,
    _padding1: u32,
    pub planes: [Plane; MAX_PLANES],
    pub ellipses: [Ellipse; MAX_ELLIPSES],
    pub portal_pairs: [PortalPair; MAX_PORTAL_PAIRS],
}

impl Default for SceneData {
    fn default() -> Self {
        Self {
            plane_count: 0,
            ellipse_count: 0,
            portal_pair_count: 0,
            _padding1: 0,
            planes: [Plane::default(); MAX_PLANES],
            ellipses: [Ellipse::default(); MAX_ELLIPSES],
            portal_pairs: [PortalPair::default(); MAX_PORTAL_PAIRS],
        }
    }
}

impl SceneData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_plane(&mut self, plane: Plane){
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

    pub fn add_portal_pair(&mut self, portal_pair: PortalPair) {
        if self.portal_pair_count < MAX_PORTAL_PAIRS as u32 {
            self.portal_pairs[self.portal_pair_count as usize] = portal_pair;
            self.portal_pair_count += 1;
        } else {
            println!("Max portal pair count reached: {}", MAX_PORTAL_PAIRS);
        }
    }
}