#![no_std]
#![warn(missing_docs)]
#![allow(non_camel_case_types)]

use core::ffi::{c_void, c_int, c_char};
use k1_math::Vec2;

#[repr(C)]
pub struct ANativeWindow { _private: [u8; 0] }

#[repr(C)]
pub struct AInputQueue { _private: [u8; 0] }

#[repr(C)]
pub struct ALooper { _private: [u8; 0] }

#[repr(C)]
pub struct AChoreographer { _private: [u8; 0] }

pub const ALOOPER_EVENT_INPUT: c_int = 1 << 0;
pub const ALOOPER_EVENT_OUTPUT: c_int = 1 << 1;
pub const ALOOPER_EVENT_ERROR: c_int = 1 << 2;
pub const ALOOPER_EVENT_HANGUP: c_int = 1 << 3;
pub const ALOOPER_EVENT_INVALID: c_int = 1 << 4;

pub const AINPUT_EVENT_TYPE_KEY: c_int = 1;
pub const AINPUT_EVENT_TYPE_MOTION: c_int = 2;

pub const AMOTION_EVENT_ACTION_DOWN: c_int = 0;
pub const AMOTION_EVENT_ACTION_UP: c_int = 1;
pub const AMOTION_EVENT_ACTION_MOVE: c_int = 2;
pub const AMOTION_EVENT_ACTION_CANCEL: c_int = 3;
pub const AMOTION_EVENT_ACTION_POINTER_DOWN: c_int = 5;
pub const AMOTION_EVENT_ACTION_POINTER_UP: c_int = 6;

pub const ANDROID_LOG_VERBOSE: c_int = 2;
pub const ANDROID_LOG_DEBUG: c_int = 3;
pub const ANDROID_LOG_INFO: c_int = 4;
pub const ANDROID_LOG_WARN: c_int = 5;
pub const ANDROID_LOG_ERROR: c_int = 6;

#[cfg(all(target_os = "android", not(feature = "mock"), not(test)))]
#[link(name = "android")]
extern "C" {
    pub fn ANativeWindow_acquire(window: *mut ANativeWindow);
    pub fn ANativeWindow_release(window: *mut ANativeWindow);
    pub fn ANativeWindow_getWidth(window: *mut ANativeWindow) -> c_int;
    pub fn ANativeWindow_getHeight(window: *mut ANativeWindow) -> c_int;
    pub fn ANativeWindow_setBuffersGeometry(window: *mut ANativeWindow, w: c_int, h: c_int, f: c_int) -> c_int;
    
    pub fn AInputQueue_finishEvent(queue: *mut AInputQueue, event: *mut c_void, handled: c_int);
    
    pub fn AInputEvent_getType(event: *const c_void) -> c_int;
    pub fn AInputEvent_getDeviceId(event: *const c_void) -> c_int;
    pub fn AInputEvent_getSource(event: *const c_void) -> c_int;
    
    pub fn AMotionEvent_getAction(event: *const c_void) -> c_int;
    pub fn AMotionEvent_getX(event: *const c_void, pointer_index: usize) -> f32;
    pub fn AMotionEvent_getY(event: *const c_void, pointer_index: usize) -> f32;
    pub fn AMotionEvent_getPointerCount(event: *const c_void) -> usize;
    pub fn AMotionEvent_getPointerId(event: *const c_void, pointer_index: usize) -> c_int;
    pub fn AMotionEvent_getEventTime(event: *const c_void) -> i64;
    
    pub fn AKeyEvent_getAction(event: *const c_void) -> c_int;
    pub fn AKeyEvent_getKeyCode(event: *const c_void) -> c_int;
    pub fn AKeyEvent_getMetaState(event: *const c_void) -> c_int;
    
    pub fn __android_log_write(prio: c_int, tag: *const c_char, text: *const c_char) -> c_int;
    
    pub fn AChoreographer_getInstance() -> *mut AChoreographer;
    pub fn AChoreographer_postFrameCallback(
        choreographer: *mut AChoreographer,
        callback: Option<unsafe extern "C" fn(i64, *mut c_void)>,
        data: *mut c_void
    );
    
    pub fn ALooper_forThread() -> *mut ALooper;
    pub fn ALooper_acquire(looper: *mut ALooper);
    pub fn ALooper_release(looper: *mut ALooper);
    pub fn ALooper_addFd(
        looper: *mut ALooper,
        fd: c_int,
        ident: c_int,
        events: c_int,
        callback: Option<unsafe extern "C" fn(c_int, c_int, *mut c_void) -> c_int>,
        data: *mut c_void
    ) -> c_int;
    pub fn ALooper_removeFd(looper: *mut ALooper, fd: c_int) -> c_int;
    pub fn ALooper_pollAll(timeoutMillis: c_int, outFd: *mut c_int, outEvents: *mut c_int, outData: *mut *mut c_void) -> c_int;
    pub fn ALooper_wake(looper: *mut ALooper);
}

