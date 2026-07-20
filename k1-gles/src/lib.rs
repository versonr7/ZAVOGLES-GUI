#![no_std]
#![allow(warnings)]

use core::ffi::{c_char, c_int, c_void};
use k1_math::{Color, Mat4, Rect, Vec2};

// ==================== EGL Constants ====================
pub const EGL_DEFAULT_DISPLAY: *mut c_void = 0 as *mut c_void;
pub const EGL_NO_CONTEXT: *mut c_void = 0 as *mut c_void;
pub const EGL_NO_SURFACE: *mut c_void = 0 as *mut c_void;
pub const EGL_NONE: c_int = 0x3038;
pub const EGL_WINDOW_BIT: c_int = 0x0004;
pub const EGL_OPENGL_ES2_BIT: c_int = 0x0004;
pub const EGL_RENDERABLE_TYPE: c_int = 0x3040;
pub const EGL_SURFACE_TYPE: c_int = 0x3033;
pub const EGL_RED_SIZE: c_int = 0x3024;
pub const EGL_GREEN_SIZE: c_int = 0x3023;
pub const EGL_BLUE_SIZE: c_int = 0x3022;
pub const EGL_ALPHA_SIZE: c_int = 0x3021; // ← 0x3021 مو 0x3027!
pub const EGL_DEPTH_SIZE: c_int = 0x3025;
pub const EGL_STENCIL_SIZE: c_int = 0x3028; // ← 0x3028 مو 0x3027!
pub const EGL_CONFIG_CAVEAT: c_int = 0x3027; // ← 0x3027 هي هذي!
pub const EGL_OPENGL_ES3_BIT: c_int = 0x0040; // جديد
pub const EGL_CONTEXT_CLIENT_VERSION: c_int = 0x3098;

// ==================== GL Constants ====================
pub const GL_VERTEX_SHADER: c_int = 0x8B31;
pub const GL_FRAGMENT_SHADER: c_int = 0x8B30;
pub const GL_ARRAY_BUFFER: c_int = 0x8892;
pub const GL_ELEMENT_ARRAY_BUFFER: c_int = 0x8893;
pub const GL_STATIC_DRAW: c_int = 0x88E4;
pub const GL_DYNAMIC_DRAW: c_int = 0x88E8;
pub const GL_FLOAT: c_int = 0x1406;
pub const GL_UNSIGNED_BYTE: c_int = 0x1401;
pub const GL_UNSIGNED_SHORT: c_int = 0x1403;
pub const GL_RGBA: c_int = 0x1908;
pub const GL_TEXTURE_2D: c_int = 0x0DE1;
pub const GL_TEXTURE0: c_int = 0x84C0;
pub const GL_ACTIVE_TEXTURE: c_int = 0x84E0;
pub const GL_TEXTURE_MIN_FILTER: c_int = 0x2801;
pub const GL_TEXTURE_MAG_FILTER: c_int = 0x2800;
pub const GL_LINEAR: c_int = 0x2601;
pub const GL_BLEND: c_int = 0x0BE2;
pub const GL_SRC_ALPHA: c_int = 0x0302;
pub const GL_ONE_MINUS_SRC_ALPHA: c_int = 0x0303;
pub const GL_COLOR_BUFFER_BIT: c_int = 0x00004000;
pub const GL_DEPTH_BUFFER_BIT: c_int = 0x00000100;
pub const GL_SCISSOR_TEST: c_int = 0x0C11;
pub const GL_TRIANGLES: c_int = 0x0004;
pub const GL_COMPILE_STATUS: c_int = 0x8B81;
pub const GL_LINK_STATUS: c_int = 0x8B82;
pub const GL_TRUE: c_int = 1;

// ==================== EGL FFI ====================
#[cfg(all(target_os = "android", not(feature = "mock"), not(test)))]
#[link(name = "GLESv2")]
#[link(name = "EGL")]
extern "C" {
    pub fn eglGetDisplay(display_id: *mut c_void) -> *mut c_void;
    pub fn eglInitialize(dpy: *mut c_void, major: *mut c_int, minor: *mut c_int) -> c_int;
    pub fn eglChooseConfig(
        dpy: *mut c_void,
        attrib_list: *const c_int,
        configs: *mut c_void,
        config_size: c_int,
        num_config: *mut c_int,
    ) -> c_int;
    pub fn eglCreateWindowSurface(
        dpy: *mut c_void,
        config: *mut c_void,
        win: *mut c_void,
        attrib_list: *const c_int,
    ) -> *mut c_void;
    pub fn eglCreateContext(
        dpy: *mut c_void,
        config: *mut c_void,
        share_context: *mut c_void,
        attrib_list: *const c_int,
    ) -> *mut c_void;
    pub fn eglMakeCurrent(
        dpy: *mut c_void,
        draw: *mut c_void,
        read: *mut c_void,
        ctx: *mut c_void,
    ) -> c_int;
    pub fn eglSwapBuffers(dpy: *mut c_void, surface: *mut c_void) -> c_int;
    pub fn eglDestroyContext(dpy: *mut c_void, ctx: *mut c_void) -> c_int;
    pub fn eglDestroySurface(dpy: *mut c_void, surface: *mut c_void) -> c_int;
    pub fn eglGetError() -> c_int;
}

