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
    reflect: bool,
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

// Calculate reflection direction of an `incident` ray bouncing off a surface with normal `normal`
fn reflect(incident: Vec3, normal: Vec3) -> Vec3 {
    incident - 2.0 * incident.dot(normal) * normal
}

pub fn trace_ray_cpu(scene: &SceneData, ray_origin: Vec3, ray_direction: Vec3) -> HitInfoCpu {
    let mut hit_info = HitInfoCpu {
        hit: false,
        t: 1000.0,
        point: Vec3::ZERO,
        normal: Vec3::ZERO,
        color: [0.0; 3],
        reflect: false,
    };

    for plane in scene.planes.iter() {
        let t = ray_plane_intersect_cpu(ray_origin, ray_direction, *plane, false);

        if t > 0.001 && t < hit_info.t {
            hit_info.hit = true;
            hit_info.t = t;
            hit_info.point = ray_origin + t * ray_direction;
            hit_info.normal = plane.normal();
            hit_info.color = plane.color.into_components().into();
            hit_info.reflect = plane.reflectivity > 0.0;
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
            hit_info.reflect = ellipse.reflectivity > 0.0;
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
                hit_info.reflect = face.reflectivity > 0.0;
            }
        }
    }

    hit_info
}

pub fn trace_debug_ray(
    scene: &SceneData,
    origin: Vec3,
    direction: Vec3,
    max_bounces: u32,
) -> DebugRay {
    let mut segments = Vec::new();
    let mut curr_ray_origin = origin;
    let mut curr_ray_direction = direction;
    let mut length = 0.0;

    for bounce in 0..max_bounces {
        // Trace the ray through the scene
        let hit_info = trace_ray_cpu(scene, curr_ray_origin, curr_ray_direction);

        if hit_info.hit {
            // Record the segment
            let segment = Segment::new_with_bounce(curr_ray_origin, hit_info.point, bounce);
            segments.push(segment);

            // Update the length of the ray
            length += (hit_info.point - curr_ray_origin).length();

            // Prepare for the next bounce
            if hit_info.reflect {
                curr_ray_origin = hit_info.point;
                curr_ray_direction = reflect(curr_ray_direction, hit_info.normal);
            } else {
                break; // Stop if the surface is not reflective
            }
        } else {
            // No hit, extend the ray to a far distance
            let far_point = curr_ray_origin + 1000.0 * curr_ray_direction;
            let segment = Segment::new_with_bounce(curr_ray_origin, far_point, bounce);
            segments.push(segment);
            length += (far_point - curr_ray_origin).length();
            break;
        }
    }

    DebugRay { segments, length }
}

// fn trace_ray(ray: Ray, max_bounces: u32) -> HitInfo {
//     var current_ray = ray;
//     var final_hit_info: HitInfo;
//     final_hit_info.hit = false;
//     final_hit_info.t = 1000.0;
//     final_hit_info.color = vec3<f32>(0.1, 0.2, 0.4);
//     final_hit_info.multiplier = 1.0;
//     final_hit_info.reflectivity = 0.0;

//     let background_color = vec3<f32>(0.1, 0.2, 0.4);
//     var accumulated_color = vec3<f32>(0.0);
//     var throughput = vec3<f32>(1.0);
//     var hit_seen = false;

//     for (var bounce: u32 = 0u; bounce < max_bounces; bounce++) {
//         let hit_info = trace_ray_single_bounce(current_ray);

//         if !hit_info.hit {
//             if hit_seen {
//                 accumulated_color += background_color * throughput;
//                 final_hit_info.hit = true;
//                 final_hit_info.color = accumulated_color;
//                 final_hit_info.multiplier = 1.0;
//             } else {
//                 final_hit_info.hit = false;
//                 final_hit_info.color = background_color;
//                 final_hit_info.multiplier = 1.0;
//             }
//             break;
//         }

//         hit_seen = true;

//         let reflectivity = clamp(hit_info.reflectivity, 0.0, 1.0);
//         accumulated_color += (1.0 - reflectivity) * hit_info.color * throughput;
//         throughput *= reflectivity;

//         if reflectivity <= 0.0001 {
//             final_hit_info = hit_info;
//             final_hit_info.hit = true;
//             final_hit_info.color = accumulated_color;
//             final_hit_info.multiplier = 1.0;
//             break;
//         }

//         let reflected_direction = reflect(current_ray.direction, hit_info.normal);
//         current_ray.origin = hit_info.point + 0.001 * reflected_direction;
//         current_ray.direction = reflected_direction;

//         if bounce == max_bounces - 1u {
//             final_hit_info = hit_info;
//             final_hit_info.hit = true;
//             final_hit_info.color = accumulated_color;
//             final_hit_info.multiplier = 1.0;
//             break;
//         }
//     }

//     if !final_hit_info.hit && hit_seen {
//         final_hit_info.hit = true;
//         final_hit_info.color = accumulated_color;
//         final_hit_info.multiplier = 1.0;
//     }

//     return final_hit_info;
// }
