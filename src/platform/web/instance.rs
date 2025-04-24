use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use super::{context::WebContext, surface::WebSurface};
use crate::{InstanceError, PowerPreference};
use glow::HasContext;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle, RawWindowHandle};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::js_sys;

lazy_static! {
    pub static ref ID: AtomicU64 = AtomicU64::from(0);
}

/// 代表一个WebGL实例，用于创建和管理WebGL上的表面和上下文。
#[derive(Debug)]
pub struct WebInstance(Option<WebSurface>);

impl WebInstance {
    /// 创建一个新的WebGL实例。
    ///
    /// # 参数
    /// - `power`: 表示GPU的功耗偏好。
    /// - `_is_vsync`: 是否启用垂直同步。
    #[inline]
    pub fn new(power: PowerPreference, _is_vsync: bool) -> Result<Self, InstanceError> {
        Ok(WebInstance(None))
    }

    /// 为提供的窗口创建一个新的WebGL表面。
    ///
    /// # 参数
    /// - `window`: 包含原始窗口句柄的窗口。
    #[inline]
    pub fn create_surface<W: HasRawWindowHandle + HasRawDisplayHandle>(
        &self,
        window: &W,
    ) -> Result<WebSurface, InstanceError> {
        // 获取窗口的原始句柄ID。
        let canvas_attribute = if let Ok(RawWindowHandle::Web(handle)) = window.raw_window_handle()  
        {
            handle.id
        } else {
            return Err(InstanceError::IncompatibleWindowHandle);
        };

        // 获取全局window对象。
        let window = web_sys::window().unwrap();

        // 尝试获取或创建canvas元素。
        let mut canvas = None;
        let user = window.navigator().user_agent();
        log::error!("navigator user : {:?}", user);
        // if let Ok(user) = user {
        //     // 检测是否为微信小游戏环境。
        //     if user.contains("wechatdevtools") || user.contains("PI_WX_GAME") || user.contains("PI_QQ_GAME") {
                // log::error!("微信小游戏环境，使用特殊的canvas处理！！！");
                canvas = Some(
                    js_sys::Reflect::get(&window, &"canvas".to_string().into())
                        .unwrap_or(JsValue::null()),
                );
        //     }
        // }

        // 如果未找到canvas元素，则尝试通过查询获取。
        if canvas.is_none() {
            canvas = Some(
                window
                    .document()
                    .and_then(|doc| {
                        // 使用查询选择器找到带有特定属性的canvas元素。
                        doc.query_selector_all(&format!("[data-raw-handle=\"{canvas_attribute}\"]"))
                            .ok()
                    })
                    .and_then(|nodes| nodes.get(0))
                    .expect("预期找到唯一的canvas元素").into(),
            );
        }

        // 获取canvas元素。
        let canvas = canvas.take().unwrap();

        // 检查canvas是否有效。
        if canvas.is_null() || canvas.is_undefined() {
            log::error!("create_surface 000");
        }

        // 将JsValue转换为HtmlCanvasElement。
        let canvas: web_sys::HtmlCanvasElement = canvas.into();

        // 获取WebGL2上下文。
        let webgl2_context: wasm_bindgen::JsValue = match canvas.get_context("webgl2") {
            Ok(v) => match v {
                Some(v) => v.into(),
                None => wasm_bindgen::throw_str("webgl2 上下文获取失败!!!"),
            },
            Err(err) => wasm_bindgen::throw_str(&format!("获取webgl2上下文失败!!!{:?}", err)),
        };

        // 生成唯一ID。
        let id = ID.fetch_add(1, Ordering::Relaxed);

        // 返回新的WebSurface实例。
        Ok(WebSurface {
            context: Arc::new(glow::Context::from_webgl2_context(webgl2_context.into())),
            id,
        })
    }

    /// 创建一个新的WebGL上下文。
    #[inline]
    pub fn create_context(&self) -> Result<WebContext, InstanceError> {
        // 创建一个新的canvas元素。
        let value: wasm_bindgen::JsValue = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("canvas")
            .unwrap()
            .into();

        // 将其转换为HtmlCanvasElement。
        let canvas: web_sys::HtmlCanvasElement = value.into();

        // 获取WebGL2上下文。
        let webgl2_context: wasm_bindgen::JsValue =
            canvas.get_context("webgl2").unwrap().unwrap().into();

        // 生成唯一ID。
        let id = ID.fetch_add(1, Ordering::Relaxed);

        // 返回新的WebContext实例。
        return Ok(WebContext {
            context: Arc::new(glow::Context::from_webgl2_context(webgl2_context.into())),
            id,
        });
    }

    /// 设置当前的上下文和表面，确保后续的图形操作在正确的上下文中进行。
    pub fn make_current<'a>(
        &'a mut self,
        surface: Option<&'a WebSurface>,
        context: Option<&WebContext>,
    ) {
        if let Some(context) = context {
            if let Some(surface) = surface {
                // 如果已经绑定了正确的上下文和表面，则直接返回。
                if let Some(bind_context) = &self.0 {
                    if bind_context == surface {
                        return;
                    }
                }
                // 更新绑定的表面。
                self.0.replace(surface.clone());
            } else {
                // 如果未提供表面，则直接绑定上下文。
                if let Some(bind_context) = &self.0 {
                    if bind_context == context {
                        return;
                    }
                }
                self.0.replace(context.clone());
            }
        }
    }

    /// 获取当前实例的Glow上下文引用。
    #[inline]
    pub fn get_glow<'a>(&'a self) -> &glow::Context {
        self.0.as_ref().unwrap().context.as_ref()
    }

    /// 交换表面的缓冲区。
    #[inline]
    pub fn swap_buffers(&self, surface: &WebSurface) {
        // let c = self.0.as_ref().unwrap().context.as_ref();
        // let e = unsafe { c.get_error() };
        // log::error!("============= swap_buffers: {}", e);
    }
}