#[cfg(any(not(target_os = "android"), feature = "mock", test))]
pub mod egl_mock {
    use super::*;
    use core::sync::atomic::{AtomicI32, Ordering};
    static NEXT: AtomicI32 = AtomicI32::new(1);

    pub unsafe fn eglGetDisplay(_: *mut c_void) -> *mut c_void {
        0x1 as *mut c_void
    }
    pub unsafe fn eglInitialize(_: *mut c_void, maj: *mut c_int, min: *mut c_int) -> c_int {
        *maj = 1;
        *min = 4;
        1
    }
    pub unsafe fn eglChooseConfig(
        _: *mut c_void,
        _: *const c_int,
        cfg: *mut c_void,
        _: c_int,
        n: *mut c_int,
    ) -> c_int {
        *n = 1;
        *(cfg as *mut usize) = 0xABC;
        1
    }
    pub unsafe fn eglCreateWindowSurface(
        _: *mut c_void,
        _: *mut c_void,
        _: *mut c_void,
        _: *const c_int,
    ) -> *mut c_void {
        NEXT.fetch_add(1, Ordering::Relaxed) as *mut c_void
    }
    pub unsafe fn eglCreateContext(
        _: *mut c_void,
        _: *mut c_void,
        _: *mut c_void,
        _: *const c_int,
    ) -> *mut c_void {
        NEXT.fetch_add(1, Ordering::Relaxed) as *mut c_void
    }
    pub unsafe fn eglMakeCurrent(
        _: *mut c_void,
        _: *mut c_void,
        _: *mut c_void,
        _: *mut c_void,
    ) -> c_int {
        1
    }
    pub unsafe fn eglSwapBuffers(_: *mut c_void, _: *mut c_void) -> c_int {
        1
    }
    pub unsafe fn eglDestroyContext(_: *mut c_void, _: *mut c_void) -> c_int {
        1
    }
    pub unsafe fn eglDestroySurface(_: *mut c_void, _: *mut c_void) -> c_int {
        1
    }
    pub unsafe fn eglGetError() -> c_int {
        0x3000
    }
}
#[cfg(any(not(target_os = "android"), feature = "mock", test))]
use egl_mock::*;

// ==================== GL FFI ====================
#[cfg(all(target_os = "android", not(feature = "mock"), not(test)))]
#[link(name = "GLESv2")]
#[link(name = "EGL")]
extern "C" {
    pub fn glViewport(x: c_int, y: c_int, width: c_int, height: c_int);
    pub fn glUniform1f(location: c_int, v: f32);
    pub fn glClearColor(r: f32, g: f32, b: f32, a: f32);
    pub fn glClear(mask: c_int);
    pub fn glEnable(cap: c_int);
    pub fn glDisable(cap: c_int);
    pub fn glBlendFunc(sfactor: c_int, dfactor: c_int);
    pub fn glScissor(x: c_int, y: c_int, width: c_int, height: c_int);
    pub fn glActiveTexture(texture: c_int);
    pub fn glCreateShader(ty: c_int) -> c_int;
    pub fn glShaderSource(
        shader: c_int,
        count: c_int,
        string: *const *const c_char,
        length: *const c_int,
    );
    pub fn glCompileShader(shader: c_int);
    pub fn glGetShaderiv(shader: c_int, pname: c_int, params: *mut c_int);
    pub fn glGetShaderInfoLog(
        shader: c_int,
        bufSize: c_int,
        length: *mut c_int,
        infoLog: *mut c_char,
    );
    pub fn glDeleteShader(shader: c_int);
    pub fn glCreateProgram() -> c_int;
    pub fn glAttachShader(program: c_int, shader: c_int);
    pub fn glLinkProgram(program: c_int);
    pub fn glGetProgramiv(program: c_int, pname: c_int, params: *mut c_int);
    pub fn glGetProgramInfoLog(
        program: c_int,
        bufSize: c_int,
        length: *mut c_int,
        infoLog: *mut c_char,
    );
    pub fn glUseProgram(program: c_int);
    pub fn glGetUniformLocation(program: c_int, name: *const c_char) -> c_int;
    pub fn glUniformMatrix4fv(location: c_int, count: c_int, transpose: c_int, value: *const f32);
    pub fn glGenBuffers(n: c_int, buffers: *mut c_int);
    pub fn glBindBuffer(target: c_int, buffer: c_int);
    pub fn glBufferData(target: c_int, size: isize, data: *const c_void, usage: c_int);
    pub fn glDeleteBuffers(n: c_int, buffers: *const c_int);
    pub fn glGenTextures(n: c_int, textures: *mut c_int);
    pub fn glBindTexture(target: c_int, texture: c_int);
    pub fn glTexImage2D(
        target: c_int,
        level: c_int,
        internalformat: c_int,
        width: c_int,
        height: c_int,
        border: c_int,
        format: c_int,
        ty: c_int,
        pixels: *const c_void,
    );
    pub fn glTexParameteri(target: c_int, pname: c_int, param: c_int);
    pub fn glDeleteTextures(n: c_int, textures: *const c_int);
    pub fn glDrawElements(mode: c_int, count: c_int, ty: c_int, indices: *const c_void);
    pub fn glVertexAttribPointer(
        idx: c_int,
        size: c_int,
        ty: c_int,
        normalized: c_int,
        stride: c_int,
        ptr: *const c_void,
    );
    pub fn glEnableVertexAttribArray(idx: c_int);
    pub fn glDisableVertexAttribArray(idx: c_int);
}

