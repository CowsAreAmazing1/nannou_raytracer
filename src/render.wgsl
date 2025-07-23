

struct Uniforms {
    resolution: vec2<f32>,
    time: f32,
    scene_id: u32,
    camera_pos: vec3<f32>,
    _padding2: f32,
    camera_dir: vec3<f32>,
    _padding3: f32,
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
    _padding4: f32,
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
    _padding5: f32,
}

struct Portal {
    ellipse: Ellipse,
    transform_matrix: mat4x4<f32>,
    inverse_transform_matrix: mat4x4<f32>,
}

struct PortalPair {
    portal_a: Portal,
    portal_b: Portal,
}

struct SceneData {
    plane_count: u32,
    ellipse_count: u32,
    portal_pair_count: u32,
    _padding1: u32,
    planes: array<Plane, 10>,
    ellipses: array<Ellipse, 4>,
    portal_pairs: array<PortalPair, 4>,
}


@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var<uniform> scene: SceneData;

struct VertexOutput {
    @builtin(position) position : vec4<f32>,
    @location(0) uv : vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    // Generate a full-screen triangle
    let x = f32((vertex_index << 1u) & 2u) * 2.0 - 1.0;
    let y = f32(vertex_index & 2u) * 2.0 - 1.0;
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>(x, y) * 0.5 + 0.5;
    return out;
}


//////////////////// Portal Functions
// Application of an affine transformation to a point
fn transform_point(point: vec3<f32>, matrix: mat4x4<f32>) -> vec3<f32> {
    let homogeneous = matrix * vec4<f32>(point, 1.0);
    return homogeneous.xyz / homogeneous.w;
}

// Application of an affine transformation to a direction vector (no translation)
fn transform_direction(direction: vec3<f32>, matrix: mat4x4<f32>) -> vec3<f32> {
    return normalize((matrix * vec4<f32>(direction, 0.0)).xyz);
}







//////////////////////////////
//   Objects and Ray Tracing Functions
struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
}

fn ray_plane_intersect(ray: Ray, plane: Plane) -> f32 {
    let denom = dot(plane.normal, ray.direction);
    if (abs(denom) < 1e-6) {
        return -1.0;
    }
    let t = dot(plane.point - ray.origin, plane.normal) / denom;
    if (t < 0.0) {
        return -1.0;
    }
    
    // Check if plane is finite
    if (plane.is_infinite < 0.5) {
        let hit_point = ray.origin + t * ray.direction;
        let local_point = hit_point - plane.point;
        
        // Create local coordinate system for the plane
        let up = vec3<f32>(0.0, 1.0, 0.0);
        var u_axis: vec3<f32>;
        var v_axis: vec3<f32>;
        
        if (abs(dot(plane.normal, up)) < 0.9) {
            u_axis = normalize(cross(plane.normal, up));
        } else {
            u_axis = normalize(cross(plane.normal, vec3<f32>(1.0, 0.0, 0.0)));
        }
        v_axis = cross(plane.normal, u_axis);
        
        let u = dot(local_point, u_axis);
        let v = dot(local_point, v_axis);
        
        // Check bounds
        if (abs(u) > plane.width * 0.5 || abs(v) > plane.height * 0.5) {
            return -1.0; // Outside bounds
        }
    }
    
    return t;
}

