fn main() {
    if let Ok(target_os) = std::env::var("CARGO_CFG_TARGET_OS") {
        if target_os == "android" {
            println!("cargo:rustc-link-lib=GLESv2");
            println!("cargo:rustc-link-lib=EGL");
        }
    }
}
