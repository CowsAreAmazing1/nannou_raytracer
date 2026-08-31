

struct Uniforms {
    resolution: vec2<f32>,
    time: f32,
    scene_id: u32,
    camera_pos: vec3<f32>,
    fov: f32,
    camera_dir: vec3<f32>,
    _padding: f32,
}

struct Plane {
    point: vec3<f32>,
    _padding1: f32,
    normal: vec3<f32>,
    _padding2: f32,
    color: vec3<f32>,
    _padding3: f32,
    width: f32,
    height: f32,
    is_infinite: f32, // 0.0 for finite, 1.0 for infinite
    reflectivity: f32,
}

struct Ellipse {
    center: vec3<f32>,
    _padding1: f32,
    normal: vec3<f32>,
    _padding2: f32,
    radius_a: f32,
    radius_b: f32,
    border_thickness: f32,
    _padding3: f32,
    color: vec3<f32>,
    _padding4: f32,
    border_color: vec3<f32>,
    reflectivity: f32,
}

struct Cube {
    planes: array<Plane, 6>,
}

struct SceneData {
    plane_count: u32,
    ellipse_count: u32,
    cube_count: u32,
    max_bounces: u32,
    planes: array<Plane, 10>,
    ellipses: array<Ellipse, 4>,
    cubes: array<Cube, 4>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var<uniform> scene: SceneData;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    // Generate a full-screen triangle ?? it actually is lol
    let x = f32((vertex_index << 1u) & 2u) * 2.0 - 1.0;
    let y = f32(vertex_index & 2u) * 2.0 - 1.0;
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>(x, y) * 0.5 + 0.5;
    return out;
}

//////////////////////////////
//   Objects and Ray Tracing Functions
struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
}

fn ray_plane_intersect(ray: Ray, plane: Plane) -> f32 {
    let denom = dot(plane.normal, ray.direction);
    if abs(denom) < 1e-6 {
        return -1.0;
    }
    let t = dot(plane.point - ray.origin, plane.normal) / denom;
    if t < 0.0 {
        return -1.0;
    }

    // Check if plane is finite
    if plane.is_infinite < 0.5 {
        let hit_point = ray.origin + t * ray.direction;
        let local_point = hit_point - plane.point;

        // Create local coordinate system for the plane
        let up = vec3<f32>(0.0, 1.0, 0.0);
        var u_axis: vec3<f32>;
        var v_axis: vec3<f32>;

        if abs(dot(plane.normal, up)) < 0.9 {
            u_axis = normalize(cross(plane.normal, up));
        } else {
            u_axis = normalize(cross(plane.normal, vec3<f32>(1.0, 0.0, 0.0)));
        }
        v_axis = cross(plane.normal, u_axis);

        let u = dot(local_point, u_axis);
        let v = dot(local_point, v_axis);

        // Check bounds
        if abs(u) > plane.width * 0.5 || abs(v) > plane.height * 0.5 {
            return -1.0; // Outside bounds
        }
    }

    return t;
}

fn ray_plane_intersect_one_way(ray: Ray, plane: Plane) -> f32 {
    if dot(ray.direction, plane.normal) > 0.0 {
        return -1.0; // Ray is going away from the plane
    }

    let denom = dot(plane.normal, ray.direction);
    if abs(denom) < 1e-6 {
        return -1.0;
    }
    let t = dot(plane.point - ray.origin, plane.normal) / denom;
    if t < 0.0 {
        return -1.0;
    }

    // Check if plane is finite
    if plane.is_infinite < 0.5 {
        let hit_point = ray.origin + t * ray.direction;
        let local_point = hit_point - plane.point;

        // Create local coordinate system for the plane
        let up = vec3<f32>(0.0, 1.0, 0.0);
        var u_axis: vec3<f32>;
        var v_axis: vec3<f32>;

        if abs(dot(plane.normal, up)) < 0.9 {
            u_axis = normalize(cross(plane.normal, up));
        } else {
            u_axis = normalize(cross(plane.normal, vec3<f32>(1.0, 0.0, 0.0)));
        }
        v_axis = cross(plane.normal, u_axis);

        let u = dot(local_point, u_axis);
        let v = dot(local_point, v_axis);

        // Check bounds
        if abs(u) > plane.width * 0.5 || abs(v) > plane.height * 0.5 {
            return -1.0; // Outside bounds
        }
    }

    return t;
}

