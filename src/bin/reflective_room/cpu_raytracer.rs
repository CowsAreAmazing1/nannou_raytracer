use nannou::prelude::*;

use crate::{
    scene::{
        SceneData,
        primitive::{ellipse::Ellipse, plane::Plane},
    },
    ui::Segment,
    util::WORLD_UP,
};

pub struct DebugRay {
    pub segments: Vec<Segment>,
    pub length: f32,
}

pub struct HitInfoCpu {
    hit: bool,
    pub(crate) t: f32,
    point: Vec3,
    normal: Vec3,
    color: [f32; 3],
}

fn ray_plane_intersect_cpu(
    ray_origin: Vec3,
    ray_direction: Vec3,
    plane: Plane,
    one_way: bool,
) -> f32 {
    let plane_point = plane.point;
    let plane_normal = plane.normal();

    let denom = plane_normal.dot(ray_direction);
    if denom.abs() < 1e-6 {
        return -1.0;
    }
    if one_way && denom > 0.0 {
        return -1.0; // Ray hit the back side of the plane
    }

    let t = (plane_point - ray_origin).dot(plane_normal) / denom;

    // Check finite plane bounds if needed
    if !plane.is_infinite {
        let hit_point = ray_origin + t * ray_direction;
        let local_point = hit_point - plane_point;
        // Add finite plane intersection logic here

        let u_axis = if plane_normal.dot(WORLD_UP).abs() < 0.9 {
            plane_normal.cross(WORLD_UP).normalize()
        } else {
            plane_normal.cross(Vec3::X).normalize()
        };
        let v_axis = plane_normal.cross(u_axis);

        let u = local_point.dot(u_axis);
        let v = local_point.dot(v_axis);

        if u.abs() > plane.width * 0.5 || v.abs() > plane.height * 0.5 {
            return -1.0; // Outside bounds
        }
    }

    t
}

fn ray_ellipse_intersect_cpu(ray_origin: Vec3, ray_direction: Vec3, ellipse: Ellipse) -> f32 {
    let center = ellipse.center;
    let normal = ellipse.normal();

    let denom = normal.dot(ray_direction);
    if denom.abs() < 1e-6 {
        return -1.0;
    }

    let t = (center - ray_origin).dot(normal) / denom;
    if t < 0.0 {
        return -1.0;
    }

    let hit_point = ray_origin + t * ray_direction;
    let local_point = hit_point - center;

    let u_axis = if normal.dot(WORLD_UP).abs() < 0.9 {
        normal.cross(WORLD_UP).normalize()
    } else {
        normal.cross(Vec3::X).normalize()
    };
    let v_axis = u_axis.cross(normal);

    let u = local_point.dot(u_axis);
    let v = local_point.dot(v_axis);

    let ellipse_test = (u * u) / (ellipse.radius_a * ellipse.radius_a)
        + (v * v) / (ellipse.radius_b * ellipse.radius_b);

    if ellipse_test > 1.0 {
        return -1.0;
    }

    t
}

pub fn trace_ray_cpu(scene: &SceneData, ray_origin: Vec3, ray_direction: Vec3) -> HitInfoCpu {
    let mut hit_info = HitInfoCpu {
        hit: false,
        t: 1000.0,
        point: Vec3::ZERO,
        normal: Vec3::ZERO,
        color: [0.0; 3],
    };

    for plane in scene.planes.iter() {
        let t = ray_plane_intersect_cpu(ray_origin, ray_direction, *plane, false);

        if t > 0.001 && t < hit_info.t {
            hit_info.hit = true;
            hit_info.t = t;
            hit_info.point = ray_origin + t * ray_direction;
            hit_info.normal = plane.normal();
            hit_info.color = plane.color.into_components().into();
        }
    }

    for ellipse in scene.ellipses.iter() {
        let t = ray_ellipse_intersect_cpu(ray_origin, ray_direction, *ellipse);

        if t > 0.001 && t < hit_info.t {
            hit_info.hit = true;
            hit_info.t = t;
            hit_info.point = ray_origin + t * ray_direction;
            hit_info.normal = ellipse.normal();
            hit_info.color = ellipse.color.into_components().into();
        }
    }

    for cube in scene.cubes.iter() {
        for face in cube.planes() {
            let t = ray_plane_intersect_cpu(ray_origin, ray_direction, face, true);

            if t > 0.001 && t < hit_info.t {
                hit_info.hit = true;
                hit_info.t = t;
                hit_info.point = ray_origin + t * ray_direction;
                hit_info.normal = face.normal();
                hit_info.color = face.color.into_components().into();
            }
        }
    }

    hit_info
}

pub fn trace_debug_ray(scene: &SceneData, origin: Vec3, direction: Vec3) -> DebugRay {
    let hit_info = trace_ray_cpu(scene, origin, direction);
    let segment = Segment::new(origin, hit_info.point, [1.0, 1.0, 0.0], 3.0);

    DebugRay {
        segments: vec![segment],
        length: hit_info.t,
    }
}
