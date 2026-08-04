use bytemuck::{Pod, Zeroable};
use nannou::{
    color::{Srgb, WHITE},
    glam::{Mat4, Vec3},
};
use std::f32::consts::PI;

use crate::scene::primitive::ellipse::{Ellipse, EllipseRaw};

#[derive(Debug, Clone, Copy)]
pub struct Portal {
    /// The visible part of the portal. The teleporting surface is in the ellipse's normal direction.
    pub ellipse: Ellipse,
    /// Transforms a point from the base portal to this portal
    pub transformation_matrix: Mat4,
    /// Transforms a point from this portal to the base portal
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
    pub fn new(
        position: Vec3,
        rotation: (f32, f32, f32),
        radius_a: f32,
        radius_b: f32,
        flipped: bool,
    ) -> Self {
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

        portal.transform_from_self(flipped);
        portal
    }

    pub fn from_ellipse(ellipse: Ellipse, flipped: bool) -> Self {
        let mut portal = Self {
            ellipse,
            transformation_matrix: Mat4::IDENTITY,
            inverse_transformation_matrix: Mat4::IDENTITY,
        };

        portal.transform_from_self(flipped);
        portal
    }

    pub fn transform_from_self(&mut self, flipped: bool) {
        let mut rotation = Mat4::from_quat(self.ellipse.quat());
        let translation = Mat4::from_translation(self.ellipse.center);

        if flipped {
            rotation *= Mat4::from_rotation_y(PI);
        }

        let transform = translation * rotation;

        self.transformation_matrix = transform;
        self.inverse_transformation_matrix = transform.inverse();
    }

    fn update_transform(&mut self, position: Vec3, rotation: (f32, f32, f32), flipped: bool) {
        self.ellipse.center = position;
        self.ellipse.rots = rotation;

        self.transform_from_self(flipped);
    }

    pub fn animate(&mut self, position: Vec3, rotation: (f32, f32, f32), flipped: bool) {
        self.update_transform(position, rotation, flipped);
    }

    pub fn normal(&self) -> Vec3 {
        self.ellipse.normal()
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct PortalRaw {
    pub ellipse: EllipseRaw,
    /// Matrices stored in collumn major order, for wgpu
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
        Self { portal_a, portal_b }
    }

    pub fn from_ellipses(ellipse_a: Ellipse, ellipse_b: Ellipse) -> Self {
        let portal_a = Portal::from_ellipse(ellipse_a, true);
        let portal_b = Portal::from_ellipse(ellipse_b, false);

        Self { portal_a, portal_b }
    }

    pub fn animate_both(
        &mut self,
        pos_a: Vec3,
        rot_a: (f32, f32, f32),
        pos_b: Vec3,
        rot_b: (f32, f32, f32),
    ) {
        self.portal_a.animate(pos_a, rot_a, true);
        self.portal_b.animate(pos_b, rot_b, false);
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