#[cfg(any(not(target_os = "android"), feature = "mock", test))]
pub mod gl_mock {
    use super::*;
    use core::sync::atomic::{AtomicI32, Ordering};
    static NEXT: AtomicI32 = AtomicI32::new(100);

    pub unsafe fn glViewport(_: c_int, _: c_int, _: c_int, _: c_int) {}
    pub unsafe fn glUniform1f(_: c_int, _: f32) {}
    pub unsafe fn glClearColor(_: f32, _: f32, _: f32, _: f32) {}
    pub unsafe fn glClear(_: c_int) {}
    pub unsafe fn glEnable(_: c_int) {}
    pub unsafe fn glDisable(_: c_int) {}
    pub unsafe fn glBlendFunc(_: c_int, _: c_int) {}
    pub unsafe fn glScissor(_: c_int, _: c_int, _: c_int, _: c_int) {}
    pub unsafe fn glActiveTexture(_: c_int) {}
    pub unsafe fn glCreateShader(_: c_int) -> c_int {
        NEXT.fetch_add(1, Ordering::Relaxed)
    }
    pub unsafe fn glShaderSource(_: c_int, _: c_int, _: *const *const c_char, _: *const c_int) {}
    pub unsafe fn glCompileShader(_: c_int) {}
    pub unsafe fn glGetShaderiv(_: c_int, _: c_int, params: *mut c_int) {
        *params = 1;
    }
    pub unsafe fn glGetShaderInfoLog(_: c_int, _: c_int, _: *mut c_int, _: *mut c_char) {}
    pub unsafe fn glDeleteShader(_: c_int) {}
    pub unsafe fn glCreateProgram() -> c_int {
        NEXT.fetch_add(1, Ordering::Relaxed)
    }
    pub unsafe fn glAttachShader(_: c_int, _: c_int) {}
    pub unsafe fn glLinkProgram(_: c_int) {}
    pub unsafe fn glGetProgramiv(_: c_int, _: c_int, params: *mut c_int) {
        *params = 1;
    }
    pub unsafe fn glGetProgramInfoLog(_: c_int, _: c_int, _: *mut c_int, _: *mut c_char) {}
    pub unsafe fn glUseProgram(_: c_int) {}
    pub unsafe fn glGetUniformLocation(_: c_int, _: *const c_char) -> c_int {
        0
    }
    pub unsafe fn glUniformMatrix4fv(_: c_int, _: c_int, _: c_int, _: *const f32) {}
    pub unsafe fn glGenBuffers(n: c_int, buffers: *mut c_int) {
        for i in 0..n {
            *buffers.offset(i as isize) = NEXT.fetch_add(1, Ordering::Relaxed);
        }
    }
    pub unsafe fn glBindBuffer(_: c_int, _: c_int) {}
    pub unsafe fn glBufferData(_: c_int, _: isize, _: *const c_void, _: c_int) {}
    pub unsafe fn glDeleteBuffers(_: c_int, _: *const c_int) {}
    pub unsafe fn glGenTextures(n: c_int, textures: *mut c_int) {
        for i in 0..n {
            *textures.offset(i as isize) = NEXT.fetch_add(1, Ordering::Relaxed);
        }
    }
    pub unsafe fn glBindTexture(_: c_int, _: c_int) {}
    pub unsafe fn glTexImage2D(
        _: c_int,
        _: c_int,
        _: c_int,
        _: c_int,
        _: c_int,
        _: c_int,
        _: c_int,
        _: c_int,
        _: *const c_void,
    ) {
    }
    pub unsafe fn glTexParameteri(_: c_int, _: c_int, _: c_int) {}
    pub unsafe fn glDeleteTextures(_: c_int, _: *const c_int) {}
    pub unsafe fn glDrawElements(_: c_int, _: c_int, _: c_int, _: *const c_void) {}
    pub unsafe fn glVertexAttribPointer(
        _: c_int,
        _: c_int,
        _: c_int,
        _: c_int,
        _: c_int,
        _: *const c_void,
    ) {
    }
    pub unsafe fn glEnableVertexAttribArray(_: c_int) {}
    pub unsafe fn glDisableVertexAttribArray(_: c_int) {}
}
#[cfg(any(not(target_os = "android"), feature = "mock", test))]
use gl_mock::*;