fn add_checkerboard_pattern(ray: Ray, plane: Plane, t: f32, hit_info: ptr<function, HitInfo>, checker_scale: f32) {

    if t > 0.001 && t < (*hit_info).t {
        (*hit_info).hit = true;
        (*hit_info).t = t;
        (*hit_info).point = ray.origin + t * ray.direction;
        (*hit_info).normal = plane.normal;
        (*hit_info).color = plane.color;

        // Create local coordinate system for the plane
        let hit_point = (*hit_info).point;
        let local_point = hit_point - plane.point;

        let up = vec3<f32>(0.0, 1.0, 0.0);
        var u_axis: vec3<f32>;
        var v_axis: vec3<f32>;

        if abs(dot(plane.normal, up)) < 0.9 {
            u_axis = normalize(cross(plane.normal, up));
        } else {
            u_axis = normalize(cross(plane.normal, vec3<f32>(1.0, 0.0, 0.0)));
        }
        v_axis = cross(plane.normal, u_axis);

        // Project hit point onto plane's local coordinates
        let u = dot(local_point, u_axis);
        let v = dot(local_point, v_axis);

        // Apply checkerboard pattern using local coordinates
        let checker_u = floor(u / checker_scale + 0.5);
        let checker_v = floor(v / checker_scale + 0.5);
        let sum = checker_u + checker_v;
        let checker_pattern = abs(sum - 2.0 * floor(sum * 0.5));

        if checker_pattern < 0.5 {
            (*hit_info).color = plane.color;
        } else {
            (*hit_info).color = plane.color * 0.5; //  - vec3<f32>(0.25, 0.25, 0.25);
        }

        (*hit_info).reflectivity = plane.reflectivity;
    }
}

fn ray_ellipse_intersect(ray: Ray, ellipse: Ellipse) -> f32 {
    // First, intersect with the plane containing the ellipse
    let plane = Plane(ellipse.center, 0.0, ellipse.normal, 0.0, ellipse.color, 0.0, 0.0, 0.0, 1.0, 0.0);
    let t = ray_plane_intersect(ray, plane);

    if t < 0.0 {
        return -1.0; // No plane intersection
    }

    // Get the intersection point on the plane
    let hit_point = ray.origin + t * ray.direction;
    let local_point = hit_point - ellipse.center;

    // Create local coordinate system for the ellipse
    // We need two perpendicular vectors in the ellipse plane
    let up = vec3<f32>(0.0, 1.0, 0.0);
    var u_axis: vec3<f32>;
    var v_axis: vec3<f32>;

    // Choose u_axis perpendicular to normal
    if abs(dot(ellipse.normal, up)) < 0.9 {
        u_axis = normalize(cross(ellipse.normal, up));
    } else {
        u_axis = normalize(cross(ellipse.normal, vec3<f32>(1.0, 0.0, 0.0)));
    }
    v_axis = cross(ellipse.normal, u_axis);

    // Project the hit point onto the ellipse's local coordinate system
    let u = dot(local_point, u_axis);
    let v = dot(local_point, v_axis);

    // Check if point is inside outer ellipse
    let outer_test = (u * u) / (ellipse.radius_a * ellipse.radius_a) +
                     (v * v) / (ellipse.radius_b * ellipse.radius_b);

    if outer_test > 1.0 {
        return -1.0;
    }

    return t;
}

fn get_ellipse_color(ellipse: Ellipse, hit_point: vec3<f32>) -> vec3<f32> {
    return add_border(ellipse, hit_point, ellipse.color);
}

fn add_border(ellipse: Ellipse, hit_point: vec3<f32>, ellipse_color: vec3<f32>) -> vec3<f32> {

    let distance_from_center = distance_from_ellipse_center(ellipse, hit_point);

    let border_start = 1.0 - ellipse.border_thickness;
    if distance_from_center > border_start {
        // let border_factor = (distance_from_center - border_start) / (1.0 - border_start);
        // return mix(ellipse_color, ellipse.border_color, border_factor);

        return ellipse.border_color;
    }

    return ellipse_color;
}

fn distance_from_ellipse_center(ellipse: Ellipse, hit_point: vec3<f32>) -> f32 {
    let local_point = hit_point - ellipse.center;

    let up = vec3<f32>(0.0, 1.0, 0.0);
    var u_axis: vec3<f32>;
    if abs(dot(ellipse.normal, up)) < 0.9 {
        u_axis = normalize(cross(ellipse.normal, up));
    } else {
        u_axis = normalize(cross(ellipse.normal, vec3<f32>(1.0, 0.0, 0.0)));
    }
    let v_axis = cross(ellipse.normal, u_axis);

    let u = dot(local_point, u_axis);
    let v = dot(local_point, v_axis);

    return sqrt((u * u) / (ellipse.radius_a * ellipse.radius_a) +
                                   (v * v) / (ellipse.radius_b * ellipse.radius_b));
}

//////////////////////////////////////////

struct HitInfo {
    hit: bool,
    t: f32,
    point: vec3<f32>,
    normal: vec3<f32>,
    color: vec3<f32>,
    multiplier: f32,
    reflectivity: f32,
}

