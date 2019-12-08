use std::{fs::File, io::BufReader};

use rendy::{
    command::{QueueId, RenderPassEncoder},
    core::types::vertex::{AsAttribute, AsVertex, VertexFormat},
    factory::{Config, Factory, ImageState},
    graph::{present::PresentNode, render::*, GraphBuilder, GraphContext, NodeBuffer, NodeImage},
    hal::{self, device::Device as _},
    init::AnyWindowedRendy,
    memory::Dynamic,
    resource::{Buffer, BufferInfo, DescriptorSet, DescriptorSetLayout, Escape, Handle},
    shader::{ShaderKind, SourceLanguage, SourceShaderInfo, SpirvReflection, SpirvShader},
    texture::{image::ImageTextureConfig, Texture},
    vulkan::Backend,
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    env_logger::Builder::from_default_env()
        .filter_module("glium_tutorial_but_its_rendy", log::LevelFilter::Trace)
        .init();

    let config: Config = Default::default();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_title("glium tutorial but it's rendy");

    let rendy = AnyWindowedRendy::init_auto(&config, window, &event_loop).unwrap();
    rendy::with_any_windowed_rendy!((rendy)
            (mut factory, mut families, surface, window) => {
                let mut graph_builder = GraphBuilder::<Backend, _>::new();

                let size = window.inner_size().to_physical(window.hidpi_factor());

                let color = graph_builder.create_image(
                    hal::image::Kind::D2(size.width as u32, size.height as u32, 1, 1),
            1,
            factory.get_surface_format(&surface),
            Some(
                hal::command::ClearValue {
                    color: hal::command::ClearColor{ float32: [0.0, 0.0, 1.0, 1.0] },
                }
            ),
        );

        let pass = graph_builder.add_node(
            TutorialRenderPipeline::builder()
            .into_subpass()
            .with_color(color)
            .into_pass(),
        );

        graph_builder.add_node(PresentNode::builder(&factory, surface, color).with_dependency(pass));

        let mut t: f32 = -0.5;

        let mut graph = graph_builder
        .build(&mut factory, &mut families, &t)
        .unwrap();

        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            Event::EventsCleared => {
                t += 0.0002;
                if t > 0.5 {
                    t = -0.5;
                }

                graph.run(&mut factory, &mut families, &t);
            }
            _ => {}
        });
    });
}

#[derive(Debug, Default)]
struct TutorialRenderPipelineDesc;

