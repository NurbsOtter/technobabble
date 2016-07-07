use gfx;
use gfx::{Device};
use gfx::traits::{Factory, FactoryExt};
use gfx_window_glutin;
use gfx_device_gl;
use glutin;
use cgmath::{Vector3, Matrix4};
use genmesh::generators::{Plane, Cube};
use genmesh::{Triangulate, MapToVertices, Vertices};
use amethyst;
use amethyst::renderer::VertexPosNormal as Vertex;
use amethyst::renderer::target::{ColorFormat, DepthFormat};
use amethyst::renderer::{Frame, Layer};
use ecs::{self, Join};

use camera;
use transform::Transform;
use PreviewMarker;

fn build_grid() -> Vec<Vertex> {
    Plane::subdivide(256, 256)
        .vertex(|(x, y)| Vertex{
            pos: [x*16., y*16., 0.],
            normal: [0., 0., 1.],
            tex_coord: [0., 0.]
        })
        .map(|mut x| {
            x.x.tex_coord = [0., 0.];
            x.y.tex_coord = [1., 0.];
            x.z.tex_coord = [1., 1.];
            x.w.tex_coord = [0., 1.];
            x
        })
        .triangulate()
        .vertices()
        .collect()
}

fn build_cube() -> Vec<Vertex> {
    Cube::new()
        .vertex(|(x, y, z)| Vertex{
            pos: [x * 0.0625, y * 0.0625, z * 0.125 + 0.125],
            normal: [x, y, z],
            tex_coord: [0., 0.]
        })
        .triangulate()
        .vertices()
        .collect()
}

fn build_texture(x: u16, y: u16) -> Vec<u8> {
    let mut out = Vec::with_capacity(x as usize * y as usize * 4);
    for ix in 0..x {
        for iy in 0..y {
            let edge = ix == 0 || x == (ix - 1) ||
                       iy == 0 || y == (iy - 1);
            let color = if edge { 0 } else { 128 };
            out.push(color);
            out.push(color);
            out.push(color);
            out.push(255);
        }
    }
    out
}

pub struct Renderer {
    device: gfx_device_gl::Device,
    _factory: gfx_device_gl::Factory,
    grid: gfx::handle::Buffer<gfx_device_gl::Resources, Vertex>,
    grid_slice: gfx::Slice<gfx_device_gl::Resources>,
    grid_texture : gfx::handle::ShaderResourceView<gfx_device_gl::Resources, [f32; 4]>,
    cube: gfx::handle::Buffer<gfx_device_gl::Resources, Vertex>,
    cube_slice: gfx::Slice<gfx_device_gl::Resources>,
    renderer: amethyst::renderer::Renderer<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>,
    frame: amethyst::renderer::Frame<gfx_device_gl::Resources>,
}

impl Renderer {
    pub fn new(builder: glutin::WindowBuilder) -> (Renderer, glutin::Window) {
        let (window, device, mut factory, main_color, main_depth) =
            gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder);

        let combuf = factory.create_command_buffer();
        let mut renderer = amethyst::renderer::Renderer::new(combuf);
        renderer.load_all(&mut factory);

        let grid = build_grid();
        let (buffer, slice) = factory.create_vertex_buffer_with_slice(&grid, ());

        let cube = build_cube();
        let (cube_buffer, cube_slice) = factory.create_vertex_buffer_with_slice(&cube, ());

        let data = build_texture(16, 16);
        let data = vec![&data[..]];
        let (_, text) = factory.create_texture_const_u8::<gfx::format::Rgba8>(
            gfx::tex::Kind::D2(16, 16, gfx::tex::AaMode::Single),
            &data[..]
        ).unwrap();

        let mut renderer = Renderer {
            device: device,
            _factory: factory,
            renderer: renderer,
            grid: buffer,
            grid_slice: slice,
            grid_texture: text,
            frame: Frame::new(),
            cube: cube_buffer,
            cube_slice: cube_slice
        };
        renderer.frame.targets.insert(
            "main".into(),
            Box::new(amethyst::renderer::target::ColorBuffer{
                color: main_color,
                output_depth: main_depth
            }
        ));
        renderer.frame.layers = vec![
            Layer::new("main",
                vec![
                    amethyst::renderer::pass::Clear::new([0., 0., 0., 1.]),
                    amethyst::renderer::pass::DrawShaded::new("main", "main")
                ]
            )
        ];
        (renderer, window)
    }

    pub fn resize(&mut self, window: &glutin::Window) {
        let output = self.frame.targets.get_mut("main").unwrap();
        let out = output.downcast_mut::<amethyst::renderer::target::ColorBuffer<gfx_device_gl::Resources>>();
        let out = out.unwrap();
        gfx_window_glutin::update_views(
            window,
            &mut out.color,
            &mut out.output_depth
        );
    }

    pub fn render(&mut self, camera: camera::Camera, world: &ecs::World) {
        let mut scene = amethyst::renderer::Scene::new();
        scene.fragments.push(amethyst::renderer::Fragment{
            buffer: self.grid.clone(),
            slice: self.grid_slice.clone(),
            ka: amethyst::renderer::Texture::Constant([0., 0., 0., 1.]),
            kd: amethyst::renderer::Texture::Texture(self.grid_texture.clone()),
            transform: Matrix4::from_translation(Vector3::new(0., 0., 0.)).into()
        });

        let transform = world.read::<Transform>();
        let preview = world.read::<PreviewMarker>();

        for (t, _) in (&transform, !&preview).iter() {
            scene.fragments.push(amethyst::renderer::Fragment{
                buffer: self.cube.clone(),
                slice: self.cube_slice.clone(),
                ka: amethyst::renderer::Texture::Constant([0., 0., 0., 1.]),
                kd: amethyst::renderer::Texture::Constant([1., 0., 0., 1.]),
                transform: t.model_matrix().into(),
            })
        }

        for (t, _) in (&transform, &preview).iter() {
            scene.fragments.push(amethyst::renderer::Fragment{
                buffer: self.cube.clone(),
                slice: self.cube_slice.clone(),
                ka: amethyst::renderer::Texture::Constant([0., 0., 0., 1.]),
                kd: amethyst::renderer::Texture::Constant([0., 1., 0., 1.]),
                transform: t.model_matrix().into(),
            })
        }

        /*scene.fragments.push(amethyst::renderer::Fragment{
            buffer: self.cube.clone(),
            slice: self.cube_slice.clone(),
            ka: amethyst::renderer::Texture::Constant([0., 0., 0., 1.]),
            kd: amethyst::renderer::Texture::Constant([1., 0., 0., 1.]),
            transform: Matrix4::from_translation(Vector3::new(b.x, b.y, b.z)).into()
        });*/
        scene.lights.push(amethyst::renderer::Light{
            color: [1., 1., 1., 1.],
            radius: 1.,
            center: [4., 0., 4.],
            propagation_constant: 0.,
            propagation_linear: 1.,
            propagation_r_square: 0.,
        });
        self.frame.cameras.insert(
            format!("main"),
            amethyst::renderer::Camera{
                projection: camera.projection().into(),
                view: camera.view().into()
            }
        );
        self.frame.scenes.insert("main".into(), scene);
        self.renderer.submit(&self.frame, &mut self.device);
    }
}