#[cfg(any(not(target_os = "android"), feature = "mock", test))]
pub mod mock {
    use super::*;
    use core::sync::atomic::{AtomicI32, Ordering};

    static MOCK_W: AtomicI32 = AtomicI32::new(1080);
    static MOCK_H: AtomicI32 = AtomicI32::new(1920);

    pub fn reset_mock_window(w: i32, h: i32) {
        MOCK_W.store(w, Ordering::Relaxed);
        MOCK_H.store(h, Ordering::Relaxed);
    }

    pub unsafe fn ANativeWindow_acquire(_: *mut ANativeWindow) {}
    pub unsafe fn ANativeWindow_release(_: *mut ANativeWindow) {}
    pub unsafe fn ANativeWindow_getWidth(_: *mut ANativeWindow) -> c_int { MOCK_W.load(Ordering::Relaxed) }
    pub unsafe fn ANativeWindow_getHeight(_: *mut ANativeWindow) -> c_int { MOCK_H.load(Ordering::Relaxed) }
    pub unsafe fn ANativeWindow_setBuffersGeometry(_: *mut ANativeWindow, w: c_int, h: c_int, _: c_int) -> c_int {
        MOCK_W.store(w, Ordering::Relaxed); MOCK_H.store(h, Ordering::Relaxed); 0
    }

    pub unsafe fn AInputQueue_getEvent(_: *mut AInputQueue, _: *mut *mut c_void) -> c_int { 0 }
    pub unsafe fn AInputQueue_preDispatchEvent(_: *mut AInputQueue, _: *mut c_void) -> c_int { 0 }
    pub unsafe fn AInputQueue_sendEvent(_: *mut AInputQueue, _: *mut c_void, _: c_int) {}
    pub unsafe fn AInputQueue_finishEvent(_: *mut AInputQueue, _: *mut c_void, _: c_int) {}

    pub unsafe fn AInputEvent_getType(_: *const c_void) -> c_int { AINPUT_EVENT_TYPE_MOTION }
    pub unsafe fn AInputEvent_getDeviceId(_: *const c_void) -> c_int { 0 }
    pub unsafe fn AInputEvent_getSource(_: *const c_void) -> c_int { 0 }

    pub unsafe fn AMotionEvent_getAction(_: *const c_void) -> c_int { AMOTION_EVENT_ACTION_DOWN }
    pub unsafe fn AMotionEvent_getX(_: *const c_void, _: usize) -> f32 { 100.0 }
    pub unsafe fn AMotionEvent_getY(_: *const c_void, _: usize) -> f32 { 200.0 }
    pub unsafe fn AMotionEvent_getPointerCount(_: *const c_void) -> usize { 1 }
    pub unsafe fn AMotionEvent_getPointerId(_: *const c_void, _: usize) -> c_int { 0 }
    pub unsafe fn AMotionEvent_getEventTime(_: *const c_void) -> i64 { 0 }

    pub unsafe fn AKeyEvent_getAction(_: *const c_void) -> c_int { 0 }
    pub unsafe fn AKeyEvent_getKeyCode(_: *const c_void) -> c_int { 0 }
    pub unsafe fn AKeyEvent_getMetaState(_: *const c_void) -> c_int { 0 }

