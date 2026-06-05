#![no_std]
#![warn(missing_docs)]
#![allow(non_camel_case_types)]

extern crate alloc;
use alloc::string::String;
use alloc::format;
use core::ffi::{c_void, c_int, c_char};
use k1_math::{Vec2, Color, Rect};
use k1_sys::{NativeWindow, android_log, LogLevel};

// ==================== EGL Types ====================

pub type EGLDisplay = *mut c_void;
pub type EGLConfig = *mut c_void;
pub type EGLSurface = *mut c_void;
pub type EGLContext = *mut c_void;
pub type EGLNativeDisplayType = *mut c_void;
pub type EGLNativeWindowType = *mut c_void;
pub type EGLint = c_int;

pub const EGL_DEFAULT_DISPLAY: EGLNativeDisplayType = core::ptr::null_mut();
pub const EGL_NO_CONTEXT: EGLContext = core::ptr::null_mut();
pub const EGL_NO_DISPLAY: EGLDisplay = core::ptr::null_mut();
pub const EGL_NO_SURFACE: EGLSurface = core::ptr::null_mut();

pub const EGL_SUCCESS: EGLint = 0x3000;
pub const EGL_NOT_INITIALIZED: EGLint = 0x3001;
pub const EGL_BAD_ACCESS: EGLint = 0x3002;
pub const EGL_BAD_ALLOC: EGLint = 0x3003;
pub const EGL_BAD_ATTRIBUTE: EGLint = 0x3004;
pub const EGL_BAD_CONFIG: EGLint = 0x3005;
pub const EGL_BAD_CONTEXT: EGLint = 0x3006;
pub const EGL_BAD_CURRENT_SURFACE: EGLint = 0x3007;
pub const EGL_BAD_DISPLAY: EGLint = 0x3008;
pub const EGL_BAD_MATCH: EGLint = 0x3009;
pub const EGL_BAD_NATIVE_PIXMAP: EGLint = 0x300A;
pub const EGL_BAD_NATIVE_WINDOW: EGLint = 0x300B;
pub const EGL_BAD_PARAMETER: EGLint = 0x300C;
pub const EGL_BAD_SURFACE: EGLint = 0x300D;

pub const EGL_BUFFER_SIZE: EGLint = 0x3020;
pub const EGL_ALPHA_SIZE: EGLint = 0x3021;
pub const EGL_BLUE_SIZE: EGLint = 0x3022;
pub const EGL_GREEN_SIZE: EGLint = 0x3023;
pub const EGL_RED_SIZE: EGLint = 0x3024;
pub const EGL_DEPTH_SIZE: EGLint = 0x3025;
pub const EGL_STENCIL_SIZE: EGLint = 0x3026;
pub const EGL_SURFACE_TYPE: EGLint = 0x3033;
pub const EGL_RENDERABLE_TYPE: EGLint = 0x3040;
pub const EGL_NONE: EGLint = 0x3038;
pub const EGL_WINDOW_BIT: EGLint = 0x0004;
pub const EGL_OPENGL_ES2_BIT: EGLint = 0x0004;
pub const EGL_OPENGL_ES3_BIT: EGLint = 0x0040;
pub const EGL_CONTEXT_CLIENT_VERSION: EGLint = 0x3098;
pub const EGL_VENDOR: EGLint = 0x3053;
pub const EGL_VERSION: EGLint = 0x3054;

// ==================== GL Constants ====================

