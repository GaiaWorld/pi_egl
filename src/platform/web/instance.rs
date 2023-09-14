use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle, RawWindowHandle};
use wasm_bindgen::JsCast;

use super::{context::WebContext, surface::WebSurface};
use crate::{InstanceError, PowerPreference};

#[derive(Debug)]
pub struct WebInstance(Option<glow::Context>);

impl WebInstance {
    pub fn new(power: PowerPreference, _is_vsync: bool) -> Result<Self, InstanceError> {
        Ok(WebInstance(None))
    }

    // 带双缓冲的 Surface
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

        Ok(WebSurface(glow::Context::from_webgl2_context(
            webgl2_context,
        )))
    }

    #[allow(non_snake_case)]
    pub fn create_context(&self) -> Result<WebContext, InstanceError> {
        return Ok(WebContext);
    }

    // 调用了这个之后，gl的函数 才能用；
    // wasm32 cfg 空实现
    pub fn make_current<'a>(
        &'a mut self,
        surface: Option<&'a WebSurface>,
        context: Option<&WebContext>,
    ) -> Option<&glow::Context> {
        if let Some(context) = context {
            if let Some(surface) = surface {
                return Some(&surface.0);
            }
        }
        None
    }

    // 交换 Surface 中的 双缓冲
    // wasm32 cfg 空实现
    pub fn swap_buffers(&self, surface: &WebSurface) {
        // unsafe { SwapBuffers(surface.0 as HDC) };
    }
}
