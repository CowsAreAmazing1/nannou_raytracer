use std::f32::consts::{FRAC_PI_2, PI};

use nannou::{
    color::*,
    glam::{Quat, Vec3, vec3},
};

use crate::{
    scene::{
        Scene,
        portal::{Portal, PortalPair},
        primitive::{cube::Cube, ellipse::Ellipse, plane::Plane},
    },
    util::quat_to,
};

pub fn create_scenes() -> Vec<Scene> {
    let mut scenes = Vec::new();

    let e_a = 0.6;
    let e_b = 1.0;
    let rim_thickness = 0.2;

    {
        let mut scene = Scene::new("test");

        scene.add_cube(Cube::new(
            vec3(2.0, 1.0, 1.0),
            0.5,
            (0.086275, 0.5098, 0.17255),
        ));

        scene.add_cube(Cube::new(Vec3::ZERO, 0.2, WHITE.into_format()));

        scene.add_portal_pair(PortalPair::from_ellipses(
            Ellipse::new(
                vec3(2.0, 0.0, 0.0),
                (0.0, FRAC_PI_2, FRAC_PI_2),
                0.6,
                1.0,
                0.1,
                ORANGERED.into_format(),
                BLACK.into_format(),
            ),
            Ellipse::new(
                vec3(-2.0, 0.0, 0.0),
                (0.0, PI, FRAC_PI_2),
                0.6,
                1.0,
                0.1,
                BLUEVIOLET.into_format(),
                BLACK.into_format(),
            ),
        ));

        scenes.push(scene);
    }

    {
        let mut scene = Scene::new("Primitives");

        scene.add_ellipse(Ellipse::new(
            vec3(3.0, 1.5, 0.0),
            (0.0, 0.0, FRAC_PI_2),
            e_a,
            e_b,
            rim_thickness,
            (0.7, 0.4, 0.0),
            (0.0, 0.0, 0.0),
        ));
        scene.add_ellipse(Ellipse::new(
            vec3(-3.0, 1.5, 0.0),
            (0.0, 0.0, FRAC_PI_2),
            e_a,
            e_b,
            rim_thickness,
            (0.0, 0.4, 0.7),
            (0.0, 0.0, 0.0),
        ));

        scene.add_plane(Plane::new_finite(
            vec3(3.0, 0.0, 2.5),
            Quat::IDENTITY,
            (1.0, 0.2, 0.2),
            5.0,
            5.0,
        ));
        scene.add_plane(Plane::new_finite(
            vec3(3.0, 0.0, -2.5),
            Quat::IDENTITY,
            (0.6, 0.6, 0.2),
            5.0,
            5.0,
        ));

        scene.add_plane(Plane::new_finite(
            vec3(-3.0, 0.0, 2.5),
            Quat::IDENTITY,
            (0.2, 1.0, 0.2),
            5.0,
            5.0,
        ));
        scene.add_plane(Plane::new_finite(
            vec3(-3.0, 0.0, -2.5),
            Quat::IDENTITY,
            (0.2, 0.2, 1.0),
            5.0,
            5.0,
        ));

        scenes.push(scene);
    }

    {
        // Scene 2: Same as 1 but with a pair of portals
        let mut scene = Scene::new("Single Portal Pair");

        for plane in scenes[1].data.planes.iter() {
            scene.add_plane(*plane);
        }

        scene.add_portal_pair(PortalPair::from_ellipses(
            scenes[1].data.ellipses[0],
            scenes[1].data.ellipses[1],
        ));

        scenes.push(scene);
    }

    {
        // Scene 3: Rooms
        let mut scene = Scene::new("Rooms");

        scene.add_plane(Plane::new_finite(
            // Red right
            (-0.5 - 1.5, 0.0 + 1.0, 0.0 - 5.0),
            quat_to(-Vec3::X),
            (0.2, 0.0, 0.0),
            3.0,
            3.0,
        ));
        scene.add_plane(Plane::new_finite(
            // Red back
            (-2.0 - 1.5, 0.0 + 1.0, -1.5 - 5.0),
            quat_to(Vec3::Z),
            (0.3, 0.0, 0.0),
            3.0,
            3.0,
        ));
        scene.add_plane(Plane::new_finite(
            // Red left
            (-3.5 - 1.5, 0.0 + 1.0, 0.0 - 5.0),
            quat_to(Vec3::X),
            (0.4, 0.0, 0.0),
            3.0,
            3.0,
        ));
        scene.add_plane(Plane::new_finite(
            // Red bottom
            (-2.0 - 1.5, -1.5 + 1.0, 0.0 - 5.0),
            quat_to(Vec3::Y),
            (0.5, 0.0, 0.0),
            3.0,
            3.0,
        ));
        scene.add_plane(Plane::new_finite(
            // Red top
            (-2.0 - 1.5, 1.5 + 1.0, 0.0 - 5.0),
            quat_to(-Vec3::Y),
            (0.6, 0.0, 0.0),
            3.0,
            3.0,
        ));

        scene.add_plane(Plane::new_finite(
            // Blue right
            (0.5 + 1.5, 0.0 + 1.0, 0.0 - 5.0),
            quat_to(-Vec3::X),
            (0.0, 0.0, 0.2),
            3.0,
            3.0,
        ));
        scene.add_plane(Plane::new_finite(
            // Blue back
            (2.0 + 1.5, 0.0 + 1.0, -1.5 - 5.0),
            quat_to(Vec3::Z),
            (0.0, 0.0, 0.3),
            3.0,
            3.0,
        ));
        scene.add_plane(Plane::new_finite(
            // Blue left
            (3.5 + 1.5, 0.0 + 1.0, 0.0 - 5.0),
            quat_to(Vec3::X),
            (0.0, 0.0, 0.4),
            3.0,
            3.0,
        ));
        scene.add_plane(Plane::new_finite(
            // Blue bottom
            (2.0 + 1.5, -1.5 + 1.0, 0.0 - 5.0),
            quat_to(Vec3::Y),
            (0.0, 0.0, 0.5),
            3.0,
            3.0,
        ));
        scene.add_plane(Plane::new_finite(
            // Blue top
            (2.0 + 1.5, 1.5 + 1.0, 0.0 - 5.0),
            quat_to(-Vec3::Y),
            (0.0, 0.0, 0.6),
            3.0,
            3.0,
        ));

        scene.add_portal_pair(PortalPair::new(
            Portal::new(
                Vec3::new(-0.51 - 1.5, 0.0 + 1.0, 0.0 - 5.0),
                (0.0, 0.0, FRAC_PI_2),
                e_a,
                e_b,
                true,
            ),
            Portal::new(
                Vec3::new(0.51 + 1.5, 0.0 + 1.0, 0.0 - 5.0),
                (0.0, 0.0, -FRAC_PI_2),
                e_a,
                e_b,
                false,
            ),
        ));

        scenes.push(scene);
    }

    {
        // Scene 4: Infinite Portal Room
        let mut scene = Scene::new("Infinite Portal Room");

        scene.add_plane(Plane::new(
            [0.0, -2.0, 0.0],
            Quat::IDENTITY,
            (0.2, 0.2, 0.2),
        ));

        scene.add_plane(Plane::new_finite(
            // Red right
            [0.6, 0.0 + 1.0, 0.0 - 5.0],
            Quat::from_rotation_z(FRAC_PI_2),
            (0.2, 0.0, 0.0),
            3.0,
            3.0,
        ));
        scene.add_plane(Plane::new_finite(
            // Red back
            [0.0, 0.0 + 1.0, -1.5 - 5.0],
            quat_to(Vec3::Z),
            (0.8, 0.8, 0.8),
            1.2,
            3.0,
        ));
        scene.add_plane(Plane::new_finite(
            // Red left
            [-0.6, 0.0 + 1.0, 0.0 - 5.0],
            Quat::from_rotation_z(FRAC_PI_2),
            (0.0, 0.6, 0.5),
            3.0,
            3.0,
        ));
        scene.add_plane(Plane::new_finite(
            // Red bottom
            [-0.0, -1.5 + 1.0, 0.0 - 5.0],
            Quat::IDENTITY,
            (0.8, 0.8, 0.8),
            3.0,
            1.2,
        ));
        scene.add_plane(Plane::new_finite(
            // Red top
            [-0.0, 1.5 + 1.0, 0.0 - 5.0],
            Quat::IDENTITY,
            (0.8, 0.8, 0.8),
            3.0,
            1.2,
        ));

        scene.add_portal_pair(PortalPair::new(
            Portal::new(
                Vec3::new(-0.55, 0.0 + 1.0, 0.0 - 5.0),
                (0.0, 0.0, -FRAC_PI_2),
                0.6,
                1.0,
                true,
            ),
            Portal::new(
                Vec3::new(0.55, 0.0 + 1.0, 0.0 - 5.0),
                (0.0, 0.0, FRAC_PI_2),
                0.6,
                1.0,
                false,
            ),
        ));
        // scene.add_portal_pair(PortalPair::new(
        //     Portal::new(
        //         Vec3::new(0.0, 0.0 + 1.0, -1.3 - 5.0),
        //         quat_to(Vec3::Z),
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

        scenes.push(scene);
    }

    {
        let mut scene = Scene::new("Door");

        scene.add_plane(Plane::new_finite(
            [0.0, 0.0, -1.25],
            Quat::IDENTITY,
            (0.4, 0.1, 0.4),
            2.5,
            5.0,
        ));

        scene.add_plane(Plane::new_finite(
            [0.0, 0.0, 1.25],
            Quat::IDENTITY,
            (0.1, 0.4, 0.4),
            2.5,
            5.0,
        ));

        scene.add_portal_pair(PortalPair::new(
            Portal::new(vec3(0.0, 1.0, 0.5), (0.0, 0.0, 0.0), 0.7, 0.9, true),
            Portal::new(vec3(0.0, 1.0, -0.5), (FRAC_PI_2, 0.0, 0.0), 0.7, 0.9, false),
        ));

        scenes.push(scene);
    }

    {
        let mut scene = Scene::new("Static Door");

        scene.add_plane(Plane::new_finite(
            [0.0, 0.0, -1.25],
            Quat::IDENTITY,
            (0.4, 0.1, 0.4),
            2.5,
            5.0,
        ));

        scene.add_plane(Plane::new_finite(
            [0.0, 0.0, 1.25],
            Quat::IDENTITY,
            (0.1, 0.4, 0.4),
            2.5,
            5.0,
        ));

        scene.add_portal_pair(PortalPair::new(
            Portal::new(vec3(0.1, 1.0, 0.0), (0.0, 0.0, FRAC_PI_2), 0.7, 0.9, true),
            Portal::new(vec3(0.0, 1.0, 0.0), (0.0, PI, FRAC_PI_2), 0.7, 0.9, false),
        ));

        scenes.push(scene);
    }

    {
        let mut scene = Scene::new("Empty");

        scene.add_portal_pair(PortalPair::from_ellipses(
            Ellipse::new(
                vec3(2.0, 0.0, 0.0),
                (0.0, 0.0, 0.0),
                e_a,
                e_b,
                0.1,
                ORANGERED.into_format(),
                BLACK.into_format(),
            ),
            Ellipse::new(
                vec3(-2.0, 0.0, 0.0),
                (0.0, 0.0, 0.0),
                e_a,
                e_b,
                0.1,
                BLUEVIOLET.into_format(),
                BLACK.into_format(),
            ),
        ));

        scenes.push(scene);
    }

    scenes
}
