use nannou::prelude::*;

use crate::{
    Model,
    scene::{
        SceneData,
        portal::Portal,
        primitive::{ellipse::Ellipse, plane::Plane},
    },
    util::WORLD_UP,
};

pub struct DebugRayEmitter {
    origin: Vec3,
    directions: (Vec3, Vec3, Vec3), // (forward, right, up)
}

impl DebugRayEmitter {
    pub fn new(origin: Vec3, directions: (Vec3, Vec3, Vec3)) -> Self {
        Self { origin, directions }
    }
}

struct DebugRay {
    pub segments: Vec<Segment>,
}

pub struct Segment {
    pub start: Vec3,
    pub end: Vec3,
    pub color: [f32; 3],
    pub weight: f32,
}

impl Segment {
    pub fn new(start: Vec3, end: Vec3, color: [f32; 3], weight: f32) -> Self {
        Self {
            start,
            end,
            color,
            weight,
        }
    }

    pub fn new_with_bounce(start: Vec3, end: Vec3, bounce: u32) -> Self {
        Self {
            start,
            end,
            color: bounce_color(bounce),
            weight: 3.0,
        }
    }
}

struct HitInfoCpu {
    hit: bool,
    t: f32,
    point: Vec3,
    normal: Vec3,
    color: [f32; 3],
}

enum PortalHitType<'a> {
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

fn trace_ray_cpu(scene: &SceneData, ray_origin: Vec3, ray_direction: Vec3) -> HitInfoCpu {
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

fn bounce_color(bounce_count: u32) -> [f32; 3] {
    match bounce_count {
        // Thanks copilot
        0 => [1.0, 1.0, 0.0], // Yellow for the first segment
        1 => [0.0, 1.0, 0.0], // Green for the second segment
        2 => [0.0, 0.0, 1.0], // Blue for the third segment
        3 => [1.0, 0.0, 1.0], // Magenta for the fourth segment
        4 => [1.0, 0.5, 0.0], // Orange for the fifth segment
        _ => [0.0, 1.0, 1.0], // Cyan for subsequent segments
    }
}

fn trace_debug_ray(scene: &SceneData, origin: Vec3, direction: Vec3, max_bounces: u32) -> DebugRay {
    let mut segments = Vec::new();
    let mut curr_ray_origin = origin;
    let mut curr_ray_direction = direction;

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
                    segments.push(Segment::new_with_bounce(
                        curr_ray_origin,
                        hit_info.point,
                        bounce,
                    ));
                } else {
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

    DebugRay { segments }
}

impl Model {
    pub fn add_debug_ray_emitter(&mut self) {
        let camera = &self.camera;
        let ray_emitter = DebugRayEmitter::new(camera.position, camera.directions());
        self.debug_ray_emitters.push(ray_emitter);
    }

    pub fn draw_debug_ray(&self, draw: &Draw, screen_size: Vec2) {
        let mut debug_rays = Vec::new();

        // Shoots a single ray directly forward from the camera
        for emitter in self.debug_ray_emitters.iter() {
            let ray_direction = emitter.directions.0;
            let debug_ray = trace_debug_ray(
                &self.scenes[self.current_scene].data,
                emitter.origin,
                ray_direction,
                10,
            );
            debug_rays.push(debug_ray);
        }

        // Shoots a spread of rays
        // let m = 0.2;
        // let res_x = 1;
        // let res_y = 3;

        // for emitter in self.debug_ray_emitters.iter() {
        //     let (forward, right, up) = emitter.directions;

        //     for x in 0..res_x {
        //         for y in 0..res_y {
        //             let uv_x = (x as f32 / res_x as f32) * 2.0 * m - m;
        //             let uv_y = (y as f32 / res_y as f32) * 2.0 * m - m;

        //             let ray_direction = (forward + uv_x * right + uv_y * up).normalize();

        //             let debug_ray = trace_debug_ray(
        //                 &self.scenes[self.current_scene].data,
        //                 emitter.origin,
        //                 ray_direction,
        //                 10,
        //             );

        //             debug_rays.push(debug_ray);
        //         }
        //     }
        // }

        for ray in debug_rays.iter() {
            for segment in &ray.segments {
                self.camera.draw_segment(draw, segment, screen_size);
            }

            // Draw the origin of the ray
            if let Some(first_segment) = ray.segments.first()
                && let Some(origin_2d) = self
                    .camera
                    .world_to_screen(first_segment.start, screen_size)
            {
                draw.ellipse()
                    .xy(pt2(origin_2d.x, origin_2d.y))
                    .radius(5.0)
                    .color(RED);
            }
        }
    }
}

pub fn check_camera_portal_teleport(
    scene: &SceneData,
    old_pos: Vec3,
    new_pos: Vec3,
) -> Option<Vec3> {
    let movement_vec = new_pos - old_pos;
    let movement_length = movement_vec.length();

    // if movement_length < 0.001 {
    //     return None;
    // }

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
    let ellipse = in_portal.ellipse();

    let t = ray_ellipse_intersect_cpu(ray_origin, ray_direction, ellipse);

    if t > 0.001 && t < max_distance {
        let portal_normal = ellipse.normal();
        if ray_direction.dot(portal_normal) < 0.0 {
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
