pub const WORLD_UP: nannou::glam::Vec3 = nannou::glam::Vec3::Y;

pub fn quat_to(vec: nannou::glam::Vec3) -> nannou::glam::Quat {
    let target = vec.normalize();
    nannou::glam::Quat::from_rotation_arc(WORLD_UP, target)
}
