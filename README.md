# pi_egl

为 各平台 实现 统一的 EGL / WebGL2.0 跨平台的 API；

| 平台      | 说明                                       |
| --------- | ------------------------------------------ |
| `Windows` | wgl / 独显-集显 设备                       |
| `Android` | egl / 帧率平滑库                           |
| `Web`     | struct-空字段，方法-空实现，只保留 gl-接口 |
| `iOS`     | `TODO`                                     |

# 相关 Rust库

| 功能          | 库                                                                               |
| ------------- | -------------------------------------------------------------------------------- |
| wgl/egl 封装  | [glutin](https://github.com/rust-windowing/glutin)                               |
| GL-函数生成   | [gl-rs](https://github.com/brendanzab/gl-rs) `gl_generator` /  `webgl_generator` |
| 窗口句柄Trait | [raw-window-handle](https://github.com/rust-windowing/raw-window-handle)         |