impl<B> SimpleGraphicsPipelineDesc<B, f32> for TutorialRenderPipelineDesc
where
    B: hal::Backend,
{
    type Pipeline = TutorialRenderPipeline<B>;

    fn depth_stencil(&self) -> Option<hal::pso::DepthStencilDesc> {
        None
    }

    fn load_shader_set(&self, factory: &mut Factory<B>, _aux: &f32) -> rendy::shader::ShaderSet<B> {
        SHADERS.build(factory, Default::default()).unwrap()
    }

    fn vertices(
        &self,
    ) -> Vec<(
        Vec<hal::pso::Element<hal::format::Format>>,
        hal::pso::ElemStride,
        hal::pso::VertexInputRate,
    )> {
        vec![SHADER_REFLECTION
            .attributes_range(..)
            .unwrap()
            .gfx_vertex_input_desc(hal::pso::VertexInputRate::Vertex)]
    }

    fn layout(&self) -> Layout {
        // SHADER_REFLECTION.layout().unwrap()
        Layout {
            sets: vec![SetLayout {
                bindings: vec![
                    hal::pso::DescriptorSetLayoutBinding {
                        binding: 0,
                        ty: hal::pso::DescriptorType::UniformBuffer,
                        count: 1,
                        stage_flags: hal::pso::ShaderStageFlags::VERTEX,
                        immutable_samplers: false,
                    },
                    hal::pso::DescriptorSetLayoutBinding {
                        binding: 1,
                        ty: hal::pso::DescriptorType::SampledImage,
                        count: 1,
                        stage_flags: hal::pso::ShaderStageFlags::FRAGMENT,
                        immutable_samplers: false,
                    },
                    hal::pso::DescriptorSetLayoutBinding {
                        binding: 2,
                        ty: hal::pso::DescriptorType::Sampler,
                        count: 1,
                        stage_flags: hal::pso::ShaderStageFlags::FRAGMENT,
                        immutable_samplers: false,
                    },
                ],
            }],
            push_constants: Vec::new(),
        }
    }

    fn build<'a>(
        self,
        _ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        queue: QueueId,
        _aux: &f32,
        buffers: Vec<NodeBuffer>,
        images: Vec<NodeImage>,
        set_layouts: &[Handle<DescriptorSetLayout<B>>],
    ) -> Result<TutorialRenderPipeline<B>, hal::pso::CreationError> {
        assert!(buffers.is_empty());
        assert!(images.is_empty());
        assert_eq!(set_layouts.len(), 1);

        let image_reader = BufReader::new(File::open("assets/opengl.png").map_err(|err| {
            log::error!("Unable to open {}: {:?}", "assets/opengl.png", err);
            hal::pso::CreationError::Other
        })?);

        let texture_builder = rendy::texture::image::load_from_image(
            image_reader,
            ImageTextureConfig {
                generate_mips: true,
                ..Default::default()
            },
        )
        .map_err(|e| {
            log::error!("Unable to load image: {:?}", e);
            hal::pso::CreationError::Other
        })?;

        let texture = texture_builder
            .build(
                ImageState {
                    queue,
                    stage: hal::pso::PipelineStage::FRAGMENT_SHADER,
                    access: hal::image::Access::SHADER_READ,
                    layout: hal::image::Layout::ShaderReadOnlyOptimal,
                },
                factory,
            )
            .unwrap();

        let uniform_buffer = factory
            .create_buffer(
                BufferInfo {
                    size: UNIFORM_LOCALS_SIZE,
                    usage: hal::buffer::Usage::UNIFORM,
                },
                Dynamic,
            )
            .unwrap();

        dbg!(set_layouts);

        let descriptor_set = factory
            .create_descriptor_set(set_layouts[0].clone())
            .unwrap();

        unsafe {
            factory.device().write_descriptor_sets(vec![
                hal::pso::DescriptorSetWrite {
                    set: descriptor_set.raw(),
                    binding: 0,
                    array_offset: 0,
                    descriptors: vec![hal::pso::Descriptor::Buffer(
                        uniform_buffer.raw(),
                        None..Some(UNIFORM_LOCALS_SIZE),
                    )],
                },
                hal::pso::DescriptorSetWrite {
                    set: descriptor_set.raw(),
                    binding: 1,
                    array_offset: 0,
                    descriptors: vec![hal::pso::Descriptor::Image(
                        texture.view().raw(),
                        hal::image::Layout::ShaderReadOnlyOptimal,
                    )],
                },
                hal::pso::DescriptorSetWrite {
                    set: descriptor_set.raw(),
                    binding: 2,
                    array_offset: 0,
                    descriptors: vec![hal::pso::Descriptor::Sampler(texture.sampler().raw())],
                },
            ])
        };

        let vbuf_size = SHADER_REFLECTION.attributes_range(..).unwrap().stride as u64 * 3;

        let mut vbuf = factory
            .create_buffer(
                BufferInfo {
                    size: vbuf_size,
                    usage: hal::buffer::Usage::VERTEX,
                },
                Dynamic,
            )
            .unwrap();

        unsafe {
            factory
                .upload_visible_buffer(
                    &mut vbuf,
                    0,
                    &[
                        Vertex {
                            position: [-0.5, 0.5].into(),
                            tex_coords: [0.0, 0.0].into(),
                        },
                        Vertex {
                            position: [0.0, -0.5].into(),
                            tex_coords: [0.0, 1.0].into(),
                        },
                        Vertex {
                            position: [0.5, 0.25].into(),
                            tex_coords: [1.0, 0.0].into(),
                        },
                    ],
                )
                .unwrap();
        }

        Ok(TutorialRenderPipeline {
            texture,
            uniform: uniform_buffer,
            vertex: vbuf,
            descriptor_set,
        })
    }
}

