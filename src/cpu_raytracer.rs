
use nannou::prelude::*;

use crate::{Model, scene::{Ellipse, Plane, Portal, SceneData}};


pub struct DebugRay {
    pub segments: Vec<RaySegment>,
}

struct RaySegment {
    pub start: Vec3,
    pub end: Vec3,
    pub color: [f32; 3],
    segment_type: RaySegmentType,
}

enum RaySegmentType {
    Primary,
    ThroughPortal,
}

struct HitInfoCpu {
    hit: bool,
    t: f32,
    point: Vec3,
    normal: Vec3,
    color: [f32; 3],
}

pub fn shoot_debug_ray(model: &mut Model) {
    let camera = &model.camera;
    let ray_origin = camera.position;
    let ray_direction = camera.forward();

    let debug_ray = trace_debug_ray(
        &model.scenes[model.current_scene as usize],
        ray_origin,
        ray_direction,
        10,
    );

    model.debug_ray = Some(debug_ray);
    model.debug_ray_active = true;
}

fn trace_debug_ray(scene: &SceneData, origin: Vec3, direction: Vec3, max_bounces: u32) -> DebugRay {
    let mut segments = Vec::new();
    let mut current_ray_origin = origin;
    let mut current_ray_direction = direction;

    for bounce in 0..max_bounces {
        let hit_info = trace_ray_cpu(scene, current_ray_origin, current_ray_direction);

        if !hit_info.hit {
            segments.push(RaySegment {
                start: current_ray_origin,
                end: current_ray_origin + current_ray_direction * 20.0,
                color: if bounce == 0 { [1.0, 1.0, 0.0] } else { [0.0, 1.0, 1.0] },
                segment_type: if bounce == 0 { RaySegmentType::Primary } else { RaySegmentType::ThroughPortal },
            });
            break;
        }

        let mut hit_portal = false;
        for i in 0..scene.portal_pair_count {
            let portal_pair = &scene.portal_pairs[i as usize];

            for (in_portal, out_portal) in [
                (&portal_pair.portal_a, &portal_pair.portal_b),
                (&portal_pair.portal_b, &portal_pair.portal_a),
            ] {
                let portal_t = ray_ellipse_intersect_cpu(current_ray_origin, current_ray_direction, in_portal.ellipse);

                if portal_t > 0.001 && portal_t <= hit_info.t + 0.001 {
                    let portal_normal = Vec3::from(in_portal.ellipse.normal);

                    if current_ray_direction.dot(portal_normal) < 0.0 {
                        let portal_hit_point = current_ray_origin + portal_t * current_ray_direction;

                        segments.push(RaySegment {
                            start: current_ray_origin,
                            end: portal_hit_point,
                            color: if bounce == 0 { [1.0, 1.0, 0.0] } else { [0.0, 1.0, 1.0] },
                            segment_type: if bounce == 0 { RaySegmentType::Primary } else { RaySegmentType::ThroughPortal },
                        });

                        let transformed_point = transform_point_through_portal(portal_hit_point, in_portal, out_portal);
                        let transformed_direction = transform_direction_through_portal(current_ray_direction, in_portal, out_portal);

                        current_ray_origin = transformed_point;
                        current_ray_direction = transformed_direction;
                        hit_portal = true;
                        break;
                    }
                }
            }

            if hit_portal {
                break;
            }
        }

        if !hit_portal {
            segments.push(RaySegment {
                start: current_ray_origin,
                end: hit_info.point,
                color: if bounce == 0 { [1.0, 1.0, 0.0] } else { [0.0, 1.0, 1.0] },
                segment_type: if bounce == 0 { RaySegmentType::Primary } else { RaySegmentType::ThroughPortal }, 
            });
            break;
        }
    }

    DebugRay {
        segments,
    }
}

fn trace_ray_cpu(scene: &SceneData, ray_origin: Vec3, ray_direction: Vec3) -> HitInfoCpu {
    let mut hit_info = HitInfoCpu {
        hit: false,
        t: 1000.0,
        point: Vec3::ZERO,
        normal: Vec3::ZERO,
        color: [0.0; 3],
    };

    for i in 0..scene.plane_count {
        let plane = &scene.planes[i as usize];
        let t = ray_plane_intersect_cpu(ray_origin, ray_direction, *plane);

        if t > 0.001 && t < hit_info.t {
            hit_info.hit = true;
            hit_info.t = t;
            hit_info.point = ray_origin + t * ray_direction;
            hit_info.normal = Vec3::from(plane.normal);
            hit_info.color = plane.color;
        }
    }

    for i in 0..scene.ellipse_count {
        let ellipse = &scene.ellipses[i as usize];
        let t = ray_ellipse_intersect_cpu(ray_origin, ray_direction, *ellipse);
        
        if t > 0.001 && t < hit_info.t {
            hit_info.hit = true;
            hit_info.t = t;
            hit_info.point = ray_origin + t * ray_direction;
            hit_info.normal = Vec3::from(ellipse.normal);
            hit_info.color = ellipse.color;
        }
    }

    hit_info
}