pub const GL_FLOAT: c_int = 0x1406;
pub const GL_UNSIGNED_BYTE: c_int = 0x1401;
pub const GL_TEXTURE_2D: c_int = 0x0DE1;
pub const GL_TEXTURE_MAG_FILTER: c_int = 0x2800;
pub const GL_TEXTURE_MIN_FILTER: c_int = 0x2801;
pub const GL_TEXTURE_WRAP_S: c_int = 0x2802;
pub const GL_TEXTURE_WRAP_T: c_int = 0x2803;
pub const GL_NEAREST: c_int = 0x2600;
pub const GL_LINEAR: c_int = 0x2601;
pub const GL_CLAMP_TO_EDGE: c_int = 0x812F;
pub const GL_COLOR_BUFFER_BIT: c_int = 0x00004000;
pub const GL_DEPTH_BUFFER_BIT: c_int = 0x00000100;
pub const GL_VERTEX_SHADER: c_int = 0x8B31;
pub const GL_FRAGMENT_SHADER: c_int = 0x8B30;
pub const GL_COMPILE_STATUS: c_int = 0x8B81;
pub const GL_LINK_STATUS: c_int = 0x8B82;
pub const GL_INFO_LOG_LENGTH: c_int = 0x8B84;
pub const GL_ARRAY_BUFFER: c_int = 0x8892;
pub const GL_ELEMENT_ARRAY_BUFFER: c_int = 0x8893;
pub const GL_STATIC_DRAW: c_int = 0x88E4;
pub const GL_DYNAMIC_DRAW: c_int = 0x88E8;
pub const GL_TRIANGLES: c_int = 0x0004;
pub const GL_TRIANGLE_STRIP: c_int = 0x0005;
pub const GL_BLEND: c_int = 0x0BE2;
pub const GL_SRC_ALPHA: c_int = 0x0302;
pub const GL_ONE_MINUS_SRC_ALPHA: c_int = 0x0303;
pub const GL_SCISSOR_TEST: c_int = 0x0C11;
pub const GL_VIEWPORT: c_int = 0x0BA2;

// ==================== Mock EGL/GL ====================

#[cfg(any(not(target_os = "android"), feature = "mock"))]
mod mock_impl {
    use super::*;
    use core::sync::atomic::{AtomicPtr, AtomicI32, Ordering};
    
    static DPY: AtomicPtr<c_void> = AtomicPtr::new(0x1000 as *mut c_void);
    static SURF: AtomicPtr<c_void> = AtomicPtr::new(0x2000 as *mut c_void);
    static CTX: AtomicPtr<c_void> = AtomicPtr::new(0x3000 as *mut c_void);
    static PROG: AtomicI32 = AtomicI32::new(1);
    static SHADER: AtomicI32 = AtomicI32::new(2);
    static BUF: AtomicI32 = AtomicI32::new(3);
    static TEX: AtomicI32 = AtomicI32::new(4);
    
    pub unsafe fn eglGetDisplay(_: EGLNativeDisplayType) -> EGLDisplay { DPY.load(Ordering::Relaxed) }
    pub unsafe fn eglInitialize(_: EGLDisplay, major: *mut EGLint, minor: *mut EGLint) -> EGLint {
        if !major.is_null() { *major = 1; }
        if !minor.is_null() { *minor = 4; }
        EGL_SUCCESS
    }
    pub unsafe fn eglTerminate(_: EGLDisplay) -> EGLint { EGL_SUCCESS }
    pub unsafe fn eglChooseConfig(_: EGLDisplay, _: *const EGLint, configs: *mut EGLConfig, _: EGLint, num: *mut EGLint) -> EGLint {
        if !num.is_null() { *num = 1; }
        if !configs.is_null() { *configs = 0x4000 as *mut c_void; }
        EGL_SUCCESS
    }
    pub unsafe fn eglCreateWindowSurface(_: EGLDisplay, _: EGLConfig, _: EGLNativeWindowType, _: *const EGLint) -> EGLSurface {
        SURF.load(Ordering::Relaxed)
    }
    pub unsafe fn eglCreateContext(_: EGLDisplay, _: EGLConfig, _: EGLContext, _: *const EGLint) -> EGLContext {
        CTX.load(Ordering::Relaxed)
    }
    pub unsafe fn eglMakeCurrent(_: EGLDisplay, _: EGLSurface, _: EGLSurface, _: EGLContext) -> EGLint { EGL_SUCCESS }
    pub unsafe fn eglSwapBuffers(_: EGLDisplay, _: EGLSurface) -> EGLint { EGL_SUCCESS }
    pub unsafe fn eglSwapInterval(_: EGLDisplay, _: EGLint) -> EGLint { EGL_SUCCESS }
    pub unsafe fn eglGetError() -> EGLint { EGL_SUCCESS }
    pub unsafe fn eglDestroySurface(_: EGLDisplay, _: EGLSurface) -> EGLint { EGL_SUCCESS }
    pub unsafe fn eglDestroyContext(_: EGLDisplay, _: EGLContext) -> EGLint { EGL_SUCCESS }
    pub unsafe fn eglQueryString(_: EGLDisplay, _: EGLint) -> *const c_char { core::ptr::null() }
    
