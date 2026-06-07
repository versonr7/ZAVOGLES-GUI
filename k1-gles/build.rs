fn main() {
    #[cfg(all(target_os = "android", not(feature = "mock"), not(test)))]
    {
        println!("cargo:rustc-link-lib=GLESv2");
        println!("cargo:rustc-link-lib=EGL");
    }
}
