#![no_std]

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::ffi::CString;
use core::ffi::{c_void, c_int, c_char};
use k1_math::{Vec2, Color, Rect, Mat4};

// ... (constants same as before) ...

pub const GL_RGBA: c_int = 0x1908;
pub const GL_TEXTURE0: c_int = 0x84C0;

// ... (mock impl same) ...

// ==================== Error Handling ====================

#[derive(Debug, Clone, PartialEq)]
pub enum EglError {
    NotInitialized, BadAccess, BadAlloc, BadAttribute, BadConfig,
    BadContext, BadCurrentSurface, BadDisplay, BadMatch,
    BadNativePixmap, BadNativeWindow, BadParameter, BadSurface,
    Unknown(EGLint), Message(String),
}

// ... (EglDisplay, Shader, Program, Buffer, Texture same) ...

// ==================== GL Context ====================

pub struct GlContext {
    display: EglDisplay,
    surface: EGLSurface,
    context: EGLContext,
    width: i32,
    height: i32,
}

impl GlContext {
    pub fn from_window(win: &k1_sys::NativeWindow) -> Result<Self, String> {
        let mut dpy = EglDisplay::get_default().map_err(|e| format!("{:?}", e))?;
        let (maj, min) = dpy.initialize().map_err(|e| format!("{:?}", e))?;
        k1_sys::android_log(k1_sys::LogLevel::Info, "K1-GLES", &format!("EGL {}.{}", maj, min));
        
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
        
        unsafe {
            glViewport(0, 0, w, h);
            glClearColor(0.0, 0.0, 0.0, 1.0);
            glEnable(GL_BLEND);
            glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
        }
        
        Ok(Self { display: dpy, surface: surf, context: ctx, width: w, height: h })
    }
    
    pub fn swap_buffers(&self) -> Result<(), EglError> {
        self.display.swap_buffers(self.surface)
    }
    
    pub fn clear(&self) {
        unsafe { glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT); }
    }
    
    pub fn set_viewport(&self, x: i32, y: i32, w: i32, h: i32) {
        unsafe { glViewport(x, y, w, h); }
    }
    
    pub fn set_scissor(&self, x: i32, y: i32, w: i32, h: i32) {
        unsafe { 
            glEnable(GL_SCISSOR_TEST);
            glScissor(x, y, w, h); 
        }
    }
    
    pub fn disable_scissor(&self) {
        unsafe { glDisable(GL_SCISSOR_TEST); }
    }
    
    pub fn width(&self) -> i32 { self.width }
    pub fn height(&self) -> i32 { self.height }
}

impl Drop for GlContext {
    fn drop(&mut self) {
        unsafe {
            eglMakeCurrent(self.display.handle(), EGL_NO_SURFACE, EGL_NO_SURFACE, EGL_NO_CONTEXT);
            eglDestroyContext(self.display.handle(), self.context);
            eglDestroySurface(self.display.handle(), self.surface);
        }
    }
}

