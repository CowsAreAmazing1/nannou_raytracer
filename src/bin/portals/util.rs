use nannou::glam::Vec3;

pub const WORLD_FORWARDS: Vec3 = Vec3::X;
pub const WORLD_RIGHT: Vec3 = Vec3::Z;
pub const WORLD_UP: Vec3 = Vec3::Y;

pub const WORLD_FRAME: (Vec3, Vec3, Vec3) = (WORLD_FORWARDS, WORLD_RIGHT, WORLD_UP);

pub fn quat_to(vec: nannou::glam::Vec3) -> nannou::glam::Quat {
    let target = vec.normalize();
    nannou::glam::Quat::from_rotation_arc(WORLD_UP, target)
}
