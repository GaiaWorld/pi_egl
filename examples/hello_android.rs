// #![cfg(any(target_os = "android"))]

use glow::{HasContext, COLOR_BUFFER_BIT};
use pi_egl::{Instance, PowerPreference, Surface};

use winit::{
    dpi::PhysicalSize,
    event::Event,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
fn main() {
    std::env::set_var("RUST_BACKTRACE", "full");
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(1024, 768))
        .build(&event_loop)
        .unwrap();

    let mut instance = Instance::new(PowerPreference::HighPerformance, false).unwrap();
    let context = instance.create_context().unwrap();

    let mut gl: Option<&'static glow::Context> = None;
    let mut surface: Option<Surface> = None;
    let mut fps = 0;
    let mut time = std::time::Instant::now();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        // println!("event: {:?}", event);
        match event {
            winit::event::Event::MainEventsCleared => unsafe {
                // gl.Viewport(0, 0, 1024, 768);
                if let Some(gl) = gl {
                    gl.clear_color(1.0, 0.0, 0.0, 1.0);
                    gl.clear(COLOR_BUFFER_BIT);
                }
                window.request_redraw();
            },
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::CloseRequested => {
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                }
                _ => (),
            },

            Event::RedrawRequested(_) => {
                if let Some(gl) = gl {
                    let err = unsafe { gl.get_error() };
                    // println!("gl error:{}", err);
                }
                if let Some(surface) = &surface {
                    instance.swap_buffers(surface);
                    fps += 1;
                    // println!("time: {:?}",time.elapsed().as_millis() );
                    if time.elapsed().as_millis() > 1000 {
                        println!("fps: {}", fps);
                        fps = 0;
                        time = std::time::Instant::now();
                    }
                }
            }
            Event::Resumed => {
                let s = instance.create_surface(&window).unwrap();
                instance.make_current(Some(&s), Some(&context));
                let context = instance.get_glow();

                let context = unsafe {
                    std::mem::transmute::<&'_ glow::Context, &'static glow::Context>(context)
                };

                gl.replace(context);

                surface.replace(s);
            }
            Event::Suspended => {
                let _ = instance.make_current(None, Some(&context));
            }
            _ => {}
        }
    });
}
