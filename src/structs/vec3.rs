use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, serde::Deserialize)]
#[repr(C)]
pub struct Vec3 {
    x: f32, 
    y: f32,
    z: f32
}

unsafe impl dataview::Pod for Vec3 {}