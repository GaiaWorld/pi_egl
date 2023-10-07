// SwappyGL_destroy
// SwappyGL_enableStats
// SwappyGL_getFenceTimeoutNS
// SwappyGL_getRefreshPeriodNanos
// SwappyGL_getStats
// SwappyGL_getSwapIntervalNS
// SwappyGL_getUseAffinity
// SwappyGL_init_internal
// SwappyGL_injectTracer
// SwappyGL_isEnabled
// SwappyGL_onChoreographer
// SwappyGL_recordFrameStart
// SwappyGL_setAutoPipelineMode
// SwappyGL_setAutoSwapInterval
// SwappyGL_setBufferStuffingFixWait
// SwappyGL_setFenceTimeoutNS
// SwappyGL_setMaxAutoSwapIntervalNS
// SwappyGL_setSwapIntervalNS
// SwappyGL_setUseAffinity
// SwappyGL_setWindow
// SwappyGL_swap
#![allow(warnings)]

use jni::{sys::jobject, JNIEnv};
use libc::{c_int, c_uchar, c_uint, c_ulonglong, c_void};

const MAX_FRAME_BUCKETS: usize = 6;

pub type EGLDisplay = *const c_void;
pub type EGLSurface = *const c_void;

#[repr(C)]
pub struct ANativeWindow {
    pub opaque: i32,
}

/** @brief 外部线程管理器返回的线程 ID。 */
type SwappyThreadId = c_ulonglong;

/**
 * 指向可以附加到 SwappyTracer::preWait 的函数的指针
 * @param userData 指向任意数据的指针，参见 SwappyTracer::userData。
 */
// typedef void (*SwappyPreWaitCallback)(void*);
pub type SwappyPreWaitCallback = extern "C" fn(_: *mut c_void);

/**
 * 指向可以附加到 SwappyTracer::postWait 的函数的指针。
 * @param userData 指向任意数据的指针，参见 SwappyTracer::userData。
 * @param cpu_time_ns CPU 处理此帧的时间（以纳秒为单位）。
 * @param gpu_time_ns GPU 处理前一帧的时间（以纳秒为单位）。
 */
// typedef void (*SwappyPostWaitCallback)(void*, int64_t cpu_time_ns, int64_t gpu_time_ns);
pub type SwappyPostWaitCallback =
    extern "C" fn(_: *mut c_void, cpu_time_ns: c_ulonglong, gpu_time_ns: c_ulonglong);

/**
 * 指向可以附加到 SwappyTracer::preSwapBuffers 的函数的指针。
 * @param userData 指向任意数据的指针，参见 SwappyTracer::userData。
 */
// typedef void (*SwappyPreSwapBuffersCallback)(void*);
pub type SwappyPreSwapBuffersCallback = extern "C" fn(_: *mut c_void);

/**
 * 指向可以附加到 SwappyTracer::postSwapBuffers 的函数的指针。
 * @param userData 指向任意数据的指针，参见 SwappyTracer::userData。
 * @param requiredPresentationTimeMillis 目标时间，以毫秒为单位，在
 * 框架将显示在屏幕上。
 */
// typedef void (*SwappyPostSwapBuffersCallback)(void*, int64_t desiredPresentationTimeMillis);
pub type SwappyPostSwapBuffersCallback =
    extern "C" fn(_: *mut c_void, desiredPresentationTimeMillis: c_ulonglong);

/**
 * 指向可以附加到 SwappyTracer::startFrame 的函数的指针。
 * @param userData 指向任意数据的指针，参见 SwappyTracer::userData。
 * @param requiredPresentationTimeMillis 计划呈现帧的时间（以毫秒为单位）。
 */
// typedef void (*SwappyStartFrameCallback)(void*, int currentFrame,
//                                          int64_t desiredPresentationTimeMillis);
pub type SwappyStartFrameCallback =
    extern "C" fn(_: *mut c_void, currentFrame: c_int, desiredPresentationTimeMillis: c_ulonglong);

/**
 * 指向可以附加到 SwappyTracer::swapIntervalChanged 的函数的指针。
 * 调用::SwappyGL_getSwapIntervalNS 或::SwappyVk_getSwapIntervalNS 获取最新的swapInterval。
 * @param userData 指向任意数据的指针，参见 SwappyTracer::userData。
 */
