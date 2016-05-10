use gfx;
use gfx::{Device};
use gfx::traits::FactoryExt;
use gfx_window_glutin;
use gfx_device_gl;
use glutin;
use cgmath::{self, Vector3, Matrix4, EuclideanVector, Transform};
use genmesh::generators::{Cube};
use genmesh::{Triangulate, MapToVertices, Vertices};
use amethyst;
use amethyst::renderer::VertexPosNormal as Vertex;
use amethyst::renderer::target::{ColorFormat, DepthFormat};
use amethyst::renderer::{Frame, Layer};
use amethyst::renderer::pass::*;

fn build_grid() -> Vec<Vertex> {
    Cube::new()
        .vertex(|(x, y, z)| Vertex{
            pos: [x*8., y*8., z],
            normal: Vector3::new(x, y, z).normalize().into()
        })
        .triangulate()
        .vertices()
        .collect()
}

pub struct Renderer {
    device: gfx_device_gl::Device,
    factory: gfx_device_gl::Factory,
    grid: gfx::handle::Buffer<gfx_device_gl::Resources, Vertex>,
    grid_slice: gfx::Slice<gfx_device_gl::Resources>,
    renderer: amethyst::renderer::Renderer<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>,
    frame: amethyst::renderer::Frame<gfx_device_gl::Resources>
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

        let mut renderer = Renderer {
            device: device,
            factory: factory,
            renderer: renderer,
            grid: buffer,
            grid_slice: slice,
            frame: Frame::new()
        };
        renderer.frame.targets.insert(
            "main".into(),
            Box::new(amethyst::renderer::target::ColorBuffer{
                color: main_color,
                output_depth: main_depth
            }
        ));
        renderer.frame.layers = vec![
            Layer::new("gbuffer",
                vec![
                    Clear::new([0., 0., 0., 1.]),
                    DrawNoShading::new("main", "main")
                ]
            ),
            Layer::new("main",
                vec![
                    BlitLayer::new("gbuffer", "ka"),
                    Lighting::new("main", "gbuffer", "main")
                ]
            )
        ];
        let view = cgmath::AffineMatrix3::look_at(
            cgmath::Point3::new(10., 10., 10.),
            cgmath::Point3::new(0., 0., 0.),
            Vector3::unit_z()
        );
        let proj = cgmath::ortho(8., -8., -6., 6., -100., 100.);
        renderer.frame.cameras.insert(
            format!("main"),
            amethyst::renderer::Camera{projection: proj.into(), view: view.mat.into()}
        );
        renderer.frame.targets.insert(
            "gbuffer".into(),
            Box::new(amethyst::renderer::target::GeometryBuffer::new(&mut renderer.factory, (800, 600)))
        );
        (renderer, window)
    }

    pub fn render(&mut self) {
        let mut scene = amethyst::renderer::Scene::new();
        scene.fragments.push(amethyst::renderer::Fragment{
            buffer: self.grid.clone(),
            slice: self.grid_slice.clone(),
            ka: [0.05, 0.05, 0.05, 1.0],
            kd: [0.5, 0.5, 0.5, 1.0],
            transform: Matrix4::from_translation(Vector3::new(0., 0., 0.)).into()
        });
        scene.lights.push(amethyst::renderer::Light{
            color: [1., 1., 1., 1.],
            radius: 1.,
            center: [4., 0., 4.],
            propagation_constant: 0.,
            propagation_linear: 1.,
            propagation_r_square: 1.,
        });
        self.frame.scenes.insert("main".into(), scene);
        self.renderer.submit(&self.frame, &mut self.device);
    }
}