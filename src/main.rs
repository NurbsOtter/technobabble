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


const SCALE: f32 = 0.1;

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
    world.register::<PreviewMarker>();
    world.register::<movement::Movement>();

    let builder = glutin::WindowBuilder::new()
        .with_title("Technobabble".to_string())
        .with_dimensions(800, 600)
        .with_vsync();

    let eid = world.create_now()
                   .with(PreviewMarker)
                   .with(movement::Movement::new(0., 0.))
                   .with(transform::Transform{
                        rectangle: rtree::Rectangle{
                            min: Point{x: -1, y: -1},
                            max: Point{x:  1, y:  1}
                        },
                        z: 0.,
                   })
                   .build();

    let (mut renderer, window) = renderer::Renderer::new(builder);
    world.add_resource(input::Events::new(&window));
    world.add_resource(camera::Camera::new());
    world.add_resource(RTree::<ecs::Entity>::new());
    world.add_resource(Player(eid));

    let mut sim = ecs::Planner::<()>::new(world, 4);
    sim.add_system(InputHandler, "Input Handler", 16);
    sim.add_system(CreateBox, "Create box", 15);
    sim.add_system(movement::System, "movement", 14);
    sim.add_system(CameraSystem, "Camera System", 13);

    while step(&sim.world, &window) {
        sim.dispatch(());

        let camera = sim.world.read_resource::<camera::Camera>();
        renderer.resize(&window);
        renderer.render(*camera, &sim.world);
        window.swap_buffers().unwrap();
    }
}

struct InputHandler;

impl ecs::System<()> for InputHandler {
    fn run(&mut self, arg: ecs::RunArg, _: ()) {
        let (mut camera, input, player, mut mov) = arg.fetch(|w| {
            (w.write_resource::<camera::Camera>(),
             w.read_resource::<input::Events>(),
             w.read_resource::<Player>(),
             w.write::<movement::Movement>())
        });

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

impl ecs::System<()> for CameraSystem {
    fn run(&mut self, arg: ecs::RunArg, _: ()) {
        let (mut camera, player, transform) = arg.fetch(|w| {
            (w.write_resource::<camera::Camera>(),
             w.read_resource::<Player>(),
             w.read::<transform::Transform>())
        });

        let transform = transform.get(player.0).unwrap();
        let (x, y) = transform.middle();
        camera.position.x = x + 5.;
        camera.position.y = y + 5.;
    }
}

#[derive(Clone, Default)]
pub struct PreviewMarker;
impl ecs::Component for PreviewMarker {
    type Storage = ecs::NullStorage<PreviewMarker>;
}

struct CreateBox;

impl ecs::System<()> for CreateBox {
    fn run(&mut self, arg: ecs::RunArg, _: ()) {
        let (camera, input, mut grid, mut trans) = arg.fetch(|w| {
            (w.read_resource::<camera::Camera>(),
             w.read_resource::<input::Events>(),
             w.write_resource::<RTree<ecs::Entity>>(),
             w.write::<transform::Transform>())
        });


        let ray = camera.pixel_ray(input.mouse_position);
        let plane = Plane::from_abcd(0., 0., 1., 0.);
        if let Some(p) = (plane, ray).intersection() {
            let x = (p.x * 8.).round() as i16;
            let y = (p.y * 8.).round() as i16;

            let rect = Rectangle{
                min: Point{x: x, y: y},
                max: Point{x: x+1, y: y+1},
            };

            // check to see if we can!
            for (&other, _) in grid.query(rect) {
                if rect.overlaps(other) {
                    return
                }
            }

            if input.is_button_down(glutin::MouseButton::Left) {
                let eid = arg.create();
                grid.extend(Some((rect, eid)));

                trans.insert(eid, transform::Transform{
                    rectangle: rect,
                    z: p.z
                });
            }
        }
    }
}
