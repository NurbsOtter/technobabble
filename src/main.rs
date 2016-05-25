extern crate cgmath;
extern crate collision;
extern crate gfx;
extern crate gfx_device_gl;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate genmesh;
extern crate amethyst;
extern crate specs as ecs;

mod renderer;
mod camera;
mod input;
mod transform;

use glutin::Event;
use glutin::VirtualKeyCode as Key;
use cgmath::Vector3;
use collision::{Plane, Intersect};

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

fn main() {
    let mut world = ecs::World::new();
    world.register::<transform::Transform>();
    world.register::<input::Events>();

    let builder = glutin::WindowBuilder::new()
        .with_title("Technobabble".to_string())
        .with_dimensions(800, 600)
        .with_vsync();

    let (mut renderer, window) = renderer::Renderer::new(builder);
    world.add_resource(input::Events::new(&window));
    world.add_resource(camera::Camera::new());

    let mut sim = ecs::Planner::<()>::new(world, 4);
    sim.add_system(CameraListener, "Camera Listener", 10);

    while step(&sim.world, &window) {
        sim.dispatch(());

        let input = sim.world.read_resource::<input::Events>();
        let camera = sim.world.read_resource::<camera::Camera>();

        let ray = camera.pixel_ray(input.mouse_position);
        let plane = Plane::from_abcd(0., 0., 1., 0.);
        if let Some(p) = (plane, ray).intersection() {
            renderer.resize(&window);
            renderer.render(*camera, p);
        }
        window.swap_buffers().unwrap();
    }
}

struct CameraListener;

impl ecs::System<()> for CameraListener{
    fn run(&mut self, arg: ecs::RunArg, _: ()) {
        let (mut camera, input) = arg.fetch(|w| {
            (w.write_resource::<camera::Camera>(), w.read_resource::<input::Events>())
        });

        camera.position = camera.position + match (input.is_key_down(Key::A), input.is_key_down(Key::D)) {
            (true, false) => Vector3::new(1.0, -1.0, 0.),
            (false, true) => Vector3::new(-1.0, 1.0, 0.),
            _ => Vector3::new(0., 0., 0.)
        } * SCALE;
        camera.position = camera.position + match (input.is_key_down(Key::S), input.is_key_down(Key::W)) {
            (true, false) => Vector3::new(1.0, 1.0, 0.),
            (false, true) => Vector3::new(-1.0, -1.0, 0.),
            _ => Vector3::new(0., 0., 0.)
        } * SCALE;
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
                    camera.position.z -= 2. * x * SCALE * 10.;
                }
                _ => ()
            }
        }

        camera.position.z = clamp(1., camera.position.z, 10.);
        camera.resize(input.window_size);
    }
}
