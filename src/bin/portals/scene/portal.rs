use bytemuck::{Pod, Zeroable};
use nannou::{
    color::{Srgb, WHITE},
    glam::{
        EulerRot::{self, XYZ},
        Mat4, Quat, Vec3,
    },
};
use std::f32::consts::PI;

use crate::scene::primitive::ellipse::{Ellipse, EllipseRaw};

#[derive(Debug)]
pub struct Portal {
    /// The visible part of the portal. The teleporting surface is in the ellipse's normal direction.
    pub(crate) ellipse: Ellipse,
    /// Transforms a point from the base portal to this portal
    pub transformation_matrix: Mat4,
    /// Transforms a point from this portal to the base portal
    pub inverse_transformation_matrix: Mat4,
    /// Lerps the portal to the position and rotation of the other portal, creating a doorway
    doorification: f32,
    /// Reference to this portal's partner
    partner: Option<*const Portal>,
}

impl Clone for Portal {
    fn clone(&self) -> Self {
        Self {
            ellipse: self.ellipse,
            transformation_matrix: self.transformation_matrix,
            inverse_transformation_matrix: self.inverse_transformation_matrix,
            doorification: self.doorification,
            partner: None, // Cloning a portal does not clone its partner reference
        }
    }
}

impl Default for Portal {
    fn default() -> Self {
        Self {
            ellipse: Ellipse::default(),
            transformation_matrix: Mat4::IDENTITY,
            inverse_transformation_matrix: Mat4::IDENTITY,
            doorification: 0.0,
            partner: None,
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
        let (a, b, c) = rotation;

        let ellipse = Ellipse::new(
            position,
            Quat::from_euler(XYZ, a, b, c),
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
            doorification: 0.0,
            partner: None,
        };

        portal.transform_from_self(flipped);
        portal
    }

    pub fn from_ellipse(ellipse: Ellipse, flipped: bool) -> Self {
        let mut portal = Self {
            ellipse,
            transformation_matrix: Mat4::IDENTITY,
            inverse_transformation_matrix: Mat4::IDENTITY,
            doorification: 0.0,
            partner: None,
        };

        portal.transform_from_self(flipped);
        portal
    }

    fn set_partner_ptr(&mut self, partner: *const Portal) {
        self.partner = Some(partner);
    }

    pub fn partner(&self) -> &Portal {
        self.partner
            .map(|ptr| unsafe { &*ptr })
            .expect("Lone portal!")
    }

    pub fn set_transform(&mut self, position: Vec3, quat: Quat, flipped: bool) {
        let mut rotation = Mat4::from_quat(quat);
        let translation = Mat4::from_translation(position);

        if flipped {
            rotation *= Mat4::from_rotation_y(PI);
        }

        let transform = translation * rotation;

        self.transformation_matrix = transform;
        self.inverse_transformation_matrix = transform.inverse();
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

    pub fn rotate_delta(&mut self, rot: EulerRot, dx: f32, dy: f32, dz: f32) {
        let delta_rotation = Quat::from_euler(rot, dx, dy, dz);
        let new_quat = delta_rotation * self.ellipse.quat;

        self.ellipse.quat = new_quat;
        self.transform_from_self(false);
    }

    pub fn position(&self) -> Vec3 {
        let partner = self.partner();
        self.ellipse
            .center
            .lerp(partner.ellipse.center, self.doorification * 0.99999) // avoid z fighting
    }

    pub fn rotation(&self) -> Quat {
        let partner = self.partner();
        let q1 = self.ellipse.quat();
        let q2 = partner.ellipse.quat() * Quat::from_rotation_z(PI);

        q1.slerp(q2, self.doorification)
    }

    fn doorify(&mut self, t: f32, flipped: bool) {
        self.doorification = t;

        let new_position = self.position();
        let new_rotation = self.rotation();

        self.set_transform(new_position, new_rotation, flipped);
    }

    pub fn ellipse(&self) -> Ellipse {
        let mut ellipse = self.ellipse;

        ellipse.center = self.position();
        ellipse.quat = self.rotation();

        ellipse
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
            ellipse: portal.ellipse().into(),
            transformation_matrix: portal.transformation_matrix.to_cols_array(),
            inverse_transformation_matrix: portal.inverse_transformation_matrix.to_cols_array(),
        }
    }
}

impl From<&Portal> for PortalRaw {
    fn from(portal: &Portal) -> Self {
        Self {
            ellipse: portal.ellipse().into(),
            transformation_matrix: portal.transformation_matrix.to_cols_array(),
            inverse_transformation_matrix: portal.inverse_transformation_matrix.to_cols_array(),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct PortalPair {
    pub portal_a: Box<Portal>,
    pub portal_b: Box<Portal>,
    pub doorification: f32,
}

impl PortalPair {
    pub fn new(portal_a: Portal, portal_b: Portal) -> Self {
        let mut portal_a = Box::new(portal_a);
        let mut portal_b = Box::new(portal_b);

        let portal_a_ptr: *const Portal = &*portal_a;
        let portal_b_ptr: *const Portal = &*portal_b;

        portal_a.set_partner_ptr(portal_b_ptr);
        portal_b.set_partner_ptr(portal_a_ptr);

        Self {
            portal_a,
            portal_b,
            doorification: 0.0,
        }
    }

    pub fn from_ellipses(ellipse_a: Ellipse, ellipse_b: Ellipse) -> Self {
        let portal_a = Portal::from_ellipse(ellipse_a, true);
        let portal_b = Portal::from_ellipse(ellipse_b, false);

        Self::new(portal_a, portal_b)
    }

    pub fn doorify_a_to_b(&mut self) {
        self.portal_a.doorify(self.doorification, true);
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable, Default)]
pub struct PortalPairRaw {
    pub portal_a: PortalRaw,
    pub portal_b: PortalRaw,
}

impl From<&PortalPair> for PortalPairRaw {
    fn from(portal_pair: &PortalPair) -> Self {
        Self {
            portal_a: portal_pair.portal_a.as_ref().into(),
            portal_b: portal_pair.portal_b.as_ref().into(),
        }
    }
}