fn ray_ellipse_intersect(ray: Ray, ellipse: Ellipse) -> f32 {
    // First, intersect with the plane containing the ellipse
    let plane = Plane(ellipse.center, 0.0, ellipse.normal, 0.0, ellipse.color, 0.0, 0.0, 0.0, 1.0, 0.0);
    let t = ray_plane_intersect(ray, plane);
    
    if (t < 0.0) {
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
    if (abs(dot(ellipse.normal, up)) < 0.9) {
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
    
    if (outer_test > 1.0) {
        return -1.0; // Outside outer ellipse
    }
    
    return t; // Valid intersection with the ring
}

fn get_ellipse_color(ellipse: Ellipse, hit_point: vec3<f32>) -> vec3<f32> {
    let local_point = hit_point - ellipse.center;

    let up = vec3<f32>(0.0, 1.0, 0.0);
    var u_axis: vec3<f32>;
    if (abs(dot(ellipse.normal, up)) < 0.9) {
        u_axis = normalize(cross(ellipse.normal, up));
    } else {
        u_axis = normalize(cross(ellipse.normal, vec3<f32>(1.0, 0.0, 0.0)));
    }
    let v_axis = cross(ellipse.normal, u_axis);
    
    let u = dot(local_point, u_axis);
    let v = dot(local_point, v_axis);
    
    let distance_from_center = sqrt((u * u) / (ellipse.radius_a * ellipse.radius_a) + 
                                   (v * v) / (ellipse.radius_b * ellipse.radius_b));
    
    let border_start = 1.0 - ellipse.border_thickness;
    if (distance_from_center > border_start) {
        // Interpolate between main color and border color
        let border_factor = (distance_from_center - border_start) / (1.0 - border_start);
        return mix(ellipse.color, ellipse.border_color, border_factor);
    }

    return ellipse.color;
}
fn add_border(ellipse: Ellipse, hit_point: vec3<f32>, ellipse_color: vec3<f32>) -> vec3<f32> {
    let local_point = hit_point - ellipse.center;

    let up = vec3<f32>(0.0, 1.0, 0.0);
    var u_axis: vec3<f32>;
    if (abs(dot(ellipse.normal, up)) < 0.9) {
        u_axis = normalize(cross(ellipse.normal, up));
    } else {
        u_axis = normalize(cross(ellipse.normal, vec3<f32>(1.0, 0.0, 0.0)));
    }
    let v_axis = cross(ellipse.normal, u_axis);
    
    let u = dot(local_point, u_axis);
    let v = dot(local_point, v_axis);
    
    let distance_from_center = sqrt((u * u) / (ellipse.radius_a * ellipse.radius_a) + 
                                   (v * v) / (ellipse.radius_b * ellipse.radius_b));
    
    let border_start = 1.0 - ellipse.border_thickness;
    if (distance_from_center > border_start) {
        // Interpolate between main color and border color
        let border_factor = (distance_from_center - border_start) / (1.0 - border_start);
        return mix(ellipse_color, ellipse.border_color, border_factor);
    }

    return ellipse_color;
}

fn ray_portal_intersect(ray: Ray, portal: Portal) -> f32 {
    return ray_ellipse_intersect(ray, portal.ellipse);
}

fn portal_ray(ray: Ray, hit_t: f32, in_portal: Portal, out_portal: Portal) -> Ray {
    let world_hit_point = ray.origin + hit_t * ray.direction;

    let portal_hit_point = transform_point(world_hit_point, in_portal.inverse_transform_matrix);
    let portal_direction = transform_direction(ray.direction, in_portal.inverse_transform_matrix);

    let new_world_origin = transform_point(portal_hit_point, out_portal.transform_matrix);
    let new_world_direction = transform_direction(portal_direction, out_portal.transform_matrix);

    return Ray(new_world_origin, new_world_direction);
}

//////////////////////////////////////////

struct HitInfo {
    hit: bool,
    t: f32,
    point: vec3<f32>,
    normal: vec3<f32>,
    color: vec3<f32>,
    _padding: f32,
}

fn trace_ray(ray: Ray) -> HitInfo {
    var hit_info: HitInfo;
    hit_info.hit = false;
    hit_info.t = 1000.0;

    for (var i: u32 = 0u; i < scene.plane_count; i++) {
        let plane = scene.planes[i];
        let t = ray_plane_intersect(ray, plane);

        if (t > 0.001 && t < hit_info.t) {
            hit_info.hit = true;
            hit_info.t = t;
            hit_info.point = ray.origin + t * ray.direction;
            hit_info.normal = plane.normal;
            hit_info.color = plane.color;

            // Create local coordinate system for the plane
            let hit_point = hit_info.point;
            let local_point = hit_point - plane.point;
            
            let up = vec3<f32>(0.0, 1.0, 0.0);
            var u_axis: vec3<f32>;
            var v_axis: vec3<f32>;
            
            if (abs(dot(plane.normal, up)) < 0.9) {
                u_axis = normalize(cross(plane.normal, up));
            } else {
                u_axis = normalize(cross(plane.normal, vec3<f32>(1.0, 0.0, 0.0)));
            }
            v_axis = cross(plane.normal, u_axis);
            
            // Project hit point onto plane's local coordinates
            let u = dot(local_point, u_axis);
            let v = dot(local_point, v_axis);
            
            // Apply checkerboard pattern using local coordinates
            let checker_scale = 0.5;
            let checker_u = floor(u / checker_scale + 0.5);
            let checker_v = floor(v / checker_scale + 0.5);
            let sum = checker_u + checker_v;
            let checker_pattern = abs(sum - 2.0 * floor(sum * 0.5));

            if (checker_pattern < 0.5) {
                hit_info.color = plane.color;
            } else {
                hit_info.color = plane.color - vec3<f32>(0.25, 0.25, 0.25);
            }
        }
    }
    
    for (var i: u32 = 0u; i < scene.ellipse_count; i++) {
        let ellipse = scene.ellipses[i];
        let t = ray_ellipse_intersect(ray, ellipse);
        
        if (t > 0.001 && t < hit_info.t) {
            hit_info.hit = true;
            hit_info.t = t;
            hit_info.point = ray.origin + t * ray.direction;
            hit_info.normal = ellipse.normal;
            hit_info.color = get_ellipse_color(ellipse, hit_info.point);
        }
    }

    var has_hit_portal = false;
    var in_portal: Portal;
    var out_portal: Portal;

    for (var i: u32 = 0u; i < scene.portal_pair_count; i++) {
        let portal_pair = scene.portal_pairs[i];

        let t_a = ray_portal_intersect(ray, portal_pair.portal_a);
        let t_b = ray_portal_intersect(ray, portal_pair.portal_b);

        // Check if the portals should be the thing rendered
        var hit_portal_a = t_a > 0.001 && t_a < hit_info.t;
        var hit_portal_b = t_b > 0.001 && t_b < hit_info.t;

        if (!hit_portal_a && !hit_portal_b) {
            continue; // No valid intersection with either portal
        }

        // Choose the closer portal
        if (hit_portal_a && (!hit_portal_b || t_a < t_b)) {
            has_hit_portal = true;
            in_portal = portal_pair.portal_a;
            out_portal = portal_pair.portal_b;
            hit_info.t = t_a;
        } else if (hit_portal_b) {
            has_hit_portal = true;
            in_portal = portal_pair.portal_b;
            out_portal = portal_pair.portal_a;
            hit_info.t = t_b;
        }
    }

    if (has_hit_portal) {
        hit_info.hit = true;
        hit_info.point = ray.origin + hit_info.t * ray.direction;

        /*
            The open side of a portal is the side with the normal pointing towards the viewer.
            Things teleported through a portal emerge from the second portals open side.

            Two portals placed back to back (normals pointing out both sides) will act as a doorway.

            A ray hitting a portal in the same direction as the portal's normal will be entering from the back,
                 and should be rendered as if it were hitting an ellipse.
        */
        
        if (dot(ray.direction, in_portal.ellipse.normal) > 0.0) {
            hit_info.color = get_ellipse_color(in_portal.ellipse, hit_info.point);
            hit_info.normal = in_portal.ellipse.normal;
        } else {
            // Ray is going into the portal, show the view through it
            let new_ray = portal_ray(ray, hit_info.t, in_portal, out_portal);
            let portal_hit_info = garbage_trace_ray(new_ray);
            
            if (portal_hit_info.hit) {
                hit_info.color = add_border(in_portal.ellipse, hit_info.point, portal_hit_info.color);
                hit_info.normal = portal_hit_info.normal;
                // Keep the portal's hit point for proper depth
            } else {
                hit_info.color = add_border(in_portal.ellipse, hit_info.point, vec3<f32>(0.1, 0.2, 0.4));
            }
        }
    }
    
    return hit_info;
}

/*
im working on the logic to render the view through a portal. I feel like what i have in the trace_ray function would work, apart from the fact that wgsl does not allow recursion. How can i get around this limitation? Id like to avoid having two trace_ray functions, with one specifically for portal interactions, but if there is no other way, this can work. 
*/

fn garbage_trace_ray(ray: Ray) -> HitInfo {
    var hit_info: HitInfo;
    hit_info.hit = false;
    hit_info.t = 1000.0;

    for (var i: u32 = 0u; i < scene.plane_count; i++) {
        let plane = scene.planes[i];
        let t = ray_plane_intersect(ray, plane);

        if (t > 0.001 && t < hit_info.t) {
            hit_info.hit = true;
            hit_info.t = t;
            hit_info.point = ray.origin + t * ray.direction;
            hit_info.normal = plane.normal;
            hit_info.color = plane.color;

            // Checker
            let world_pos = hit_info.point;
            let checker_scale = 1.0;
            let checker_x = floor(world_pos.x / checker_scale + 0.5);
            let checker_z = floor(world_pos.z / checker_scale + 0.5);
            let sum = checker_x + checker_z;
            let checker_pattern = abs(sum - 2.0 * floor(sum * 0.5));

            if (checker_pattern < 0.5) {
                hit_info.color = plane.color;
            } else {
                hit_info.color = plane.color - vec3<f32>(0.15, 0.15, 0.15); // Darker color for checkerboard
            }
        }

    }
    
    for (var i: u32 = 0u; i < scene.ellipse_count; i++) {
        let ellipse = scene.ellipses[i];
        let t = ray_ellipse_intersect(ray, ellipse);
        
        if (t > 0.001 && t < hit_info.t) {
            hit_info.hit = true;
            hit_info.t = t;
            hit_info.point = ray.origin + t * ray.direction;
            hit_info.normal = ellipse.normal;
            hit_info.color = get_ellipse_color(ellipse, hit_info.point);
        }
    }

    // var has_hit_portal = false;
    // var in_portal: Portal;
    // var out_portal: Portal;

    // for (var i: u32 = 0u; i < scene.portal_pair_count; i++) {
    //     let portal_pair = scene.portal_pairs[i];

    //     let t_a = ray_portal_intersect(ray, portal_pair.portal_a);
    //     let t_b = ray_portal_intersect(ray, portal_pair.portal_b);

    //     if (!(t_a > 0.001 && t_a < hit_info.t) && !(t_b > 0.001 && t_b < hit_info.t)) {
    //         continue; // No intersection with either portal in this pair
    //     }

    //     has_hit_portal = true;

    //     if (t_a > 0.001 && t_a < hit_info.t && t_a < t_b) { // Into portal a
    //         in_portal = portal_pair.portal_a;
    //         out_portal = portal_pair.portal_b;
    //         hit_info.t = t_a;
    //     } else { // Int portal b
    //         in_portal = portal_pair.portal_b;
    //         out_portal = portal_pair.portal_a;
    //         hit_info.t = t_b;
    //     }
    // }

    // if (has_hit_portal) {
    //     hit_info.hit = true;
    //     hit_info.point = ray.origin + hit_info.t * ray.direction;
    //     // hit_info.normal = ellipse.normal;
    //     hit_info.color = get_ellipse_color(in_portal.ellipse, hit_info.point);
    // }

    return hit_info;
}

// Calculate reflection direction
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
    let fov = 1.0;
    let ray_direction = normalize(
        camera_forward + 
        uv.x * camera_right * fov +
        uv.y * camera_up * fov
    );
    
    let primary_ray = Ray(ray_origin, ray_direction);
    
    // Trace the ray
    let hit_info = trace_ray(primary_ray);
    
    var color = vec3<f32>(0.1, 0.2, 0.4); // Blue gradient background
    
    if (hit_info.hit) {
        color = hit_info.color;
    }
    
    return vec4<f32>(color, 1.0);
}