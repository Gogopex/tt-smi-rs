fn main() {
    let luwen_lib_path = "./luwen/target/release";

    println!("cargo:rustc-link-search=native={luwen_lib_path}");
    println!("cargo:rustc-link-lib=dylib=luwencpp");
    println!("cargo:warning=Using locally built luwencpp library from {luwen_lib_path}");

    println!("cargo:rerun-if-changed=build.rs");

    #[cfg(target_os = "linux")]
    println!("cargo:rerun-if-changed=luwen/target/release/libluwencpp.so");

    #[cfg(target_os = "macos")]
    println!("cargo:rerun-if-changed=luwen/target/release/libluwencpp.dylib");
}
