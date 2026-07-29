use nannou::prelude::*;

use crate::{
    Camera, Model,
    scene::{
        SceneData,
        portal::Portal,
        primitive::{ellipse::Ellipse, plane::Plane},
    },
    util::WORLD_UP,
};

pub struct DebugRay {
    pub segments: Vec<RaySegment>,
}

pub struct RaySegment {
    pub start: Vec3,
    pub end: Vec3,
    pub color: [f32; 3],
}

struct HitInfoCpu {
    hit: bool,
    t: f32,
    point: Vec3,
    normal: Vec3,
    color: [f32; 3],
}

impl Model {
    pub fn shoot_debug_ray(&mut self) {
        let camera = &self.camera;

        let camera_forward = camera.forward();
        let camera_right = camera_forward.cross(WORLD_UP).normalize();
        let camera_up = camera_right.cross(camera_forward);

        let mut debug_rays = vec![];

        let m = 0.2;
        let res_x = 8;
        let res_y = 8;

        for x in 0..res_x {
            for y in 0..res_y {
                let uv_x = (x as f32 / res_x as f32) * 2.0 * m - m;
                let uv_y = (y as f32 / res_y as f32) * 2.0 * m - m;

                let ray_direction = (camera_forward
                    + uv_x * camera_right * camera.fov_multiplier
                    + uv_y * camera_up * camera.fov_multiplier)
                    .normalize();

                let debug_ray = trace_debug_ray(
                    &self.scenes[self.current_scene].data,
                    camera.position,
                    ray_direction,
                    10,
                );

                debug_rays.push(debug_ray);
            }
        }

        self.debug_rays.append(&mut debug_rays);
    }

    pub fn draw_debug_ray(&self, draw: &Draw, screen_size: Vec2) {
        for ray in self.debug_rays.iter() {
            for segment in &ray.segments {
                // Try to get screen positions for both points
                let start_2d = self.camera.world_to_screen(segment.start, screen_size);
                let end_2d = self.camera.world_to_screen(segment.end, screen_size);

                // Handle different visibility cases
                match (start_2d, end_2d) {
                    // Both points visible - draw normally
                    (Some(start), Some(end)) => {
                        draw.line()
                            .start(pt2(start.x, start.y))
                            .end(pt2(end.x, end.y))
                            .color(rgb(segment.color[0], segment.color[1], segment.color[2]))
                            .weight(3.0);
                    }
                    // Only start visible - clip to screen edge
                    (Some(start), None) => {
                        if let Some(clipped_end) = Camera::clip_ray_to_screen(
                            segment.start,
                            segment.end,
                            &self.camera,
                            screen_size,
                        ) {
                            draw.line()
                                .start(pt2(start.x, start.y))
                                .end(pt2(clipped_end.x, clipped_end.y))
                                .color(rgb(segment.color[0], segment.color[1], segment.color[2]))
                                .weight(3.0);
                        }
                    }
                    // Only end visible - clip from screen edge
                    (None, Some(end)) => {
                        if let Some(clipped_start) = Camera::clip_ray_to_screen(
                            segment.end,
                            segment.start,
                            &self.camera,
                            screen_size,
                        ) {
                            draw.line()
                                .start(pt2(clipped_start.x, clipped_start.y))
                                .end(pt2(end.x, end.y))
                                .color(rgb(segment.color[0], segment.color[1], segment.color[2]))
                                .weight(3.0);
                        }
                    }
                    // Neither visible - try to find screen intersection
                    (None, None) => {
                        if let Some((clipped_start, clipped_end)) =
                            Camera::clip_line_segment_to_screen(
                                segment.start,
                                segment.end,
                                &self.camera,
                                screen_size,
                            )
                        {
                            draw.line()
                                .start(pt2(clipped_start.x, clipped_start.y))
                                .end(pt2(clipped_end.x, clipped_end.y))
                                .color(rgb(segment.color[0], segment.color[1], segment.color[2]))
                                .weight(3.0);
                        }
                    }
                }
            }

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
                color: if bounce == 0 {
                    [1.0, 1.0, 0.0]
                } else {
                    [0.0, 1.0, 1.0]
                },
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
                let portal_t = ray_ellipse_intersect_cpu(
                    current_ray_origin,
                    current_ray_direction,
                    in_portal.ellipse,
                );

                if portal_t > 0.001 && portal_t <= hit_info.t + 0.001 {
                    let portal_normal = Vec3::from(in_portal.ellipse.normal);

                    if current_ray_direction.dot(portal_normal) < 0.0 {
                        let portal_hit_point =
                            current_ray_origin + portal_t * current_ray_direction;

                        segments.push(RaySegment {
                            start: current_ray_origin,
                            end: portal_hit_point,
                            color: if bounce == 0 {
                                [1.0, 1.0, 0.0]
                            } else {
                                [0.0, 1.0, 1.0]
                            },
                        });

                        let transformed_point =
                            transform_point_through_portal(portal_hit_point, in_portal, out_portal);
                        let transformed_direction = transform_direction_through_portal(
                            current_ray_direction,
                            in_portal,
                            out_portal,
                        );

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
                color: if bounce == 0 {
                    [1.0, 1.0, 0.0]
                } else {
                    [0.0, 1.0, 1.0]
                },
            });
            break;
        }
    }

    DebugRay { segments }
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
            hit_info.normal = plane.normal();
            hit_info.color = plane.color.into_components().into();
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
    let plane_point = plane.point;
    let plane_normal = plane.normal();

    let denom = plane_normal.dot(ray_direction);
    if denom.abs() < 1e-6 {
        return -1.0;
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

            let teleported_point = transform_point_through_portal(hit_point, in_portal, out_portal);

            let transformed_direction =
                transform_direction_through_portal(ray_direction, in_portal, out_portal);

            return Some(teleported_point + remaining_distance * transformed_direction);
        }
    }

    None
}

fn transform_point_through_portal(point: Vec3, in_portal: &Portal, out_portal: &Portal) -> Vec3 {
    let in_transform = Mat4::from_cols_array(&in_portal.inverse_transformation_matrix);
    let out_transform = Mat4::from_cols_array(&out_portal.transformation_matrix);

    let local_point = in_transform.transform_point3(point);
    out_transform.transform_point3(local_point)
}

fn transform_direction_through_portal(
    direction: Vec3,
    in_portal: &Portal,
    out_portal: &Portal,
) -> Vec3 {
    let in_transform = Mat4::from_cols_array(&in_portal.inverse_transformation_matrix);
    let out_transform = Mat4::from_cols_array(&out_portal.transformation_matrix);

    let local_direction = in_transform.transform_vector3(direction);
    out_transform.transform_vector3(local_direction).normalize()
}