    pub unsafe fn glCreateProgram() -> c_int { PROG.load(Ordering::Relaxed) }
    pub unsafe fn glCreateShader(_: c_int) -> c_int { SHADER.load(Ordering::Relaxed) }
    pub unsafe fn glGenBuffers(n: c_int, b: *mut c_int) { if !b.is_null() && n > 0 { *b = BUF.load(Ordering::Relaxed); } }
    pub unsafe fn glGenTextures(n: c_int, t: *mut c_int) { if !t.is_null() && n > 0 { *t = TEX.load(Ordering::Relaxed); } }
    pub unsafe fn glDeleteProgram(_: c_int) {}
    pub unsafe fn glDeleteShader(_: c_int) {}
    pub unsafe fn glDeleteBuffers(_: c_int, _: *const c_int) {}
    pub unsafe fn glDeleteTextures(_: c_int, _: *const c_int) {}
    pub unsafe fn glAttachShader(_: c_int, _: c_int) {}
    pub unsafe fn glCompileShader(_: c_int) {}
    pub unsafe fn glLinkProgram(_: c_int) {}
    pub unsafe fn glUseProgram(_: c_int) {}
    pub unsafe fn glShaderSource(_: c_int, _: c_int, _: *const *const c_char, _: *const c_int) {}
    pub unsafe fn glGetShaderiv(_: c_int, _: c_int, p: *mut c_int) { if !p.is_null() { *p = 1; } }
    pub unsafe fn glGetProgramiv(_: c_int, _: c_int, p: *mut c_int) { if !p.is_null() { *p = 1; } }
    pub unsafe fn glGetShaderInfoLog(_: c_int, _: c_int, _: *mut c_int, _: *mut c_char) {}
    pub unsafe fn glGetProgramInfoLog(_: c_int, _: c_int, _: *mut c_int, _: *mut c_char) {}
    pub unsafe fn glGetAttribLocation(_: c_int, _: *const c_char) -> c_int { 0 }
    pub unsafe fn glGetUniformLocation(_: c_int, _: *const c_char) -> c_int { 0 }
    pub unsafe fn glUniform1f(_: c_int, _: f32) {}
    pub unsafe fn glUniform1i(_: c_int, _: c_int) {}
    pub unsafe fn glUniform2f(_: c_int, _: f32, _: f32) {}
    pub unsafe fn glUniform4f(_: c_int, _: f32, _: f32, _: f32, _: f32) {}
    pub unsafe fn glUniformMatrix4fv(_: c_int, _: c_int, _: u8, _: *const f32) {}
    pub unsafe fn glBindBuffer(_: c_int, _: c_int) {}
    pub unsafe fn glBufferData(_: c_int, _: isize, _: *const c_void, _: c_int) {}
    pub unsafe fn glBufferSubData(_: c_int, _: isize, _: isize, _: *const c_void) {}
    pub unsafe fn glBindTexture(_: c_int, _: c_int) {}
    pub unsafe fn glTexImage2D(_: c_int, _: c_int, _: c_int, _: c_int, _: c_int, _: c_int, _: c_int, _: c_int, _: *const c_void) {}
    pub unsafe fn glTexParameteri(_: c_int, _: c_int, _: c_int) {}
    pub unsafe fn glActiveTexture(_: c_int) {}
    pub unsafe fn glEnable(_: c_int) {}
    pub unsafe fn glDisable(_: c_int) {}
    pub unsafe fn glBlendFunc(_: c_int, _: c_int) {}
    pub unsafe fn glClear(_: c_int) {}
    pub unsafe fn glClearColor(_: f32, _: f32, _: f32, _: f32) {}
    pub unsafe fn glViewport(_: c_int, _: c_int, _: c_int, _: c_int) {}
    pub unsafe fn glScissor(_: c_int, _: c_int, _: c_int, _: c_int) {}
    pub unsafe fn glDrawArrays(_: c_int, _: c_int, _: c_int) {}
    pub unsafe fn glDrawElements(_: c_int, _: c_int, _: c_int, _: *const c_void) {}
    pub unsafe fn glEnableVertexAttribArray(_: c_int) {}
    pub unsafe fn glVertexAttribPointer(_: c_int, _: c_int, _: c_int, _: u8, _: c_int, _: *const c_void) {}
    pub unsafe fn glGetError() -> c_int { 0 }
    pub unsafe fn glGetString(_: c_int) -> *const c_char { core::ptr::null() }
}

