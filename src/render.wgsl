

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

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Convert screen coordinates to ray direction
    let uv = (in.uv * 2.0 - 1.0) * vec2<f32>(uniforms.resolution.x / uniforms.resolution.y, 1.0);
    
    // Simple camera setup
    let ray_origin = uniforms.camera_pos;
    let ray_direction = normalize(vec3<f32>(uv.x, uv.y, -1.0));
    
    let ray = Ray(ray_origin, ray_direction);
    
    // Simple sphere to ray trace
    let sphere = Sphere(vec3<f32>(0.0, 0.0, -3.0), 1.0);
    
    let t = ray_sphere_intersect(ray, sphere);
    
    if (t > 0.0) {
        // Hit the sphere - calculate simple shading
        let hit_point = ray.origin + t * ray.direction;
        let normal = normalize(hit_point - sphere.center);
        let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
        let diffuse = max(dot(normal, light_dir), 0.1);
        return vec4<f32>(diffuse, diffuse * 0.5, diffuse * 0.8, 1.0);
    } else {
        // Background gradient
        let gradient = uv.y * 0.5 + 0.5;
        return vec4<f32>(0.1, 0.2, 0.3 + gradient * 0.3, 1.0);
    }
}