use nannou::{
    color::{Component, Srgb},
    glam::{Quat, Vec3},
};

pub const WORLD_UP: Vec3 = Vec3::Y;

pub fn quat_to(vec: Vec3) -> Quat {
    let target = vec.normalize();
    Quat::from_rotation_arc(WORLD_UP, target)
}

pub fn vec_to(quat: Quat) -> Vec3 {
    (quat * WORLD_UP).normalize()
}

pub fn color_convert<T: Component>(color: Srgb<T>) -> Srgb<f32> {
    color.into_format::<f32>()
}
