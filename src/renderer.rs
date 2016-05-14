use gfx;
use gfx::{Device};
use gfx::traits::{Factory, FactoryExt};
use gfx_window_glutin;
use gfx_device_gl;
use glutin;
use cgmath::{self, Vector3, Matrix4, Transform};
use genmesh::generators::Plane;
use genmesh::{Triangulate, MapToVertices, Vertices};
use amethyst;
use amethyst::renderer::VertexPosNormal as Vertex;
use amethyst::renderer::target::{ColorFormat, DepthFormat};
use amethyst::renderer::{Frame, Layer};

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
    factory: gfx_device_gl::Factory,
    grid: gfx::handle::Buffer<gfx_device_gl::Resources, Vertex>,
    grid_slice: gfx::Slice<gfx_device_gl::Resources>,
    grid_texture : gfx::handle::ShaderResourceView<gfx_device_gl::Resources, [f32; 4]>,
    renderer: amethyst::renderer::Renderer<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>,
    frame: amethyst::renderer::Frame<gfx_device_gl::Resources>,
    size: (f32, f32),
    scale: f32
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

        let data = build_texture(16, 16);
        let data = vec![&data[..]];
        let (_, text) = factory.create_texture_const_u8::<gfx::format::Rgba8>(
            gfx::tex::Kind::D2(16, 16, gfx::tex::AaMode::Single),
            &data[..]
        ).unwrap();

        let mut renderer = Renderer {
            device: device,
            factory: factory,
            renderer: renderer,
            grid: buffer,
            grid_slice: slice,
            grid_texture: text,
            frame: Frame::new(),
            size: (800., 600.),
            scale: 5.
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

        let (w, h) = window.get_inner_size_points().unwrap();
        self.size = (w as f32, h as f32);
    }

    pub fn render(&mut self, x: f32, y: f32, zm: f32) {
        self.scale = zm;
        let mut scene = amethyst::renderer::Scene::new();
        scene.fragments.push(amethyst::renderer::Fragment{
            buffer: self.grid.clone(),
            slice: self.grid_slice.clone(),
            ka: amethyst::renderer::Texture::Constant([0., 0., 0., 1.]),
            kd: amethyst::renderer::Texture::Texture(self.grid_texture.clone()),
            transform: Matrix4::from_translation(Vector3::new(0., 0., 0.)).into()
        });
        scene.lights.push(amethyst::renderer::Light{
            color: [1., 1., 1., 1.],
            radius: 1.,
            center: [4., 0., 4.],
            propagation_constant: 0.,
            propagation_linear: 1.,
            propagation_r_square: 0.,
        });
        let view = cgmath::AffineMatrix3::look_at(
            cgmath::Point3::new(x + 1., y + 1., 1.),
            cgmath::Point3::new(x, y, 0.),
            Vector3::unit_z()
        );

        let angle = (self.size.0 / self.size.1).atan();
        let w = angle.sin() * self.scale;
        let h = angle.cos() * self.scale;
        let proj = cgmath::ortho(-w, w, -h, h, -1000., 1000.);
        self.frame.cameras.insert(
            format!("main"),
            amethyst::renderer::Camera{projection: proj.into(), view: view.mat.into()}
        );
        self.frame.scenes.insert("main".into(), scene);
        self.renderer.submit(&self.frame, &mut self.device);
    }
}