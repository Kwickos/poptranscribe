fn main() {
    // Add Swift runtime rpath for ScreenCaptureKit dependency
    println!("cargo:rustc-link-arg=-Wl,-rpath,/usr/lib/swift");
    tauri_build::build()
}
