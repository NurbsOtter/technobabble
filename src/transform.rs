use ecs;
use cgmath::{Vector3, Matrix4};

#[derive(Debug, Copy, Clone)]
pub struct Transform {
    pub translate: Vector3<f32>,
}

impl Transform {
    /// convert the transform into a model matrix
    pub fn model_matrix(&self) -> Matrix4<f32> {
        Matrix4::from_translation(self.translate)
    }
}

impl ecs::Component for Transform {
    type Storage = ecs::VecStorage<Transform>;
}