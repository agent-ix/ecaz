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
    emit_git_rerun_paths();
    println!("cargo:rerun-if-changed=build.rs");
}

fn emit_git_rerun_paths() {
    let git_path = |name: &str| {
        std::process::Command::new("git")
            .args(["rev-parse", "--git-path", name])
            .output()
            .ok()
            .filter(|out| out.status.success())
            .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_owned())
            .filter(|path| !path.is_empty())
    };
    for name in ["HEAD", "index", "packed-refs"] {
        if let Some(path) = git_path(name) {
            println!("cargo:rerun-if-changed={path}");
        }
    }
    let symbolic = std::process::Command::new("git")
        .args(["symbolic-ref", "-q", "HEAD"])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_owned());
    if let Some(reference) = symbolic
        .as_deref()
        .filter(|reference| !reference.is_empty())
    {
        if let Some(path) = git_path(reference) {
            println!("cargo:rerun-if-changed={path}");
        }
    }
}
