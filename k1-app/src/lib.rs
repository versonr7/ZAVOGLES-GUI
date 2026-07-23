#![no_std]
#![allow(warnings)]
#![cfg_attr(not(test), feature(lang_items))]

use core::ffi::{c_int, c_void};
use core::sync::atomic::{AtomicBool, AtomicI32, AtomicPtr, AtomicU32, Ordering};
use k1_gles::{BatchRenderer, GlContext};
use k1_math::{Color, Mat4, Rect};
use k1_sys::NativeWindow;

// ===== LOGGING =====
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

use core::mem::MaybeUninit;

// ===== STATE =====
static RUNNING: AtomicBool = AtomicBool::new(false);
static WIDTH: AtomicI32 = AtomicI32::new(0);
static HEIGHT: AtomicI32 = AtomicI32::new(0);
static FRAME_COUNT: AtomicU32 = AtomicU32::new(0);
static INITIALIZED: AtomicBool = AtomicBool::new(false);
static FRAME_LOCK: AtomicBool = AtomicBool::new(false);

static mut GL_CTX_STORAGE: MaybeUninit<GlContext> = MaybeUninit::uninit();
static GL_CTX: AtomicPtr<GlContext> = AtomicPtr::new(core::ptr::null_mut());

static mut BATCH_STORAGE: MaybeUninit<BatchRenderer<400, 600>> = MaybeUninit::uninit();
static BATCH: AtomicPtr<BatchRenderer<400, 600>> = AtomicPtr::new(core::ptr::null_mut());

// ===== JNI EXPORTS =====
#[no_mangle]
pub extern "C" fn Java_com_versonr7_zavogles_ZavoglesActivity_nativeOnRenderThreadExit(
    _env: *mut c_void,
    _class: *mut c_void,
) {
    unsafe {
        // نسقط BatchRenderer أولًا (يحتاج سياق GL ساريًا)
        if !BATCH.load(Ordering::Relaxed).is_null() {
            core::ptr::drop_in_place(BATCH_STORAGE.as_mut_ptr());
            BATCH.store(core::ptr::null_mut(), Ordering::Release);
        }
        // ثم نسقط GlContext (الذي بداخله NativeWindow وسياق EGL)
        if !GL_CTX.load(Ordering::Relaxed).is_null() {
            core::ptr::drop_in_place(GL_CTX_STORAGE.as_mut_ptr());
            GL_CTX.store(core::ptr::null_mut(), Ordering::Release);
        }
        INITIALIZED.store(false, Ordering::Release);
    }
}

#[no_mangle]
pub extern "C" fn Java_com_versonr7_zavogles_ZavoglesActivity_nativeOnCreate(
    _env: *mut c_void,
    _class: *mut c_void,
) {
    logfox!("ZAVOGLES", "Native onCreate");
}

#[no_mangle]
pub extern "C" fn Java_com_versonr7_zavogles_ZavoglesActivity_nativeOnSurfaceCreated(
    _env: *mut c_void,
    _class: *mut c_void,
    surface: *mut c_void,
) {
    logfox!("ZAVOGLES", "Native surfaceCreated");

    unsafe {
        let anw = k1_sys::ANativeWindow_fromSurface(_env, surface);
        if anw.is_null() {
            logfox!("ZAVOGLES", "ERROR: ANativeWindow_fromSurface returned null");
            return;
        }

        if let Some(win) = NativeWindow::from_raw(anw) {
            // 1. اقرأ الأبعاد قبل نقل ملكية win
            let w = win.width();
            let h = win.height();

            // 2. مرر win كقيمة (وليس كمرجع)
            match GlContext::from_window(win) {
                Ok(ctx) => {
                    GL_CTX_STORAGE.write(ctx);
                    GL_CTX.store(GL_CTX_STORAGE.as_mut_ptr(), Ordering::Release);

                    // لم نعد ننشئ BatchRenderer هنا، بل على خيط الرسم

                    WIDTH.store(w, Ordering::Release);
                    HEIGHT.store(h, Ordering::Release);
                    INITIALIZED.store(true, Ordering::Release);
                    RUNNING.store(true, Ordering::Release);

                    logfox!("ZAVOGLES", "EGL context ready: {}x{}", w, h);
                }
                Err(e) => logfox!("ZAVOGLES", "ERROR: GlContext failed: {}", e),
            }
        } else {
            logfox!("ZAVOGLES", "ERROR: NativeWindow::from_raw failed");
        }
    }
}

