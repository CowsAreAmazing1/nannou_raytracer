use bytemuck::{Pod, Zeroable};
use nannou::{
    color::{Srgb, WHITE},
    glam::{Mat4, Quat, Vec3},
};

use crate::scene::primitive::ellipse::{Ellipse, EllipseRaw};

#[derive(Debug, Clone, Copy)]
pub struct Portal {
    pub ellipse: Ellipse,
    pub transformation_matrix: Mat4,
    pub inverse_transformation_matrix: Mat4,
}

impl Default for Portal {
    fn default() -> Self {
        Self {
            ellipse: Ellipse::default(),
            transformation_matrix: Mat4::IDENTITY,
            inverse_transformation_matrix: Mat4::IDENTITY,
        }
    }
}

impl Portal {
    pub fn new(position: Vec3, rotation: Quat, radius_a: f32, radius_b: f32) -> Self {
        let ellipse = Ellipse::new(
            position,
            rotation,
            radius_a,
            radius_b,
            0.1,
            WHITE.into_format(),
            Srgb::default(),
        );

        let mut portal = Self {
            ellipse,
            transformation_matrix: Mat4::IDENTITY,
            inverse_transformation_matrix: Mat4::IDENTITY,
        };

        // portal.update_transform(position, rotation);
        portal.transform_from_self();
        portal
    }

    pub fn from_ellipse(ellipse: Ellipse) -> Self {
        let mut portal = Self {
            ellipse,
            transformation_matrix: Mat4::IDENTITY,
            inverse_transformation_matrix: Mat4::IDENTITY,
        };

        portal.transform_from_self();
        portal
    }

    pub fn transform_from_self(&mut self) {
        let transform = Mat4::from_rotation_translation(self.ellipse.quat, self.ellipse.center);

        self.transformation_matrix = transform;
        self.inverse_transformation_matrix = transform.inverse();
    }

    fn update_transform(&mut self, position: Vec3, rotation: Quat) {
        self.ellipse.center = position;
        self.ellipse.quat = rotation;

        let transform = Mat4::from_rotation_translation(rotation, position);

        self.transformation_matrix = transform;
        self.inverse_transformation_matrix = transform.inverse();
    }

    fn apply_flip(&mut self) {
        let flip_matrix = Mat4::from_rotation_z(std::f32::consts::PI);
        let flipped_transform = self.transformation_matrix * flip_matrix;

        self.transformation_matrix = flipped_transform;
        self.inverse_transformation_matrix = flipped_transform.inverse();
    }

    #[allow(dead_code)]
    pub fn set_position(&mut self, position: Vec3) {
        let current_rotation = self.ellipse.quat;
        self.update_transform(position, current_rotation);
    }

    #[allow(dead_code)]
    pub fn set_rotation(&mut self, rotation: Quat) {
        self.update_transform(self.position(), rotation);
    }

    pub fn animate(&mut self, position: Vec3, rotation: Quat) {
        self.update_transform(position, rotation);
    }

    pub fn normal(&self) -> Vec3 {
        self.ellipse.normal()
    }

    pub fn position(&self) -> Vec3 {
        self.ellipse.center
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct PortalRaw {
    pub ellipse: EllipseRaw,
    pub transformation_matrix: [f32; 16],
    pub inverse_transformation_matrix: [f32; 16],
}

impl Default for PortalRaw {
    fn default() -> Self {
        Self {
            ellipse: EllipseRaw::default(),
            transformation_matrix: Mat4::IDENTITY.to_cols_array(),
            inverse_transformation_matrix: Mat4::IDENTITY.to_cols_array(),
        }
    }
}

impl From<Portal> for PortalRaw {
    fn from(portal: Portal) -> Self {
        Self {
            ellipse: portal.ellipse.into(),
            transformation_matrix: portal.transformation_matrix.to_cols_array(),
            inverse_transformation_matrix: portal.inverse_transformation_matrix.to_cols_array(),
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct PortalPair {
    pub portal_a: Portal,
    pub portal_b: Portal,
}

impl PortalPair {
    pub fn new(portal_a: Portal, portal_b: Portal) -> Self {
        let mut flipped_a = portal_a;
        flipped_a.apply_flip(); // Always flip portal A

        Self {
            portal_a: flipped_a,
            portal_b,
        }
    }

    #[allow(dead_code)]
    pub fn animate_portal_a(&mut self, position: Vec3, rotation: Quat) {
        self.portal_a.animate(position, rotation);
        self.portal_a.apply_flip();
    }

    #[allow(dead_code)]
    pub fn animate_portal_b(&mut self, position: Vec3, rotation: Quat) {
        self.portal_b.animate(position, rotation);
    }

    pub fn animate_both(&mut self, pos_a: Vec3, rot_a: Quat, pos_b: Vec3, rot_b: Quat) {
        self.portal_a.animate(pos_a, rot_a);
        self.portal_a.apply_flip();

        self.portal_b.animate(pos_b, rot_b);
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable, Default)]
pub struct PortalPairRaw {
    pub portal_a: PortalRaw,
    pub portal_b: PortalRaw,
}

impl From<PortalPair> for PortalPairRaw {
    fn from(portal_pair: PortalPair) -> Self {
        Self {
            portal_a: portal_pair.portal_a.into(),
            portal_b: portal_pair.portal_b.into(),
        }
    }
}
