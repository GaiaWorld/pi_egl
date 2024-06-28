use std::ptr::NonNull;

use glow::{HasContext, COLOR_BUFFER_BIT};
use libc::c_void;
use log::LevelFilter;
use log4rs::{
    append::console::ConsoleAppender,
    config::{Appender, Root},
    encode::{json::JsonEncoder, writer::simple::SimpleWriter},
};
use pi_egl::{Instance, PowerPreference};
use winit::{
    dpi::PhysicalSize,
    event::Event,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
    let r: *mut c_void  = std::ptr::null_mut();
    let r = NonNull::new(r).unwrap();
    let r = r.as_ptr()
    let stdout = ConsoleAppender::builder().build();
    let log_config = log4rs::config::Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Info))
        .unwrap();
    log4rs::init_config(log_config).unwrap();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(1024, 768))
        .build(&event_loop)
        .unwrap();

    let mut instance = Instance::new(PowerPreference::HighPerformance, false).unwrap();
    let context = instance.create_context().unwrap();
    let surface = instance.create_surface(&window).unwrap();

    instance.make_current(Some(&surface), Some(&context));

    let mut r = 0.0;

    let mut fps = 0;
    let mut time = std::time::Instant::now();
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
                    let gl = instance.get_glow();

                    r += 0.01;
                    if r > 1.0 {
                        r = 0.0;
                    }

                    gl.clear_color(r, 0.0, 0.0, 1.0);
                    gl.clear(COLOR_BUFFER_BIT);

                    // instance.make_current(None, None);
                }

                {
                    // instance.make_current(Some(&surface), Some(&context));

                    instance.swap_buffers(&surface);
                    fps += 1;
                    // println!("time: {:?}",time.elapsed().as_millis() );
                    if time.elapsed().as_millis() > 1000 {
                        log::info!("fps: {}", fps);
                        println!("fps: {}", fps);
                        fps = 0;
                        time = std::time::Instant::now();
                    }
                    // instance.make_current(None, None);
                }
            }
            _ => {}
        }
    });
}
