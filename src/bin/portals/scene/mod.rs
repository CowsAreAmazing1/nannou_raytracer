pub mod data;
pub mod portal;
pub mod primitive;

use bytemuck::{Pod, Zeroable};

use crate::scene::{
    portal::PortalPair,
    primitive::{Ellipse, Plane, PlaneRaw},
};

const MAX_PLANES: usize = 10;
const MAX_ELLIPSES: usize = 4;
const MAX_PORTAL_PAIRS: usize = 4;

#[derive(Default)]
pub struct SceneData {
    pub plane_count: u32,
    pub ellipse_count: u32,
    pub portal_pair_count: u32,
    pub planes: Vec<Plane>,
    pub ellipses: Vec<Ellipse>,
    pub portal_pairs: Vec<PortalPair>,
}

pub struct Scene {
    pub name: String,
    pub data: SceneData,
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
            self.data.planes.push(plane);
            self.data.plane_count += 1;
        } else {
            println!("Max plane count reached: {}", MAX_PLANES);
        }
    }

    pub fn add_ellipse(&mut self, ellipse: Ellipse) {
        if self.data.ellipse_count < MAX_ELLIPSES as u32 {
            println!("ellipses length {}", self.data.ellipses.len());
            self.data.ellipses.push(ellipse);
            self.data.ellipse_count += 1;
        } else {
            println!("Max ellipse count reached: {}", MAX_ELLIPSES);
        }
    }

    pub fn add_portal_pair(&mut self, portal_pair: PortalPair) {
        if self.data.portal_pair_count < MAX_PORTAL_PAIRS as u32 {
            self.data.portal_pairs.push(portal_pair);
            self.data.portal_pair_count += 1;
        } else {
            println!("Max portal pair count reached: {}", MAX_PORTAL_PAIRS);
        }
    }

    pub fn to_raw(&self) -> SceneDataRaw {
        let mut planes = [PlaneRaw::default(); MAX_PLANES];
        let mut ellipses = [Ellipse::default(); MAX_ELLIPSES];
        let mut portal_pairs = [PortalPair::default(); MAX_PORTAL_PAIRS];

        for (i, plane) in self.data.planes.iter().enumerate() {
            if i < MAX_PLANES {
                planes[i] = (*plane).into();
            }
        }
        for (i, ellipse) in self.data.ellipses.iter().enumerate() {
            if i < MAX_ELLIPSES {
                ellipses[i] = *ellipse;
            }
        }
        for (i, portal_pair) in self.data.portal_pairs.iter().enumerate() {
            if i < MAX_PORTAL_PAIRS {
                portal_pairs[i] = *portal_pair;
            }
        }

        SceneDataRaw {
            plane_count: self.data.plane_count,
            ellipse_count: self.data.ellipse_count,
            portal_pair_count: self.data.portal_pair_count,
            _padding1: 0,
            planes,
            ellipses,
            portal_pairs,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct SceneDataRaw {
    pub plane_count: u32,
    pub ellipse_count: u32,
    pub portal_pair_count: u32,
    _padding1: u32,
    pub planes: [PlaneRaw; MAX_PLANES],
    pub ellipses: [Ellipse; MAX_ELLIPSES],
    pub portal_pairs: [PortalPair; MAX_PORTAL_PAIRS],
}
