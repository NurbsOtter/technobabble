use cgmath::{Vector3, Vector4, Point3, Matrix4, ortho,
             AffineMatrix3, Transform, SquareMatrix,
             EuclideanVector};
use collision::{Ray3, Ray};

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

    /// Calculate the position of the camera
    pub fn origin(&self) -> Point3<f32> {
        self.position
    }

    /// resize the viewport
    pub fn resize(&mut self, (w, h): (u32, u32)) {
        self.viewport_size = (w as f32, h as f32);
    }

    /// Creates a Ray into the world from the point of view of the camera
    /// this takes a pixel coordinate and turns it into a ray
    pub fn pixel_ray(&self, (x, y): (i32, i32)) -> Ray3<f32> {
        let ray_nds = Vector4::new(
            2. * x as f32 / self.viewport_size.0 - 1.,
            1. - 2. * y as f32 / self.viewport_size.1,
            0.,
            1.
        );
        let ray_clip = Vector4::new(ray_nds.x, ray_nds.y, -1., 1.);

        let iview = self.view().invert().unwrap();
        let iproj = self.projection().invert().unwrap();
        let iview_iproj = iview * iproj;

        let nds = iview_iproj *ray_nds;
        let clip = iview_iproj *ray_clip;

        let ray = (nds - clip).normalize();
        let origin = Point3::new(nds.x, nds.y, nds.z);
        Ray::new(origin, Vector3::new(ray.x, ray.y, ray.z))
    }
}