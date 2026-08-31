use std::f32::consts::FRAC_PI_2;

use nannou::glam::{EulerRot::XYZ, Quat, Vec2, Vec3, vec2, vec3};

use crate::{
    scene::{
        Scene,
        primitive::{cube::Cube, ellipse::Ellipse, plane::Plane},
    },
    util::{WORLD_UP, quat_to},
};

pub fn create_scenes() -> Vec<Scene> {
    let mut scenes = Vec::new();

    {
        let mut scene = Scene::with_max_bounces("test", 5);

        let two_to_three = |vec: &Vec2| vec3(vec.x, 0.0, vec.y);

        let vertices_2d = [
            vec2(5.3, -6.0),
            vec2(1.35, 1.23),
            vec2(-4.55, -0.57),
            vec2(-5.33, -5.9),
            vec2(-1.3, -10.5),
        ];

        let quats = (0..vertices_2d.len())
            .map(|i| {
                let o2o1 = vertices_2d[(i + 1) % vertices_2d.len()] - vertices_2d[i];
                let rot90 = vec2(-o2o1.y, o2o1.x);

                quat_to(two_to_three(&rot90))
            })
            .collect::<Vec<_>>();

        let colors = [
            (1.0, 0.0, 0.0), // Red
            (0.0, 1.0, 0.0), // Green
            (0.0, 0.0, 1.0), // Blue
            (1.0, 1.0, 0.0), // Yellow
            (1.0, 0.5, 0.5), // Pink
        ];

        for i in 0..vertices_2d.len() {
            let point = two_to_three(&vertices_2d[i]);
            let quat = quats[i];
            let color = colors[i];

            scene.add_plane(Plane::new(point, quat, color).make_reflective(0.5));
        }

        scene.add_plane(
            Plane::new(5.0 * WORLD_UP, Quat::IDENTITY, (0.8, 0.8, 0.8)).make_reflective(0.5),
        );

        scene.add_plane(
            Plane::new(-5.0 * WORLD_UP, Quat::IDENTITY, (0.8, 0.8, 0.8)).make_reflective(0.5),
        );

        scene.add_cube(Cube::new(Vec3::ZERO, 0.5, (0.0, 1.0, 0.0)).make_reflective(0.5));

        scenes.push(scene);
    }

    {
        let mut scene = Scene::new("Primitives");

        scene.add_ellipse(
            Ellipse::new(
                vec3(3.0, 1.5, 0.0),
                Quat::from_euler(XYZ, 0.0, 0.0, FRAC_PI_2),
                (0.7, 0.4, 0.0),
            )
            .make_reflective(0.5),
        );
        scene.add_ellipse(
            Ellipse::new(
                vec3(-3.0, 1.5, 0.0),
                Quat::from_euler(XYZ, 0.0, 0.0, FRAC_PI_2),
                (0.0, 0.4, 0.7),
            )
            .make_reflective(0.5),
        );

        scene.add_plane(
            Plane::new(vec3(3.0, 0.0, 2.5), Quat::IDENTITY, (1.0, 0.2, 0.2))
                .make_finite(5.0, 5.0)
                .make_reflective(0.5),
        );
        scene.add_plane(
            Plane::new(vec3(3.0, 0.0, -2.5), Quat::IDENTITY, (0.6, 0.6, 0.2)).make_finite(5.0, 5.0).make_reflective(0.5),
        );

        scene.add_plane(
            Plane::new(vec3(-3.0, 0.0, 2.5), Quat::IDENTITY, (0.2, 1.0, 0.2)).make_finite(5.0, 5.0).make_reflective(0.5),
        );
        scene.add_plane(
            Plane::new(vec3(-3.0, 0.0, -2.5), Quat::IDENTITY, (0.2, 0.2, 1.0))
                .make_finite(5.0, 5.0)
                .make_reflective(0.5),
        );

        scenes.push(scene);
    }

    scenes
}
