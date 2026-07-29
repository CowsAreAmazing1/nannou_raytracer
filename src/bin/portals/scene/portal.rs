use bytemuck::{Pod, Zeroable};
use nannou::glam::{Mat4, Quat, Vec3};

use crate::{
    scene::primitive::ellipse::Ellipse,
    util::{WORLD_UP, quat_to},
};

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
    pub fn new(position: Vec3, rotation: Quat, radius_a: f32, radius_b: f32) -> Self {
        let ellipse = Ellipse::new(
            position.to_array(),
            (rotation * WORLD_UP).to_array(),
            radius_a,
            radius_b,
            0.1,
            [1.0; 3],
            [0.0, 0.0, 0.0],
        );

        let mut portal = Self {
            ellipse,
            transformation_matrix: Mat4::IDENTITY.to_cols_array(),
            inverse_transformation_matrix: Mat4::IDENTITY.to_cols_array(),
        };

        portal.update_transform(position, rotation);
        portal
    }

    pub fn from_ellipse(ellipse: Ellipse) -> Self {
        let mut portal = Self {
            ellipse,
            transformation_matrix: Mat4::IDENTITY.to_cols_array(),
            inverse_transformation_matrix: Mat4::IDENTITY.to_cols_array(),
        };

        portal.transform_from_self();
        portal
    }

    fn transform_from_self(&mut self) {
        let position = Vec3::from(self.ellipse.center);
        let rotation = quat_to(Vec3::from(self.ellipse.normal).normalize());

        let transform = Mat4::from_rotation_translation(rotation, position);

        self.transformation_matrix = transform.to_cols_array();
        self.inverse_transformation_matrix = transform.inverse().to_cols_array();
    }

    fn update_transform(&mut self, position: Vec3, rotation: Quat) {
        self.ellipse.center = position.to_array();
        self.ellipse.normal = (rotation * WORLD_UP).to_array();

        let transform = Mat4::from_rotation_translation(rotation, position);

        self.transformation_matrix = transform.to_cols_array();
        self.inverse_transformation_matrix = transform.inverse().to_cols_array();
    }

    fn apply_flip(&mut self) {
        let current_transform = Mat4::from_cols_array(&self.transformation_matrix);
        let flip_matrix = Mat4::from_rotation_z(std::f32::consts::PI);
        let flipped_transform = current_transform * flip_matrix;

        self.transformation_matrix = flipped_transform.to_cols_array();
        self.inverse_transformation_matrix = flipped_transform.inverse().to_cols_array();
    }

    #[allow(dead_code)]
    pub fn set_position(&mut self, position: Vec3) {
        let current_rotation = self.get_rotation();
        self.update_transform(position, current_rotation);
    }

    #[allow(dead_code)]
    pub fn set_rotation(&mut self, rotation: Quat) {
        let current_position = Vec3::from(self.ellipse.center);
        self.update_transform(current_position, rotation);
    }

    pub fn animate(&mut self, position: Vec3, rotation: Quat) {
        self.update_transform(position, rotation);
    }

    fn get_rotation(&self) -> Quat {
        let current_normal = Vec3::from(self.ellipse.normal);
        quat_to(current_normal)
    }

    pub fn position(&self) -> Vec3 {
        Vec3::from(self.ellipse.center)
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable, Default)]
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
