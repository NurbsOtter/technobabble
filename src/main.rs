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

use glutin::ElementState::{Pressed, Released};
use glutin::VirtualKeyCode as Key;
use cgmath::{Vector3, Point3};
use collision::{Plane, Intersect};

enum Dir {
    Positive,
    Zero,
    Negative
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

fn main() {
    let builder = glutin::WindowBuilder::new()
        .with_title("Technobabble".to_string())
        .with_dimensions(800, 600)
        .with_vsync();


    let (mut renderer, window) = renderer::Renderer::new(builder);

    let mut camera = camera::Camera::new();
    let (mut dx, mut dy, mut dzm) = (Dir::Zero, Dir::Zero, Dir::Zero);
    let mut mouse = (400, 600);

    'main: loop {
        for event in window.poll_events() {
            match event {
                glutin::Event::Closed => {
                    break 'main;
                },
                glutin::Event::KeyboardInput(Pressed, _, Some(Key::W)) => {
                    dy = Dir::Positive;
                },
                glutin::Event::KeyboardInput(Pressed, _, Some(Key::S)) => {
                    dy = Dir::Negative;
                },
                glutin::Event::KeyboardInput(Released, _, Some(Key::W)) |
                glutin::Event::KeyboardInput(Released, _, Some(Key::S)) => {
                    dy = Dir::Zero;
                },
                glutin::Event::KeyboardInput(Pressed, _, Some(Key::D)) => {
                    dx = Dir::Positive;
                },
                glutin::Event::KeyboardInput(Pressed, _, Some(Key::A)) => {
                    dx = Dir::Negative;
                },
                glutin::Event::KeyboardInput(Released, _, Some(Key::A)) |
                glutin::Event::KeyboardInput(Released, _, Some(Key::D)) => {
                    dx = Dir::Zero;
                },
                glutin::Event::KeyboardInput(Pressed, _, Some(Key::Q)) => {
                    dzm = Dir::Negative;
                },
                glutin::Event::KeyboardInput(Pressed, _, Some(Key::Z)) => {
                    dzm = Dir::Positive;
                },
                glutin::Event::KeyboardInput(Released, _, Some(Key::Q)) |
                glutin::Event::KeyboardInput(Released, _, Some(Key::Z)) => {
                    dzm = Dir::Zero;
                },
                glutin::Event::MouseMoved(x, y) => {
                    mouse = (x, y);
                }
                glutin::Event::Resized(_, _) => {
                    renderer.resize(&window);
                    camera.resize(window.get_inner_size_points().unwrap());
                }
                _ => ()
            }
        }

        camera.position = camera.position + match dy {
            Dir::Negative => Vector3::new( 0.1,  0.1, 0.),
            Dir::Positive => Vector3::new(-0.1, -0.1, 0.),
            Dir::Zero => Vector3::new(0., 0., 0.),
        };
        camera.position = camera.position + match dx {
            Dir::Negative => Vector3::new( 0.1, -0.1, 0.),
            Dir::Positive => Vector3::new(-0.1,  0.1, 0.),
            Dir::Zero => Vector3::new(0., 0., 0.),
        };
        camera.position = camera.position + match dzm {
            Dir::Positive => Vector3::new(0., 0., 0.1),
            Dir::Negative => Vector3::new(0., 0., -0.1),
            Dir::Zero => Vector3::new(0., 0., 0.)
        };

        camera.position.z = clamp(1., camera.position.z, 10.);

        let ray = camera.pixel_ray(mouse);
        let plane = Plane::from_abcd(0., 0., 1., 0.);
        println!("{:?} {:?}", ray.origin, ray.direction);
        if let Some(p) = (plane, ray).intersection() {
            renderer.render(camera, p);
        }
        window.swap_buffers().unwrap();
    }
}