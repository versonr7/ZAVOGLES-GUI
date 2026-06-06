use std::ffi::{c_void, c_int};
use std::sync::atomic::{AtomicBool, Ordering};
use k1_math::{Vec2, Color, Rect, Mat4};
use k1_gles::{GlContext, BatchRenderer};
use k1_sys::{NativeWindow, android_log, LogLevel, post_frame_callback};

#[repr(C)]
pub struct ANativeActivityCallbacks {
    pub on_start: Option<<extern "C" fn(*mut c_void)>,
    pub on_resume: Option<<extern "C" fn(*mut c_void)>,
    pub on_save_instance_state: Option<<extern "C" fn(*mut c_void, *mut usize) -> *mut c_void>,
    pub on_pause: Option<<extern "C" fn(*mut c_void)>,
    pub on_stop: Option<<extern "C" fn(*mut c_void)>,
    pub on_destroy: Option<<extern "C" fn(*mut c_void)>,
    pub on_window_focus_changed: Option<<extern "C" fn(*mut c_void, c_int)>,
    pub on_native_window_created: Option<<extern "C" fn(*mut c_void, *mut c_void)>,
    pub on_native_window_resized: Option<<extern "C" fn(*mut c_void, *mut c_void)>,
    pub on_native_window_redraw_needed: Option<<extern "C" fn(*mut c_void, *mut c_void)>,
    pub on_native_window_destroyed: Option<<extern "C" fn(*mut c_void, *mut c_void)>,
    pub on_input_queue_created: Option<<extern "C" fn(*mut c_void, *mut c_void)>,
    pub on_input_queue_destroyed: Option<<extern "C" fn(*mut c_void, *mut c_void)>,
    pub on_content_rect_changed: Option<<extern "C" fn(*mut c_void, *const c_void)>,
    pub on_configuration_changed: Option<<extern "C" fn(*mut c_void)>,
    pub on_low_memory: Option<<extern "C" fn(*mut c_void)>,
}

#[repr(C)]
pub struct ANativeActivity {
    pub callbacks: *mut ANativeActivityCallbacks,
    pub vm: *mut c_void,
    pub env: *mut c_void,
    pub object: *mut c_void,
    pub internal_data_path: *const u8,
    pub external_data_path: *const u8,
    pub sdk_version: i32,
    pub instance: *mut c_void,
    pub asset_manager: *mut c_void,
    pub obb_path: *const u8,
}

static RUNNING: AtomicBool = AtomicBool::new(false);
static mut WINDOW_PTR: *mut c_void = std::ptr::null_mut();
static mut GL_CTX: Option<<GlContext> = None;
static mut BATCH: Option<<BatchRenderer<<400, 600>> = None;

#[no_mangle]
pub extern "C" fn ANativeActivity_onCreate(activity: *mut ANativeActivity, _saved_state: *mut c_void, _saved_state_size: usize) {
    android_log(LogLevel::Info, "K1-APP", "ANativeActivity_onCreate");
    unsafe {
        let callbacks = (*activity).callbacks;
        (*callbacks).on_native_window_created = Some(on_window_created);
        (*callbacks).on_native_window_destroyed = Some(on_window_destroyed);
        (*callbacks).on_destroy = Some(on_destroy);
    }
}

extern "C" fn on_window_created(_activity: *mut c_void, window: *mut c_void) {
    android_log(LogLevel::Info, "K1-APP", "Window created");
    unsafe {
        WINDOW_PTR = window;
        if let Some(w) = NativeWindow::from_raw(window as *mut k1_sys::ANativeWindow) {
            match GlContext::from_window(&w) {
                Ok(ctx) => {
                    match BatchRenderer::<400, 600>::new() {
                        Ok(batch) => {
                            GL_CTX = Some(ctx);
                            BATCH = Some(batch);
                            RUNNING.store(true, Ordering::Relaxed);
                            post_frame_callback(render_frame, std::ptr::null_mut());
                            android_log(LogLevel::Info, "K1-APP", "Render loop started");
                        }
                        Err(_) => android_log(LogLevel::Error, "K1-APP", "BatchRenderer failed"),
                    }
                }
                Err(_) => android_log(LogLevel::Error, "K1-APP", "GLContext failed"),
            }
        }
    }
}

extern "C" fn on_window_destroyed(_activity: *mut c_void, _window: *mut c_void) {
    android_log(LogLevel::Info, "K1-APP", "Window destroyed");
    RUNNING.store(false, Ordering::Relaxed);
    unsafe {
        BATCH = None;
        GL_CTX = None;
        WINDOW_PTR = std::ptr::null_mut();
    }
}

extern "C" fn on_destroy(_activity: *mut c_void) {
    android_log(LogLevel::Info, "K1-APP", "Destroy");
    RUNNING.store(false, Ordering::Relaxed);
}

extern "C" fn render_frame(_frame_time: i64, _data: *mut c_void) {
    if !RUNNING.load(Ordering::Relaxed) { return; }
    unsafe {
        if let Some(ref ctx) = GL_CTX {
            ctx.clear();
            if let Some(ref mut batch) = BATCH {
                batch.begin_frame();
                let w = ctx.width() as f32;
                let h = ctx.height() as f32;
                let size = 200.0;
                let x = (w - size) / 2.0;
                let y = (h - size) / 2.0;
                batch.draw_quad(
                    Rect::from_coords(x, y, size, size),
                    Rect::from_coords(0.0, 0.0, 1.0, 1.0),
                    Color::RED
                );
                let matrix = Mat4::ortho(0.0, w, h, 0.0, -1.0, 1.0);
                batch.end_frame(&matrix);
            }
            let _ = ctx.swap_buffers();
        }
        if RUNNING.load(Ordering::Relaxed) {
            post_frame_callback(render_frame, std::ptr::null_mut());
        }
    }
}