fn ray_plane_intersect_cpu(ray_origin: Vec3, ray_direction: Vec3, plane: Plane) -> f32 {
    let plane_point = Vec3::from(plane.point);
    let plane_normal = Vec3::from(plane.normal);
    
    let denom = plane_normal.dot(ray_direction);
    if denom.abs() < 1e-6 {
        return -1.0;
    }
    
    let t = (plane_point - ray_origin).dot(plane_normal) / denom;
    
    // Check finite plane bounds if needed
    if plane.is_infinite < 0.5 {
        let hit_point = ray_origin + t * ray_direction;
        // Add finite plane intersection logic here
    }
    
    t
}

fn ray_ellipse_intersect_cpu(
    ray_origin: Vec3,
    ray_direction: Vec3,
    ellipse: Ellipse
) -> f32 {
    let center = Vec3::from(ellipse.center);
    let normal = Vec3::from(ellipse.normal);

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

    let up = Vec3::Y;
    let u_axis = if normal.dot(up).abs() < 0.9 {
        normal.cross(up).normalize()
    } else {
        normal.cross(Vec3::X).normalize()
    };
    let v_axis = u_axis.cross(normal);

    let u = local_point.dot(u_axis);
    let v = local_point.dot(v_axis);

    let ellipse_test = (u*u) / (ellipse.radius_a * ellipse.radius_a) + 
                       (v*v) / (ellipse.radius_b * ellipse.radius_b);

    if ellipse_test > 1.0 {
        return -1.0;
    }

    t
}


















pub fn check_camera_portal_teleport(
    scene: &SceneData,
    old_pos: Vec3,
    new_pos: Vec3,
) -> Option<Vec3> {
    let movement_vec = new_pos - old_pos;
    let movement_length = movement_vec.length();

    if movement_length < 0.001 {
        return None;
    }

    let ray_direction = movement_vec / movement_length;

    for i in 0..scene.portal_pair_count {
        let portal_pair = &scene.portal_pairs[i as usize];

        if let Some(teleport_pos) = check_single_portal_teleport(
            old_pos,
            ray_direction,
            movement_length,
            &portal_pair.portal_a,
            &portal_pair.portal_b,
        ) {
            return Some(teleport_pos);
        }

        if let Some(teleport_pos) = check_single_portal_teleport(
            old_pos,
            ray_direction,
            movement_length,
            &portal_pair.portal_b,
            &portal_pair.portal_a,
        ) {
            return Some(teleport_pos);
        }
    }

    None
}

fn check_single_portal_teleport(
    ray_origin: Vec3,
    ray_direction: Vec3,
    max_distance: f32,
    in_portal: &Portal,
    out_portal: &Portal,
) -> Option<Vec3> {
    let ellipse = in_portal.ellipse;

    let t = ray_ellipse_intersect_cpu(ray_origin, ray_direction, ellipse);

    if t > 0.001 && t < max_distance {
        let portal_normal = Vec3::from(ellipse.normal);
        if ray_direction.dot(portal_normal) < 0.0 {
            let hit_point = ray_origin + t * ray_direction;
            let remaining_distance = max_distance - t;

            let teleported_point = transform_point_through_portal(
                hit_point,
                in_portal,
                out_portal,
            );

            let transformed_direction = transform_direction_through_portal(
                ray_direction,
                in_portal,
                out_portal,
            );

            return Some(teleported_point + remaining_distance * transformed_direction);
        }
    }

    None
}



fn transform_point_through_portal(
    point: Vec3, 
    in_portal: &Portal, 
    out_portal: &Portal
) -> Vec3 {
    let in_transform = Mat4::from_cols_array(&in_portal.inverse_transformation_matrix);
    let out_transform = Mat4::from_cols_array(&out_portal.transformation_matrix);
    
    let local_point = in_transform.transform_point3(point);
    out_transform.transform_point3(local_point)
}

fn transform_direction_through_portal(
    direction: Vec3, 
    in_portal: &Portal, 
    out_portal: &Portal
) -> Vec3 {
    let in_transform = Mat4::from_cols_array(&in_portal.inverse_transformation_matrix);
    let out_transform = Mat4::from_cols_array(&out_portal.transformation_matrix);
    
    let local_direction = in_transform.transform_vector3(direction);
    out_transform.transform_vector3(local_direction).normalize()
}










