pub mod data;
pub mod primitive;

use bytemuck::{Pod, Zeroable};

use crate::scene::primitive::{
    cube::{Cube, CubeRaw},
    ellipse::{Ellipse, EllipseRaw},
    plane::{Plane, PlaneRaw},
};

const MAX_PLANES: usize = 10;
const MAX_ELLIPSES: usize = 4;
const MAX_CUBES: usize = 4;

#[derive(Default)]
pub struct SceneData {
    pub plane_count: u32,
    pub ellipse_count: u32,
    pub cube_count: u32,
    pub planes: Vec<Plane>,
    pub ellipses: Vec<Ellipse>,
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
        let mut cubes = [CubeRaw::default(); MAX_CUBES];

        for (i, plane) in self.data.planes.iter().enumerate() {
            if i < MAX_PLANES {
                planes[i] = plane.into();
            }
        }
        for (i, ellipse) in self.data.ellipses.iter().enumerate() {
            if i < MAX_ELLIPSES {
                ellipses[i] = ellipse.into();
            }
        }
        for (i, cube) in self.data.cubes.iter().enumerate() {
            if i < MAX_CUBES {
                cubes[i] = cube.into();
            }
        }

        SceneDataRaw {
            plane_count: self.data.plane_count,
            ellipse_count: self.data.ellipse_count,
            cube_count: self.data.cube_count,
            _padding: 0,
            planes,
            ellipses,
            cubes,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct SceneDataRaw {
    pub plane_count: u32,
    pub ellipse_count: u32,
    pub cube_count: u32,
    _padding: u32,
    pub planes: [PlaneRaw; MAX_PLANES],
    pub ellipses: [EllipseRaw; MAX_ELLIPSES],
    pub cubes: [CubeRaw; MAX_CUBES],
}