// ==================== EGL Types ====================
pub struct EglDisplay {
    handle: *mut c_void,
}

impl EglDisplay {
    pub fn get_default() -> Result<Self, i32> {
        let h = unsafe { eglGetDisplay(EGL_DEFAULT_DISPLAY) };
        if h.is_null() {
            return Err(0x3008);
        }
        Ok(Self { handle: h })
    }
    pub fn initialize(&mut self) -> Result<(i32, i32), i32> {
        let mut maj: c_int = 0;
        let mut min: c_int = 0;
        if unsafe { eglInitialize(self.handle, &mut maj, &mut min) } == 1 {
            Ok((maj, min))
        } else {
            Err(unsafe { eglGetError() })
        }
    }
    pub fn choose_config(&mut self, attribs: &[c_int]) -> Result<*mut c_void, i32> {
        let mut cfg: *mut c_void = core::ptr::null_mut();
        let mut n: c_int = 0;
        if unsafe {
            eglChooseConfig(
                self.handle,
                attribs.as_ptr(),
                &mut cfg as *mut *mut c_void as *mut c_void,
                1,
                &mut n,
            )
        } == 1
            && n > 0
        {
            Ok(cfg)
        } else {
            Err(unsafe { eglGetError() })
        }
    }
    pub fn create_window_surface(
        &mut self,
        config: *mut c_void,
        win: &k1_sys::NativeWindow,
    ) -> Result<*mut c_void, i32> {
        let surf = unsafe {
            eglCreateWindowSurface(
                self.handle,
                config,
                win.as_raw() as *mut c_void,
                core::ptr::null(),
            )
        };
        if surf.is_null() {
            Err(unsafe { eglGetError() })
        } else {
            Ok(surf)
        }
    }
    pub fn create_context(
        &mut self,
        config: *mut c_void,
        version: i32,
    ) -> Result<*mut c_void, i32> {
        let attribs = [EGL_CONTEXT_CLIENT_VERSION, version, EGL_NONE];
        let ctx =
            unsafe { eglCreateContext(self.handle, config, EGL_NO_CONTEXT, attribs.as_ptr()) };
        if ctx.is_null() {
            Err(unsafe { eglGetError() })
        } else {
            Ok(ctx)
        }
    }
    pub fn make_current(&self, surf: *mut c_void, ctx: *mut c_void) -> Result<(), i32> {
        if unsafe { eglMakeCurrent(self.handle, surf, surf, ctx) } == 1 {
            Ok(())
        } else {
            Err(unsafe { eglGetError() })
        }
    }
    pub fn swap_buffers(&self, surf: *mut c_void) -> Result<(), i32> {
        if unsafe { eglSwapBuffers(self.handle, surf) } == 1 {
            Ok(())
        } else {
            Err(unsafe { eglGetError() })
        }
    }
    pub fn handle(&self) -> *mut c_void {
        self.handle
    }
}

// ==================== Shader ====================
pub struct Shader {
    handle: c_int,
    shader_type: c_int,
}

impl Shader {
    pub fn from_source(ty: c_int, source: &str) -> Result<Self, i32> {
        let handle = unsafe { glCreateShader(ty) };
        if handle == 0 {
            return Err(0x9999);
        }
        let src_ptr = source.as_bytes().as_ptr() as *const c_char;
        let len = source.len() as c_int;
        unsafe {
            glShaderSource(handle, 1, &src_ptr, &len);
        }
        unsafe {
            glCompileShader(handle);
        }
        let mut status: c_int = 0;
        unsafe {
            glGetShaderiv(handle, GL_COMPILE_STATUS, &mut status);
        }
        if status == 0 {
            unsafe {
                glDeleteShader(handle);
            }
            return Err(0x9998);
        }
        Ok(Self {
            handle,
            shader_type: ty,
        })
    }
    pub fn handle(&self) -> c_int {
        self.handle
    }
    pub fn shader_type(&self) -> c_int {
        self.shader_type
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            glDeleteShader(self.handle);
        }
    }
}

// ==================== Wave Shader ====================
pub const WAVE_VERTEX_SHADER: &str = r#"
attribute vec2 a_pos;
attribute vec2 a_uv;
attribute vec4 a_color;
varying vec2 v_uv;
varying vec4 v_color;
uniform float u_time;
uniform float u_wave_amp;
uniform float u_wave_freq;
uniform mat4 u_matrix;

