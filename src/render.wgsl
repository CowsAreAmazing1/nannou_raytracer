

struct Uniforms {
    resolution: vec2<f32>,
    time: f32,
    _padding: f32,
    camera_pos: vec3<f32>,
    _padding2: f32,
    camera_dir: vec3<f32>,
    _padding3: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

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



//////////////////////////////
//   Objects and Ray Tracing Functions
struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
}

struct Sphere {
    center: vec3<f32>,
    radius: f32,
}
fn ray_sphere_intersect(ray: Ray, sphere: Sphere) -> f32 {
    let oc = ray.origin - sphere.center;
    let a = dot(ray.direction, ray.direction);
    let b = 2.0 * dot(oc, ray.direction);
    let c = dot(oc, oc) - sphere.radius * sphere.radius;
    let discriminant = b * b - 4.0 * a * c;
    
    if (discriminant < 0.0) {
        return -1.0;
    }
    
    return (-b - sqrt(discriminant)) / (2.0 * a);
}

struct Plane {
    point: vec3<f32>,
    normal: vec3<f32>,
}
fn ray_plane_intersect(ray: Ray, plane: Plane) -> f32 {
    let denom = dot(plane.normal, ray.direction);
    if (abs(denom) < 1e-6) {
        return -1.0; // Ray is parallel to the plane
    }
    let t = dot(plane.point - ray.origin, plane.normal) / denom;
    if (t < 0.0) {
        return -1.0; // Intersection behind the ray origin
    }
    return t; // Return the distance to the intersection point
}

struct Ellipse {
    center: vec3<f32>,
    normal: vec3<f32>,        // Normal to the ellipse plane
    radius_a: f32,            // Radius in first axis
    radius_b: f32,            // Radius in second axis
    inner_radius_a: f32,      // Inner radius in first axis (for ring)
    inner_radius_b: f32,      // Inner radius in second axis (for ring)
}

fn ray_ellipse_intersect(ray: Ray, ellipse: Ellipse) -> f32 {
    // First, intersect with the plane containing the ellipse
    let plane = Plane(ellipse.center, ellipse.normal);
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
    
    // Check if point is outside inner ellipse (for ring shape)
    let inner_test = (u * u) / (ellipse.inner_radius_a * ellipse.inner_radius_a) + 
                     (v * v) / (ellipse.inner_radius_b * ellipse.inner_radius_b);
    
    if (inner_test < 1.0) {
        return -1.0; // Inside inner ellipse (hole)
    }
    
    return t; // Valid intersection with the ring
}

//////////////////////////////////////////

// Structure to hold intersection information
struct HitInfo {
    hit: bool,
    t: f32,
    point: vec3<f32>,
    normal: vec3<f32>,
    color: vec3<f32>,
    reflectivity: f32,
}

