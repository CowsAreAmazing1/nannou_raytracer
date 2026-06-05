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
    pub width: f32,
    pub height: f32,
    pub is_infinite: f32, // 1.0 for infinite, 0.0 for finite
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

    pub fn new_finite(
        point: [f32; 3],
        normal: [f32; 3],
        color: [f32; 3],
        width: f32,
        height: f32,
    ) -> Self {
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
    pub fn new(
        center: [f32; 3],
        normal: [f32; 3],
        radius_a: f32,
        radius_b: f32,
        border_thickness: f32,
        color: [f32; 3],
        border_color: [f32; 3],
    ) -> Self {
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
    pub fn new(position: Vec3, rotation: Quat, radius_a: f32, radius_b: f32) -> Self {
        let ellipse = Ellipse::new(
            position.to_array(),
            (rotation * Vec3::Y).to_array(),
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

    #[allow(dead_code)]
    fn from_ellipse(ellipse: Ellipse) -> Self {
        let mut portal = Self {
            ellipse,
            transformation_matrix: Mat4::IDENTITY.to_cols_array(),
            inverse_transformation_matrix: Mat4::IDENTITY.to_cols_array(),
        };

        portal.transform_from_self();
        portal
    }

    #[allow(dead_code)]
    fn transform_from_self(&mut self) {
        let position = Vec3::from(self.ellipse.center);
        let rotation = Quat::from_rotation_arc(Vec3::Y, Vec3::from(self.ellipse.normal));

        let base_transform = Mat4::from_rotation_translation(rotation, position);

        let flip_matrix = Mat4::IDENTITY;
        // if self.flip_transform < 0.0 {
        //     Mat4::from_rotation_z(std::f32::consts::PI)
        // } else {
        //     Mat4::IDENTITY
        // };

        let final_transform = base_transform * flip_matrix;

        self.transformation_matrix = final_transform.to_cols_array();
        self.inverse_transformation_matrix = final_transform.inverse().to_cols_array();
    }

    fn update_transform(&mut self, position: Vec3, rotation: Quat) {
        self.ellipse.center = position.to_array();
        self.ellipse.normal = (rotation * Vec3::Y).to_array();

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

    // Animation methods
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

    #[allow(dead_code)]
    fn get_rotation(&self) -> Quat {
        let current_normal = Vec3::from(self.ellipse.normal);
        Quat::from_rotation_arc(Vec3::Y, current_normal)
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

const MAX_PLANES: usize = 10;
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

    pub fn add_portal_pair(&mut self, portal_pair: PortalPair) {
        if self.portal_pair_count < MAX_PORTAL_PAIRS as u32 {
            self.portal_pairs[self.portal_pair_count as usize] = portal_pair;
            self.portal_pair_count += 1;
        } else {
            println!("Max portal pair count reached: {}", MAX_PORTAL_PAIRS);
        }
    }

    pub fn create_scenes() -> Vec<SceneData> {
        let mut scenes = Vec::new();

        let e_a = 0.6;
        let e_b = 1.0;
        let rim_thickness = 0.2;

        {
            // Scene 1: Ellipse Showcase
            let mut scene1 = SceneData::new();

            scene1.add_plane(Plane::new(
                [0.0, -2.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.2, 0.0, 0.0],
            ));

            scene1.add_ellipse(Ellipse::new(
                [1.5, 1.0, -4.0],
                [0.0, -0.5, 1.0],
                e_a,
                e_b,
                rim_thickness,
                [0.7, 0.4, 0.0],
                [0.0, 0.0, 0.0],
            ));

            scene1.add_ellipse(Ellipse::new(
                [-1.5, 1.0, -4.0],
                [0.0, -0.5, 1.0],
                e_a,
                e_b,
                rim_thickness,
                [0.0, 0.4, 0.7],
                [0.0, 0.0, 0.0],
            ));

            scenes.push(scene1);
        }

        {
            // Scene 2: Single Portal Pair Setup
            let mut scene2 = SceneData::new();

            scene2.add_plane(Plane::new(
                [0.1, 0.0, 0.1],
                [-0.1, 1.0, -0.1],
                [0.5, 0.0, 0.0],
            ));

            scene2.add_plane(Plane::new(
                [-0.1, 0.0, 0.1],
                [0.1, 1.0, -0.1],
                [0.35, 0.35, 0.0],
            ));

            scene2.add_plane(Plane::new(
                [0.1, 0.0, -0.1],
                [-0.1, 1.0, 0.1],
                [0.0, 0.5, 0.0],
            ));

            scene2.add_plane(Plane::new(
                [-0.1, 0.0, -0.1],
                [0.1, 1.0, 0.1],
                [0.0, 0.2, 0.5],
            ));

            scene2.add_ellipse(Ellipse::new(
                [-1.0, 1.7, -4.0],
                [0.0, 0.0, 1.0],
                e_a,
                e_b,
                rim_thickness,
                [0.7, 0.4, 0.0],
                [0.0, 0.0, 0.0],
            ));

            scene2.add_ellipse(Ellipse::new(
                [1.0, 1.7, -4.1],
                [0.0, 0.0, -1.0],
                e_a,
                e_b,
                rim_thickness,
                [0.0, 0.4, 0.7],
                [0.0, 0.0, 0.0],
            ));

            scenes.push(scene2);
        }

        {
            // Scene 3: Single Portal Pair
            let mut scene3 = SceneData::new();

            scene3.add_plane(Plane::new(
                [0.1, 0.0, 0.1],
                [-0.1, 1.0, -0.1],
                [0.5, 0.0, 0.0],
            ));

            scene3.add_plane(Plane::new(
                [-0.1, 0.0, 0.1],
                [0.1, 1.0, -0.1],
                [0.35, 0.35, 0.0],
            ));

            scene3.add_plane(Plane::new(
                [0.1, 0.0, -0.1],
                [-0.1, 1.0, 0.1],
                [0.0, 0.5, 0.0],
            ));

            scene3.add_plane(Plane::new(
                [-0.1, 0.0, -0.1],
                [0.1, 1.0, 0.1],
                [0.0, 0.2, 0.5],
            ));

            scene3.add_portal_pair(PortalPair::new(
                Portal::new(
                    scenes[1].ellipses[0].center.into(),
                    Quat::from_rotation_arc(Vec3::Y, Vec3::Z),
                    0.6,
                    1.0,
                ),
                Portal::new(
                    scenes[1].ellipses[1].center.into(),
                    Quat::from_rotation_arc(Vec3::Y, Vec3::Z) * Quat::from_rotation_z(PI),
                    0.6,
                    1.0,
                ),
            ));

            scenes.push(scene3);
        }

        {
            // Scene 4: Rooms
            let mut scene4 = SceneData::new();

            scene4.add_plane(Plane::new_finite(
                // Red right
                [-0.5 - 1.5, 0.0 + 1.0, 0.0 - 5.0],
                [-1.0, 0.0, 0.0],
                [0.2, 0.0, 0.0],
                3.0,
                3.0,
            ));
            scene4.add_plane(Plane::new_finite(
                // Red back
                [-2.0 - 1.5, 0.0 + 1.0, -1.5 - 5.0],
                [0.0, 0.0, 1.0],
                [0.3, 0.0, 0.0],
                3.0,
                3.0,
            ));
            scene4.add_plane(Plane::new_finite(
                // Red left
                [-3.5 - 1.5, 0.0 + 1.0, 0.0 - 5.0],
                [1.0, 0.0, 0.0],
                [0.4, 0.0, 0.0],
                3.0,
                3.0,
            ));
            scene4.add_plane(Plane::new_finite(
                // Red bottom
                [-2.0 - 1.5, -1.5 + 1.0, 0.0 - 5.0],
                [0.0, 1.0, 0.0],
                [0.5, 0.0, 0.0],
                3.0,
                3.0,
            ));
            scene4.add_plane(Plane::new_finite(
                // Red top
                [-2.0 - 1.5, 1.5 + 1.0, 0.0 - 5.0],
                [0.0, -1.0, 0.0],
                [0.6, 0.0, 0.0],
                3.0,
                3.0,
            ));

            scene4.add_plane(Plane::new_finite(
                // Blue right
                [0.5 + 1.5, 0.0 + 1.0, 0.0 - 5.0],
                [-1.0, 0.0, 0.0],
                [0.0, 0.0, 0.2],
                3.0,
                3.0,
            ));
            scene4.add_plane(Plane::new_finite(
                // Blue back
                [2.0 + 1.5, 0.0 + 1.0, -1.5 - 5.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 0.3],
                3.0,
                3.0,
            ));
            scene4.add_plane(Plane::new_finite(
                // Blue left
                [3.5 + 1.5, 0.0 + 1.0, 0.0 - 5.0],
                [1.0, 0.0, 0.0],
                [0.0, 0.0, 0.4],
                3.0,
                3.0,
            ));
            scene4.add_plane(Plane::new_finite(
                // Blue bottom
                [2.0 + 1.5, -1.5 + 1.0, 0.0 - 5.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 0.5],
                3.0,
                3.0,
            ));
            scene4.add_plane(Plane::new_finite(
                // Blue top
                [2.0 + 1.5, 1.5 + 1.0, 0.0 - 5.0],
                [0.0, -1.0, 0.0],
                [0.0, 0.0, 0.6],
                3.0,
                3.0,
            ));

            scene4.add_portal_pair(PortalPair::new(
                Portal::new(
                    Vec3::new(-0.51 - 1.5, 0.0 + 1.0, 0.0 - 5.0),
                    Quat::from_rotation_arc(Vec3::Y, -Vec3::X),
                    e_a,
                    e_b,
                ),
                Portal::new(
                    Vec3::new(0.51 + 1.5, 0.0 + 1.0, 0.0 - 5.0),
                    Quat::from_rotation_arc(Vec3::Y, Vec3::X),
                    e_a,
                    e_b,
                ),
            ));

            scenes.push(scene4);
        }

        {
            // Scene 5: Infinite Portal Room
            let mut scene5 = SceneData::new();

            scene5.add_plane(Plane::new(
                [0.0, -2.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.2, 0.2, 0.2],
            ));

            scene5.add_plane(Plane::new_finite(
                // Red right
                [0.6, 0.0 + 1.0, 0.0 - 5.0],
                [-1.0, 0.0, 0.0],
                [0.2, 0.0, 0.0],
                3.0,
                3.0,
            ));
            scene5.add_plane(Plane::new_finite(
                // Red back
                [0.0, 0.0 + 1.0, -1.5 - 5.0],
                [0.0, 0.0, 1.0],
                [0.8, 0.8, 0.8],
                1.2,
                3.0,
            ));
            scene5.add_plane(Plane::new_finite(
                // Red left
                [-0.6, 0.0 + 1.0, 0.0 - 5.0],
                [1.0, 0.0, 0.0],
                [0.0, 0.6, 0.5],
                3.0,
                3.0,
            ));
            scene5.add_plane(Plane::new_finite(
                // Red bottom
                [-0.0, -1.5 + 1.0, 0.0 - 5.0],
                [0.0, 1.0, 0.0],
                [0.8, 0.8, 0.8],
                3.0,
                1.2,
            ));
            scene5.add_plane(Plane::new_finite(
                // Red top
                [-0.0, 1.5 + 1.0, 0.0 - 5.0],
                [0.0, -1.0, 0.0],
                [0.8, 0.8, 0.8],
                3.0,
                1.2,
            ));

            scene5.add_portal_pair(PortalPair::new(
                Portal::new(
                    Vec3::new(-0.55, 0.0 + 1.0, 0.0 - 5.0),
                    Quat::from_rotation_arc(Vec3::Y, Vec3::X),
                    0.6,
                    1.0,
                ),
                Portal::new(
                    Vec3::new(0.55, 0.0 + 1.0, 0.0 - 5.0),
                    Quat::from_rotation_arc(Vec3::Y, -Vec3::X) * Quat::from_rotation_x(0.0),
                    0.6,
                    1.0,
                ),
            ));
            // scene5.add_portal_pair(PortalPair::new(
            //     Portal::new(
            //         Vec3::new(0.0, 0.0 + 1.0, -1.3 - 5.0),
            //         Quat::from_rotation_arc(Vec3::Y, Vec3::Z),
            //         1.0,
            //         1.0,
            //     ),
            //     Portal::new(
            //         Vec3::new(1.4, 0.0 + 1.0, 4.0 - 5.0),
            //         Quat::from_rotation_z(PI/2.0) * Quat::from_rotation_y(-PI/2.0),
            //         1.0,
            //         1.0,
            //     ),
            // ));

            scenes.push(scene5);
        }

        {
            // Scene 6: Non-Euclidean Portal Maze
            let mut scene6 = SceneData::new();

            // Central Hub Room (Green theme)
            scene6.add_plane(Plane::new_finite(
                // Hub floor
                [0.0, -1.5, -5.0],
                [0.0, 1.0, 0.0],
                [0.1, 0.3, 0.1],
                4.0,
                4.0,
            ));
            scene6.add_plane(Plane::new_finite(
                // Hub ceiling
                [0.0, 1.5, -5.0],
                [0.0, -1.0, 0.0],
                [0.1, 0.3, 0.1],
                4.0,
                4.0,
            ));

            // North Corridor (leads to floating room)
            scene6.add_plane(Plane::new_finite(
                // North wall with portal opening
                [0.0, 0.0, -7.0],
                [0.0, 0.0, 1.0],
                [0.2, 0.4, 0.2],
                3.0,
                3.0,
            ));

            // East Tower Room (Red theme, elevated)
            scene6.add_plane(Plane::new_finite(
                // Tower floor
                [8.0, 2.0, -5.0],
                [0.0, 1.0, 0.0],
                [0.4, 0.1, 0.1],
                3.0,
                3.0,
            ));
            scene6.add_plane(Plane::new_finite(
                // Tower east wall
                [9.5, 3.5, -5.0],
                [-1.0, 0.0, 0.0],
                [0.5, 0.1, 0.1],
                3.0,
                3.0,
            ));
            scene6.add_plane(Plane::new_finite(
                // Tower north wall
                [8.0, 3.5, -6.5],
                [0.0, 0.0, 1.0],
                [0.3, 0.1, 0.1],
                3.0,
                3.0,
            ));

            // Underground Chamber (Blue theme, below hub)
            scene6.add_plane(Plane::new_finite(
                // Underground floor
                [0.0, -8.0, -5.0],
                [0.0, 1.0, 0.0],
                [0.1, 0.1, 0.4],
                5.0,
                5.0,
            ));
            scene6.add_plane(Plane::new_finite(
                // Underground ceiling
                [0.0, -5.5, -5.0],
                [0.0, -1.0, 0.0],
                [0.1, 0.2, 0.5],
                5.0,
                5.0,
            ));

            // Floating Island Room (Purple theme)
            scene6.add_plane(Plane::new_finite(
                // Floating platform
                [-12.0, 10.0, -2.0],
                [0.0, 1.0, 0.0],
                [0.4, 0.1, 0.4],
                2.5,
                2.5,
            ));

            // Portal Network:
            // 1. Hub to Tower (East wall of hub connects to west wall of tower)
            scene6.add_portal_pair(PortalPair::new(
                Portal::new(
                    Vec3::new(1.8, 0.0, -5.0),
                    Quat::from_rotation_arc(Vec3::Y, -Vec3::X), // Facing east
                    0.8,
                    1.2,
                ),
                Portal::new(
                    Vec3::new(6.5, 3.5, -5.0),
                    Quat::from_rotation_arc(Vec3::Y, Vec3::X), // Facing west
                    0.8,
                    1.2,
                ),
            ));

            // 2. Hub to Underground (Floor portal in hub connects to ceiling of underground)
            scene6.add_portal_pair(PortalPair::new(
                Portal::new(
                    Vec3::new(-1.0, -1.4, -4.0),
                    Quat::from_rotation_arc(Vec3::Y, -Vec3::Y), // Facing down
                    0.6,
                    0.6,
                ),
                Portal::new(
                    Vec3::new(0.0, -5.6, -5.0),
                    Quat::from_rotation_arc(Vec3::Y, Vec3::Y), // Facing up
                    0.6,
                    0.6,
                ),
            ));

            // 3. Tower to Floating Island (impossible connection - tower ceiling to floating platform)
            scene6.add_portal_pair(PortalPair::new(
                Portal::new(
                    Vec3::new(8.0, 4.8, -5.0),
                    Quat::from_rotation_arc(Vec3::Y, Vec3::Y), // Facing up
                    0.5,
                    0.5,
                ),
                Portal::new(
                    Vec3::new(-12.0, 9.9, -2.0),
                    Quat::from_rotation_arc(Vec3::Y, -Vec3::Y), // Facing down
                    0.5,
                    0.5,
                ),
            ));

            // 4. Floating Island back to Hub North Wall (creates impossible loop)
            scene6.add_portal_pair(PortalPair::new(
                Portal::new(
                    Vec3::new(-12.0, 11.0, -0.5),
                    Quat::from_rotation_arc(Vec3::Y, -Vec3::Z), // Facing north
                    0.7,
                    0.9,
                ),
                Portal::new(
                    Vec3::new(0.0, 0.5, -6.9),
                    Quat::from_rotation_arc(Vec3::Y, Vec3::Z), // Facing south
                    0.7,
                    0.9,
                ),
            ));

            scenes.push(scene6);
        }

        scenes
    }
}
