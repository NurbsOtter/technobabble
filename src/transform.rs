use ecs::{self, Join};
use cgmath::{Vector3, Matrix4, EuclideanVector};
use rtree::Rectangle;
use Step;

#[derive(Debug, Copy, Clone)]
pub struct Height(pub f32);

#[derive(Debug, Copy, Clone)]
pub struct Transform {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

impl Transform {
    /// convert the transform into a model matrix
    pub fn model_matrix(&self) -> Matrix4<f32> {
        let vec = Vector3::new(
            self.x,
            self.y,
            self.z
        );

        Matrix4::from_translation(vec)
    }
}

impl ecs::Component for Transform {
    type Storage = ecs::VecStorage<Transform>;
}

#[derive(Debug, Copy, Clone)]
pub struct Location(pub Rectangle);

impl Location {
    /// convert the transform into a model matrix
    pub fn transform(&self, z: f32) -> Transform {
        let (x, y) = self.middle();

        Transform{
            x: x,
            y: y,
            z: z
        }
    }

    pub fn middle(&self) -> (f32, f32) {
        let (ax, bx) = (self.0.min.x as f32, self.0.max.x as f32);
        let (ay, by) = (self.0.min.y as f32, self.0.max.y as f32);
        ((ax + bx) / 16., (ay + by) / 16.)
    }
}

impl ecs::Component for Location {
    type Storage = ecs::VecStorage<Location>;
}


#[derive(Debug, Copy, Clone)]
pub struct MovingTo(pub Rectangle);

impl ecs::Component for MovingTo {
    type Storage = ecs::VecStorage<MovingTo>;
}

pub struct LocationToTransform;

impl ecs::System<Step> for LocationToTransform {
    fn run(&mut self, arg: ecs::RunArg, step: Step) {
        let (eids, mut trans, loc, mov) = arg.fetch(|w| {
            (w.entities(),
             w.write::<Transform>(),
             w.read::<Location>(),
             w.read::<MovingTo>()
            )
        });

        if !step.is_render() {
            return
        }

        let delta = step.fstep().fract() as f32 ;

        for (eid, loc) in (&eids, &loc).iter() {
            if let Some(to) = mov.get(eid) {
                let (x, y) = loc.middle();
                let from = Vector3::new(x, y, 0.);
                let (x, y) = Location(to.0).middle();
                let to = Vector3::new(x, y, 0.);
                let t = from.lerp(to, delta);
                trans.insert(eid, Transform{
                    x: t.x,
                    y: t.y,
                    z: t.z,
                });
            } else {
                trans.insert(eid, loc.transform(0.));
            }
        }
    }
}