use nannou::{
    Draw,
    color::RED,
    glam::{Vec2, Vec3},
};

use crate::{Model, cpu_raytracer::trace_debug_ray};

pub struct DebugRayEmitter {
    origin: Vec3,
    directions: (Vec3, Vec3, Vec3), // (forward, right, up)
}

impl DebugRayEmitter {
    pub fn new(origin: Vec3, directions: (Vec3, Vec3, Vec3)) -> Self {
        Self { origin, directions }
    }
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
            color: Self::bounce_color(bounce),
            weight: 3.0,
        }
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
                draw.ellipse().xy(origin_2d).radius(5.0).color(RED);
            }
        }
    }

    pub fn draw_look_ellipse(&self, draw: &Draw, screen_size: Vec2) {
        let origin = self.camera.position;
        let direction = self.camera.forward();

        let dr = trace_debug_ray(&self.scenes[self.current_scene].data, origin, direction, 1);

        if let Some(end_pos) = dr.segments.last().map(|s| s.end)
            && let Some(end_pos_2d) = self.camera.world_to_screen(end_pos, screen_size)
        {
            draw.ellipse()
                .xy(end_pos_2d)
                .radius(5.0 / dr.length)
                .color(RED);
        }
    }
}
