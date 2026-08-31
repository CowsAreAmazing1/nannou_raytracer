use nannou::prelude::*;
use std::f32::consts::{PI, TAU};

use crate::{
    cpu_raytracer::check_camera_portal_teleport,
    scene::SceneData,
    ui::Segment,
    util::{WORLD_FORWARDS, WORLD_FRAME, WORLD_RIGHT, WORLD_UP},
};

pub struct Camera {
    pub position: Vec3,
    pub rotation: Quat,
    pub roll: f32,
    pub speed: f32,
    pub sensitivity: f32,
    pub fov_multiplier: f32,
    pub(crate) use_free_roll_camera: bool,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            position: vec3(0.0, 1.0, 0.0),
            rotation: Quat::from_rotation_y(-PI / 2.0),
            roll: 0.0,
            speed: 5.0,
            sensitivity: 0.003,
            fov_multiplier: 1.0,
            use_free_roll_camera: false,
        }
    }

    pub fn forward(&self) -> Vec3 {
        (self.rotation * WORLD_FORWARDS).normalize()
    }

    pub fn _right(&self) -> Vec3 {
        (self.rotation * WORLD_RIGHT).normalize()
    }

    pub fn _up(&self) -> Vec3 {
        (self.rotation * WORLD_UP).normalize()
    }

    /// Returns the forward, right, and up vectors of the camera. Used for debug rays
    pub fn directions(&self) -> (Vec3, Vec3, Vec3) {
        let (f, r, u) = WORLD_FRAME;
        (self.rotation * f, self.rotation * r, self.rotation * u)
    }

    pub fn movement(&mut self, app: &App, dt: f32, scene_data: &SceneData) {
        let old_position = self.position;
        let dt = dt.clamp(0.0, 0.1);

        let (forward, right, up) = self.directions();

        let mut moved = false;
        let mut movement = Vec3::ZERO;

        if app.keys.down.contains(&Key::W) {
            movement += forward;
            moved = true;
        }
        if app.keys.down.contains(&Key::A) {
            movement -= right;
            moved = true;
        }
        if app.keys.down.contains(&Key::S) {
            movement -= forward;
            moved = true;
        }
        if app.keys.down.contains(&Key::D) {
            movement += right;
            moved = true;
        }
        if app.keys.down.contains(&Key::Space) {
            movement += up;
            moved = true;
        }
        if app.keys.down.contains(&Key::LShift) {
            movement -= up;
            moved = true;
        }

        if moved {
            let movement_length = movement.length_squared();
            if movement_length <= f32::EPSILON || !movement_length.is_finite() {
                return;
            }

            movement = movement / movement_length.sqrt() * self.speed * dt;
            let new_position = self.position + movement;

            if let Some((teleported_pos, teleported_rotation)) =
                check_camera_portal_teleport(scene_data, old_position, new_position, self.rotation)
            {
                self.position = teleported_pos;
                self.rotation = teleported_rotation;
                self.roll = Self::roll_from_rotation(self.rotation);
            } else {
                self.position = new_position;
            }
        }

        let mut rotation = 0.0;
        if app.keys.down.contains(&Key::E) {
            rotation += 1.0;
        }
        if app.keys.down.contains(&Key::Q) {
            rotation -= 1.0;
        }
        if rotation != 0.0 {
            let angle = rotation * dt;
            self.roll += angle;
            self.rotation = Self::rotation_from_forward_roll(self.forward(), self.roll);
        }

        if app.keys.down.contains(&Key::Equals) {
            self.fov_multiplier = (self.fov_multiplier + 0.01).min(3.0);
            // println!("FOV: {:.2}", self.fov_multiplier);
        }
        if app.keys.down.contains(&Key::Minus) {
            self.fov_multiplier = (self.fov_multiplier - 0.01).max(0.1);
            // println!("FOV: {:.2}", self.fov_multiplier);
        }
    }

    pub fn rotate_view(&mut self, pos: Vec2) {
        if self.use_free_roll_camera {
            self.rotate_view_free(pos);
        } else {
            self.rotate_view_rollless(pos);
        }
    }

    fn rotate_view_rollless(&mut self, pos: Vec2) {
        let delta = pos * self.sensitivity;
        let current_orientation = Self::rotation_from_forward_roll(self.forward(), self.roll);
        let yaw_axis = (current_orientation * WORLD_UP).normalize();
        let camera_right = (current_orientation * WORLD_RIGHT).normalize();

        // Reduce yaw input as the view approaches the rolled yaw axis.
        let yaw_scale = (1.0 - self.forward().dot(yaw_axis).powi(2)).max(0.0).sqrt();
        let yaw = Quat::from_axis_angle(yaw_axis, -delta.x * yaw_scale);
        let pitch = Quat::from_axis_angle(camera_right, delta.y);
        let aimed_forward = (pitch * yaw * current_orientation * WORLD_FORWARDS).normalize();

        // Keep the direction away from both frame-construction singularities.
        let max_axis_alignment = 0.95;
        let next_forward = Self::clamp_forward_from_axis(
            Self::clamp_forward_from_axis(
                aimed_forward,
                yaw_axis,
                max_axis_alignment,
                camera_right,
            ),
            WORLD_UP,
            max_axis_alignment,
            camera_right,
        );

        self.rotation = Self::rotation_from_forward_roll(next_forward, self.roll);
    }

    fn clamp_forward_from_axis(forward: Vec3, axis: Vec3, max_dot: f32, fallback: Vec3) -> Vec3 {
        let dot = forward.dot(axis);
        if dot.abs() <= max_dot {
            return forward;
        }

        let perpendicular = forward - axis * dot;
        let perpendicular = if perpendicular.length_squared() > f32::EPSILON {
            perpendicular.normalize()
        } else {
            let fallback = fallback - axis * fallback.dot(axis);
            if fallback.length_squared() > f32::EPSILON {
                fallback.normalize()
            } else {
                WORLD_FORWARDS - axis * WORLD_FORWARDS.dot(axis)
            }
            .normalize()
        };

        perpendicular * (1.0 - max_dot * max_dot).sqrt() + axis * dot.signum() * max_dot
    }

    fn rotation_from_forward_roll(forward: Vec3, roll: f32) -> Quat {
        let forward = forward.normalize();

        // Project world up onto the plane perpendicular to forward to create a no-roll frame.
        let projected_up = WORLD_UP - forward * WORLD_UP.dot(forward);
        let base_up = if projected_up.length_squared() > f32::EPSILON {
            projected_up.normalize()
        } else {
            // World up is parallel to forward, so use world right at the pole instead.
            (WORLD_RIGHT - forward * WORLD_RIGHT.dot(forward)).normalize()
        };

        // Apply only the explicitly stored roll, then complete the orthonormal camera basis.
        let roll_rotation = Quat::from_axis_angle(forward, roll);
        let up = (roll_rotation * base_up).normalize();
        let right = forward.cross(up).normalize();

        Quat::from_mat3(&Mat3::from_cols(forward, up, right)).normalize()
    }

    fn roll_from_rotation(rotation: Quat) -> f32 {
        let forward = (rotation * WORLD_FORWARDS).normalize();

        // Recreate the same no-roll reference frame used by rotation_from_forward_roll.
        let projected_up = WORLD_UP - forward * WORLD_UP.dot(forward);
        let base_up = if projected_up.length_squared() > f32::EPSILON {
            projected_up.normalize()
        } else {
            (WORLD_RIGHT - forward * WORLD_RIGHT.dot(forward)).normalize()
        };
        let base_right = forward.cross(base_up).normalize();
        let camera_up = (rotation * WORLD_UP).normalize();

        // Measure the signed twist of the transformed camera up around forward.
        camera_up.dot(base_right).atan2(camera_up.dot(base_up))
    }

    fn rotate_view_free(&mut self, pos: Vec2) {
        let delta = pos * self.sensitivity;

        // Free quaternion look retained for a future control-mode toggle.
        let yaw_quat = Quat::from_axis_angle(WORLD_UP, -delta.x);
        let pitch_quat = Quat::from_axis_angle(WORLD_RIGHT, delta.y);

        self.rotation = (self.rotation * yaw_quat * pitch_quat).normalize();
    }

    fn shader_camera_right(&self) -> Vec3 {
        (self.rotation * WORLD_RIGHT).normalize()
    }

    fn shader_camera_up(&self) -> Vec3 {
        (self.rotation * WORLD_UP).normalize()
    }

    /// Performs 3D to 2D mapping of the world position of an object for the camera.
    ///
    /// `screen_size` must take the scale_factor into account. (??)
    fn world_to_screen_unbounded(&self, world_pos: Vec3, screen_size: Vec2) -> Option<Vec2> {
        let relative_pos = world_pos - self.position;

        let camera_forward = self.forward();
        let camera_right = self.shader_camera_right();
        let camera_up = self.shader_camera_up();

        let forward_dist = relative_pos.dot(camera_forward);

        // Check if behind camera
        if forward_dist <= 0.01 {
            return None;
        }

        // Project to camera's right/up plane
        let right_offset = relative_pos.dot(camera_right);
        let up_offset = relative_pos.dot(camera_up);

        let aspect_ratio = screen_size.x / screen_size.y;

        let scale = self.fov_multiplier.atan();

        let x = right_offset / forward_dist / scale;
        let y = up_offset / forward_dist / scale;

        // UVs
        let ndc_x = x / aspect_ratio;
        let ndc_y = y;

        // Convert to screen coordinates
        let screen_x = ndc_x * screen_size.x * 0.5;
        let screen_y = ndc_y * screen_size.y * 0.5;

        Some(vec2(screen_x, screen_y))
    }

    /// Clips the output of the camera's 3D to 2D map to the screen, returning None if the transformed `world_pos` does not fit on the screen
    pub fn world_to_screen(&self, world_pos: Vec3, screen_size: Vec2) -> Option<Vec2> {
        self.world_to_screen_unbounded(world_pos, screen_size)
            .and_then(|screen_pos| {
                // Check bounds
                if screen_pos.x.abs() > screen_size.x * 0.5
                    || screen_pos.y.abs() > screen_size.y * 0.5
                {
                    return None;
                }

                Some(screen_pos)
            })
    }

    pub fn clip_ray_to_screen(
        &self,
        visible_point: Vec3,
        invisible_point: Vec3,
        screen_size: Vec2,
    ) -> Option<Vec2> {
        let ray_dir = (invisible_point - visible_point).normalize();
        let screen_bounds = screen_size * 0.5;

        // Sample points along the ray to find screen intersection
        for i in 1..100 {
            let t = i as f32 * 0.01;
            let test_point = visible_point + ray_dir * t;

            // If the point is in front of the camera but outside the screen bounds
            // Once this happens the rest of the points will not be on screen
            if let Some(screen_pos) = self.world_to_screen_unbounded(test_point, screen_size)
                && screen_pos.x.abs() >= screen_bounds.x
                && screen_pos.y.abs() >= screen_bounds.y
            {
                // Check if we've reached screen bounds
                let clamped_x = screen_pos.x.clamp(-screen_bounds.x, screen_bounds.x);
                let clamped_y = screen_pos.y.clamp(-screen_bounds.y, screen_bounds.y);

                return Some(vec2(clamped_x, clamped_y));
            }
            //  else if screen_pos.x.abs() <= screen_bounds.x {
            //     let clamped_x = screen_pos.x.clamp(-screen_bounds.x, screen_bounds.x);

            //     return Some(vec2(clamped_x, screen_pos.y));
            // } else if screen_pos.y.abs() <= screen_bounds.y {
            //     screen_pos.y.clamp(-screen_bounds.y, screen_bounds.y)

            //     return Some(vec2(screen_pos.x, clamped_y));
            // }
        }
        None
    }

    pub fn clip_line_segment_to_screen(
        &self,
        start: Vec3,
        end: Vec3,
        screen_size: Vec2,
    ) -> Option<(Vec2, Vec2)> {
        let screen_bounds = vec2(screen_size.x * 0.5, screen_size.y * 0.5);
        let mut clipped_points = Vec::new();

        let offset = end - start;

        // Sample points along the line segment
        for i in 0..=50 {
            let t = i as f32 / 50.0;
            let test_point = start + t * offset;

            // Project the point ..
            if let Some(screen_pos) = self.world_to_screen_unbounded(test_point, screen_size)
                // .. and check if point is within screen bounds
                && screen_pos.x.abs() <= screen_bounds.x
                && screen_pos.y.abs() <= screen_bounds.y
            {
                clipped_points.push(screen_pos);
            }
        }

        if clipped_points.len() >= 2 {
            Some((
                *clipped_points.first().unwrap(),
                *clipped_points.last().unwrap(),
            ))
        } else {
            None
        }
    }

    pub fn draw_segment(&self, draw: &Draw, segment: &Segment, screen_size: Vec2) {
        // Try to get screen positions for both points
        let start_2d = self.world_to_screen_unbounded(segment.start, screen_size);
        let end_2d = self.world_to_screen_unbounded(segment.end, screen_size);

        // Handle different visibility cases
        match (start_2d, end_2d) {
            // Both points visible - draw normally
            (Some(start), Some(end)) => {
                draw.line()
                    .start(start)
                    .end(end)
                    .color(Srgb::from_components(segment.color.into()))
                    .weight(segment.weight);
            }
            // Only start visible - clip to screen edge
            (Some(start), None) => {
                if let Some(clipped_end) =
                    self.clip_ray_to_screen(segment.start, segment.end, screen_size)
                {
                    draw.line()
                        .start(start)
                        .end(clipped_end)
                        .color(Srgb::from_components(segment.color.into()))
                        .weight(segment.weight);
                }
            }
            // Only end visible - clip from screen edge
            (None, Some(end)) => {
                if let Some(clipped_start) =
                    self.clip_ray_to_screen(segment.end, segment.start, screen_size)
                {
                    draw.line()
                        .start(clipped_start)
                        .end(end)
                        .color(Srgb::from_components(segment.color.into()))
                        .weight(segment.weight);
                }
            }
            // Neither visible - try to find screen intersection
            (None, None) => {
                if let Some((clipped_start, clipped_end)) =
                    self.clip_line_segment_to_screen(segment.start, segment.end, screen_size)
                {
                    draw.line()
                        .start(clipped_start)
                        .end(clipped_end)
                        .color(Srgb::from_components(segment.color.into()))
                        .weight(segment.weight);
                }
            }
        }
    }

    pub fn draw_ring(&self, draw: &Draw, screen_size: Vec2) {
        let points = (0..100)
            .map(|i| {
                let rad = i as f32 / 100.0 * TAU;
                vec3(rad.cos(), rad.sin(), 0.0)
            })
            .collect::<Vec<_>>();

        (0..100).for_each(|i| {
            let next_i = (i + 1) % 100;
            let p1 = points[i];
            let p2 = points[next_i];

            let segment = Segment::new(p1, p2, [1.0, 1.0, 1.0], 2.0);
            self.draw_segment(draw, &segment, screen_size);
        });
    }
}