    pub unsafe fn ALooper_forThread() -> *mut ALooper { core::ptr::null_mut() }
    pub unsafe fn ALooper_acquire(_: *mut ALooper) {}
    pub unsafe fn ALooper_release(_: *mut ALooper) {}
    pub unsafe fn ALooper_addFd(_: *mut ALooper, _: c_int, _: c_int, _: c_int, _: Option<unsafe extern "C" fn(c_int, c_int, *mut c_void) -> c_int>, _: *mut c_void) -> c_int { 1 }
    pub unsafe fn ALooper_removeFd(_: *mut ALooper, _: c_int) -> c_int { 1 }
    pub unsafe fn ALooper_pollAll(_: c_int, _: *mut c_int, _: *mut c_int, _: *mut *mut c_void) -> c_int { -1 }
    pub unsafe fn ALooper_wake(_: *mut ALooper) {}

    pub unsafe fn AChoreographer_getInstance() -> *mut AChoreographer { core::ptr::null_mut() }
    pub unsafe fn AChoreographer_postFrameCallback(_: *mut AChoreographer, _: Option<unsafe extern "C" fn(i64, *mut c_void)>, _: *mut c_void) {}

    pub unsafe fn __android_log_write(_: c_int, _: *const c_char, _: *const c_char) -> c_int { 0 }
}

#[cfg(any(not(target_os = "android"), feature = "mock", test ))]
use mock::*;

pub struct NativeWindow {
    ptr: *mut ANativeWindow,
    width: i32,
    height: i32,
}

impl NativeWindow {
    pub unsafe fn from_raw(ptr: *mut ANativeWindow) -> Option<Self> {
        if ptr.is_null() { return None; }
        ANativeWindow_acquire(ptr);
        Some(Self { ptr, width: ANativeWindow_getWidth(ptr), height: ANativeWindow_getHeight(ptr) })
    }

    pub fn width(&self) -> i32 { self.width }
    pub fn height(&self) -> i32 { self.height }
    pub fn size(&self) -> Vec2 { Vec2::new(self.width as f32, self.height as f32) }

    pub fn set_buffer_geometry(&mut self, w: i32, h: i32, f: i32) -> Result<(), ()> {
        if unsafe { ANativeWindow_setBuffersGeometry(self.ptr, w, h, f) } == 0 {
            self.width = w; self.height = h; Ok(())
        } else { Err(()) }
    }

    pub unsafe fn as_raw(&self) -> *mut ANativeWindow { self.ptr }
}