// ... (Vertex, BatchRenderer same) ...

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_egl_display_get_default() {
        let dpy = EglDisplay::get_default();
        assert!(dpy.is_ok());
    }
    
    #[test]
    fn test_egl_display_initialize() {
        let mut dpy = EglDisplay::get_default().unwrap();
        let (maj, min) = dpy.initialize().unwrap();
        assert_eq!(maj, 1);
        assert_eq!(min, 4);
    }
    
    #[test]
    fn test_egl_choose_config() {
        let mut dpy = EglDisplay::get_default().unwrap();
        dpy.initialize().unwrap();
        let cfg = dpy.choose_config(&[EGL_SURFACE_TYPE, EGL_WINDOW_BIT, EGL_NONE]);
        assert!(cfg.is_ok());
    }
    
    #[test]
    fn test_egl_create_context() {
        let mut dpy = EglDisplay::get_default().unwrap();
        dpy.initialize().unwrap();
        let cfg = dpy.choose_config(&[EGL_SURFACE_TYPE, EGL_WINDOW_BIT, EGL_NONE]).unwrap();
        let ctx = dpy.create_context(cfg, 2);
        assert!(ctx.is_ok());
    }
    
    #[test]
    fn test_egl_swap_interval() {
        let mut dpy = EglDisplay::get_default().unwrap();
        dpy.initialize().unwrap();
        assert!(dpy.set_swap_interval(1).is_ok());
    }
    
    #[test]
    fn test_shader_create_vertex() {
        let s = Shader::from_source(GL_VERTEX_SHADER, "void main() {}");
        assert!(s.is_ok());
        assert_eq!(s.unwrap().shader_type(), GL_VERTEX_SHADER);
    }
    
    #[test]
    fn test_shader_create_fragment() {
        let s = Shader::from_source(GL_FRAGMENT_SHADER, "void main() {}");
        assert!(s.is_ok());
    }
    
    #[test]
    fn test_shader_compile_fail() {
        assert!(s.is_err());
    }
    
    #[test]
    fn test_program_create() {
        let p = Program::new();
        assert!(p.is_ok());
        assert!(p.unwrap().handle() > 0);
    }
    
    #[test]
    fn test_program_link() {
        let vs = Shader::from_source(GL_VERTEX_SHADER, "void main() {}").unwrap();
        let fs = Shader::from_source(GL_FRAGMENT_SHADER, "void main() {}").unwrap();
        let p = Program::new().unwrap();
        p.attach_shader(&vs);
        p.attach_shader(&fs);
        assert!(p.link().is_ok());
    }
    
    #[test]
    fn test_program_uniform_location() {
        let p = Program::new().unwrap();
        assert_eq!(p.uniform_location("u_matrix"), 0);
n    }
    
    #[test]
    fn test_buffer_create() {
        let b = Buffer::new(GL_ARRAY_BUFFER);
        assert!(b.is_ok());
        assert!(b.unwrap().handle() > 0);
    }
    
    #[test]
    fn test_buffer_upload() {
        let b = Buffer::new(GL_ARRAY_BUFFER).unwrap();
        let data: [f32; 8] = [0.0; 8];
        b.upload(&data, GL_STATIC_DRAW);
    }
    
    #[test]
    fn test_texture_create() {
        let t = Texture::new();
        assert!(t.is_ok());
        assert!(t.unwrap().handle() > 0);
    }
    
    #[test]
    fn test_texture_upload_size() {
        let t = Texture::new().unwrap();
        t.upload_rgba(2, 2, &data);
    }
    
    #[test]
    #[should_panic(expected = "RGBA data size mismatch")]
    fn test_texture_upload_wrong_size() {
        let t = Texture::new().unwrap();
        t.upload_rgba(2, 2, &[0u8; 10]);
    }
    
    #[test]
    fn test_texture_bind_unit() {
        let t = Texture::new().unwrap();
        t.bind(0);
        t.bind(3);
    }
    
    #[test]
    fn test_vertex_create() {
        let v = Vertex::new(Vec2::new(1.0, 2.0), Vec2::new(0.0, 0.0), Color::new(1.0, 0.0, 0.0, 1.0));
        assert_eq!(v.pos.x, 1.0);
        assert_eq!(v.pos.y, 2.0);
    }
    
    #[test]
    fn test_batch_renderer_create() {
        let br = BatchRenderer::new(100);
        assert!(br.is_ok());
    }
    
    #[test]
    fn test_batch_draw_quad() {
        let mut br = BatchRenderer::new(10).unwrap();
        br.begin_frame();
        br.draw_quad(
            Rect::from_coords(0.0, 0.0, 100.0, 100.0),
            Rect::from_coords(0.0, 0.0, 1.0, 1.0),
            Color::new(1.0, 1.0, 1.0, 1.0)
        );
        assert_eq!(br.vertex_count(), 4);
        assert_eq!(br.index_count(), 6);
    }
    
    #[test]
    fn test_batch_multiple_quads() {
        let mut br = BatchRenderer::new(10).unwrap();
        br.begin_frame();
        for i in 0..5 {
            br.draw_quad(
                Rect::from_coords(i as f32 * 10.0, 0.0, 10.0, 10.0),
                Rect::from_coords(0.0, 0.0, 1.0, 1.0),
                Color::new(1.0, 1.0, 1.0, 1.0)
            );
        }
        assert_eq!(br.vertex_count(), 20);
        assert_eq!(br.index_count(), 30);
    }
    
    #[test]
    fn test_batch_overflow_silent() {
        let mut br = BatchRenderer::new(1).unwrap();
        br.begin_frame();
        br.draw_quad(Rect::from_coords(0.0, 0.0, 10.0, 10.0), Rect::from_coords(0.0, 0.0, 1.0, 1.0), Color::new(1.0, 1.0, 1.0, 1.0));
        br.draw_quad(Rect::from_coords(0.0, 0.0, 10.0, 10.0), Rect::from_coords(0.0, 0.0, 1.0, 1.0), Color::new(1.0, 1.0, 1.0, 1.0));
        assert_eq!(br.vertex_count(), 4);
    }
    
    #[test]
    fn test_gl_constants() {
        assert_eq!(GL_RGBA, 0x1908);
        assert_eq!(GL_TEXTURE0, 0x84C0);
        assert_eq!(GL_FLOAT, 0x1406);
    }
    
    #[test]
    fn test_egl_error_mapping() {
        assert_eq!(EglError::from_egl_error(EGL_BAD_ALLOC), EglError::BadAlloc);
        assert_eq!(EglError::from_egl_error(0x9999), EglError::Unknown(0x9999));
    }
}