#[no_mangle]
pub extern "C" fn Java_com_versonr7_zavogles_ZavoglesActivity_nativeOnSurfaceChanged(
    _env: *mut c_void,
    _class: *mut c_void,
    width: i32,
    height: i32,
) {
    logfox!("ZAVOGLES", "Native surfaceChanged: {}x{}", width, height);
    WIDTH.store(width, Ordering::Release);
    HEIGHT.store(height, Ordering::Release);
}

#[no_mangle]
pub extern "C" fn Java_com_versonr7_zavogles_ZavoglesActivity_nativeOnSurfaceDestroyed(
    _env: *mut c_void,
    _class: *mut c_void,
) {
    logfox!("ZAVOGLES", "Native surfaceDestroyed");
    RUNNING.store(false, Ordering::Release);
}

#[no_mangle]
pub extern "C" fn Java_com_versonr7_zavogles_ZavoglesActivity_nativeOnPause(
    _env: *mut c_void,
    _class: *mut c_void,
) {
    logfox!("ZAVOGLES", "Native onPause");
    RUNNING.store(false, Ordering::Release);
}

#[no_mangle]
pub extern "C" fn Java_com_versonr7_zavogles_ZavoglesActivity_nativeOnResume(
    _env: *mut c_void,
    _class: *mut c_void,
) {
    logfox!("ZAVOGLES", "Native onResume");
}

#[no_mangle]
pub extern "C" fn Java_com_versonr7_zavogles_ZavoglesActivity_nativeOnDestroy(
    _env: *mut c_void,
    _class: *mut c_void,
) {
    logfox!("ZAVOGLES", "Native onDestroy");
    RUNNING.store(false, Ordering::Release);
}

#[no_mangle]
pub extern "C" fn Java_com_versonr7_zavogles_ZavoglesActivity_nativeOnTouch(
    _env: *mut c_void,
    _class: *mut c_void,
    x: f32,
    y: f32,
    action: i32,
) {
    if action == 0 || action == 1 {
        logfox!(
            "ZAVOGLES",
            "Touch: x={} y={} action={}",
            x as i32,
            y as i32,
            action
        );
    }
}

