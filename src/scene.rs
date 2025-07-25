
use bytemuck::{Pod, Zeroable};



// Scene Objects
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct Sphere {
    center: [f32; 3],
    radius: f32,
    color: [f32; 3],
    _padding: f32,
}

impl Default for Sphere {
    fn default() -> Self {
        Self {
            center: [0.0; 3],
            radius: 1.0,
            color: [0.0; 3],
            _padding: 0.0,
        }
    }
}




#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct Plane {
    point: [f32; 3],
    _padding1: f32,
    normal: [f32; 3],
    _padding2: f32,
    color: [f32; 3],
    _padding3: f32,
    width: f32,
    height: f32,
    is_infinite: f32, // 1.0 for infinite, 0.0 for finite
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

    #[allow(dead_code)]
    pub fn new_finite(point: [f32; 3], normal: [f32; 3], color: [f32; 3], width: f32, height: f32) -> Self {
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
    center: [f32; 3],
    _padding1: f32,
    normal: [f32; 3],
    _padding2: f32,
    radius_a: f32,
    radius_b: f32,
    border_thickness: f32,
    _padding3: f32,
    color: [f32; 3],
    _padding4: f32,
    border_color: [f32; 3],
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


const MAX_SPHERES: usize = 8;
const MAX_PLANES: usize = 4;
const MAX_ELLIPSES: usize = 8;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct SceneData {
    pub sphere_count: u32,
    pub plane_count: u32,
    pub ellipse_count: u32,
    pub _padding: u32,
    pub spheres: [Sphere; MAX_SPHERES],
    pub planes: [Plane; MAX_PLANES],
    pub ellipses: [Ellipse; MAX_ELLIPSES],
}

impl Default for SceneData {
    fn default() -> Self {
        Self {
            plane_count: 0,
            sphere_count: 0,
            ellipse_count: 0,
            _padding: 0,
            spheres: [Sphere::default(); MAX_SPHERES],
            planes: [Plane::default(); MAX_PLANES],
            ellipses: [Ellipse::default(); MAX_ELLIPSES],
        }
    }
}

impl SceneData {
    pub fn new(sphere_count: u32, plane_count: u32, ellipse_count: u32) -> Self {
        Self {
            sphere_count,
            plane_count,
            ellipse_count,
            _padding: 0,
            spheres: [Sphere::default(); MAX_SPHERES],
            planes: [Plane::default(); MAX_PLANES],
            ellipses: [Ellipse::default(); MAX_ELLIPSES],
        }
    }

    #[allow(dead_code)]
    pub fn add_sphere(&mut self, sphere: Sphere) {
        if self.sphere_count < MAX_SPHERES as u32 {
            self.spheres[self.sphere_count as usize] = sphere;
            self.sphere_count += 1;
        } else {
            println!("Max sphere count reached: {}", MAX_SPHERES);
        }
    }

    pub fn add_plane(&mut self, plane: Plane) {
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

    pub fn create_scenes() -> Vec<SceneData> {
        let mut scenes = Vec::new();

        let e_a = 0.6;
        let e_b = 1.0;
        let rim_thickness = 0.1;

        {
        let mut scene1 = SceneData::new(0, 1, 2);

        scene1.add_plane(
            Plane::new(
                [0.0, -5.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.2, 0.0, 0.0],
            )
        );

        scene1.add_ellipse(
            Ellipse::new(
                [1.5, 1.0, -4.0],
                [0.0, -0.5, 1.0],
                e_a,
                e_b,
                rim_thickness,
                [0.7, 0.4, 0.0],
                [0.0, 0.0, 0.0],
            )
        );

        scene1.add_ellipse(
            Ellipse::new(
                [-1.5, 1.0, -4.0],
                [0.0, -0.5, 1.0],
                e_a,
                e_b,
                rim_thickness,
                [0.0, 0.4, 0.7],
                [0.0, 0.0, 0.0],
            )
        );

        scenes.push(scene1);
        }

        scenes
    }
}