#[cfg(any(not(target_os = "android"), feature = "mock"))]
use mock_impl::*;

// ==================== Error Handling ====================

#[derive(Debug, Clone, PartialEq)]
pub enum EglError {
    NotInitialized, BadAccess, BadAlloc, BadAttribute, BadConfig,
    BadContext, BadCurrentSurface, BadDisplay, BadMatch,
    BadNativePixmap, BadNativeWindow, BadParameter, BadSurface,
    Unknown(EGLint), Message(String),
}

impl EglError {
    fn from_egl_error(err: EGLint) -> Self {
        match err {
            EGL_NOT_INITIALIZED => Self::NotInitialized,
            EGL_BAD_ACCESS => Self::BadAccess,
            EGL_BAD_ALLOC => Self::BadAlloc,
            EGL_BAD_ATTRIBUTE => Self::BadAttribute,
            EGL_BAD_CONFIG => Self::BadConfig,
            EGL_BAD_CONTEXT => Self::BadContext,
            EGL_BAD_CURRENT_SURFACE => Self::BadCurrentSurface,
            EGL_BAD_DISPLAY => Self::BadDisplay,
            EGL_BAD_MATCH => Self::BadMatch,
            EGL_BAD_NATIVE_PIXMAP => Self::BadNativePixmap,
            EGL_BAD_NATIVE_WINDOW => Self::BadNativeWindow,
            EGL_BAD_PARAMETER => Self::BadParameter,
            EGL_BAD_SURFACE => Self::BadSurface,
            _ => Self::Unknown(err),
        }
    }
}

// ==================== Safe EGL Types ====================

pub struct EglDisplay {
    handle: EGLDisplay,
    initialized: bool,
}

impl EglDisplay {
    pub fn get_default() -> Result<Self, EglError> {
        unsafe {
            let h = eglGetDisplay(EGL_DEFAULT_DISPLAY);
            if h.is_null() { Err(EglError::Message(String::from("null display"))) }
            else { Ok(Self { handle: h, initialized: false }) }
        }
    }
    
    pub fn initialize(&mut self) -> Result<(i32, i32), EglError> {
        unsafe {
            let mut major: EGLint = 0;
            let mut minor: EGLint = 0;
            let r = eglInitialize(self.handle, &mut major, &mut minor);
            if r == EGL_SUCCESS { self.initialized = true; Ok((major as i32, minor as i32)) }
            else { Err(EglError::from_egl_error(r)) }
        }
    }
    
