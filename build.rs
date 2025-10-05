use std::process::Command;
use std::path::Path;

fn main() {
    let luwen_lib_path = "./luwen/target/release";
    
    let lib_name = if cfg!(target_os = "linux") {
        "libluwencpp.so"
    } else if cfg!(target_os = "macos") {
        "libluwencpp.dylib"
    } else {
        panic!("Unsupported OS");
    };
    
    let lib_full_path = format!("{}/{}", luwen_lib_path, lib_name);
    
    if !Path::new(&lib_full_path).exists() {
        println!("cargo:warning=Building luwencpp library...");
        
        let output = Command::new("cargo")
            .args(&["build", "--release", "-p", "luwencpp"])
            .current_dir("./luwen")
            .output()
            .expect("Failed to build luwencpp");
        
        if !output.status.success() {
            panic!("Failed to build luwencpp: {}", 
                   String::from_utf8_lossy(&output.stderr));
        }
        
        println!("cargo:warning=Successfully built luwencpp library");
    }

    println!("cargo:rustc-link-search=native={luwen_lib_path}");
    println!("cargo:rustc-link-lib=dylib=luwencpp");
    println!("cargo:warning=Using locally built luwencpp library from {luwen_lib_path}");

    println!("cargo:rerun-if-changed=build.rs");

    #[cfg(target_os = "linux")]
    println!("cargo:rerun-if-changed=luwen/target/release/libluwencpp.so");

    #[cfg(target_os = "macos")]
    println!("cargo:rerun-if-changed=luwen/target/release/libluwencpp.dylib");
}