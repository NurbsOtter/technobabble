use ecs;
use cgmath::{Vector3, Matrix4};
use rtree::Rectangle;

#[derive(Debug, Copy, Clone)]
pub struct Transform {
    pub rectangle: Rectangle,
    pub z: f32
}

impl Transform {
    /// convert the transform into a model matrix
    pub fn model_matrix(&self) -> Matrix4<f32> {
        let (ax, bx) = (self.rectangle.min.x as f32, self.rectangle.max.x as f32);
        let (ay, by) = (self.rectangle.min.y as f32, self.rectangle.max.y as f32);

        let vec = Vector3::new(
            (ax + bx) / 16.,
            (ay + by) / 16.,
            self.z
        );

        Matrix4::from_translation(vec)
    }

    pub fn middle(&self) -> (f32, f32) {
        let (ax, bx) = (self.rectangle.min.x as f32, self.rectangle.max.x as f32);
        let (ay, by) = (self.rectangle.min.y as f32, self.rectangle.max.y as f32);
        ((ax + bx) / 16.,
         (ay + by) / 16.)
    }
}

impl ecs::Component for Transform {
    type Storage = ecs::VecStorage<Transform>;
}