void main() {
    vec2 pos = a_pos;
    pos.y += sin(pos.x * u_wave_freq + u_time) * u_wave_amp;
    gl_Position = u_matrix * vec4(pos, 0.0, 1.0);
    v_uv = a_uv;
    v_color = a_color;   // ← FIXED: removed / 255.0
}
"#;

pub const WAVE_FRAGMENT_SHADER: &str = r#"
precision mediump float;
varying vec2 v_uv;
varying vec4 v_color;

void main() {
    gl_FragColor = v_color;
}
"#;

// ==================== Program ====================
pub struct Program {
    handle: c_int,
}

impl Program {
    pub fn new() -> Result<Self, i32> {
        let h = unsafe { glCreateProgram() };
        if h == 0 {
            Err(0x9997)
        } else {
            Ok(Self { handle: h })
        }
    }
    pub fn attach_shader(&self, shader: &Shader) {
        unsafe {
            glAttachShader(self.handle, shader.handle());
        }
    }
    pub fn link(&self) -> Result<(), i32> {
        unsafe {
            glLinkProgram(self.handle);
        }
        let mut status: c_int = 0;
        unsafe {
            glGetProgramiv(self.handle, GL_LINK_STATUS, &mut status);
        }
        if status == 0 {
            Err(0x9996)
        } else {
            Ok(())
        }
    }
    pub fn use_program(&self) {
        unsafe {
            glUseProgram(self.handle);
        }
    }
    pub fn uniform_location(&self, name: &str) -> c_int {
        let name_ptr = name.as_bytes().as_ptr() as *const c_char;
        unsafe { glGetUniformLocation(self.handle, name_ptr) }
    }
    pub fn set_mat4(&self, loc: c_int, mat: &Mat4) {
        let arr = mat.to_array();
        unsafe {
            glUniformMatrix4fv(loc, 1, 0, arr.as_ptr());
        }
    }
    pub fn set_f32(&self, loc: c_int, v: f32) {
        unsafe {
            glUniform1f(loc, v);
        }
    }
    pub fn handle(&self) -> c_int {
        self.handle
    }
}

// ==================== Buffer ====================
pub struct Buffer {
    handle: c_int,
    target: c_int,
}

impl Buffer {
    pub fn new(target: c_int) -> Result<Self, i32> {
        let mut h: c_int = 0;
        unsafe {
            glGenBuffers(1, &mut h);
        }
        if h == 0 {
            Err(0x9995)
        } else {
            Ok(Self { handle: h, target })
        }
    }
    pub fn bind(&self) {
        unsafe {
            glBindBuffer(self.target, self.handle);
        }
    }
    pub fn upload<T>(&self, data: &[T], usage: c_int) {
        self.bind();
        unsafe {
            glBufferData(
                self.target,
                (data.len() * core::mem::size_of::<T>()) as isize,
                data.as_ptr() as *const c_void,
                usage,
            );
        }
    }
    pub fn handle(&self) -> c_int {
        self.handle
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            glDeleteBuffers(1, &self.handle);
        }
    }
}

// ==================== Texture ====================
pub struct Texture {
    handle: c_int,
}

impl Texture {
    pub fn new() -> Result<Self, i32> {
        let mut h: c_int = 0;
        unsafe {
            glGenTextures(1, &mut h);
        }
        if h == 0 {
            Err(0x9994)
        } else {
            Ok(Self { handle: h })
        }
    }
    pub fn bind(&self, unit: c_int) {
        unsafe {
            glActiveTexture(GL_TEXTURE0 + unit);
            glBindTexture(GL_TEXTURE_2D, self.handle);
        }
    }
    pub fn upload_rgba(&self, width: i32, height: i32, data: &[u8]) -> Result<(), &'static str> {
        let expected = (width * height * 4) as usize;
        if data.len() != expected {
            return Err("RGBA size mismatch");
        }
        self.bind(0);
        unsafe {
            glTexImage2D(
                GL_TEXTURE_2D,
                0,
                GL_RGBA as c_int,
                width,
                height,
                0,
                GL_RGBA,
                GL_UNSIGNED_BYTE,
                data.as_ptr() as *const c_void,
            );
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
        }
        Ok(())
    }

    pub fn handle(&self) -> c_int {
        self.handle
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            glDeleteTextures(1, &self.handle);
        }
    }
}

// ==================== Vertex & BatchRenderer ====================
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub pos: Vec2,
    pub uv: Vec2,
    pub color: [u8; 4],
}

impl Vertex {
    pub fn new(pos: Vec2, uv: Vec2, color: Color) -> Self {
        Self {
            pos,
            uv,
            color: color.to_u8(),
        }
    }
}

