fn main() {
    println!("cargo::rerun-if-changed=build.rs");

    // Declare known cfg values to prevent warnings about unknown cfgs
    println!("cargo::rustc-check-cfg=cfg(windows_dll)");
}
