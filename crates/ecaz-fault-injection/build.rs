use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=src/ldpreload_provider.c");

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "linux" {
        return;
    }

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set by cargo"));
    let provider = out_dir.join("libecaz_fault_provider.so");
    let compiler = cc::Build::new().get_compiler();
    let status = Command::new(compiler.path())
        .args(compiler.args())
        .arg("-shared")
        .arg("-fPIC")
        .arg("-O2")
        .arg("src/ldpreload_provider.c")
        .arg("-o")
        .arg(&provider)
        .arg("-ldl")
        .status()
        .expect("invoke C compiler for fault provider");
    assert!(status.success(), "fault provider C build failed");

    println!(
        "cargo:rustc-env=ECAZ_FAULT_PROVIDER_SO={}",
        provider.display()
    );
}
