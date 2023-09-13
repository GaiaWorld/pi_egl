use glow::{HasContext, COLOR_BUFFER_BIT};
use pi_egl::{init_env, Instance, PowerPreference};

use winit::{
    dpi::PhysicalSize,
    event::Event,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

init_env!();

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(1024, 768))
        .build(&event_loop)
        .unwrap();

    let mut instance = Instance::new(PowerPreference::HighPerformance, true).unwrap();
    let context = instance.create_context().unwrap();
    let surface = instance.create_surface(&window).unwrap();

    let gl = instance
        .make_current(Some(&surface), Some(&context))
        .unwrap();
    let gl = unsafe { std::mem::transmute::<&'_  glow::Context, &'static glow::Context>(gl) };

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            winit::event::Event::MainEventsCleared => unsafe {
                // gl.Viewport(0, 0, 1024, 768);
                gl.clear_color(1.0, 0.0, 0.0, 1.0);
                gl.clear(COLOR_BUFFER_BIT);
            },
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::CloseRequested => {
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                }
                _ => (),
            },

            Event::RedrawRequested(_) => {
                unsafe { println!("error: {}", gl.get_error()) };
                instance.swap_buffers(&surface)
            }
            _ => {}
        }
    });
}
