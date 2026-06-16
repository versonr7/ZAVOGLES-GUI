use std::process::Command;
use std::env;

fn main() {
    println!("cargo:rerun-if-changed=shaders/");
    
    // Compile shaders to SPIR-V
    let shaders = &[
        ("shaders/ui.vert", "shaders/ui.vert.spv"),
        ("shaders/ui.frag", "shaders/ui.frag.spv"),
        ("shaders/wave.vert", "shaders/wave.vert.spv"),
        ("shaders/wave.frag", "shaders/wave.frag.spv"),
    ];
    
    for (src, dst) in shaders {
        let status = Command::new("glslangValidator")
            .args(&["-V", src, "-o", dst])
            .status();
        
        match status {
            Ok(s) if s.success() => {
                println!("cargo:warning=Compiled shader: {}", src);
            }
            _ => {
                println!("cargo:warning=Failed to compile shader: {} (glslangValidator not found?)", src);
            }
        }
    }
    
    // TODO: Generate Arabic glyph atlas
    // Run harfbuzz on host to generate ARABIC_GLYPHS array
    // This requires Python + harfbuzz on build machine
    
    println!("cargo:rerun-if-changed=assets/");
}
