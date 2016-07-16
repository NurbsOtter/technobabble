extern crate cgmath;
extern crate collision;
extern crate gfx;
extern crate gfx_device_gl;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate genmesh;
extern crate amethyst;
extern crate specs as ecs;

#[cfg(test)]
extern crate quickcheck;

mod renderer;
mod camera;
mod input;
mod transform;
mod movement;
mod rtree;

use glutin::Event;
use glutin::VirtualKeyCode as Key;
use cgmath::Vector3;
use collision::{Plane, Intersect};
use rtree::{RTree, Rectangle, Point};
use ecs::Join;
pub use transform::MovingTo;

const SCALE: f32 = 0.1;

#[derive(Copy, Clone, Debug)]
pub enum Step {
    Game(u64),
    Render(u64, f32)
}

impl Step {
    pub fn is_game(&self) -> bool {
        if let Step::Game(_) = *self { true } else { false }
    }

    pub fn is_render(&self) -> bool {
        if let Step::Render(_, _) = *self { true } else { false }
    }

    pub fn step(&self) -> u64 {
        match *self {
            Step::Game(i) => i,
            Step::Render(i, _) => i
        }
    }

    pub fn fstep(&self) -> f64 {
        match *self {
            Step::Game(i) => i as f64,
            Step::Render(i, off) => i as f64 + off as f64
        }
    }
}

