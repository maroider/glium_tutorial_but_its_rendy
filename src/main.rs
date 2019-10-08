use rendy::{
    factory::{Config, Factory},
    graph::{
        present::PresentNode,
        render::{RenderPassNodeBuilder, SubpassBuilder},
        GraphBuilder,
    },
    hal,
    vulkan::{Backend, Instance},
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
    let (mut factory, mut families): (Factory<Backend>, _) = rendy::factory::init(config).unwrap();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("glium tutorial but it's rendy")
        .build(&event_loop)
        .unwrap();

    let surface = unsafe {
        factory.create_surface_with(|instance: &Instance| {
            instance.create_surface_from_raw(&window).unwrap()
        })
    };

    let mut graph_builder = GraphBuilder::<Backend, ()>::new();

    let size = window.inner_size().to_physical(window.hidpi_factor());

    let color = graph_builder.create_image(
        hal::image::Kind::D2(size.width as u32, size.height as u32, 1, 1),
        1,
        factory.get_surface_format(&surface),
        Some(hal::command::ClearValue::Color([0.0, 0.0, 1.0, 1.0].into())),
    );

    let pass = graph_builder.add_node(
        RenderPassNodeBuilder::new().with_subpass(SubpassBuilder::new().with_color(color)),
    );

    graph_builder.add_node(PresentNode::builder(&factory, surface, color).with_dependency(pass));

    let mut graph = graph_builder
        .build(&mut factory, &mut families, &())
        .unwrap();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            window_id,
        } if window_id == window.id() => *control_flow = ControlFlow::Exit,
        Event::EventsCleared => {
            graph.run(&mut factory, &mut families, &());
        }
        _ => {}
    });
}