// typedef void (*SwappySwapIntervalChangedCallback)(void*);
pub type SwappySwapIntervalChangedCallback = extern "C" fn(_: *mut c_void);

/**
 * @brief 一种结构，使您可以通过调用 ::Swappy_setThreadFunctions 来设置 Swappy 如何启动和加入线程。
 *
 * 此功能的使用是可选的。
 */
pub type thread_func = extern "C" fn(_: *mut c_void) -> *mut c_void;
#[repr(C)]
pub struct SwappyThreadFunctions {
    /** @brief 线程开始回调。
     *
     * 该函数由 Swappy 调用以在新线程上启动 thread_func。
     * @param user_data 要传递给线程函数的值。
     * 如果线程已经启动，这个函数应该设置thread_id并返回0。如果线程没有启动，这个函数应该返回一个非零值。
     */
    // int (*start)(SwappyThreadId* thread_id, void* (*thread_func)(void*), void* user_data);
    pub start:
        extern "C" fn(thread_id: *mut SwappyThreadId, thread_func, user_data: *mut c_void) -> c_int,

    /** @brief 线程加入回调。
     *
     * 这个函数被 Swappy 调用以加入给定 id 的线程。
     */
    // void (*join)(SwappyThreadId thread_id);
    pub join: extern "C" fn(thread_id: SwappyThreadId),

    /** @brief 线程可连接回调。
     *
     * 该函数由 Swappy 调用以发现具有给定 id 的线程是否可加入。
     */
    // bool (*joinable)(SwappyThreadId thread_id);
    pub joinable: extern "C" fn(thread_id: SwappyThreadId),
}

/**
 * @brief Swappy 统计信息，如果使用 ::SwappyGL_enableStats 切换，则收集。
 * @see SwappyGL_getStats
 */
#[repr(C)]
pub struct SwappyStats {
    /** @brief swappy 交换的总帧数 */
    pub totalFrames: c_ulonglong,

    /** @brief 屏幕刷新次数的直方图
     * 渲染完成后的合成器队列。
     *
     * 例如：
     * 如果一个帧在渲染完成后在合成器队列中等待了 2 个刷新周期，则该帧将被计入 idleFrames[2]
     */
    pub idleFrames: [c_ulonglong; MAX_FRAME_BUCKETS],

    /** @brief 屏幕刷新次数的直方图
     * 要求的演示时间和实际的当前时间。
     *
     * 例如：
     * 如果在请求的时间戳交换集之后出现 2 个刷新周期的帧，则该帧将计入 lateFrames[2]
     */
    pub lateFrames: [c_ulonglong; MAX_FRAME_BUCKETS],

    /** @brief 两次屏幕刷新次数直方图
     * 连续帧
     *
     * 例如：
     * 如果第 N 帧出现在第 N-1 帧之后的 2 个刷新周期，第 N 帧将计入 offsetFromPreviousFrame[2]
     */
    pub offsetFromPreviousFrame: [c_ulonglong; MAX_FRAME_BUCKETS],

    /** @brief 屏幕刷新次数的直方图
     * 调用 Swappy_recordFrameStart 和实际当前时间。
     *
     * 例如：
     * 如果在调用 `Swappy_recordFrameStart` 后出现 2 个刷新周期的帧，该帧将被计入latencyFrames[2]
     */
    pub latencyFrames: [c_ulonglong; MAX_FRAME_BUCKETS],
}

/**
 * @brief 每帧要调用的回调集合以跟踪执行。
 *
 * 这些的注入是可选的。
 */
#[repr(C)]
pub struct SwappyTracer {
    /**
     * 在等待将帧排队到 Composer 之前调用的回调。
     */
    pub preWait: SwappyPreWaitCallback,

    /**
     * 在等待将帧排队到 Composer 后调用的回调完成。
     */
    pub postWait: SwappyPostWaitCallback,

    /**
     * 在调用函数以将帧排队到composer之前调用的回调。
     */
    pub preSwapBuffers: SwappyPreSwapBuffersCallback,

    /**
     * 调用函数后调用的回调将帧排队到composer。
     */
    pub postSwapBuffers: SwappyPostSwapBuffersCallback,

    /**
     * 在帧开始时调用的回调。
     */
    pub startFrame: SwappyStartFrameCallback,

