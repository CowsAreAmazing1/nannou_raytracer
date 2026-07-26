pub mod data;
pub mod portal;
pub mod primitive;

use bytemuck::{Pod, Zeroable};

use crate::scene::{
    portal::PortalPair,
    primitive::{Ellipse, Plane},
};

const MAX_PLANES: usize = 10;
const MAX_ELLIPSES: usize = 4;
const MAX_PORTAL_PAIRS: usize = 4;

pub struct Scene {
    pub name: String,
    pub data: SceneData,
}

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

impl Scene {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            data: SceneData::default(),
        }
    }

    pub fn add_plane(&mut self, plane: Plane) {
        if self.data.plane_count < MAX_PLANES as u32 {
            self.data.planes[self.data.plane_count as usize] = plane;
            self.data.plane_count += 1;
        } else {
            println!("Max plane count reached: {}", MAX_PLANES);
        }
    }

    pub fn add_ellipse(&mut self, ellipse: Ellipse) {
        if self.data.ellipse_count < MAX_ELLIPSES as u32 {
            self.data.ellipses[self.data.ellipse_count as usize] = ellipse;
            self.data.ellipse_count += 1;
        } else {
            println!("Max ellipse count reached: {}", MAX_ELLIPSES);
        }
    }

    pub fn add_portal_pair(&mut self, portal_pair: PortalPair) {
        if self.data.portal_pair_count < MAX_PORTAL_PAIRS as u32 {
            self.data.portal_pairs[self.data.portal_pair_count as usize] = portal_pair;
            self.data.portal_pair_count += 1;
        } else {
            println!("Max portal pair count reached: {}", MAX_PORTAL_PAIRS);
        }
    }
}
