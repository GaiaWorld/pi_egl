# pi_egl - 跨平台OpenGL/WebGL2抽象层

[![Crates.io](https://img.shields.io/crates/v/pi_egl)](https://crates.io/crates/pi_egl)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

提供跨平台的EGL/WebGL2统一抽象，支持Windows、Android和Web平台，实现OpenGL ES图形上下文管理。

## 特性

- **多平台支持**:
  - 🪟 Windows: 通过WGL实现（支持独立/集成显卡切换）
  - 🤖 Android: 基于EGL + [Swappy](https://github.com/google/swappy)帧率优化
  - 🌐 Web: WebGL2.0标准实现
  - 🍎 iOS/macOS: *计划中*

- **核心功能**:
  - 自动生成OpenGL ES绑定（基于gl_generator）
  - 窗口句柄抽象（raw-window-handle集成）
  - 跨平台编译条件处理（cfg属性宏）
  - Android帧率平滑控制（Swappy静态链接）

- **依赖生态**:
  - [glutin](https://github.com/rust-windowing/glutin) 的WGL/EGL实现参考
  - [raw-window-handle](https://github.com/rust-windowing/raw-window-handle) 窗口抽象
  - [swappy](https://developer.android.com/games/sdk/swappy) Android帧率优化

## 快速开始

添加依赖到Cargo.toml：

```toml
[dependencies]
pi_egl = "0.1"
```

基础使用示例：

```rust
use pi_egl::{Context, ContextBuilder};
use raw_window_handle::RawWindowHandle;

// 创建OpenGL上下文
let ctx = ContextBuilder::new()
    .with_window_handle(raw_handle)
    .build()
    .expect("Failed to create GL context");

unsafe {
    ctx.make_current();
    gl::ClearColor(0.0, 0.0, 0.0, 1.0);
    gl::Clear(gl::COLOR_BUFFER_BIT);
    ctx.swap_buffers();
}
```

## 平台支持

### Windows
- 支持WGL扩展
- 自动检测可用显卡设备
- 多窗口上下文管理

### Android
```toml
[target.'cfg(target_os = "android")'.dependencies]
pi_egl = { version = "0.1", features = ["swappy"] }
```
- 集成Swappy库实现帧率平滑
- EGL上下文生命周期管理
- 支持多线程渲染

### WebAssembly
```toml
[target.'cfg(target_arch = "wasm32")'.dependencies]
pi_egl = { version = "0.1", default-features = false }
```
- 轻量级WebGL2实现
- 自动适配Canvas尺寸
- 支持requestAnimationFrame循环

## 构建指南

### 通用构建
```bash
cargo build --features "swappy"  # Android需要swappy特性
```

### Android构建
1. 安装NDK工具链
2. 添加交叉编译目标：
```bash
rustup target add aarch64-linux-android armv7-linux-androideabi
```
3. 运行示例：
```bash
cargo apk run --example hello_android
```

### Web构建
```bash
wasm-pack build --target web --features webgl
```

## 贡献

欢迎通过GitHub Issues和Pull Requests参与贡献。需要特别注意：

1. 平台特定代码应放在`platform/`目录
2. 新增特性需提供至少一个平台实现
3. 涉及OpenGL状态修改需添加安全注释

## 许可证

双协议授权：[MIT](LICENSE-MIT) 或 [Apache-2.0](LICENSE-APACHE)

## 致谢

- [glutin](https://github.com/rust-windowing/glutin) - OpenGL上下文管理参考
- [winit](https://github.com/rust-windowing/winit) - 跨平台窗口实现
- [swappy-rs](https://github.com/google/swappy) - Android帧率控制绑定
