# pi_egl

为 各平台 实现 统一的 EGL / WebGL2.0 跨平台的 API；

实现 顺序 如下：

| 平台      | 说明                                       |
| --------- | ------------------------------------------ |
| `Windows` | `wgl` / 独显-集显 设备                     |
| `Web`     | struct-空字段，方法-空实现，只保留 gl-接口 |
| `Android` | `egl` / 帧率平滑库                         |
| `iOS`     | `TODO` 无实现计划                          |

# 相关 Rust库

| 功能               | 库                                                                               |
| ------------------ | -------------------------------------------------------------------------------- |
| `wgl` / `egl` 封装 | [glutin](https://github.com/rust-windowing/glutin)                               |
| GL-函数生成        | [gl-rs](https://github.com/brendanzab/gl-rs) `gl_generator` /  `webgl_generator` |
| 窗口句柄 Trait     | [raw-window-handle](https://github.com/rust-windowing/raw-window-handle)         |

* android 实例 执行 cargo apk run --example hello_android

* 内部crates推送命令 cargo publish --no-verify --manifest-path Cargo.toml --index=http://ser.yinengyun.com:10082/tech/crates-io.git --token 8c3fd4b32020b5041d70142da --allow-dirty