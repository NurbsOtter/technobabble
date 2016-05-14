extern crate cgmath;
extern crate gfx;
extern crate gfx_device_gl;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate genmesh;
extern crate amethyst;

mod renderer;
use glutin::ElementState::{Pressed, Released};
use glutin::VirtualKeyCode as Key;

enum Dir {
    Positive,
    Zero,
    Negative
}

fn main() {
    let builder = glutin::WindowBuilder::new()
        .with_title("Technobabble".to_string())
        .with_dimensions(800, 600)
        .with_vsync();


    let (mut renderer, window) = renderer::Renderer::new(builder);

    let (mut x, mut y , mut zm) = (0., 0., 8.);
    let (mut dx, mut dy, mut dzm) = (Dir::Zero, Dir::Zero, Dir::Zero);

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
                    dzm = Dir::Positive;
                },
                glutin::Event::KeyboardInput(Pressed, _, Some(Key::Z)) => {
                    dzm = Dir::Negative;
                },
                glutin::Event::KeyboardInput(Released, _, Some(Key::Q)) |
                glutin::Event::KeyboardInput(Released, _, Some(Key::Z)) => {
                    dzm = Dir::Zero;
                },
                glutin::Event::Resized(_, _) => {
                    renderer.resize(&window);
                }
                _ => ()
            }
        }

        match dy {
            Dir::Negative => {
                x += 0.1;
                y += 0.1;
            }
            Dir::Positive => {
                x -= 0.1;
                y -= 0.1;
            }
            Dir::Zero => ()
        }

        match dx {
            Dir::Positive => {
                x -= 0.1;
                y += 0.1;
            }
            Dir::Negative => {
                x += 0.1;
                y -= 0.1;
            }
            Dir::Zero => ()
        }
        
        match dzm {
            Dir::Positive => {
                zm -= 0.1;   
            }
            Dir::Negative => {
                zm += 0.1;    
            }
            Dir::Zero => ()
        }

        renderer.render(x, y, zm);
        window.swap_buffers().unwrap();
    }
}