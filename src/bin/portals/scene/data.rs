use std::f32::consts::PI;

use nannou::glam::{Quat, Vec3, vec3};

use crate::scene::{
    Scene,
    portal::{Portal, PortalPair},
    primitive::{Ellipse, Plane},
};

pub fn create_scenes() -> Vec<Scene> {
    let mut scenes = Vec::new();

    let e_a = 0.6;
    let e_b = 1.0;
    let rim_thickness = 0.2;

    {
        let mut scene = Scene::new("Ellipse Showcase");

        scene.add_plane(Plane::new(
            vec3(0.0, -2.0, 0.0),
            Quat::IDENTITY,
            (0.2, 0.0, 0.0),
        ));

        scene.add_ellipse(Ellipse::new(
            [1.5, 1.0, -4.0],
            [0.0, -0.5, 1.0],
            e_a,
            e_b,
            rim_thickness,
            [0.7, 0.4, 0.0],
            [0.0, 0.0, 0.0],
        ));

        scene.add_ellipse(Ellipse::new(
            [-1.5, 1.0, -4.0],
            [0.0, -0.5, 1.0],
            e_a,
            e_b,
            rim_thickness,
            [0.0, 0.4, 0.7],
            [0.0, 0.0, 0.0],
        ));

        scenes.push(scene);
    }

    {
        let mut scene = Scene::new("Single Portal Pair Setup");

        scene.add_plane(Plane::new((0.1, 0.0, 0.1), Quat::IDENTITY, (0.5, 0.0, 0.0)));

        scene.add_plane(Plane::new(
            (-0.1, 0.0, 0.1),
            Quat::IDENTITY,
            (0.35, 0.35, 0.0),
        ));

        scene.add_plane(Plane::new(
            (0.1, 0.0, -0.1),
            Quat::IDENTITY,
            (0.0, 0.5, 0.0),
        ));

        scene.add_plane(Plane::new(
            (-0.1, 0.0, -0.1),
            Quat::IDENTITY,
            (0.0, 0.2, 0.5),
        ));

        scene.add_ellipse(Ellipse::new(
            [-1.0, 1.7, -4.0],
            [0.0, 0.0, 1.0],
            e_a,
            e_b,
            rim_thickness,
            [0.7, 0.4, 0.0],
            [0.0, 0.0, 0.0],
        ));

        scene.add_ellipse(Ellipse::new(
            [1.0, 1.7, -4.1],
            [0.0, 0.0, -1.0],
            e_a,
            e_b,
            rim_thickness,
            [0.0, 0.4, 0.7],
            [0.0, 0.0, 0.0],
        ));

        scenes.push(scene);
    }

    {
        let mut scene = Scene::new("Single Portal Pair");

        scene.add_plane(Plane::new((0.1, 0.0, 0.1), Quat::IDENTITY, (0.5, 0.0, 0.0)));

        scene.add_plane(Plane::new(
            (-0.1, 0.0, 0.1),
            Quat::IDENTITY,
            (0.35, 0.35, 0.0),
        ));

        scene.add_plane(Plane::new(
            (0.1, 0.0, -0.1),
            Quat::IDENTITY,
            (0.0, 0.5, 0.0),
        ));

        scene.add_plane(Plane::new(
            (-0.1, 0.0, -0.1),
            Quat::IDENTITY,
            (0.0, 0.2, 0.5),
        ));

        scene.add_portal_pair(PortalPair::new(
            Portal::from_ellipse(scenes[1].data.ellipses[0]),
            Portal::from_ellipse(scenes[1].data.ellipses[1]),
        ));

        scenes.push(scene);
    }

    {
        // Scene 4: Rooms
        let mut scene = Scene::new("Rooms");

        scene.add_plane(Plane::new_finite(
            // Red right
            (-0.5 - 1.5, 0.0 + 1.0, 0.0 - 5.0),
            Quat::IDENTITY, // [-1.0, 0.0, 0.0],
            (0.2, 0.0, 0.0),
            3.0,
            3.0,
        ));
        scene.add_plane(Plane::new_finite(
            // Red back
            (-2.0 - 1.5, 0.0 + 1.0, -1.5 - 5.0),
            Quat::IDENTITY, // [0.0, 0.0, 1.0],
            (0.3, 0.0, 0.0),
            3.0,
            3.0,
        ));
        scene.add_plane(Plane::new_finite(
            // Red left
            (-3.5 - 1.5, 0.0 + 1.0, 0.0 - 5.0),
            Quat::IDENTITY, // [1.0, 0.0, 0.0],
            (0.4, 0.0, 0.0),
            3.0,
            3.0,
        ));
        scene.add_plane(Plane::new_finite(
            // Red bottom
            (-2.0 - 1.5, -1.5 + 1.0, 0.0 - 5.0),
            Quat::IDENTITY, // [0.0, 1.0, 0.0],
            (0.5, 0.0, 0.0),
            3.0,
            3.0,
        ));
        scene.add_plane(Plane::new_finite(
            // Red top
            (-2.0 - 1.5, 1.5 + 1.0, 0.0 - 5.0),
            Quat::IDENTITY, // [0.0, -1.0, 0.0],
            (0.6, 0.0, 0.0),
            3.0,
            3.0,
        ));

        scene.add_plane(Plane::new_finite(
            // Blue right
            (0.5 + 1.5, 0.0 + 1.0, 0.0 - 5.0),
            Quat::IDENTITY, // [-1.0, 0.0, 0.0],
            (0.0, 0.0, 0.2),
            3.0,
            3.0,
        ));
        scene.add_plane(Plane::new_finite(
            // Blue back
            (2.0 + 1.5, 0.0 + 1.0, -1.5 - 5.0),
            Quat::IDENTITY, // [0.0, 0.0, 1.0],
            (0.0, 0.0, 0.3),
            3.0,
            3.0,
        ));
        scene.add_plane(Plane::new_finite(
            // Blue left
            (3.5 + 1.5, 0.0 + 1.0, 0.0 - 5.0),
            Quat::IDENTITY, // [1.0, 0.0, 0.0],
            (0.0, 0.0, 0.4),
            3.0,
            3.0,
        ));
        scene.add_plane(Plane::new_finite(
            // Blue bottom
            (2.0 + 1.5, -1.5 + 1.0, 0.0 - 5.0),
            Quat::IDENTITY, // [0.0, 1.0, 0.0],
            (0.0, 0.0, 0.5),
            3.0,
            3.0,
        ));
        scene.add_plane(Plane::new_finite(
            // Blue top
            (2.0 + 1.5, 1.5 + 1.0, 0.0 - 5.0),
            Quat::IDENTITY, // [0.0, -1.0, 0.0],
            (0.0, 0.0, 0.6),
            3.0,
            3.0,
        ));

        scene.add_portal_pair(PortalPair::new(
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

        scenes.push(scene);
    }

    {
        // Scene 5: Infinite Portal Room
        let mut scene = Scene::new("Infinite Portal Room");

        scene.add_plane(Plane::new(
            [0.0, -2.0, 0.0],
            Quat::from_rotation_z(0.5 * PI),
            (0.2, 0.2, 0.2),
        ));

        scene.add_plane(Plane::new_finite(
            // Red right
            [0.6, 0.0 + 1.0, 0.0 - 5.0],
            Quat::from_rotation_z(PI),
            (0.2, 0.0, 0.0),
            3.0,
            3.0,
        ));
        scene.add_plane(Plane::new_finite(
            // Red back
            [0.0, 0.0 + 1.0, -1.5 - 5.0],
            Quat::from_rotation_arc(Vec3::Y, Vec3::Z),
            (0.8, 0.8, 0.8),
            1.2,
            3.0,
        ));
        scene.add_plane(Plane::new_finite(
            // Red left
            [-0.6, 0.0 + 1.0, 0.0 - 5.0],
            Quat::IDENTITY,
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
        // scene.add_portal_pair(PortalPair::new(
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

        scenes.push(scene);
    }

    {
        let mut scene = Scene::new("Portal Animation Test");

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
            Portal::new(
                vec3(0.0, 1.0, 0.5),
                Quat::from_rotation_arc(Vec3::Y, -Vec3::Z)
                    * Quat::from_rotation_y(PI)
                    * Quat::from_rotation_z(0.1),
                0.7,
                0.9,
            ),
            Portal::new(
                vec3(0.0, 1.0, -0.5),
                Quat::from_rotation_arc(Vec3::Y, Vec3::Z),
                0.7,
                0.9,
            ),
        ));

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
            Portal::new(
                vec3(0.0, 1.0, 0.5),
                Quat::from_rotation_arc(Vec3::Y, -Vec3::Z)
                    * Quat::from_rotation_y(PI)
                    * Quat::from_rotation_z(0.1),
                0.7,
                0.9,
            ),
            Portal::new(
                vec3(0.0, 1.0, -0.5),
                Quat::from_rotation_arc(Vec3::Y, Vec3::Z),
                0.7,
                0.9,
            ),
        ));

        scenes.push(scene);
    }

    scenes
}
