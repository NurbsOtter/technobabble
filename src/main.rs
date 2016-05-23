extern crate cgmath;
extern crate collision;
extern crate gfx;
extern crate gfx_device_gl;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate genmesh;
extern crate amethyst;

mod renderer;
mod camera;
mod input;

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

fn main() {
    let builder = glutin::WindowBuilder::new()
        .with_title("Technobabble".to_string())
        .with_dimensions(800, 600)
        .with_vsync();


    let (mut renderer, window) = renderer::Renderer::new(builder);

    let mut camera = camera::Camera::new();
    let mut input = input::Events::new(&window);
    while input.running {
        input.next_frame(&window);

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
        renderer.resize(&window);


        let ray = camera.pixel_ray(input.mouse_position);
        let plane = Plane::from_abcd(0., 0., 1., 0.);
        if let Some(p) = (plane, ray).intersection() {
            renderer.render(camera, p);
        }
        window.swap_buffers().unwrap();
    }
}