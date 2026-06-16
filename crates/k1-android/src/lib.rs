//! K1-Android: Android entry point
//! NativeActivity with Vulkan rendering

use android_activity::{AndroidApp, InputStatus, MainEvent, PollEvent};
use android_logger::Config;
use log::LevelFilter;
use k1_ui::xmb::XMBMenu;
use std::sync::Arc;

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn android_main(app: AndroidApp) {
    android_logger::init_once(
        Config::default()
            .with_max_level(LevelFilter::Debug),
    );
    
    log::info!("K1-APP: Starting K1 Platform");
    
    let mut renderer = None;
    let mut xmb_menu = XMBMenu::new();
    let mut running = true;
    
    // Initialize XMB menu
    init_xmb_menu(&mut xmb_menu);
    
    while running {
        app.poll_events(Some(std::time::Duration::from_millis(16)), |event| {
            match event {
                PollEvent::Main(main_event) => {
                    match main_event {
                        MainEvent::InitWindow { .. } => {
                            log::info!("K1-APP: Window created");
                            
                            if let Some(window) = app.native_window() {
                                let config = k1_vulkan::RendererConfig::new(
                                    window.width() as u32,
                                    window.height() as u32,
                                );
                                
                                match k1_vulkan::VulkanRenderer::new(
                                    window.ptr() as *mut _,
                                    config,
                                ) {
                                    Ok(r) => {
                                        log::info!("K1-GLES: Vulkan initialized");
                                        renderer = Some(r);
                                    }
                                    Err(e) => {
                                        log::error!("K1-APP: Vulkan init failed: {:?}", e);
                                    }
                                }
                            }
                        }
                        
                        MainEvent::TermWindow { .. } => {
                            log::info!("K1-APP: Window destroyed");
                            renderer = None;
                        }
                        
                        MainEvent::InputAvailable { .. } => {
                            handle_input(&app, &mut xmb_menu);
                        }
                        
                        MainEvent::Pause { .. } => {
                            log::info!("K1-APP: Paused");
                        }
                        
                        MainEvent::Resume { .. } => {
                            log::info!("K1-APP: Resumed");
                        }
                        
                        MainEvent::Destroy => {
                            log::info!("K1-APP: Destroying");
                            running = false;
                        }
                        
                        _ => {}
                    }
                }
                
                PollEvent::Input(input_event) => {
                    handle_input_event(input_event, &mut xmb_menu);
                }
                
                _ => {}
            }
            
            InputStatus::Unhandled
        });
        
        // Render frame
        if let Some(ref mut r) = renderer {
            match r.begin_frame() {
                Ok(mut ctx) => {
                    xmb_menu.render(
                        1920.0, 1080.0, // TODO: Get actual screen size
                        ctx.vertices,
                        ctx.vertex_count,
                    );
                    
                    if let Err(e) = r.end_frame() {
                        log::error!("K1-APP: Render error: {:?}", e);
                    }
                }
                Err(e) => {
                    log::error!("K1-APP: Begin frame error: {:?}", e);
                }
            }
        }
    }
}

fn init_xmb_menu(menu: &mut XMBMenu) {
    // Games column
    if let Ok(col) = menu.add_column("Games", 0) {
        col.add_item("PS1 Emulator", 1, "com.epsxe.ePSXe").ok();
        col.add_item("PS2 Emulator", 2, "com.damonps2.damonps2").ok();
        col.add_item("N64 Emulator", 3, "org.mupen64plusae.v3.fzurita").ok();
        col.add_item("SNES Emulator", 4, "com.explusalpha.Snes9xPlus").ok();
    }
    
    // Media column
    if let Ok(col) = menu.add_column("Media", 5) {
        col.add_item("Music", 6, "music").ok();
        col.add_item("Videos", 7, "videos").ok();
        col.add_item("Photos", 8, "photos").ok();
    }
    
    // Settings column
    if let Ok(col) = menu.add_column("Settings", 9) {
        col.add_item("Display", 10, "display").ok();
        col.add_item("Sound", 11, "sound").ok();
        col.add_item("Controls", 12, "controls").ok();
    }
    
    log::info!("K1-APP: XMB menu initialized with {} columns", menu.column_count);
}

fn handle_input(app: &AndroidApp, menu: &mut XMBMenu) {
    // Handle gamepad/touch input
}

fn handle_input_event(event: android_activity::input::InputEvent, menu: &mut XMBMenu) {
    use android_activity::input::{KeyAction, KeyCode, KeyEvent, MotionAction, MotionEvent};
    
    match event {
        android_activity::input::InputEvent::KeyEvent(key) => {
            if key.action == KeyAction::Down {
                match key.key_code {
                    KeyCode::DpadRight => menu.navigate_right(),
                    KeyCode::DpadLeft => menu.navigate_left(),
                    KeyCode::DpadDown => menu.navigate_down(),
                    KeyCode::DpadUp => menu.navigate_up(),
                    KeyCode::ButtonA | KeyCode::Enter => {
                        log::info!("K1-APP: Selected item");
                        // Launch emulator or app
                    }
                    KeyCode::ButtonB | KeyCode::Escape => {
                        log::info!("K1-APP: Back pressed");
                    }
                    _ => {}
                }
            }
        }
        
        android_activity::input::InputEvent::MotionEvent(motion) => {
            // Touch input handling
            if motion.action == MotionAction::Down {
                // Convert touch to navigation
            }
        }
        
        _ => {}
    }
}