/// Main single-bounce ray tracing function that checks intersections with the scene
fn trace_ray_single_bounce(ray: Ray) -> HitInfo {
    var hit_info: HitInfo;
    hit_info.hit = false;
    hit_info.t = 1000.0;
    hit_info.multiplier = 1.0;
    hit_info.reflectivity = 0.0;

    for (var i: u32 = 0u; i < scene.plane_count; i++) {
        let plane = scene.planes[i];
        let t = ray_plane_intersect(ray, plane);

        add_checkerboard_pattern(ray, plane, t, &hit_info, 0.5);
    }

    for (var i: u32 = 0u; i < scene.ellipse_count; i++) {
        let ellipse = scene.ellipses[i];
        let t = ray_ellipse_intersect(ray, ellipse);

        if t > 0.001 && t < hit_info.t {
            hit_info.hit = true;
            hit_info.t = t;
            hit_info.point = ray.origin + t * ray.direction;
            hit_info.normal = ellipse.normal;
            hit_info.color = get_ellipse_color(ellipse, hit_info.point);
            hit_info.reflectivity = ellipse.reflectivity;;
        }
    }

    for (var i: u32 = 0u; i < scene.cube_count; i++) {
        let cube = scene.cubes[i];

        let plane1 = cube.planes[0];
        let t1 = ray_plane_intersect_one_way(ray, plane1);
        add_checkerboard_pattern(ray, plane1, t1, &hit_info, 0.25);

        let plane2 = cube.planes[1];
        let t2 = ray_plane_intersect_one_way(ray, plane2);
        add_checkerboard_pattern(ray, plane2, t2, &hit_info, 0.25);

        let plane3 = cube.planes[2];
        let t3 = ray_plane_intersect_one_way(ray, plane3);
        add_checkerboard_pattern(ray, plane3, t3, &hit_info, 0.25);

        let plane4 = cube.planes[3];
        let t4 = ray_plane_intersect_one_way(ray, plane4);
        add_checkerboard_pattern(ray, plane4, t4, &hit_info, 0.25);

        let plane5 = cube.planes[4];
        let t5 = ray_plane_intersect_one_way(ray, plane5);
        add_checkerboard_pattern(ray, plane5, t5, &hit_info, 0.25);

        let plane6 = cube.planes[5];
        let t6 = ray_plane_intersect_one_way(ray, plane6);
        add_checkerboard_pattern(ray, plane6, t6, &hit_info, 0.25);

        /// Doesnt work cause `array` cant be indexed with a variable ??
        // for (var j: u32 = 0u; j < 6u; j++) {
        //     let plane = cube.planes[j];
        //     let t = ray_plane_intersect(ray, plane);

        //     add_checkerboard_pattern(ray, plane, t, &hit_info);
        // }
    }

    return hit_info;
}

fn trace_ray(ray: Ray, max_bounces: u32) -> HitInfo {
    var current_ray = ray;
    var final_hit_info: HitInfo;
    final_hit_info.hit = false;
    final_hit_info.t = 1000.0;
    final_hit_info.multiplier = 1.0;
    final_hit_info.reflectivity = 0.0;

    for (var bounce: u32 = 0u; bounce < max_bounces; bounce++) {
        // Send ray through the scene
        let hit_info = trace_ray_single_bounce(current_ray);

        if bounce == max_bounces - 1u {
            // Last bounce, use the hit info
            final_hit_info = hit_info;
            break;
        } else if hit_info.reflectivity > 0.0 {
            // Hit surface was reflective
            // Calculate reflection direction
            let reflected_direction = reflect(current_ray.direction, hit_info.normal);
            current_ray.origin = hit_info.point + 0.001 * reflected_direction; // Offset to avoid self-intersection
            current_ray.direction = reflected_direction;

            // Update multiplier for color contribution
            final_hit_info.reflectivity = hit_info.reflectivity;
            final_hit_info.multiplier *= hit_info.reflectivity;
        } else { // Hit surface was not reflective, just use the hit info
            let mult = final_hit_info.multiplier;
            final_hit_info = hit_info; // <-
            final_hit_info.multiplier = mult;
            break;
        }
    }

    return final_hit_info;
}

// Calculate reflection direction of an `incident` ray bouncing off a surface with normal `normal`
fn reflect(incident: vec3<f32>, normal: vec3<f32>) -> vec3<f32> {
    return incident - 2.0 * dot(incident, normal) * normal;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Convert screen coordinates to ray direction
    let uv = (in.uv * 2.0 - 1.0) * vec2<f32>(uniforms.resolution.x / uniforms.resolution.y, 1.0);

    let ray_origin = uniforms.camera_pos;

    // Use the camera_dir from uniforms
    let camera_forward = normalize(uniforms.camera_dir);

    // Create camera coordinate system
    let world_up = vec3<f32>(0.0, 1.0, 0.0);
    let camera_right = normalize(cross(camera_forward, world_up));
    let camera_up = cross(camera_right, camera_forward);

    // Calculate ray direction
    let ray_direction = normalize(
        camera_forward +
        uv.x * camera_right * atan(uniforms.fov) +
        uv.y * camera_up * atan(uniforms.fov)
    );

    let primary_ray = Ray(ray_origin, ray_direction);

    // Trace the ray
    let hit_info = trace_ray(primary_ray, scene.max_bounces);

    var color = vec3<f32>(0.1, 0.2, 0.4); // Blue gradient background

    if hit_info.hit {
        color = hit_info.color;
    }

    color *= hit_info.multiplier;

    return vec4<f32>(color, 1.0);
}
