//! Android ANativeActivity bindings

use std::ffi::{c_void, c_int};

#[repr(C)]
pub struct ANativeActivityCallbacks {
    pub on_start: Option<extern "C" fn(*mut c_void)>,
    pub on_resume: Option<extern "C" fn(*mut c_void)>,
    pub on_save_instance_state: Option<extern "C" fn(*mut c_void, *mut usize) -> *mut c_void>,
    pub on_pause: Option<extern "C" fn(*mut c_void)>,
    pub on_stop: Option<extern "C" fn(*mut c_void)>,
    pub on_destroy: Option<extern "C" fn(*mut c_void)>,
    pub on_window_focus_changed: Option<extern "C" fn(*mut c_void, c_int)>,
    pub on_native_window_created: Option<extern "C" fn(*mut c_void, *mut c_void)>,
    pub on_native_window_resized: Option<extern "C" fn(*mut c_void, *mut c_void)>,
    pub on_native_window_redraw_needed: Option<extern "C" fn(*mut c_void, *mut c_void)>,
    pub on_native_window_destroyed: Option<extern "C" fn(*mut c_void, *mut c_void)>,
    pub on_input_queue_created: Option<extern "C" fn(*mut c_void, *mut c_void)>,
    pub on_input_queue_destroyed: Option<extern "C" fn(*mut c_void, *mut c_void)>,
    pub on_content_rect_changed: Option<extern "C" fn(*mut c_void, *const c_void)>,
    pub on_configuration_changed: Option<extern "C" fn(*mut c_void)>,
    pub on_low_memory: Option<extern "C" fn(*mut c_void)>,
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