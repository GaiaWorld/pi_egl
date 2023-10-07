use glow::{HasContext, COLOR_BUFFER_BIT};
use pi_egl::{Instance, PowerPreference};

use winit::{
    dpi::PhysicalSize,
    event::Event,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(1024, 768))
        .build(&event_loop)
        .unwrap();

    let mut instance = Instance::new(PowerPreference::HighPerformance, false).unwrap();
    let context = instance.create_context().unwrap();
    let surface = instance.create_surface(&window).unwrap();

    let mut r = 0.0;
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            winit::event::Event::MainEventsCleared => {
                window.request_redraw();
            }
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::CloseRequested => {
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                }
                _ => (),
            },

            Event::RedrawRequested(_) => {
                unsafe {
                    instance
                        .make_current(Some(&surface), Some(&context))
                        .unwrap();

                    let gl = instance.get_glow();

                    r += 0.01;
                    if r > 1.0 {
                        r = 0.0;
                    }

                    gl.clear_color(r, 0.0, 0.0, 1.0);
                    gl.clear(COLOR_BUFFER_BIT);

                    instance.make_current(None, None);
                }

                {
                    instance
                        .make_current(Some(&surface), Some(&context))
                        .unwrap();

                    instance.swap_buffers(&surface);

                    instance.make_current(None, None);
                }
            }
            _ => {}
        }
    });
}