#[derive(Debug)]
struct TutorialRenderPipeline<B: hal::Backend> {
    texture: Texture<B>,
    uniform: Escape<Buffer<B>>,
    vertex: Escape<Buffer<B>>,
    descriptor_set: Escape<DescriptorSet<B>>,
}

impl<B> SimpleGraphicsPipeline<B, f32> for TutorialRenderPipeline<B>
where
    B: hal::Backend,
{
    type Desc = TutorialRenderPipelineDesc;

    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        _set_layouts: &[Handle<DescriptorSetLayout<B>>],
        _index: usize,
        aux: &f32,
    ) -> PrepareResult {
        unsafe {
            factory
                .upload_visible_buffer(
                    &mut self.uniform,
                    0,
                    &[UniformLocals {
                        matrix: [
                            [aux.cos(), aux.sin(), 0.0, 0.0],
                            [-aux.sin(), aux.cos(), 0.0, 0.0],
                            [0.0, 0.0, 1.0, 0.0],
                            [0.0, 0.0, 0.0, 1.0],
                        ],
                    }],
                )
                .unwrap()
        };
        PrepareResult::DrawReuse
    }

    fn draw(
        &mut self,
        layout: &B::PipelineLayout,
        mut encoder: RenderPassEncoder<'_, B>,
        _index: usize,
        _aux: &f32,
    ) {
        unsafe {
            encoder.bind_graphics_descriptor_sets(
                layout,
                0,
                std::iter::once(self.descriptor_set.raw()),
                std::iter::empty(),
            );
            encoder.bind_vertex_buffers(0, Some((self.vertex.raw(), 0)));
            encoder.draw(0..3, 0..1);
        }
    }

    fn dispose(self, _factory: &mut Factory<B>, _aux: &f32) {}
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
struct Vertex {
    position: Position,
    tex_coords: TexCoords,
}

impl AsVertex for Vertex {
    fn vertex() -> VertexFormat {
        VertexFormat::new((Position::vertex(), TexCoords::vertex()))
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Position(pub [f32; 2]);
impl<T> From<T> for Position
where
    T: Into<[f32; 2]>,
{
    fn from(from: T) -> Self {
        Position(from.into())
    }
}
impl AsAttribute for Position {
    const NAME: &'static str = "position";
    const FORMAT: hal::format::Format = hal::format::Format::Rg32Sfloat;
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct TexCoords(pub [f32; 2]);
impl<T> From<T> for TexCoords
where
    T: Into<[f32; 2]>,
{
    fn from(from: T) -> Self {
        TexCoords(from.into())
    }
}
impl AsAttribute for TexCoords {
    const NAME: &'static str = "tex_coords";
    const FORMAT: hal::format::Format = hal::format::Format::Rg32Sfloat;
}

#[derive(Clone, Copy)]
#[repr(C, align(16))]
struct UniformLocals {
    matrix: [[f32; 4]; 4],
}

const UNIFORM_LOCALS_SIZE: u64 = std::mem::size_of::<UniformLocals>() as u64;

lazy_static::lazy_static! {
    static ref VERTEX: SpirvShader = SourceShaderInfo::new(
        include_str!("06.shader.vert"),
        concat!(env!("CARGO_MANIFEST_DIR"), "/src/06.shader.vert").into(),
        ShaderKind::Vertex,
        SourceLanguage::GLSL,
        "main",
    ).precompile().unwrap();

    static ref FRAGMENT: SpirvShader = SourceShaderInfo::new(
        include_str!("06.shader.frag"),
        concat!(env!("CARGO_MANIFEST_DIR"), "/src/06.shader.frag").into(),
        ShaderKind::Fragment,
        SourceLanguage::GLSL,
        "main",
    ).precompile().unwrap();

    static ref SHADERS: rendy::shader::ShaderSetBuilder = rendy::shader::ShaderSetBuilder::default()
        .with_vertex(&*VERTEX).unwrap()
        .with_fragment(&*FRAGMENT).unwrap();
}

lazy_static::lazy_static! {
    static ref SHADER_REFLECTION: SpirvReflection = SHADERS.reflect().unwrap();
}
