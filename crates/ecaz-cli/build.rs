fn main() {
    // Provenance stamp (NFR-007): suite manifests record which commit the
    // runner binary was built from, so CLI/extension skew is visible.
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
    println!("cargo:rerun-if-changed=../../.git/HEAD");
    println!("cargo:rerun-if-changed=../../.git/index");
    println!("cargo:rerun-if-changed=build.rs");
}
