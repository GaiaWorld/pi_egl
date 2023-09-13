use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle, RawWindowHandle};
use wasm_bindgen::JsCast;

use crate::{InstanceError, PowerPreference};

use super::{context::WebContext, surface::WebSurface};

pub struct WebInstance(Option<glow::Context>);

impl WebInstance {
    pub fn new(power: PowerPreference, _is_vsync: bool) -> Result<Self, InstanceError> {
        Ok(WebInstance)
    }

    // 带双缓冲的 Surface
    pub fn create_surface<W: HasRawWindowHandle + HasRawDisplayHandle>(
        &self,
        window: &W,
    ) -> Result<WebSurface, InstanceError> {
        Ok(WebSurface)
    }

    #[allow(non_snake_case)]
    pub fn create_context<W: HasRawWindowHandle + HasRawDisplayHandle>(
        &self,
        window: &W,
    ) -> Result<WebContext, InstanceError> {
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

        if self.0.is_none() {
            self.0
                .replace(glow::Context::from_webgl2_context(webgl2_context));
        }
        return Ok(WebContext);
    }

    // 调用了这个之后，gl的函数 才能用；
    // wasm32 cfg 空实现
    pub fn make_current(
        &self,
        surface: Option<&WebSurface>,
        context: Option<&WebContext>,
    ) -> Option<&glow::Context> {
        if let Some(context) = context {
            if let Some(surface) = surface {
                return Some(self.0.as_ref().unwrap());
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
