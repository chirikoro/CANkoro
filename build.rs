fn main() {
    #[cfg(target_os = "windows")]
    {
        // Link to vxlapi64.dll at runtime via LoadLibrary (no link-time dependency)
        // The FFI module uses runtime dynamic loading
        println!("cargo:rerun-if-changed=build.rs");
    }
}
