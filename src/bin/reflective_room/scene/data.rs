use nannou::glam::{Quat, Vec2, Vec3, vec2, vec3};

use crate::{
    scene::{
        Scene,
        primitive::{cube::Cube, plane::Plane},
    },
    util::{WORLD_UP, quat_to},
};

pub fn create_scenes() -> Vec<Scene> {
    let mut scenes = Vec::new();

    {
        let mut scene = Scene::new("test");

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

            scene.add_plane(Plane::new_reflective(point, quat, color, 0.5));
        }

        scene.add_plane(Plane::new_reflective(
            5.0 * WORLD_UP,
            Quat::IDENTITY,
            (0.8, 0.8, 0.8),
            0.5,
        ));
        scene.add_plane(Plane::new_reflective(
            -5.0 * WORLD_UP,
            Quat::IDENTITY,
            (0.8, 0.8, 0.8),
            0.5,
        ));

        scene.add_cube(Cube::new(Vec3::ZERO, 0.5, (0.0, 1.0, 0.0)));

        scenes.push(scene);
    }

    scenes
}
