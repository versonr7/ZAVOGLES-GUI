#![no_std]
#![allow(warnings)]

mod glyphs_msdf;
use glyphs_msdf::FONT_GLYPHS;

use core::ffi::{c_int, c_void};
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicBool, AtomicI32, AtomicPtr, AtomicU32, Ordering};

use za_gles::font::BitmapFont;
use za_gles::{BatchRenderer, GlContext};
use za_math::Mat4;
use za_sys::NativeWindow;
use za_xmb::XmbState;

// ===== LOGGING =====
#[macro_export]
macro_rules! logfox {
    ($tag:expr, $msg:expr) => {
        {
            za_sys::android_log(za_sys::LogLevel::Info, $tag, $msg);
        }
    };
    ($tag:expr, $($arg:tt)*) => {
        {
            use core::fmt::Write;
            let mut buf = heapless::String::<256>::new();
            let _ = core::write!(buf, $($arg)*);
            za_sys::android_log(za_sys::LogLevel::Info, $tag, buf.as_str());
        }
    };
}

// ===== STATE =====
static RUNNING: AtomicBool = AtomicBool::new(false);
static WIDTH: AtomicI32 = AtomicI32::new(0);
static HEIGHT: AtomicI32 = AtomicI32::new(0);
static FRAME_COUNT: AtomicU32 = AtomicU32::new(0);
static INITIALIZED: AtomicBool = AtomicBool::new(false);
static FRAME_LOCK: AtomicBool = AtomicBool::new(false);

// ✅ إضافة الـ Guard
struct FrameLockGuard;
impl FrameLockGuard {
    fn try_acquire() -> Option<Self> {
        if FRAME_LOCK
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Acquire)
            .is_ok()
        {
            Some(FrameLockGuard)
        } else {
            None
        }
    }
}
impl Drop for FrameLockGuard {
    fn drop(&mut self) {
        FRAME_LOCK.store(false, Ordering::Release);
    }
}

static XMB_STATE: XmbState = XmbState::new();

static mut GL_CTX_STORAGE: MaybeUninit<GlContext> = MaybeUninit::uninit();
static GL_CTX: AtomicPtr<GlContext> = AtomicPtr::new(core::ptr::null_mut());

static mut BATCH_STORAGE: MaybeUninit<BatchRenderer<400, 600>> = MaybeUninit::uninit();
static BATCH: AtomicPtr<BatchRenderer<400, 600>> = AtomicPtr::new(core::ptr::null_mut());

// --- Font atlas ---
static FONT_ATLAS_BYTES: &[u8] = include_bytes!("../../assets/font_msdf.rgba");
const FONT_ATLAS_W: i32 = 512;
const FONT_ATLAS_H: i32 = 512;

static mut FONT_STORAGE: MaybeUninit<BitmapFont> = MaybeUninit::uninit();
static FONT: AtomicPtr<BitmapFont> = AtomicPtr::new(core::ptr::null_mut());

