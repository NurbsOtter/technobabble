mod q8p24;
mod vector;

use ecs;
use ecs::Join;
use rtree::Rectangle;
pub use self::vector::Vector;
use transform::Transform;

impl ecs::Component for Movement {
    type Storage = ecs::VecStorage<Movement>;
}

pub struct Movement {
    pub vector: Vector,
    remainder: Vector
}

impl Movement {
    /// create a new movement vector
    pub fn new<T>(x: T, y: T) -> Movement
        where q8p24::Q8p24: From<T>
    {
        Movement {
            vector: (x, y).into(),
            remainder: (0i8, 0i8).into()
        }
    }

    // take a rectangle and move it based on the remainder
    pub fn next(&mut self, mut pos: Rectangle) -> Rectangle {
        let sum = self.remainder + self.vector;
        let rem = sum.trunc();
        self.remainder = sum.fract();

        if self.vector.x == q8p24::Q8p24(0) {
            self.remainder.x = (0.).into();
        }
        if self.vector.y == q8p24::Q8p24(0) {
            self.remainder.y = (0.).into();
        }

        let (x, y): (i8, i8) = (rem.x.into(), rem.y.into());
        pos.min.x = pos.min.x + x as i16;
        pos.min.y = pos.min.y + y as i16;
        pos.max.x = pos.max.x + x as i16;
        pos.max.y = pos.max.y + y as i16;
        pos
    }
}

pub struct System;

impl ecs::System<()> for System {
    fn run(&mut self, arg: ecs::RunArg, _: ()) {
        let (mut movement, mut transform) = arg.fetch(|w| {
            (w.write::<Movement>(), w.write::<Transform>())
        });

        for (mov, trans) in (&mut movement, &mut transform).iter() {
            trans.rectangle = mov.next(trans.rectangle);
        }
    }
}