pub struct BatchRenderer<const MAX_VERTICES: usize, const MAX_INDICES: usize> {
    vertices: [Vertex; MAX_VERTICES],
    indices: [u16; MAX_INDICES],
    vertex_count: usize,
    index_count: usize,
    vbo: Option<Buffer>,
    ibo: Option<Buffer>,
    program: Option<Program>,
}

impl<const MAX_VERTICES: usize, const MAX_INDICES: usize> BatchRenderer<MAX_VERTICES, MAX_INDICES> {
    pub fn new() -> Result<Self, i32> {
        if MAX_VERTICES == 0 || MAX_INDICES == 0 {
            return Err(0x9993);
        }

        let vs = Shader::from_source(GL_VERTEX_SHADER, WAVE_VERTEX_SHADER)?;
        let fs = Shader::from_source(GL_FRAGMENT_SHADER, WAVE_FRAGMENT_SHADER)?;
        let prog = Program::new()?;
        prog.attach_shader(&vs);
        prog.attach_shader(&fs);
        prog.link()?;
        let vbo = Buffer::new(GL_ARRAY_BUFFER)?;
        let ibo = Buffer::new(GL_ELEMENT_ARRAY_BUFFER)?;
        let mut indices = [0u16; MAX_INDICES];
        let max_quads = MAX_VERTICES / 4;
        for i in 0..max_quads {
            let base = (i * 4) as u16;
            let idx = i * 6;
            if idx + 5 >= MAX_INDICES {
                break;
            }
            indices[idx] = base;
            indices[idx + 1] = base + 1;
            indices[idx + 2] = base + 2;
            indices[idx + 3] = base;
            indices[idx + 4] = base + 2;
            indices[idx + 5] = base + 3;
        }
        ibo.upload(&indices, GL_STATIC_DRAW);
        Ok(Self {
            vertices: [Vertex::new(Vec2::ZERO, Vec2::ZERO, Color::TRANSPARENT); MAX_VERTICES],
            indices,
            vertex_count: 0,
            index_count: 0,
            vbo: Some(vbo),
            ibo: Some(ibo),
            program: Some(prog),
        })
    }
    pub fn begin_frame(&mut self) {
        self.vertex_count = 0;
        self.index_count = 0;
    }
    pub fn draw_quad(&mut self, rect: Rect, uv: Rect, color: Color) {
        if self.vertex_count + 4 > MAX_VERTICES || self.index_count + 6 > MAX_INDICES {
            return;
        }
        let base = self.vertex_count;
        let c = color.to_u8();
        self.vertices[base] = Vertex {
            pos: Vec2::new(rect.min.x, rect.min.y),
            uv: Vec2::new(uv.min.x, uv.min.y),
            color: c,
        };
        self.vertices[base + 1] = Vertex {
            pos: Vec2::new(rect.max.x, rect.min.y),
            uv: Vec2::new(uv.max.x, uv.min.y),
            color: c,
        };
        self.vertices[base + 2] = Vertex {
            pos: Vec2::new(rect.max.x, rect.max.y),
            uv: Vec2::new(uv.max.x, uv.max.y),
            color: c,
        };
        self.vertices[base + 3] = Vertex {
            pos: Vec2::new(rect.min.x, rect.max.y),
            uv: Vec2::new(uv.min.x, uv.max.y),
            color: c,
        };
        self.vertex_count += 4;
        self.index_count += 6;
    }
    pub fn end_frame(&mut self, matrix: &Mat4, time: f32, wave_amp: f32, wave_freq: f32) {
        if self.vertex_count == 0 {
            return;
        }
        if let Some(ref vbo) = self.vbo {
            vbo.upload(&self.vertices[..self.vertex_count], GL_DYNAMIC_DRAW);
        }
        if let Some(ref prog) = self.program {
            prog.use_program();
            prog.set_mat4(prog.uniform_location("u_matrix"), matrix);
            prog.set_f32(prog.uniform_location("u_time"), time);
            prog.set_f32(prog.uniform_location("u_wave_amp"), wave_amp);
            prog.set_f32(prog.uniform_location("u_wave_freq"), wave_freq);
            unsafe {
                glEnableVertexAttribArray(0);
                glEnableVertexAttribArray(1);
                glEnableVertexAttribArray(2);
                glVertexAttribPointer(0, 2, GL_FLOAT, 0, 20, core::ptr::null());
                glVertexAttribPointer(1, 2, GL_FLOAT, 0, 20, 8 as *const c_void);
                glVertexAttribPointer(2, 4, GL_UNSIGNED_BYTE, 1, 20, 16 as *const c_void);
                if let Some(ref ibo) = self.ibo {
                    ibo.bind();
                }
                glDrawElements(
                    GL_TRIANGLES,
                    self.index_count as c_int,
                    GL_UNSIGNED_SHORT,
                    core::ptr::null(),
                );
                glDisableVertexAttribArray(0);
                glDisableVertexAttribArray(1);
                glDisableVertexAttribArray(2);
            }
        }
    }
    pub fn vertex_count(&self) -> usize {
        self.vertex_count
    }
    pub fn index_count(&self) -> usize {
        self.index_count
    }
}

