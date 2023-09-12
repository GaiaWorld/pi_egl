use pi_egl::{
    init_env, platform::windows::util::get_proc_address, Instance, PowerPreference,
    COLOR_BUFFER_BIT, GL,
};

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

    let instance = Instance::new(PowerPreference::HighPerformance, true).unwrap();
    let context = instance.create_context(&window).unwrap();
    let surface = instance.create_surface(&window).unwrap();

    let gl = instance
        .make_current(Some(&surface), Some(&context))
        .unwrap();
    let gl = unsafe { std::mem::transmute::<&'_ GL, &'static GL>(gl) };

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            winit::event::Event::MainEventsCleared => unsafe {
                // gl.Viewport(0, 0, 1024, 768);
                gl.ClearColor(1.0, 0.0, 0.0, 1.0);
                gl.Clear(COLOR_BUFFER_BIT);
            },
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::CloseRequested => {
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                }
                _ => (),
            },

            Event::RedrawRequested(_) => {
                unsafe { println!("error: {}", gl.GetError()) };
                instance.swap_buffers(&surface)
            }
            _ => {}
        }
    });
}