    /**
     * 指向一些任意数据的指针，这些数据将作为回调的第一个参数传递。
     */
    pub userData: *mut c_void,

    /**
     * 交换间隔更改时调用的回调。
     */
    pub swapIntervalChanged: SwappySwapIntervalChangedCallback,
}

extern "C" {
    /**
     * @brief 在运行时返回 Swappy 库的版本。
     */
    pub fn Swappy_version() -> c_uint;

    /**
     * @brief 在任何其他函数之前调用它以使用自定义线程管理器;
     *
     * 此功能的使用完全是可选的。 Swappy 默认使用 std::thread;
     *
     */
    pub fn Swappy_setThreadFunctions(thread_functions: *const SwappyThreadFunctions);
}

/**
 * @defgroup swappyGL Swappy for OpenGL
 * Swappy 的 OpenGL 部分。
 * @{
 */
extern "C" {

    // 内部初始化函数。 不要直接调用。
    pub fn SwappyGL_init_internal(env: JNIEnv, jactivity: jobject) -> c_uchar;

    /**
     * @brief 检查 Swappy 是否已成功初始化。
     * @return false 如果 `swappy.disable` 系统属性不是 `false` 或者所需的 OpenGL 扩展对于 Swappy 工作不可用。
     */
    pub fn SwappyGL_isEnabled() -> c_uchar;

    /**
     * @brief 销毁资源并停止 Swappy 创建的所有线程。
     * @see SwappyGL_init
     */
    pub fn SwappyGL_destroy();

    /**
     * @brief 告诉 Swappy 在调用 ANativeWindow_API 时使用哪个 ANativeWindow。
     * @param window 用于创建 EGLSurface 的 ANativeWindow。
     * @return 成功时为真，如果 Swappy 未初始化则为假。
     */
    pub fn SwappyGL_setWindow(window: *const c_void) -> c_uchar;

    /**
     * @brief 用这个替换对 eglSwapBuffers 的调用
     * @return 成功时为真，否则为假
     * 1) Swappy 未初始化或 2) eglSwapBuffers 未返回 EGL_TRUE。
     * 在后一种情况下，可以使用 eglGetError 获取错误代码。
     */
    pub fn SwappyGL_swap(display: EGLDisplay, surface: EGLSurface) -> c_uchar;

    // 参数设置器：
    pub fn SwappyGL_setUseAffinity(tf: c_uchar);

    /**
     * @brief 覆盖交换间隔
     *
     * 默认情况下，Swappy 会根据实际帧渲染时间调整交换间隔。
     *
     * 如果一个应用想要覆盖 Swappy 计算的交换间隔，它可以调用这个函数：
     *
     * * 这将暂时覆盖 Swappy 的帧计时，但是，除非调用 `SwappyGL_setAutoSwapInterval(false)`，计时将继续动态更新，因此交换间隔可能会改变。
     *
     * * 这将设置运行的**最小**间隔。 例如，`SwappyGL_setSwapIntervalNS(SWAPPY_SWAP_30FPS)` 将不允许 Swappy 更快地交换，即使自动模式决定它可以。 但是如果自动模式打开，它会变慢。
     *
     * @param swap_ns 新的交换间隔值，以纳秒为单位。
     */
    pub fn SwappyGL_setSwapIntervalNS(swap_ns: c_ulonglong);

    /**
     * @brief 为驱动程序有问题的设备设置栅栏超时参数。 其默认值为 50,000,000ns (50ms)。
     */
    pub fn SwappyGL_setFenceTimeoutNS(fence_timeout_ns: c_ulonglong);

    // Parameter getters:

    /**
     * @brief 获取刷新周期值，以纳秒为单位。
     */
    pub fn SwappyGL_getRefreshPeriodNanos() -> c_ulonglong;

    /**
     * @brief 获取交换间隔值，以纳秒为单位。
     */
    pub fn SwappyGL_getSwapIntervalNS() -> c_ulonglong;

    pub fn SwappyGL_getUseAffinity() -> c_uchar;

    /**
     * @brief 获取栅栏超时值，以纳秒为单位。
     */
    pub fn SwappyGL_getFenceTimeoutNS() -> c_ulonglong;

    /**
     * @brief 设置在应用缓冲区填充修复之前要等待的坏帧数。 设置为零以关闭此功能。 默认值 = 0。
     */
    pub fn SwappyGL_setBufferStuffingFixWait(n_frames: c_int);
}

