use std::f32::consts::PI;

use nannou::glam::{Vec2, Vec3, vec2, vec3};

use crate::util::WORLD_UP;

pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub speed: f32,
    pub sensitivity: f32,
    pub fov_multiplier: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            position: vec3(0.0, 1.0, 0.0),
            yaw: -PI / 2.0,
            pitch: 0.0,
            speed: 5.0,
            sensitivity: 0.003,
            fov_multiplier: 1.0,
        }
    }

    pub fn forward(&self) -> Vec3 {
        vec3(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
    }

    pub fn right(&self) -> Vec3 {
        vec3(
            (self.yaw - PI / 2.0).cos(),
            0.0,
            (self.yaw - PI / 2.0).sin(),
        )
    }

    pub fn up(&self) -> Vec3 {
        Vec3::Y // eventually `up` will include camera roll
    }

    /// Returns the forward, right, and up vectors of the camera. Used for debug rays. Change this in the new camera orientation fix
    pub fn directions(&self) -> (Vec3, Vec3, Vec3) {
        let forward = self.forward();
        let right = forward.cross(WORLD_UP).normalize();
        let up = right.cross(forward);
        (forward, right, up)
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
        // Transform to camera space
        let relative_pos = world_pos - self.position;

        let camera_forward = self.forward();
        let camera_right = self.shader_camera_right();
        let camera_up = self.shader_camera_up();

        // Project onto camera plane
        let forward_dist = relative_pos.dot(camera_forward);

        // Check if behind camera
        if forward_dist <= 0.01 {
            return None;
        }

        // Project to camera's right/up plane
        let right_offset = relative_pos.dot(camera_right);
        let up_offset = relative_pos.dot(camera_up);

        // Perspective
        let aspect_ratio = screen_size.x / screen_size.y;

        // Convert to UV coordinates like the shader does
        let fov_radians = 2.0 * self.fov_multiplier.atan();
        let uv_x = right_offset / forward_dist / fov_radians / aspect_ratio;
        let uv_y = up_offset / forward_dist / fov_radians;

        // Convert to screen coordinates
        let screen_x = uv_x * screen_size.x * 0.5;
        let screen_y = uv_y * screen_size.y * 0.5;

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
        visible_point: Vec3,
        invisible_point: Vec3,
        camera: &Camera,
        screen_size: Vec2,
    ) -> Option<Vec2> {
        let ray_dir = (invisible_point - visible_point).normalize();
        let screen_bounds = screen_size * 0.5;

        // Sample points along the ray to find screen intersection
        for i in 1..100 {
            let t = i as f32 * 0.1;
            let test_point = visible_point + ray_dir * t;

            if let Some(screen_pos) = camera.world_to_screen_unbounded(test_point, screen_size) {
                // Check if we've reached screen bounds
                if screen_pos.x.abs() >= screen_bounds.x || screen_pos.y.abs() >= screen_bounds.y {
                    // Clamp to screen bounds
                    let clamped_x = screen_pos.x.clamp(-screen_bounds.x, screen_bounds.x);
                    let clamped_y = screen_pos.y.clamp(-screen_bounds.y, screen_bounds.y);
                    return Some(vec2(clamped_x, clamped_y));
                }
            }
        }
        None
    }

    pub fn clip_line_segment_to_screen(
        start: Vec3,
        end: Vec3,
        camera: &Camera,
        screen_size: Vec2,
    ) -> Option<(Vec2, Vec2)> {
        let screen_bounds = vec2(screen_size.x * 0.5, screen_size.y * 0.5);
        let mut clipped_points = Vec::new();

        // Sample points along the line segment
        for i in 0..=50 {
            let t = i as f32 / 50.0;
            let test_point = start + t * (end - start);

            if let Some(screen_pos) = camera.world_to_screen_unbounded(test_point, screen_size) {
                // Check if point is within screen bounds
                if screen_pos.x.abs() <= screen_bounds.x && screen_pos.y.abs() <= screen_bounds.y {
                    clipped_points.push(screen_pos);
                }
            }
        }

        if clipped_points.len() >= 2 {
            Some((clipped_points[0], clipped_points[clipped_points.len() - 1]))
        } else {
            None
        }
    }
}