    pub fn choose_config(&self, attribs: &[EGLint]) -> Result<EGLConfig, EglError> {
        unsafe {
            let mut cfg: EGLConfig = core::ptr::null_mut();
            let mut n: EGLint = 0;
            let r = eglChooseConfig(self.handle, attribs.as_ptr(), &mut cfg, 1, &mut n);
            if r == EGL_SUCCESS && n > 0 && !cfg.is_null() { Ok(cfg) }
            else { Err(EglError::from_egl_error(r)) }
        }
    }
    
    pub fn create_window_surface(&self, cfg: EGLConfig, win: &NativeWindow) -> Result<EGLSurface, EglError> {
        unsafe {
            let s = eglCreateWindowSurface(self.handle, cfg, win.as_raw() as EGLNativeWindowType, core::ptr::null());
            if s.is_null() { Err(EglError::Message(String::from("null surface"))) }
            else { Ok(s) }
        }
    }
    
    pub fn create_context(&self, cfg: EGLConfig, ver: i32) -> Result<EGLContext, EglError> {
        unsafe {
            let a = [EGL_CONTEXT_CLIENT_VERSION, ver as EGLint, EGL_NONE];
            let c = eglCreateContext(self.handle, cfg, EGL_NO_CONTEXT, a.as_ptr());
            if c.is_null() { Err(EglError::Message(String::from("null context"))) }
            else { Ok(c) }
        }
    }
    
    pub fn make_current(&self, surf: EGLSurface, ctx: EGLContext) -> Result<(), EglError> {
        unsafe {
            let r = eglMakeCurrent(self.handle, surf, surf, ctx);
            if r == EGL_SUCCESS { Ok(()) } else { Err(EglError::from_egl_error(r)) }
        }
    }
    
    pub fn swap_buffers(&self, surf: EGLSurface) -> Result<(), EglError> {
        unsafe {
            let r = eglSwapBuffers(self.handle, surf);
            if r == EGL_SUCCESS { Ok(()) } else { Err(EglError::from_egl_error(r)) }
        }
    }
    
    pub fn set_swap_interval(&self, i: i32) -> Result<(), EglError> {
        unsafe {
            let r = eglSwapInterval(self.handle, i as EGLint);
            if r == EGL_SUCCESS { Ok(()) } else { Err(EglError::from_egl_error(r)) }
        }
    }
    
    pub fn handle(&self) -> EGLDisplay { self.handle }
}

impl Drop for EglDisplay {
    fn drop(&mut self) { if self.initialized { unsafe { eglTerminate(self.handle); } } }
}

// ==================== GL Types ====================

pub struct Shader {
    handle: c_int,
    shader_type: c_int,
}

impl Shader {
    pub fn from_source(st: c_int, src: &str) -> Result<Self, String> {
        unsafe {
            let h = glCreateShader(st);
            if h == 0 { return Err(String::from("glCreateShader failed")); }
            let p = src.as_ptr() as *const c_char;
            let l = src.len() as c_int;
            glShaderSource(h, 1, &p, &l);
            glCompileShader(h);
            let mut s: c_int = 0;
            glGetShaderiv(h, GL_COMPILE_STATUS, &mut s);
            if s == 0 { glDeleteShader(h); return Err(String::from("compile failed")); }
            Ok(Self { handle: h, shader_type: st })
        }
    }
    pub fn handle(&self) -> c_int { self.handle }
    pub fn shader_type(&self) -> c_int { self.shader_type }
}

impl Drop for Shader {
    fn drop(&mut self) { unsafe { glDeleteShader(self.handle); } }
}

pub struct Program {
    handle: c_int,
}