/**
 * 最长持续时间，以刷新周期为单位，由统计数据表示。
 * @see SwappyStats
 */
extern "C" {
    /**
     * @brief 如果应用程序希望使用 Android choreographer 为 Swappy 提供滴答声，它可以调用此函数。
     *
     * @warning 这个函数*必须*在第一次`Swappy_swap()`调用之前被调用。 之后，在每个编舞者滴答声中调用此函数。
     */
    pub fn SwappyGL_onChoreographer(frameTimeNanos: c_ulonglong);

    /** @brief 传递每帧要调用的回调来跟踪执行。 */
    pub fn SwappyGL_injectTracer(t: *const SwappyTracer);

    /**
     * @brief 开启/关闭自动交换间隔检测
     *
     * 默认情况下，Swappy 会根据实际帧渲染时间调整交换间隔。 如果应用程序想要覆盖 Swappy 计算的交换间隔，它可以调用 `SwappyGL_setSwapIntervalNS`。
     * 这将暂时覆盖 Swappy 的帧计时，但是，除非调用 `SwappyGL_setAutoSwapInterval(false)`，计时将继续动态更新，因此交换间隔可能会改变。
     */
    pub fn SwappyGL_setAutoSwapInterval(enabled: c_uchar);

    /**
     * @brief 以毫秒为单位设置自动交换间隔的最大持续时间。
     *
     * 如果 Swappy 运行在自动交换间隔并且帧持续时间长于`max_swap_ns`，则 Swappy 不会做任何步调，只是尽快提交帧。
     */
    pub fn SwappyGL_setMaxAutoSwapIntervalNS(max_swap_ns: c_ulonglong);

    /**
     * @brief 开启/关闭自动流水线模式
     *
     * 默认情况下，如果自动交换间隔打开，自动流水线是打开的，
     * 并且 Swappy 将尝试通过在同一流水线阶段调度 cpu 和 gpu 工作来减少延迟（如果合适）。
     */
    pub fn SwappyGL_setAutoPipelineMode(enabled: c_ulonglong);

    /**
     * @brief 打开/关闭统计信息收集
     *
     * 默认情况下，统计信息收集是关闭的，并且没有与统计信息相关的开销。
     * 应用程序可以通过调用 `SwappyGL_enableStats(true)` 来打开统计信息收集。
     * 然后，在开始执行任何与 CPU 相关的工作之前，应用程序应该为每一帧调用 ::SwappyGL_recordFrameStart。
     * 统计信息将被记录到带有“FrameStatistics”标签的 logcat。
     * 应用程序可以通过调用::SwappyGL_getStats 来获取统计信息。
     */
    pub fn SwappyGL_enableStats(enabled: c_uchar);

    /**
     * @brief 如果已使用 SwappyGL_enableStats 启用统计信息，则应调用。
     *
     * 当使用 SwappyGL_enableStats 启用统计信息收集时，应用程序应在开始执行任何 CPU 相关工作之前为每一帧调用此函数。
     *
     * @see SwappyGL_enableStats。
     */
    pub fn SwappyGL_recordFrameStart(display: EGLDisplay, surface: EGLSurface);

    /**
     * @brief 返回收集的统计信息，如果统计信息收集被打开。
     *
     * @param swappyStats 指向 SwappyStats 的指针，该 SwappyStats 将填充收集的统计数据。
     * @see SwappyStats
     * @see SwappyGL_enableStats
     */
    pub fn SwappyGL_getStats(swappyStats: *mut SwappyStats);

}

/**
 * @brief 初始化 Swappy，通过 JNI 从显示子系统获取所需的 Android 参数。
 * @param env 使用 Swappy 的 JNI 环境
 * @param jactivity 使用 Swappy 的活动
 * @return false 如果 Swappy 初始化失败。
 * @see SwappyGL_destroy
 */
pub unsafe fn SwappyGL_init(env: JNIEnv, jactivity: jobject) -> c_uchar {
    // 此调用确保标头和链接库来自同一版本（如果不是，则会由于未定义的符号P 触发链接器错误）。
    return SwappyGL_init_internal(env, jactivity);
}

#[test]
fn test() {
    let r = unsafe { Swappy_version() };
    println!("============= Swappy_version = {}", r);
}
