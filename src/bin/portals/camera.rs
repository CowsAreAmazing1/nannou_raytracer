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
    pub speed: f32,
    pub sensitivity: f32,
    pub fov_multiplier: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            position: vec3(0.0, 1.0, 0.0),
            rotation: Quat::from_rotation_y(-PI / 2.0),
            speed: 5.0,
            sensitivity: 0.003,
            fov_multiplier: 1.0,
        }
    }

    pub fn forward(&self) -> Vec3 {
        (self.rotation * WORLD_FORWARDS).normalize()
    }

    pub fn right(&self) -> Vec3 {
        (self.rotation * WORLD_RIGHT).normalize()
    }

    pub fn up(&self) -> Vec3 {
        (self.rotation * WORLD_UP).normalize()
    }

    /// Returns the forward, right, and up vectors of the camera. Used for debug rays
    pub fn directions(&self) -> (Vec3, Vec3, Vec3) {
        let (f, r, u) = WORLD_FRAME;
        (self.rotation * f, self.rotation * r, self.rotation * u)
    }

    pub fn movement(&mut self, app: &App, dt: f32, scene_data: &SceneData) {
        let old_position = self.position;

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
            movement = movement.normalize() * self.speed * dt;
            let new_position = self.position + movement;

            if let Some(teleported_pos) =
                check_camera_portal_teleport(scene_data, old_position, new_position)
            {
                self.position = teleported_pos;
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
            let angle = rotation * self.sensitivity;
            let roll_quat = Quat::from_axis_angle(self.forward(), angle);

            println!("{:?}", roll_quat);

            self.rotation = roll_quat * self.rotation;
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
        let delta = pos * self.sensitivity;

        let yaw_quat = Quat::from_axis_angle(self.up(), -delta.x);
        let pitch_quat = Quat::from_axis_angle(self.right(), delta.y);

        self.rotation = yaw_quat * pitch_quat * self.rotation;
    }

    fn shader_camera_right(&self) -> Vec3 {
        let camera_forward = self.forward();
        camera_forward.cross(WORLD_UP).normalize()
    }

    fn shader_camera_up(&self) -> Vec3 {
        let camera_right = self.shader_camera_right();
        let camera_forward = self.forward();
        camera_right.cross(camera_forward)
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