// Function to trace a ray and return hit information
fn trace_ray(ray: Ray) -> HitInfo {
    var hit_info: HitInfo;
    hit_info.hit = false;
    hit_info.t = 1000.0;
    
    // // Objects positioned nicely
    let ground_plane = Plane(vec3<f32>(0.0, -2.0, 0.0), vec3<f32>(0.0, 1.0, 0.0));
    // let sphere1 = Sphere(vec3<f32>(-1.5, -1.0, -4.0), 1.0);  // Left sphere (mirror)
    // let sphere2 = Sphere(vec3<f32>(1.0, -1.0, -3.0), 1.0);   // Right sphere (less reflective)
    // let sphere3 = Sphere(vec3<f32>(0.0, 0.5, -5.0), 0.8);    // Floating sphere (non-reflective)

    let e_a = 2.0;
    let e_b = 2.5;
    let rim_thickness = 0.2;
    
    let ellipse1out = Ellipse(
        vec3<f32>(0.0, 1.8, -4.0),
        vec3<f32>(0.0, -0.5, 1.0),
        e_a,
        e_b,
        e_a - rim_thickness,
        e_b - rim_thickness,
    );
    let ellipse1in = Ellipse(
        ellipse1out.center,
        ellipse1out.normal,
        ellipse1out.inner_radius_a,  
        ellipse1out.inner_radius_b,
        0.0,
        0.0,
    );
    
    let ellipse2out = Ellipse(
        vec3<f32>(2.5, 0.0, -5.0),
        vec3<f32>(1.0, 0.0, 0.0),
        e_a,
        e_b,
        e_a - rim_thickness,
        e_b - rim_thickness,
    );
    let ellipse2in = Ellipse(
        ellipse2out.center,
        ellipse2out.normal,
        ellipse2out.inner_radius_a,  
        ellipse2out.inner_radius_b,
        0.0,
        0.0,
    );
    
    // Check plane
    let plane_t = ray_plane_intersect(ray, ground_plane);
    if (plane_t > 0.001 && plane_t < hit_info.t) {
        hit_info.hit = true;
        hit_info.t = plane_t;
        hit_info.point = ray.origin + plane_t * ray.direction;
        hit_info.normal = ground_plane.normal;
        
        // Checkerboard pattern
        let world_pos = hit_info.point;
        let checker_scale = 1.0; // Size of each checker square
        
        // Use a more robust method that works with negative coordinates
        let checker_x = floor(world_pos.x / checker_scale + 0.5);
        let checker_z = floor(world_pos.z / checker_scale + 0.5);
        
        // Use bitwise AND equivalent for better cross-platform compatibility
        let sum = checker_x + checker_z;
        let checker_pattern = abs(sum - 2.0 * floor(sum * 0.5));
        
        // Alternate between two colors
        if (checker_pattern < 0.5) {
            hit_info.color = vec3<f32>(0.5, 0.5, 0.5); // Light gray
        } else {
            hit_info.color = vec3<f32>(0.05, 0.05, 0.05); // Dark gray
        }
        
        hit_info.reflectivity = 0.05; // Slightly reflective ground
    }
    
    // Check ellipse 1 (horizontal ring)
    let ellipse1in_t = ray_ellipse_intersect(ray, ellipse1in);
    if (ellipse1in_t > 0.001 && ellipse1in_t < hit_info.t) {
        hit_info.hit = true;
        hit_info.t = ellipse1in_t;
        hit_info.point = ray.origin + ellipse1in_t * ray.direction;
        hit_info.normal = ellipse1in.normal;
        hit_info.color = vec3<f32>(1.0, 0.7, 0.2);
        hit_info.reflectivity = 0.0;
    }
    
    // Check ellipse 2 (vertical ring)
    let ellipse1out_t = ray_ellipse_intersect(ray, ellipse1out);
    if (ellipse1out_t > 0.001 && ellipse1out_t < hit_info.t) {
        hit_info.hit = true;
        hit_info.t = ellipse1out_t;
        hit_info.point = ray.origin + ellipse1out_t * ray.direction;
        hit_info.normal = ellipse1out.normal;
        hit_info.color = vec3<f32>(0.7, 0.4, 0.0);
        hit_info.reflectivity = 0.0;
    }

    let ellipse2in_t = ray_ellipse_intersect(ray, ellipse2in);
    if (ellipse2in_t > 0.001 && ellipse2in_t < hit_info.t) {
        hit_info.hit = true;
        hit_info.t = ellipse2in_t;
        hit_info.point = ray.origin + ellipse2in_t * ray.direction;
        hit_info.normal = ellipse2in.normal;
        hit_info.color = vec3<f32>(0.8, 0.2, 0.8);
        hit_info.reflectivity = 0.0;
    }
    
    // Check ellipse 2 (vertical ring)
    let ellipse2out_t = ray_ellipse_intersect(ray, ellipse2out);
    if (ellipse2out_t > 0.001 && ellipse2out_t < hit_info.t) {
        hit_info.hit = true;
        hit_info.t = ellipse2out_t;
        hit_info.point = ray.origin + ellipse2out_t * ray.direction;
        hit_info.normal = ellipse2out.normal;
        hit_info.color = vec3<f32>(0.5, 0.0, 0.5);
        hit_info.reflectivity = 0.0; 
    }
    
    // Check sphere 1 (mirror sphere)
    // let sphere1_t = ray_sphere_intersect(ray, sphere1);
    // if (sphere1_t > 0.001 && sphere1_t < hit_info.t) {
    //     hit_info.hit = true;
    //     hit_info.t = sphere1_t;
    //     hit_info.point = ray.origin + sphere1_t * ray.direction;
    //     hit_info.normal = normalize(hit_info.point - sphere1.center);
    //     hit_info.color = vec3<f32>(0.9, 0.9, 0.9); // Almost white for mirror
    //     hit_info.reflectivity = 0.9; // Highly reflective
    // }
    
    // // Check sphere 2 (partially reflective)
    // let sphere2_t = ray_sphere_intersect(ray, sphere2);
    // if (sphere2_t > 0.001 && sphere2_t < hit_info.t) {
    //     hit_info.hit = true;
    //     hit_info.t = sphere2_t;
    //     hit_info.point = ray.origin + sphere2_t * ray.direction;
    //     hit_info.normal = normalize(hit_info.point - sphere2.center);
    //     hit_info.color = vec3<f32>(0.3, 0.3, 1.0); // Blue
    //     hit_info.reflectivity = 0.3; // Somewhat reflective
    // }
    
    // // Check sphere 3 (non-reflective)
    // let sphere3_t = ray_sphere_intersect(ray, sphere3);
    // if (sphere3_t > 0.001 && sphere3_t < hit_info.t) {
    //     hit_info.hit = true;
    //     hit_info.t = sphere3_t;
    //     hit_info.point = ray.origin + sphere3_t * ray.direction;
    //     hit_info.normal = normalize(hit_info.point - sphere3.center);
    //     hit_info.color = vec3<f32>(0.3, 1.0, 0.3); // Green
    //     hit_info.reflectivity = 0.05; // Almost no reflection
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
    
    // Use camera position from uniforms (controlled by mouse in Rust)
    let ray_origin = uniforms.camera_pos;
    
    // Look-at target (center of our scene)
    let camera_target = vec3<f32>(0.0, -0.5, -4.0);
    
    // Create camera coordinate system
    let camera_forward = normalize(camera_target - ray_origin);
    let world_up = vec3<f32>(0.0, 1.0, 0.0);
    let camera_right = normalize(cross(camera_forward, world_up));
    let camera_up = cross(camera_right, camera_forward);
    
    // Calculate ray direction using camera coordinate system
    let ray_direction = normalize(
        camera_forward + 
        uv.x * camera_right * 0.8 +  // FOV control (smaller = more zoomed in)
        uv.y * camera_up * 0.8
    );
    
    let primary_ray = Ray(ray_origin, ray_direction);
    
    // Trace primary ray
    let hit = trace_ray(primary_ray);
    
    if (!hit.hit) {
        // Background gradient
        let gradient = uv.y * 0.5 + 0.5;
        return vec4<f32>(0.1, 0.2, 0.3 + gradient * 0.3, 1.0);
    }
    
    // Calculate basic lighting
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let diffuse = max(dot(hit.normal, light_dir), 0.1);
    var final_color = hit.color * diffuse;
    
    // Add reflection if the surface is reflective
    if (hit.reflectivity > 0.0) {
        let reflection_dir = reflect(primary_ray.direction, hit.normal);
        let reflection_ray = Ray(hit.point + hit.normal * 0.001, reflection_dir); // Offset to prevent self-intersection
        
        let reflection_hit = trace_ray(reflection_ray);
        
        if (reflection_hit.hit) {
            // Calculate lighting for reflected surface
            let reflected_diffuse = max(dot(reflection_hit.normal, light_dir), 0.1);
            let reflection_color = reflection_hit.color * reflected_diffuse;
            
            // Blend the reflection with the surface color
            final_color = mix(final_color, reflection_color, hit.reflectivity);
        } else {
            // Reflect the background
            let bg_gradient = reflection_dir.y * 0.5 + 0.5;
            let bg_color = vec3<f32>(0.1, 0.2, 0.3 + bg_gradient * 0.3);
            final_color = mix(final_color, bg_color, hit.reflectivity);
        }
    }
    
    return vec4<f32>(final_color, 1.0);
}