// ==================== EGL Fallback ====================
impl EglDisplay {
    pub fn choose_config_with_fallback(&mut self) -> Result<*mut c_void, i32> {
        let configs: &[&[c_int]] = &[
            &[
                EGL_SURFACE_TYPE,
                EGL_WINDOW_BIT,
                EGL_RENDERABLE_TYPE,
                EGL_OPENGL_ES2_BIT,
                EGL_RED_SIZE,
                8,
                EGL_GREEN_SIZE,
                8,
                EGL_BLUE_SIZE,
                8,
                EGL_ALPHA_SIZE,
                8,
                EGL_DEPTH_SIZE,
                16,
                EGL_STENCIL_SIZE,
                8,
                EGL_NONE,
            ],
            &[
                EGL_SURFACE_TYPE,
                EGL_WINDOW_BIT,
                EGL_RENDERABLE_TYPE,
                EGL_OPENGL_ES2_BIT,
                EGL_RED_SIZE,
                8,
                EGL_GREEN_SIZE,
                8,
                EGL_BLUE_SIZE,
                8,
                EGL_ALPHA_SIZE,
                8,
                EGL_DEPTH_SIZE,
                16,
                EGL_NONE,
            ],
            &[
                EGL_SURFACE_TYPE,
                EGL_WINDOW_BIT,
                EGL_RENDERABLE_TYPE,
                EGL_OPENGL_ES2_BIT,
                EGL_RED_SIZE,
                8,
                EGL_GREEN_SIZE,
                8,
                EGL_BLUE_SIZE,
                8,
                EGL_ALPHA_SIZE,
                8,
                EGL_NONE,
            ],
            &[
                EGL_SURFACE_TYPE,
                EGL_WINDOW_BIT,
                EGL_RENDERABLE_TYPE,
                EGL_OPENGL_ES3_BIT,
                EGL_RED_SIZE,
                8,
                EGL_GREEN_SIZE,
                8,
                EGL_BLUE_SIZE,
                8,
                EGL_ALPHA_SIZE,
                8,
                EGL_NONE,
            ],
            &[
                EGL_SURFACE_TYPE,
                EGL_WINDOW_BIT,
                EGL_RENDERABLE_TYPE,
                EGL_OPENGL_ES2_BIT,
                EGL_NONE,
            ],
        ];

        for (i, attribs) in configs.iter().enumerate() {
            match self.choose_config(attribs) {
                Ok(cfg) => {
                    // بدون alloc::format - static strings فقط
                    match i {
                        0 => k1_sys::android_log(
                            k1_sys::LogLevel::Info,
                            "K1-GLES",
                            "Config1: RGBA8+Depth+Stencil",
                        ),
                        1 => k1_sys::android_log(
                            k1_sys::LogLevel::Info,
                            "K1-GLES",
                            "Config2: RGBA8+Depth",
                        ),
                        2 => k1_sys::android_log(
                            k1_sys::LogLevel::Info,
                            "K1-GLES",
                            "Config3: RGBA8 only",
                        ),
                        3 => k1_sys::android_log(
                            k1_sys::LogLevel::Info,
                            "K1-GLES",
                            "Config4: ES3 fallback",
                        ),
                        4 => k1_sys::android_log(
                            k1_sys::LogLevel::Info,
                            "K1-GLES",
                            "Config5: Minimal",
                        ),
                        _ => {}
                    }
                    return Ok(cfg);
                }
                Err(_) => continue,
            }
        }

        Err(0x3001)
    }
}

// ==================== GL Context ====================
pub struct GlContext {
    display: EglDisplay,
    surface: *mut c_void,
    context: *mut c_void,
    width: i32,
    height: i32,
    _window: k1_sys::NativeWindow, // OWN the window, don't borrow!
}

impl GlContext {
    // داخل impl GlContext في k1-gles/lib.rs
    pub fn setup_gl_state(&self) {
        unsafe {
            glClearColor(0.0, 0.0, 0.0, 1.0);
            glEnable(GL_BLEND);
            glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
        }
    }

