// ===== MODULES =====
pub mod activity;

use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};
use k1_math::{Color, Rect, Mat4};
use k1_gles::{GlContext, BatchRenderer};
use k1_sys::{NativeWindow, post_frame_callback};
use activity::{ANativeActivity, ANativeActivityCallbacks};
use heapless::String as HString;

// ===== LOGGING (Zero Allocation) =====
#[macro_export]
macro_rules! logfox {
    ($tag:expr, $msg:expr) => {
        {
            k1_sys::android_log(k1_sys::LogLevel::Info, $tag, $msg);
        }
    };
    ($tag:expr, $($arg:tt)*) => {
        {
            use core::fmt::Write;
            let mut buf = heapless::String::<256>::new();
            let _ = core::write!(buf, $($arg)*);
            k1_sys::android_log(k1_sys::LogLevel::Info, $tag, buf.as_str());
        }
    };
}

// ===== STATE =====
static RUNNING: AtomicBool = AtomicBool::new(false);
static mut WINDOW_PTR: *mut c_void = std::ptr::null_mut();
static mut GL_CTX: Option<GlContext> = None;
static mut BATCH: Option<BatchRenderer<400, 600>> = None;

// ===== ENTRY POINT =====
#[no_mangle]
pub extern "C" fn ANativeActivity_onCreate(
    activity: *mut ANativeActivity,
    _saved_state: *mut c_void,
    _saved_state_size: usize,
) {
    logfox!("ZAVOGLES", "ANativeActivity_onCreate");
    
    unsafe {
        let callbacks = (*activity).callbacks;
        (*callbacks).on_native_window_created = Some(on_window_created);
        (*callbacks).on_native_window_destroyed = Some(on_window_destroyed);
        (*callbacks).on_destroy = Some(on_destroy);
        (*callbacks).on_input_queue_created = Some(on_input_queue_created);
        (*callbacks).on_input_queue_destroyed = Some(on_input_queue_destroyed);
    }
}
extern "C" fn on_input_queue_created(_activity: *mut c_void, _queue: *mut c_void) {
    logfox!("ZAVOGLES", "Input queue created");
}

extern "C" fn on_input_queue_destroyed(_activity: *mut c_void, _queue: *mut c_void) {
    logfox!("ZAVOGLES", "Input queue destroyed");
}

// ===== CALLBACKS =====
extern "C" fn on_window_created(_activity: *mut c_void, window: *mut c_void) {
    logfox!("ZAVOGLES", "Window created");
    
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
                            logfox!("ZAVOGLES", "Render loop started");
                        }
                        Err(_) => logfox!("ZAVOGLES", "ERROR: BatchRenderer failed"),
                    }
                }
                Err(_) => logfox!("ZAVOGLES", "ERROR: GLContext failed"),
            }
        }
    }
}

extern "C" fn on_window_destroyed(_activity: *mut c_void, _window: *mut c_void) {
    logfox!("ZAVOGLES", "Window destroyed");
    RUNNING.store(false, Ordering::Relaxed);
    unsafe {
        BATCH = None;
        GL_CTX = None;
        WINDOW_PTR = std::ptr::null_mut();
    }
}

extern "C" fn on_destroy(_activity: *mut c_void) {
    logfox!("ZAVOGLES", "Destroy");
    RUNNING.store(false, Ordering::Relaxed);
}

// ===== RENDER LOOP =====
extern "C" fn render_frame(frame_time: i64, _data: *mut c_void) {
    if !RUNNING.load(Ordering::Relaxed) { return; }
    
    let time = (frame_time as f32) / 1_000_000_000.0;
    
    unsafe {
        if let Some(ref ctx) = GL_CTX {
            ctx.clear();
            
            if let Some(ref mut batch) = BATCH {
                let w = ctx.width() as f32;
                let h = ctx.height() as f32;
                
                logfox!("ZAVOGLES", "Frame: {}x{} time={}", w as i32, h as i32, time);
                
                let matrix = Mat4::ortho(0.0, w, h, 0.0, -1.0, 1.0);
                
                // خلفية
                batch.begin_frame();
                batch.draw_quad(
                    Rect::from_coords(0.0, 0.0, w, h),
                    Rect::from_coords(0.0, 0.0, 1.0, 1.0),
                    Color::new(0.05, 0.05, 0.1, 1.0),
                );
                batch.end_frame(&matrix, time, 10.0, 0.005);
                
                // XMB
                batch.begin_frame();
                draw_xmb(batch, w, h, time);
                batch.end_frame(&matrix, time, 0.0, 0.0);
            } else {
                logfox!("ZAVOGLES", "BATCH is None!");
            }
            
            if let Err(e) = ctx.swap_buffers() {
                logfox!("ZAVOGLES", "swap_buffers error: {}", e);
            }
        } else {
            logfox!("ZAVOGLES", "GL_CTX is None!");
        }
        
        if RUNNING.load(Ordering::Relaxed) {
            post_frame_callback(render_frame, std::ptr::null_mut());
        }
    }
}

// ===== XMB UI =====
fn draw_xmb(batch: &mut BatchRenderer<400, 600>, w: f32, h: f32, time: f32) {
    let categories = ["Settings", "Games", "Media"];
    let y = h * 0.2;
    
    for (i, _cat) in categories.iter().enumerate() {
        let x = w * 0.15 + (i as f32 * w * 0.25);
        let alpha = 0.6 + (time + i as f32).sin() * 0.2;
        
        batch.draw_quad(
            Rect::from_coords(x - 40.0, y - 20.0, 80.0, 40.0),
            Rect::from_coords(0.0, 0.0, 1.0, 1.0),
            Color::new(0.0, 0.3, 0.6, alpha),
        );
    }
}

// ===== TDD TESTS =====
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_running() {
        RUNNING.store(true, Ordering::Relaxed);
        assert!(RUNNING.load(Ordering::Relaxed));
    }
}