#[no_mangle]
pub extern "C" fn Java_com_versonr7_zavogles_ZavoglesActivity_nativeOnFrame(
    _env: *mut c_void,
    _class: *mut c_void,
) {
    if FRAME_LOCK
        .compare_exchange(false, true, Ordering::Acquire, Ordering::Acquire)
        .is_err()
    {
        return;
    }

    if !RUNNING.load(Ordering::Acquire) {
        FRAME_LOCK.store(false, Ordering::Release);
        return;
    }

    unsafe {
        let ctx_ptr = GL_CTX.load(Ordering::Acquire);
        if ctx_ptr.is_null() {
            FRAME_LOCK.store(false, Ordering::Release);
            return;
        }

        let ctx = &mut *ctx_ptr;

        let batch_ptr = BATCH.load(Ordering::Acquire);
        if batch_ptr.is_null() {
            if let Err(e) = ctx.make_current() {
                logfox!("ZAVOGLES", "ERROR: make_current failed: {}", e);
                FRAME_LOCK.store(false, Ordering::Release);
                return;
            }
            ctx.setup_gl_state();

            match BatchRenderer::<400, 600>::new() {
                Ok(batch) => {
                    BATCH_STORAGE.write(batch);
                    BATCH.store(BATCH_STORAGE.as_mut_ptr(), Ordering::Release);
                    logfox!("ZAVOGLES", "BatchRenderer created on render thread");
                }
                Err(e) => {
                    logfox!("ZAVOGLES", "ERROR: BatchRenderer failed: {}", e);
                    FRAME_LOCK.store(false, Ordering::Release);
                    return;
                }
            }
        }

        let batch = &mut *BATCH.load(Ordering::Acquire);

        let w = WIDTH.load(Ordering::Acquire) as f32;
        let h = HEIGHT.load(Ordering::Acquire) as f32;

        ctx.update_viewport(w as i32, h as i32);
        ctx.clear();

        let frame = FRAME_COUNT.fetch_add(1, Ordering::Relaxed);
        let time = (frame as f32) / 60.0;

        let matrix = Mat4::ortho(0.0, w, h, 0.0, -1.0, 1.0);

        // --- خلفية بموجة مركزية (منطقة الموجة) ---
        let wave_margin = h * 0.15; // هامش 15% من الأعلى والأسفل
        batch.begin_frame();
        batch.draw_quad(
            Rect::from_coords(0.0, wave_margin, w, h - wave_margin * 2.0),
            Rect::from_coords(0.0, 0.0, 1.0, 1.0),
            Color::new(0.05, 0.05, 0.15, 1.0), // أزرق غامق لمنطقة الموجة
        );
        batch.end_frame(&matrix, time, 15.0, 0.008);

        // --- شريط علوي ثابت (بدون موجة) ---
        batch.begin_frame();
        batch.draw_quad(
            Rect::from_coords(0.0, 0.0, w, wave_margin),
            Rect::from_coords(0.0, 0.0, 1.0, 1.0),
            Color::new(0.02, 0.02, 0.08, 1.0), // أزرق داكن جدًا للشريط العلوي
        );
        batch.end_frame(&matrix, time, 0.0, 0.0);

        // --- شريط سفلي ثابت (بدون موجة) ---
        batch.begin_frame();
        batch.draw_quad(
            Rect::from_coords(0.0, h - wave_margin, w, wave_margin),
            Rect::from_coords(0.0, 0.0, 1.0, 1.0),
            Color::new(0.02, 0.02, 0.08, 1.0), // أزرق داكن جدًا للشريط السفلي
        );
        batch.end_frame(&matrix, time, 0.0, 0.0);

        // --- واجهة XMB (أزرار التصنيفات) ---
        batch.begin_frame();
        draw_xmb(batch, w, h, time);
        batch.end_frame(&matrix, time, 0.0, 0.0);

        // تبديل المخازن
        if RUNNING.load(Ordering::Acquire) {
            if let Err(e) = ctx.swap_buffers() {
                logfox!("ZAVOGLES", "ERROR: swap_buffers: {}", e);
            }
        }
    }

    FRAME_LOCK.store(false, Ordering::Release);
}

// ===== XMB UI =====
fn draw_xmb(batch: &mut BatchRenderer<400, 600>, w: f32, h: f32, time: f32) {
    let categories = ["Settings", "Games", "Media"];

    // أبعاد متجاوبة: التصنيفات في منتصف الارتفاع 20% من الأعلى
    let y = h * 0.20;

    // توزيع أفقي متجاوب
    let start_x = w * 0.15;
    let spacing = w * 0.25;
    let button_w = w * 0.12; // عرض الزر يستجيب لعرض الشاشة
    let button_h = h * 0.05; // ارتفاع الزر يستجيب لارتفاع الشاشة

    for (i, _cat) in categories.iter().enumerate() {
        let x = start_x + (i as f32 * spacing);
        let alpha = 0.6 + libm::sinf(time + i as f32) * 0.2;

        batch.draw_quad(
            Rect::from_coords(x - button_w / 2.0, y - button_h / 2.0, button_w, button_h),
            Rect::from_coords(0.0, 0.0, 1.0, 1.0),
            Color::new(0.2, 0.6, 1.0, alpha),
        );
    }
}

#[cfg(not(test))]
#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    if let Some(loc) = info.location() {
        k1_sys::android_log(
            k1_sys::LogLevel::Error,
            "ZAVOGLES",
            "PANIC! (see logcat for details)",
        );
    } else {
        k1_sys::android_log(k1_sys::LogLevel::Error, "ZAVOGLES", "PANIC!");
    }
    loop {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_running() {
        RUNNING.store(true, Ordering::Relaxed);
        assert!(RUNNING.load(Ordering::Relaxed));
    }

    #[test]
    fn test_frame_lock() {
        assert!(FRAME_LOCK
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Acquire)
            .is_ok());
        assert!(FRAME_LOCK
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Acquire)
            .is_err());
        FRAME_LOCK.store(false, Ordering::Release);
    }
}