impl Program {
    pub fn new() -> Result<Self, String> {
        unsafe {
            let h = glCreateProgram();
            if h == 0 { Err(String::from("glCreateProgram failed")) } else { Ok(Self { handle: h }) }
        }
    }
    pub fn attach_shader(&self, s: &Shader) { unsafe { glAttachShader(self.handle, s.handle()); } }
    pub fn link(&self) -> Result<(), String> {
        unsafe {
            glLinkProgram(self.handle);
            let mut s: c_int = 0;
            glGetProgramiv(self.handle, GL_LINK_STATUS, &mut s);
            if s == 0 { Err(String::from("link failed")) } else { Ok(()) }
        }
    }
    pub fn use_program(&self) { unsafe { glUseProgram(self.handle); } }
    pub fn uniform_location(&self, name: &str) -> c_int {
        unsafe {
            let mut buf = [0u8; 32];
            let n = name.len().min(31);
            buf[..n].copy_from_slice(name.as_bytes());
            glGetUniformLocation(self.handle, buf.as_ptr() as *const c_char)
        }
    }
    pub fn attrib_location(&self, name: &str) -> c_int {
        unsafe {
            let mut buf = [0u8; 32];
            let n = name.len().min(31);
            buf[..n].copy_from_slice(name.as_bytes());
            glGetAttribLocation(self.handle, buf.as_ptr() as *const c_char)
        }
    }
    pub fn set_uniform_1f(&self, loc: c_int, v: f32) { unsafe { glUniform1f(loc, v); } }
    pub fn set_uniform_1i(&self, loc: c_int, v: c_int) { unsafe { glUniform1i(loc, v); } }
    pub fn set_uniform_2f(&self, loc: c_int, a: f32, b: f32) { unsafe { glUniform2f(loc, a, b); } }
    pub fn set_uniform_4f(&self, loc: c_int, a: f32, b: f32, c: f32, d: f32) { unsafe { glUniform4f(loc, a, b, c, d); } }
    pub fn set_uniform_mat4(&self, loc: c_int, t: bool, v: &[f32; 16]) {
        unsafe { glUniformMatrix4fv(loc, 1, if t { 1 } else { 0 }, v.as_ptr()); }
    }
    pub fn handle(&self) -> c_int { self.handle }
}

impl Drop for Program {
    fn drop(&mut self) { unsafe { glDeleteProgram(self.handle); } }
}

pub struct Buffer {
    handle: c_int,
    target: c_int,
}

impl Buffer {
    pub fn new(target: c_int) -> Result<Self, String> {
        unsafe {
            let mut h: c_int = 0;
            glGenBuffers(1, &mut h);
            if h == 0 { Err(String::from("glGenBuffers failed")) } else { glBindBuffer(target, h); Ok(Self { handle: h, target }) }
        }
    }
    pub fn upload<T>(&self, data: &[T], usage: c_int) {
        unsafe {
            glBindBuffer(self.target, self.handle);
            let s = (data.len() * core::mem::size_of::<T>()) as isize;
            glBufferData(self.target, s, data.as_ptr() as *const c_void, usage);
        }
    }
    pub fn bind(&self) { unsafe { glBindBuffer(self.target, self.handle); } }
    pub fn handle(&self) -> c_int { self.handle }
}

impl Drop for Buffer {
    fn drop(&mut self) { unsafe { glDeleteBuffers(1, &self.handle); } }
}

pub struct Texture {
    handle: c_int,
}

impl Texture {
    pub fn new() -> Result<Self, String> {
        unsafe {
            let mut h: c_int = 0;
            glGenTextures(1, &mut h);
            if h == 0 { Err(String::from("glGenTextures failed")) } else {
                glBindTexture(GL_TEXTURE_2D, h);
                glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
                glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
                Ok(Self { handle: h })
            }
        }
    }
    pub fn upload_rgba(&self, w: i32, h: i32, data: &[u8]) {
        unsafe {
            glBindTexture(GL_TEXTURE_2D, self.handle);
            glTexImage2D(GL_TEXTURE_2D, 0, GL_TEXTURE_2D as i32, w, h, 0, GL_TEXTURE_2D as i32, GL_UNSIGNED_BYTE, data.as_ptr() as *const c_void);
        }
    }
    pub fn bind(&self, unit: c_int) {
        unsafe {
            glActiveTexture(GL_TEXTURE_2D + unit);
            glBindTexture(GL_TEXTURE_2D, self.handle);
        }
    }
    pub fn handle(&self) -> c_int { self.handle }
}

