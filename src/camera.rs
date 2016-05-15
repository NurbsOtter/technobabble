use cgmath::{Vector3, Point3, Matrix4, ortho, AffineMatrix3, Transform};

#[derive(Copy, Clone)]
pub struct Camera {
    // The Camera's position in 3D
    pub position: Point3<f32>,
    pub viewport_size: (f32, f32)
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            position: Point3::new(0., 0., 4.),
            viewport_size: (800., 600.)
        }
    }

    /// Create a projection matrix for the camera
    pub fn projection(&self) -> Matrix4<f32> {
        let angle = (self.viewport_size.0 / self.viewport_size.1).atan();
        let w = angle.sin() * self.position.z;
        let h = angle.cos() * self.position.z;
        ortho(-w, w, -h, h, -1000., 1000.)
    }

    /// Create the view matrix for the camera
    pub fn view(&self) -> Matrix4<f32> {
        AffineMatrix3::look_at(
            self.position + Vector3::new(1., 1., 1.),
            self.position,
            Vector3::unit_z()
        ).mat
    }

    /// resize the viewport
    pub fn resize(&mut self, (w, h): (u32, u32)) {
        self.viewport_size = (w as f32, h as f32);
    }
}