impl Drop for NativeWindow {
    fn drop(&mut self) { unsafe { ANativeWindow_release(self.ptr); } }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputEvent {
    Motion { action: MotionAction, pos: Vec2, pointer_id: i32, pointer_count: usize, time: i64 },
    Key { action: KeyAction, key_code: i32, meta_state: i32 },
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MotionAction { Down, Up, Move, Cancel, PointerDown, PointerUp, Unknown }

impl From<c_int> for MotionAction {
    fn from(action: c_int) -> Self {
        match action & 0xFF {
            AMOTION_EVENT_ACTION_DOWN => Self::Down,
            AMOTION_EVENT_ACTION_UP => Self::Up,
            AMOTION_EVENT_ACTION_MOVE => Self::Move,
            AMOTION_EVENT_ACTION_CANCEL => Self::Cancel,
            AMOTION_EVENT_ACTION_POINTER_DOWN => Self::PointerDown,
            AMOTION_EVENT_ACTION_POINTER_UP => Self::PointerUp,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyAction { Down, Up, Unknown }

impl From<c_int> for KeyAction {
    fn from(action: c_int) -> Self {
        match action { 0 => Self::Down, 1 => Self::Up, _ => Self::Unknown }
    }
}

pub struct InputEventHandle { raw: *mut c_void, queue: *mut AInputQueue }

impl InputEventHandle {
    pub unsafe fn from_raw(raw: *mut c_void, queue: *mut AInputQueue) -> Self {
        Self { raw, queue }
    }

    pub fn parse(&self) -> InputEvent {
        unsafe {
            match AInputEvent_getType(self.raw) {
                AINPUT_EVENT_TYPE_MOTION => InputEvent::Motion {
                    action: AMotionEvent_getAction(self.raw).into(),
                    pos: Vec2::new(AMotionEvent_getX(self.raw, 0), AMotionEvent_getY(self.raw, 0)),
                    pointer_id: AMotionEvent_getPointerId(self.raw, 0),
                    pointer_count: AMotionEvent_getPointerCount(self.raw),
                    time: AMotionEvent_getEventTime(self.raw),
                },
                AINPUT_EVENT_TYPE_KEY => InputEvent::Key {
                    action: AKeyEvent_getAction(self.raw).into(),
                    key_code: AKeyEvent_getKeyCode(self.raw),
                    meta_state: AKeyEvent_getMetaState(self.raw),
                },
                _ => InputEvent::Unknown,
            }
        }
    }

    pub fn finish(self, handled: bool) {
        unsafe { AInputQueue_finishEvent(self.queue, self.raw, if handled { 1 } else { 0 }); }
        core::mem::forget(self);
    }
}

impl Drop for InputEventHandle {
    fn drop(&mut self) { unsafe { AInputQueue_finishEvent(self.queue, self.raw, 0); } }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogLevel { Verbose, Debug, Info, Warn, Error }

impl LogLevel {
    fn to_android_prio(self) -> c_int {
        match self {
            LogLevel::Verbose => ANDROID_LOG_VERBOSE,
            LogLevel::Debug => ANDROID_LOG_DEBUG,
            LogLevel::Info => ANDROID_LOG_INFO,
            LogLevel::Warn => ANDROID_LOG_WARN,
            LogLevel::Error => ANDROID_LOG_ERROR,
        }
    }
}

pub fn android_log(level: LogLevel, tag: &str, msg: &str) {
    let mut tag_buf = [0u8; 32];
    let mut msg_buf = [0u8; 256];
    let tag_len = tag.len().min(31);
    let msg_len = msg.len().min(255);
    tag_buf[..tag_len].copy_from_slice(&tag.as_bytes()[..tag_len]);
    msg_buf[..msg_len].copy_from_slice(&msg.as_bytes()[..msg_len]);
    unsafe {
        __android_log_write(level.to_android_prio(), tag_buf.as_ptr() as *const c_char, msg_buf.as_ptr() as *const c_char);
    }
}

pub type FrameCallback = extern "C" fn(frame_time_nanos: i64, data: *mut c_void);

pub unsafe fn post_frame_callback(callback: FrameCallback, data: *mut c_void) {
    let ch = AChoreographer_getInstance();
    if !ch.is_null() { AChoreographer_postFrameCallback(ch, Some(callback), data); }
}

pub fn looper_poll_all(timeout_ms: i32) -> i32 {
    unsafe { ALooper_pollAll(timeout_ms, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock;

    #[test]
    fn test_native_window_null() {
        unsafe { assert!(NativeWindow::from_raw(core::ptr::null_mut()).is_none()); }
    }

    #[test]
    fn test_native_window_size() {
        mock::reset_mock_window(1080, 1920);
        unsafe {
            let w = NativeWindow::from_raw(0x2 as *mut ANativeWindow).unwrap();
            assert_eq!(w.width(), 1080);
            assert_eq!(w.height(), 1920);
            assert_eq!(w.size(), Vec2::new(1080.0, 1920.0));
        }
    }

    #[test]
    fn test_native_window_set_geometry() {
        unsafe {
            let mut w = NativeWindow::from_raw(0x1 as *mut ANativeWindow).unwrap();
            assert!(w.set_buffer_geometry(720, 1280, 1).is_ok());
            assert_eq!(w.width(), 720);
            assert_eq!(w.height(), 1280);
        }
    }

    #[test]
    fn test_motion_action_down() {
        assert_eq!(MotionAction::from(AMOTION_EVENT_ACTION_DOWN), MotionAction::Down);
    }

    #[test]
    fn test_motion_action_up() {
        assert_eq!(MotionAction::from(AMOTION_EVENT_ACTION_UP), MotionAction::Up);
    }

    #[test]
    fn test_motion_action_move() {
        assert_eq!(MotionAction::from(AMOTION_EVENT_ACTION_MOVE), MotionAction::Move);
    }

    #[test]
    fn test_motion_action_cancel() {
        assert_eq!(MotionAction::from(AMOTION_EVENT_ACTION_CANCEL), MotionAction::Cancel);
    }

    #[test]
    fn test_motion_action_pointer_down() {
        assert_eq!(MotionAction::from(AMOTION_EVENT_ACTION_POINTER_DOWN), MotionAction::PointerDown);
    }

    #[test]
    fn test_motion_action_pointer_up() {
        assert_eq!(MotionAction::from(AMOTION_EVENT_ACTION_POINTER_UP), MotionAction::PointerUp);
    }

    #[test]
    fn test_motion_action_unknown() {
        assert_eq!(MotionAction::from(999), MotionAction::Unknown);
    }

    #[test]
    fn test_motion_action_mask() {
        let action_with_pointer = (1 << 8) | AMOTION_EVENT_ACTION_POINTER_DOWN;
        assert_eq!(MotionAction::from(action_with_pointer), MotionAction::PointerDown);
    }

    #[test]
    fn test_key_action_down() {
        assert_eq!(KeyAction::from(0), KeyAction::Down);
    }

    #[test]
    fn test_key_action_up() {
        assert_eq!(KeyAction::from(1), KeyAction::Up);
    }

    #[test]
    fn test_key_action_unknown() {
        assert_eq!(KeyAction::from(2), KeyAction::Unknown);
    }

    #[test]
    fn test_input_event_parse_motion() {
        unsafe {
            let h = InputEventHandle::from_raw(0x1 as *mut c_void, 0x1 as *mut AInputQueue);
            match h.parse() {
                InputEvent::Motion { pos, .. } => assert_eq!(pos, Vec2::new(100.0, 200.0)),
                _ => panic!("Expected motion event"),
            }
            h.finish(true);
        }
    }

    #[test]
    fn test_looper_poll_timeout() {
        assert_eq!(looper_poll_all(0), -1);
    }

    #[test]
    fn test_log_level_verbose() {
        assert_eq!(LogLevel::Verbose.to_android_prio(), ANDROID_LOG_VERBOSE);
    }

    #[test]
    fn test_log_level_debug() {
        assert_eq!(LogLevel::Debug.to_android_prio(), ANDROID_LOG_DEBUG);
    }

    #[test]
    fn test_log_level_info() {
        assert_eq!(LogLevel::Info.to_android_prio(), ANDROID_LOG_INFO);
    }

    #[test]
    fn test_log_level_warn() {
        assert_eq!(LogLevel::Warn.to_android_prio(), ANDROID_LOG_WARN);
    }

    #[test]
    fn test_log_level_error() {
        assert_eq!(LogLevel::Error.to_android_prio(), ANDROID_LOG_ERROR);
    }

    #[test]
    fn test_looper_events() {
        assert_eq!(ALOOPER_EVENT_INPUT, 1);
        assert_eq!(ALOOPER_EVENT_OUTPUT, 2);
        assert_eq!(ALOOPER_EVENT_ERROR, 4);
    }

    #[test]
    fn test_motion_actions() {
        assert_eq!(AMOTION_EVENT_ACTION_DOWN, 0);
        assert_eq!(AMOTION_EVENT_ACTION_UP, 1);
        assert_eq!(AMOTION_EVENT_ACTION_MOVE, 2);
    }

    #[test]
    fn test_opaque_types_size() {
        assert_eq!(core::mem::size_of::<ANativeWindow>(), 0);
        assert_eq!(core::mem::size_of::<AInputQueue>(), 0);
        assert_eq!(core::mem::size_of::<ALooper>(), 0);
        assert_eq!(core::mem::size_of::<AChoreographer>(), 0);
    }
}
