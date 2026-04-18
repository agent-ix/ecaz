fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=csrc/standalone_pg_backend_stubs.c");

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();

    if target_os == "linux" && target_arch == "x86_64" {
        cc::Build::new()
            .file("csrc/standalone_pg_backend_stubs.c")
            .cargo_metadata(false)
            .flag_if_supported("-std=c11")
            .compile("tqvector_pg_test_stubs");
        println!(
            "cargo:rustc-link-search=native={}",
            std::env::var("OUT_DIR").expect("OUT_DIR should be present for build scripts")
        );
    }
}