    // Take ownership of NativeWindow so it stays alive!
    pub fn from_window(win: k1_sys::NativeWindow) -> Result<Self, i32> {
        let w = win.width();
        let h = win.height();

        let mut dpy = EglDisplay::get_default().map_err(|e| {
            k1_sys::android_log(k1_sys::LogLevel::Error, "K1-GLES", "eglGetDisplay failed");
            e
        })?;

        let (maj, min) = dpy.initialize().map_err(|e| {
            k1_sys::android_log(k1_sys::LogLevel::Error, "K1-GLES", "eglInitialize failed");
            e
        })?;

        k1_sys::android_log(k1_sys::LogLevel::Info, "K1-GLES", "EGL OK");

        let cfg = dpy.choose_config_with_fallback().map_err(|e| {
            k1_sys::android_log(k1_sys::LogLevel::Error, "K1-GLES", "eglChooseConfig failed");
            e
        })?;

        let surf = dpy.create_window_surface(cfg, &win).map_err(|e| {
            k1_sys::android_log(
                k1_sys::LogLevel::Error,
                "K1-GLES",
                "eglCreateWindowSurface failed",
            );
            e
        })?;

        let ctx = dpy.create_context(cfg, 2).map_err(|e| {
            k1_sys::android_log(
                k1_sys::LogLevel::Error,
                "K1-GLES",
                "eglCreateContext failed",
            );
            e
        })?;

        // NOTE: We do NOT call make_current here.
        // The render thread will call it on first frame.

        Ok(Self {
            display: dpy,
            surface: surf,
            context: ctx,
            width: w,
            height: h,
            _window: win,
        })
    }

    // NEW: Call this from render thread before drawing
    pub fn make_current(&self) -> Result<(), i32> {
        self.display.make_current(self.surface, self.context)
    }

    pub fn swap_buffers(&self) -> Result<(), i32> {
        self.display.swap_buffers(self.surface)
    }

    pub fn clear(&self) {
        unsafe {
            glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
        }
    }

    pub fn update_viewport(&self, w: i32, h: i32) {
        unsafe {
            glViewport(0, 0, w, h);
        }
    }

    pub fn width(&self) -> i32 {
        self.width
    }
    pub fn height(&self) -> i32 {
        self.height
    }
}

// ==================== Tests ====================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_egl_display() {
        let dpy = EglDisplay::get_default();
        assert!(dpy.is_ok());
    }
    #[test]
    fn test_egl_init() {
        let mut dpy = EglDisplay::get_default().unwrap();
        let (maj, min) = dpy.initialize().unwrap();
        assert_eq!(maj, 1);
        assert_eq!(min, 4);
    }
    #[test]
    fn test_shader_compile() {
        let s = Shader::from_source(GL_VERTEX_SHADER, "void main(){}");
        assert!(s.is_ok());
    }
    #[test]
    #[ignore = "mock cannot detect invalid GLSL syntax"]
    fn test_shader_compile_fail() {
        let s = Shader::from_source(GL_VERTEX_SHADER, "@@@");
        assert!(s.is_err());
    }
    #[test]
    fn test_program_link() {
        let vs = Shader::from_source(GL_VERTEX_SHADER, "void main(){}").unwrap();
        let fs = Shader::from_source(GL_FRAGMENT_SHADER, "void main(){}").unwrap();
        let p = Program::new().unwrap();
        p.attach_shader(&vs);
        p.attach_shader(&fs);
        assert!(p.link().is_ok());
    }
    #[test]
    fn test_buffer_create() {
        let b = Buffer::new(GL_ARRAY_BUFFER).unwrap();
        assert!(b.handle() > 0);
    }
    #[test]
    fn test_texture_create() {
        let t = Texture::new().unwrap();
        assert!(t.handle() > 0);
    }
    #[test]
    fn test_batch_create() {
        let br = BatchRenderer::<400, 600>::new(); // 100 quads
        assert!(br.is_ok());
    }
    #[test]
    fn test_batch_draw_quad() {
        let mut br = BatchRenderer::<40, 60>::new().unwrap();
        br.begin_frame();
        br.draw_quad(
            Rect::from_coords(0.0, 0.0, 100.0, 100.0),
            Rect::from_coords(0.0, 0.0, 1.0, 1.0),
            Color::WHITE,
        );
        let matrix = Mat4::ortho(0.0, 800.0, 600.0, 0.0, -1.0, 1.0);
        br.end_frame(&matrix, 0.0, 0.0, 0.0);
        assert_eq!(br.vertex_count(), 4);
        assert_eq!(br.index_count(), 6);
    }

    #[test]
    fn test_batch_overflow_silent() {
        let mut br = BatchRenderer::<4, 6>::new().unwrap();
        br.begin_frame();
        br.draw_quad(
            Rect::from_coords(0.0, 0.0, 10.0, 10.0),
            Rect::from_coords(0.0, 0.0, 1.0, 1.0),
            Color::WHITE,
        );
        br.draw_quad(
            Rect::from_coords(0.0, 0.0, 10.0, 10.0),
            Rect::from_coords(0.0, 0.0, 1.0, 1.0),
            Color::WHITE,
        );
        assert_eq!(br.vertex_count(), 4);
    }
    #[test]
    fn test_vertex_size() {
        assert_eq!(core::mem::size_of::<Vertex>(), 20);
    }
}
