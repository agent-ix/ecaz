fn emit_git_sha_env() {
    // Provenance stamp (NFR-007): suite manifests must record which commit
    // produced the installed extension. "unknown" keeps builds working
    // outside a git checkout (e.g. crate tarballs).
    let sha = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .filter(|sha| !sha.is_empty());
    let dirty = std::process::Command::new("git")
        .args(["status", "--porcelain", "--untracked-files=no"])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| !out.stdout.is_empty())
        .unwrap_or(false);
    let stamp = match sha {
        Some(sha) if dirty => format!("{sha}-dirty"),
        Some(sha) => sha,
        None => "unknown".to_string(),
    };
    println!("cargo:rustc-env=ECAZ_GIT_SHA={stamp}");
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/index");
}

fn main() {
    println!("cargo:rustc-check-cfg=cfg(kani)");
    println!("cargo:rustc-check-cfg=cfg(miri)");
    println!("cargo:rerun-if-changed=build.rs");
    emit_git_sha_env();
    println!("cargo:rerun-if-changed=csrc/standalone_pg_backend_stubs.c");
    println!("cargo:rerun-if-changed=csrc/pg18_pgstat_shim.c");
    println!("cargo:rerun-if-env-changed=PGRX_PG_CONFIG_PATH");

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR should be present for build scripts");

    if (target_os == "linux" && target_arch == "x86_64") || target_os == "macos" {
        cc::Build::new()
            .file("csrc/standalone_pg_backend_stubs.c")
            .cargo_metadata(false)
            .flag_if_supported("-std=c11")
            .compile("ecaz_pg_test_stubs");
        println!("cargo:rustc-link-search=native={out_dir}");
    }

    if std::env::var_os("CARGO_FEATURE_PG18").is_some() {
        let pgrx =
            pgrx_pg_config::Pgrx::from_config().expect("PG18 shim build requires pgrx config");
        let pg_config = pgrx
            .get("pg18")
            .expect("PG18 shim build requires a managed pg18 config");
        let include_dir = pg_config
            .includedir_server()
            .expect("PG18 shim build requires PostgreSQL server headers");
        let cppflags = pg_config
            .cppflags()
            .expect("PG18 shim build requires PostgreSQL cppflags");

        let mut build = cc::Build::new();
        build
            .file("csrc/pg18_pgstat_shim.c")
            .include(include_dir)
            .cargo_metadata(false)
            .flag_if_supported("-std=c11");
        for flag in cppflags.to_string_lossy().split_whitespace() {
            build.flag(flag);
        }
        build.compile("ecaz_pg18_pgstat_shim");
        println!("cargo:rustc-link-search=native={out_dir}");
    }
}
