fn main() {
    // Automatically enable alias feature on supported platforms
    #[cfg(any(unix, windows))]
    println!("cargo:rustc-cfg=feature=\"alias\"");
}
