use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle, RawWindowHandle};
use wasm_bindgen::JsCast;

use super::{context::WebContext, surface::WebSurface};
use crate::{InstanceError, PowerPreference};

lazy_static! {
    pub static ref ID: AtomicU64 = AtomicU64::from(0);
}

#[derive(Debug)]
pub struct WebInstance(Option<WebSurface>);

impl WebInstance {
    #[inline]
    pub fn new(power: PowerPreference, _is_vsync: bool) -> Result<Self, InstanceError> {
        Ok(WebInstance(None))
    }

    pub fn create_surface<W: HasRawWindowHandle + HasRawDisplayHandle>(
        &self,
        window: &W,
    ) -> Result<WebSurface, InstanceError> {
        let canvas_attribute = if let RawWindowHandle::Web(handle) = window.raw_window_handle() {
            handle.id
        } else {
            return Err(InstanceError::IncompatibleWindowHandle);
        };

        let canvas_node: wasm_bindgen::JsValue = web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                doc.query_selector_all(&format!("[data-raw-handle=\"{canvas_attribute}\"]"))
                    .ok()
            })
            .and_then(|nodes| nodes.get(0))
            .expect("expected to find single canvas")
            .into();
        let canvas: web_sys::HtmlCanvasElement = canvas_node.into();
        let webgl2_context: web_sys::WebGl2RenderingContext = canvas
            .get_context("webgl2")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::WebGl2RenderingContext>()
            .unwrap();
        let id = ID.fetch_add(1, Ordering::Relaxed);

        Ok(WebSurface {
            context: Arc::new(glow::Context::from_webgl2_context(webgl2_context)),
            id,
        })
    }

    #[inline]
    pub fn create_context(&self) -> Result<WebContext, InstanceError> {
        return Ok(WebContext);
    }

    pub fn make_current<'a>(
        &'a mut self,
        surface: Option<&'a WebSurface>,
        context: Option<&WebContext>,
    ) {
        if let Some(surface) = surface {
            if let Some(bind_surface) = &self.0 {
                if bind_surface == surface {
                    return;
                }
            }
            self.0.replace(surface.clone());
        }
    }

    #[inline]
    pub fn get_glow<'a>(&'a self) -> &glow::Context {
        self.0.as_ref().unwrap().context.as_ref()
    }

    #[inline]
    pub fn swap_buffers(&self, surface: &WebSurface) {}
}
