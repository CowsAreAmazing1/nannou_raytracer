pub mod data;
pub mod portal;
pub mod primitive;

use bytemuck::{Pod, Zeroable};

use crate::scene::{
    portal::{PortalPair, PortalPairRaw},
    primitive::{
        cube::{Cube, CubeRaw},
        ellipse::{Ellipse, EllipseRaw},
        plane::{Plane, PlaneRaw},
    },
};

const MAX_PLANES: usize = 10;
const MAX_ELLIPSES: usize = 4;
const MAX_PORTAL_PAIRS: usize = 4;
const MAX_CUBES: usize = 4;

#[derive(Default)]
pub struct SceneData {
    pub plane_count: u32,
    pub ellipse_count: u32,
    pub portal_pair_count: u32,
    pub cube_count: u32,
    pub planes: Vec<Plane>,
    pub ellipses: Vec<Ellipse>,
    pub portal_pairs: Vec<PortalPair>,
    pub cubes: Vec<Cube>,
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

    pub fn add_cube(&mut self, cube: Cube) {
        if self.data.cube_count < MAX_CUBES as u32 {
            self.data.cubes.push(cube);
            self.data.cube_count += 1;
        } else {
            println!("Max cube count reached: {}", MAX_CUBES);
        }
    }

    pub fn to_raw(&self) -> SceneDataRaw {
        let mut planes = [PlaneRaw::default(); MAX_PLANES];
        let mut ellipses = [EllipseRaw::default(); MAX_ELLIPSES];
        let mut portal_pairs = [PortalPairRaw::default(); MAX_PORTAL_PAIRS];
        let mut cubes = [CubeRaw::default(); MAX_CUBES];

        for (i, plane) in self.data.planes.iter().enumerate() {
            if i < MAX_PLANES {
                planes[i] = (*plane).into();
            }
        }
        for (i, ellipse) in self.data.ellipses.iter().enumerate() {
            if i < MAX_ELLIPSES {
                ellipses[i] = (*ellipse).into();
            }
        }
        for (i, portal_pair) in self.data.portal_pairs.iter().enumerate() {
            if i < MAX_PORTAL_PAIRS {
                portal_pairs[i] = (*portal_pair).into();
            }
        }
        for (i, cube) in self.data.cubes.iter().enumerate() {
            if i < MAX_CUBES {
                cubes[i] = (*cube).into();
            }
        }

        SceneDataRaw {
            plane_count: self.data.plane_count,
            ellipse_count: self.data.ellipse_count,
            portal_pair_count: self.data.portal_pair_count,
            cube_count: self.data.cube_count,
            planes,
            ellipses,
            portal_pairs,
            cubes,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct SceneDataRaw {
    pub plane_count: u32,
    pub ellipse_count: u32,
    pub portal_pair_count: u32,
    pub cube_count: u32,
    pub planes: [PlaneRaw; MAX_PLANES],
    pub ellipses: [EllipseRaw; MAX_ELLIPSES],
    pub portal_pairs: [PortalPairRaw; MAX_PORTAL_PAIRS],
    pub cubes: [CubeRaw; MAX_CUBES],
}
