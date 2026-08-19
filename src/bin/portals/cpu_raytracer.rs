use nannou::prelude::*;

use crate::{
    scene::{
        SceneData,
        portal::Portal,
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

pub enum PortalHitType<'a> {
    Front(&'a Portal, &'a Portal),
    Back,
    None,
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

        // Check for portal intersections
        let mut closest_portal_t = hit_info.t;
        let mut portal_hit_type = PortalHitType::None;

        // Check all portal pairs for intersections, to find the closest portal hit, if any
        for i in 0..scene.portal_pair_count {
            let portal_pair = &scene.portal_pairs[i as usize];

            let p_a = &portal_pair.portal_a;
            let p_b = &portal_pair.portal_b;

            // Check portal A
            let t_a = ray_ellipse_intersect_cpu(curr_ray_origin, curr_ray_direction, p_a.ellipse());
            if t_a > 0.001 && t_a < closest_portal_t {
                closest_portal_t = t_a;

                portal_hit_type = if curr_ray_direction.dot(p_a.normal()) < 0.0 {
                    PortalHitType::Front(p_a, p_b)
                } else {
                    PortalHitType::Back
                };
            }

            // Check portal B
            let t_b = ray_ellipse_intersect_cpu(curr_ray_origin, curr_ray_direction, p_b.ellipse());
            if t_b > 0.001 && t_b < closest_portal_t {
                closest_portal_t = t_b;

                portal_hit_type = if curr_ray_direction.dot(p_b.normal()) > 0.0 {
                    PortalHitType::Front(p_b, p_a)
                } else {
                    PortalHitType::Back
                };
            }
        }

        match portal_hit_type {
            // Hit the front side of a portal; draw the segment to the hit point, teleport the ray, and continue bouncing
            PortalHitType::Front(portal_a, portal_b) => {
                let portal_hit_point = curr_ray_origin + closest_portal_t * curr_ray_direction;

                length += closest_portal_t;
                segments.push(Segment::new_with_bounce(
                    curr_ray_origin,
                    portal_hit_point,
                    bounce,
                ));

                curr_ray_origin =
                    transform_point_through_portal(portal_hit_point, portal_a, portal_b);
                curr_ray_direction =
                    transform_direction_through_portal(curr_ray_direction, portal_a, portal_b);
            }

            // Hit the back side of a portal; draw the segment to the hit point and break the bounce loop
            PortalHitType::Back => {
                let portal_hit_point = curr_ray_origin + closest_portal_t * curr_ray_direction;
                length += closest_portal_t;
                segments.push(Segment::new_with_bounce(
                    curr_ray_origin,
                    portal_hit_point,
                    bounce,
                ));
                break;
            }

            // No portal hit; draw the segment to the scene hit point and break the bounce loop
            PortalHitType::None => {
                if hit_info.hit {
                    length += hit_info.t;
                    segments.push(Segment::new_with_bounce(
                        curr_ray_origin,
                        hit_info.point,
                        bounce,
                    ));
                } else {
                    length += 20.0;
                    segments.push(Segment::new_with_bounce(
                        curr_ray_origin,
                        curr_ray_origin + 20.0 * curr_ray_direction,
                        bounce,
                    ));
                }
                break;
            }
        }
    }

    DebugRay { segments, length }
}

fn transform_point_through_portal(point: Vec3, in_portal: &Portal, out_portal: &Portal) -> Vec3 {
    let in_mat = in_portal.inverse_transformation_matrix;
    let out_mat = out_portal.transformation_matrix;

    out_mat.project_point3(in_mat.project_point3(point))
}

fn transform_direction_through_portal(
    direction: Vec3,
    in_portal: &Portal,
    out_portal: &Portal,
) -> Vec3 {
    let in_mat = in_portal.inverse_transformation_matrix;
    let out_mat = out_portal.transformation_matrix;

    out_mat
        .transform_vector3(in_mat.transform_vector3(direction))
        .normalize()
}

pub fn check_camera_portal_teleport(
    scene: &SceneData,
    old_pos: Vec3,
    new_pos: Vec3,
) -> Option<Vec3> {
    let movement_vec = new_pos - old_pos;
    let movement_length = movement_vec.length();

    let ray_direction = movement_vec / movement_length;

    for pair in scene.portal_pairs.iter() {
        if let Some(teleport_pos) = check_single_portal_teleport(
            old_pos,
            ray_direction,
            movement_length,
            &pair.portal_a,
            &pair.portal_b,
            |d| d < 0.0,
        ) {
            return Some(teleport_pos);
        }

        if let Some(teleport_pos) = check_single_portal_teleport(
            old_pos,
            ray_direction,
            movement_length,
            &pair.portal_b,
            &pair.portal_a,
            |d| d > 0.0,
        ) {
            return Some(teleport_pos);
        }
    }

    None
}

/// Returns the new position of the "ray" after teleportation, if it intersects a portal. Otherwise, returns None.
/// `comp` is a comparison function used to determine if the "ray" is approaching the portal from the correct side.
/// This function is currently only used for camera teleports.
fn check_single_portal_teleport(
    ray_origin: Vec3,
    ray_direction: Vec3,
    max_distance: f32,
    in_portal: &Portal,
    out_portal: &Portal,
    comp: fn(f32) -> bool,
) -> Option<Vec3> {
    let ellipse = in_portal.ellipse();
    let t = ray_ellipse_intersect_cpu(ray_origin, ray_direction, ellipse);

    if t > 0.001 && t < max_distance {
        let portal_normal = ellipse.normal();
        if comp(ray_direction.dot(portal_normal)) {
            let hit_point = ray_origin + t * ray_direction;
            let remaining_distance = max_distance - t;

            let teleported_point = transform_point_through_portal(hit_point, in_portal, out_portal);

            let transformed_direction =
                transform_direction_through_portal(ray_direction, in_portal, out_portal);

            return Some(teleported_point + remaining_distance * transformed_direction);
        }
    }

    None
}
