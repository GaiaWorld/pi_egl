use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use super::{context::WebContext, surface::WebSurface};
use crate::{InstanceError, PowerPreference};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle, RawWindowHandle};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::js_sys;

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
        let canvas_attribute = if let Ok(RawWindowHandle::Web(handle)) = window.raw_window_handle()
        {
            handle.id
        } else {
            return Err(InstanceError::IncompatibleWindowHandle);
        };
        let window = web_sys::window().unwrap();
        let mut canvas = None;
        let user = window.navigator().user_agent();
        log::error!("navigator user : {:?}", user);
        if let Ok(user) = user {
            if user.contains("wechatdevtools") || user.contains("PI_WX_GAME") {
                log::error!("egl minigame!!!!!");
                canvas = Some(
                    js_sys::Reflect::get(&window, &"canvas".to_string().into())
                        .unwrap_or(JsValue::null()),
                );
            }
        }

        if canvas.is_none() {
            canvas = Some(
                window
                    .document()
                    .and_then(|doc| {
                        doc.query_selector_all(&format!("[data-raw-handle=\"{canvas_attribute}\"]"))
                            .ok()
                    })
                    .and_then(|nodes| nodes.get(0))
                    .expect("expected to find single canvas").into(),
            );
        }

        let canvas = canvas.take().unwrap();

        if canvas.is_null() || canvas.is_undefined() {
            log::error!("create_surface 000");
        }

        log::error!("create_surface 1111");
        let canvas: web_sys::HtmlCanvasElement = canvas.into();
        log::error!("create_surface 222");

        let webgl2_context: wasm_bindgen::JsValue = match canvas.get_context("webgl2") {
            Ok(v) => match v {
                Some(v) => v.into(),
                None => wasm_bindgen::throw_str("webgl2 is none!!!"),
            },
            Err(err) => wasm_bindgen::throw_str(&format!("get webgl2 failed!!!{:?}", err)),
        };

        let id = ID.fetch_add(1, Ordering::Relaxed);

        Ok(WebSurface {
            context: Arc::new(glow::Context::from_webgl2_context(webgl2_context.into())),
            id,
        })
    }

    #[inline]
    pub fn create_context(&self) -> Result<WebContext, InstanceError> {
        let value: wasm_bindgen::JsValue = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("canvas")
            .unwrap()
            .into();

        let canvas: web_sys::HtmlCanvasElement = value.into();

        let webgl2_context: wasm_bindgen::JsValue =
            canvas.get_context("webgl2").unwrap().unwrap().into();

        let id = ID.fetch_add(1, Ordering::Relaxed);

        return Ok(WebContext {
            context: Arc::new(glow::Context::from_webgl2_context(webgl2_context.into())),
            id,
        });
    }

    pub fn make_current<'a>(
        &'a mut self,
        surface: Option<&'a WebSurface>,
        context: Option<&WebContext>,
    ) {
        if let Some(context) = context {
            if let Some(surface) = surface {
                if let Some(bind_context) = &self.0 {
                    if bind_context == surface {
                        return;
                    }
                }
                self.0.replace(surface.clone());
            } else {
                if let Some(bind_context) = &self.0 {
                    if bind_context == context {
                        return;
                    }
                }
                self.0.replace(context.clone());
            }
        }
    }

    #[inline]
    pub fn get_glow<'a>(&'a self) -> &glow::Context {
        self.0.as_ref().unwrap().context.as_ref()
    }

    #[inline]
    pub fn swap_buffers(&self, surface: &WebSurface) {}
}