impl Drop for Texture {
    fn drop(&mut self) { unsafe { glDeleteTextures(1, &self.handle); } }
}

// ==================== GL Context ====================

pub struct GlContext {
    display: EglDisplay,
    surface: EGLSurface,
    context: EGLContext,
    width: i32,
    height: i32,
}

impl GlContext {
    pub fn from_window(win: &NativeWindow) -> Result<Self, String> {
        let mut dpy = EglDisplay::get_default().map_err(|e| format!("{:?}", e))?;
        let (maj, min) = dpy.initialize().map_err(|e| format!("{:?}", e))?;
        android_log(LogLevel::Info, "K1-GLES", &format!("EGL {}.{}", maj, min));
        
        let attribs = [
            EGL_SURFACE_TYPE, EGL_WINDOW_BIT,
            EGL_RENDERABLE_TYPE, EGL_OPENGL_ES2_BIT,
            EGL_RED_SIZE, 8, EGL_GREEN_SIZE, 8, EGL_BLUE_SIZE, 8, EGL_ALPHA_SIZE, 8,
            EGL_DEPTH_SIZE, 16, EGL_STENCIL_SIZE, 0,
            EGL_NONE,
        ];
        let cfg = dpy.choose_config(&attribs).map_err(|e| format!("{:?}", e))?;
        let surf = dpy.create_window_surface(cfg, win).map_err(|e| format!("{:?}", e))?;
        let ctx = dpy.create_context(cfg, 2).map_err(|e| format!("{:?}", e))?;
        dpy.make_current(surf, ctx).map_err(|e| format!("{:?}", e))?;
        
        let w = win.width();
        let h = win.height();
        unsafe { glViewport(0, 0, w, h); glClearColor(0.0, 0.0, 0.0, 1.0); }
        
        Ok(Self { display: dpy, surface: surf, context: ctx, width: w, height: h })
    }
    
    pub fn clear(&self, c: Color) {
        unsafe { glClearColor(c.r, c.g, c.b, c.a); glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT); }
    }
    pub fn set_viewport(&mut self, x: i32, y: i32, w: i32, h: i32) {
        self.width = w; self.height = h;
        unsafe { glViewport(x, y, w, h); }
    }
    pub fn set_scissor(&self, r: Option<Rect>) {
        unsafe {
            if let Some(rect) = r {
                glEnable(GL_SCISSOR_TEST);
                glScissor(rect.min.x as i32, rect.min.y as i32, rect.width() as i32, rect.height() as i32);
            } else { glDisable(GL_SCISSOR_TEST); }
        }
    }
    pub fn enable_blend(&self) { unsafe { glEnable(GL_BLEND); glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA); } }
    pub fn disable_blend(&self) { unsafe { glDisable(GL_BLEND); } }
    pub fn swap_buffers(&self) -> Result<(), EglError> { self.display.swap_buffers(self.surface) }
    pub fn set_vsync(&self, en: bool) -> Result<(), EglError> { self.display.set_swap_interval(if en { 1 } else { 0 }) }
    pub fn size(&self) -> Vec2 { Vec2::new(self.width as f32, self.height as f32) }
    pub fn width(&self) -> i32 { self.width }
    pub fn height(&self) -> i32 { self.height }
}

impl Drop for GlContext {
    fn drop(&mut self) {
        unsafe {
            eglMakeCurrent(self.display.handle(), EGL_NO_SURFACE, EGL_NO_SURFACE, EGL_NO_CONTEXT);
            eglDestroySurface(self.display.handle(), self.surface);
            eglDestroyContext(self.display.handle(), self.context);
        }
    }
}