// ===== JNI EXPORTS =====
#[no_mangle]
pub extern "C" fn Java_com_versonr7_zavogles_ZavoglesActivity_nativeOnRenderThreadExit(
    _env: *mut c_void,
    _class: *mut c_void,
) {
    unsafe {
        // تأكد أن السياق الحالي ساري قبل حذف الموارد
        let ctx_ptr = GL_CTX.load(Ordering::Acquire);
        if !ctx_ptr.is_null() {
            let _ = (*ctx_ptr).make_current();
        }

        // احذف الخط أولاً (يستخدم Textures)
        if !FONT.load(Ordering::Relaxed).is_null() {
            core::ptr::drop_in_place(FONT_STORAGE.as_mut_ptr());
            FONT.store(core::ptr::null_mut(), Ordering::Release);
        }

        // ثم احذف BatchRenderer
        if !BATCH.load(Ordering::Relaxed).is_null() {
            core::ptr::drop_in_place(BATCH_STORAGE.as_mut_ptr());
            BATCH.store(core::ptr::null_mut(), Ordering::Release);
        }

        // أخيرًا احذف سياق GL
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
        let anw = za_sys::ANativeWindow_fromSurface(_env, surface);
        if anw.is_null() {
            return;
        }

        if let Some(win) = NativeWindow::from_raw(anw) {
            let w = win.width();
            let h = win.height();

            // ✅ لا تلمس GL_CTX إذا كان موجوداً، فقط إذا كان null أنشئه
            if GL_CTX.load(Ordering::Acquire).is_null() {
                match GlContext::from_window(win) {
                    Ok(ctx) => {
                        GL_CTX_STORAGE.write(ctx);
                        GL_CTX.store(GL_CTX_STORAGE.as_mut_ptr(), Ordering::Release);
                        logfox!("ZAVOGLES", "EGL context created");
                    }
                    Err(e) => logfox!("ZAVOGLES", "ERROR: GlContext failed: {}", e),
                }
            }
            // تحديث الأبعاد دائماً
            WIDTH.store(w, Ordering::Release);
            HEIGHT.store(h, Ordering::Release);
            INITIALIZED.store(true, Ordering::Release);
            RUNNING.store(true, Ordering::Release);
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
    _y: f32,
    action: i32,
) {
    if action == 0 {
        let w = WIDTH.load(Ordering::Acquire) as f32;
        XMB_STATE.handle_touch(x, w);

        logfox!(
            "ZAVOGLES",
            "Selected category: {}",
            XMB_STATE.get_selected()
        );
    }
}

#[no_mangle]
pub extern "C" fn Java_com_versonr7_zavogles_ZavoglesActivity_nativeOnFrame(
    _env: *mut c_void,
    _class: *mut c_void,
) {
    // ✅ استخدام الـ Guard
    let _guard = match FrameLockGuard::try_acquire() {
        Some(g) => g,
        None => return,
    };

    if !RUNNING.load(Ordering::Acquire) {
        return; // الـ Guard سيفتح القفل تلقائياً
    }

    unsafe {
        let ctx_ptr = GL_CTX.load(Ordering::Acquire);
        if ctx_ptr.is_null() {
            return;
        }

        let ctx = &mut *ctx_ptr;

        let batch_ptr = BATCH.load(Ordering::Acquire);
        if batch_ptr.is_null() {
            if let Err(e) = ctx.make_current() {
                logfox!("ZAVOGLES", "ERROR: make_current failed: {}", e);
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

        // --- BACKGROUND ---
        batch.begin_frame();
        za_xmb::draw_background(batch, time, w, h);
        batch.end_frame(&matrix, time, 0.0, 0.0);

        // --- WAVE BAND ---
        batch.begin_frame();
        za_xmb::draw_wave(batch, time, w, h);
        batch.end_frame(&matrix, time, 5.0, 0.015);

        // --- XMB BUTTONS ---
        batch.begin_frame();
        za_xmb::draw_xmb_buttons(batch, &XMB_STATE, w, h);
        batch.end_frame(&matrix, time, 0.0, 0.0);

        // --- تحميل الخط إن لزم ---
        let font_ptr2 = FONT.load(Ordering::Acquire);
        if font_ptr2.is_null() {
            match BitmapFont::from_atlas_data(
                FONT_ATLAS_BYTES,
                FONT_ATLAS_W,
                FONT_ATLAS_H,
                FONT_GLYPHS,
                32.0,
            ) {
                Ok(font) => {
                    FONT_STORAGE.write(font);
                    FONT.store(FONT_STORAGE.as_mut_ptr(), Ordering::Release);
                    logfox!("ZAVOGLES", "✅ Font loaded successfully");
                }
                Err(e) => {
                    logfox!("ZAVOGLES", "❌ Font init failed: {}", e);
                }
            }
        }

        // --- XMB TEXT ---
        let font_ptr_final = FONT.load(Ordering::Acquire);
        if !font_ptr_final.is_null() {
            let font = &*font_ptr_final;
            batch.begin_frame();
            za_xmb::draw_xmb_text(batch, &XMB_STATE, font, w, h);
            batch.end_frame(&matrix, time, 0.0, 0.0);
        }

        // --- SWAP ---
        if RUNNING.load(Ordering::Acquire) {
            if let Err(e) = ctx.swap_buffers() {
                logfox!("ZAVOGLES", "ERROR: swap_buffers: {}", e);
            }
        }
    }

    FRAME_LOCK.store(false, Ordering::Release);
}

// ===== PANIC HANDLER =====
// مطلوب بسبب فجوة معروفة برستك (rust-lang/rust#56152) لبناء
// no_std + cdylib بوضع dev، حتى مع panic="abort".
#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    if info.location().is_some() {
        za_sys::android_log(za_sys::LogLevel::Error, "ZAVOGLES", "PANIC! (see logcat)");
    } else {
        za_sys::android_log(za_sys::LogLevel::Error, "ZAVOGLES", "PANIC!");
    }
    // ✅ إنهاء فوري بدل التجميد
    unsafe { libc::abort() }
}

// ===== TESTS =====
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