fn clamp(min: f32, value: f32, max: f32) -> f32 {
    if min > value {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

fn step(world: &ecs::World, window: &glutin::Window) -> bool {
    let mut input = world.write_resource::<input::Events>();
    input.next_frame(window);
    input.running
}

struct Player(ecs::Entity);

fn main() {
    let mut world = ecs::World::new();
    world.register::<transform::Transform>();
    world.register::<transform::Location>();
    world.register::<transform::MovingTo>();
    world.register::<PreviewMarker>();
    world.register::<BulletMarker>();
    world.register::<movement::Movement>();
    world.register::<Decay>();


    let builder = glutin::WindowBuilder::new()
        .with_title("Technobabble".to_string())
        .with_dimensions(800, 600)
        .with_vsync();

    let eid = world.create_now()
                   .with(PreviewMarker)
                   .with(movement::Movement::new(0., 0.))
                   .with(transform::Location(
                        rtree::Rectangle{
                            min: Point{x: -1, y: -1},
                            max: Point{x:  1, y:  1}
                        }
                   ))
                   .with(transform::Transform{
                        x: 0., y: 0., z: 0.
                   })
                   .build();

    let (mut renderer, window) = renderer::Renderer::new(builder);
    world.add_resource(input::Events::new(&window));
    world.add_resource(camera::Camera::new());
    world.add_resource(RTree::<ecs::Entity>::new());
    world.add_resource(Player(eid));

    let mut sim = ecs::Planner::<Step>::new(world, 4);
    sim.add_system(InputHandler, "Input Handler", 16);
    sim.add_system(ShootShit, "Create box", 15);
    sim.add_system(movement::System, "Movement", 14);
    sim.add_system(CameraSystem, "Camera System", 13);
    sim.add_system(DecaySystem, "Decay System", 12);
    sim.add_system(transform::LocationToTransform, "Location Sync", 11);

    let start = std::time::SystemTime::now();
    let mut index = 0;
    while step(sim.mut_world(), &window) {
        let now = std::time::SystemTime::now();
        let delta = now.duration_since(start).unwrap();
        let delta = (delta.as_secs() as f64 + delta.subsec_nanos() as f64 / 1e9) * 20.;
        while delta.trunc() as u64 - index > 0 {
            index += 1;
            sim.dispatch(Step::Game(index));
        }
        sim.dispatch(Step::Render(index, delta.fract() as f32));

        let camera = {
            *sim.mut_world().read_resource::<camera::Camera>()
        };
        renderer.resize(&window);
        renderer.render(camera, sim.mut_world());
        window.swap_buffers().unwrap();
    }
}

struct InputHandler;

impl ecs::System<Step> for InputHandler {
    fn run(&mut self, arg: ecs::RunArg, step: Step) {
        let (mut camera, input, player, mut mov) = arg.fetch(|w| {
            (w.write_resource::<camera::Camera>(),
             w.read_resource::<input::Events>(),
             w.read_resource::<Player>(),
             w.write::<movement::Movement>())
        });

        if !step.is_game() {
            return
        }

        camera.position = camera.position + match (input.is_key_down(Key::Equals), input.is_key_down(Key::Subtract)) {
            (true, false) => Vector3::new(0., 0., -1.),
            (false, true) => Vector3::new(0., 0., 1.),
            _ => Vector3::new(0., 0., 0.)
        } * SCALE;

        for e in &input.events {
            use glutin::MouseScrollDelta;
            match e {
                &Event::MouseWheel(MouseScrollDelta::LineDelta(_, x), _) => {
                    camera.position.z -= 2. * x * SCALE;
                }
                &Event::MouseWheel(MouseScrollDelta::PixelDelta(_, x), _) => {
                    camera.position.z -= 2. * x * SCALE / 10.;
                }
                _ => ()
            }
        }

        camera.position.z = clamp(1., camera.position.z, 10.);
        camera.resize(input.window_size);

        let rate = if input.is_key_down(Key::LShift) { 0.55 } else { 0.20 };
        let left_right: movement::Vector = match (input.is_key_down(Key::A), input.is_key_down(Key::D)) {
            (true, false) => (rate, -rate).into(),
            (false, true) => (-rate, rate).into(),
            _ => (0., 0.).into()
        };
        let up_down: movement::Vector = match (input.is_key_down(Key::S), input.is_key_down(Key::W)) {
            (true, false) => (rate, rate).into(),
            (false, true) => (-rate, -rate).into(),
            _ => (0., 0.).into()
        };

        let movement = mov.get_mut(player.0).unwrap();
        movement.vector = left_right + up_down;
    }
}

struct CameraSystem;

impl ecs::System<Step> for CameraSystem {
    fn run(&mut self, arg: ecs::RunArg, step: Step) {
        let (mut camera, player, transform) = arg.fetch(|w| {
            (w.write_resource::<camera::Camera>(),
             w.read_resource::<Player>(),
             w.read::<transform::Transform>())
        });

        if !step.is_render() {
            return
        }

        let transform = transform.get(player.0).unwrap();
        camera.position.x = transform.x + 5.;
        camera.position.y = transform.y + 5.;
    }
}

#[derive(Clone, Default)]
pub struct PreviewMarker;
impl ecs::Component for PreviewMarker {
    type Storage = ecs::NullStorage<PreviewMarker>;
}

#[derive(Clone, Default)]
pub struct BulletMarker;
impl ecs::Component for BulletMarker {
    type Storage = ecs::NullStorage<BulletMarker>;
}

struct ShootShit;

impl ecs::System<Step> for ShootShit {
    fn run(&mut self, arg: ecs::RunArg, step: Step) {
        let (camera, input, player, mut bullet, mut trans, mut mov, mut decay) = arg.fetch(|w| {
            (w.read_resource::<camera::Camera>(),
             w.read_resource::<input::Events>(),
             w.read_resource::<Player>(),
             w.write::<BulletMarker>(),
             w.write::<transform::Location>(),
             w.write::<movement::Movement>(),
             w.write::<Decay>())
        });

        if !step.is_game() {
            return
        }

        let ray = camera.pixel_ray(input.mouse_position);
        let plane = Plane::from_abcd(0., 0., 1., 0.);
        if let Some(p) = (plane, ray).intersection() {
            let x = (p.x * 8.).round() as i32;
            let y = (p.y * 8.).round() as i32;

            let pos = *trans.get(player.0).unwrap();
            let (mx, my) = pos.middle();

            if input.is_button_down(glutin::MouseButton::Left) {
                for x in (x-2)..(x+2) {
                    for y in (y-2)..(y+2) {
                        let x = x as f32;
                        let y = y as f32;

                        let eid = arg.create();

                        trans.insert(eid, transform::Location(pos.0));
                        bullet.insert(eid, BulletMarker);

                        let (dx, dy) = (x - mx, y - my);
                        let mag = (dx * dx + dy * dy).sqrt();

                        mov.insert(eid, movement::Movement::new(
                            4. * (x - mx) / mag,
                            4. * (y - my) / mag
                        ));

                        decay.insert(eid, Decay(60));
                    }
                }
            }
        }
    }
}


/// decay will kill an entity ofer x turns
#[derive(Clone, Default, Debug)]
pub struct Decay(u16);

impl ecs::Component for Decay {
    type Storage = ecs::VecStorage<Decay>;
}

struct DecaySystem;

impl ecs::System<Step> for DecaySystem {
    fn run(&mut self, arg: ecs::RunArg, step: Step) {
        let (eids, mut decay) = arg.fetch(|w| {
            (w.entities(), w.write::<Decay>())
        });


        if !step.is_game() {
            return
        }

        for (eid, d) in (&eids, &mut decay).iter() {
            if d.0 == 0 {
                arg.delete(eid);
            } else {
                d.0 -= 1;
            }
        }
    }
}