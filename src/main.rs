extern crate cgmath;
extern crate gfx;
extern crate gfx_device_gl;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate genmesh;
extern crate amethyst;

mod renderer;

fn main() {
    let builder = glutin::WindowBuilder::new()
        .with_title("Amethyst Renderer Demo".to_string())
        .with_dimensions(800, 600)
        .with_vsync();


    let (mut renderer, window) = renderer::Renderer::new(builder);

    'main: loop {
        for event in window.poll_events() {
            match event {
                glutin::Event::Closed => {
                    break 'main;
                },
                _ => ()
            }
        }
        renderer.render();
        window.swap_buffers().unwrap();
    }
}