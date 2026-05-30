use clap::Args;
use color_eyre::eyre::{eyre, Result};
use std::path::PathBuf;

use crate::{aws, profiles::Profile, ssm, terraform::Terraform};

#[derive(Args, Debug)]
pub struct InstallArgs {
    #[arg(long)]
    pub profile: Profile,

    /// Git ref to build and install. Defaults to the ref baked into the
    /// instance at provision time; pass an explicit ref to upgrade.
    #[arg(long, default_value = "main")]
    pub git_ref: String,

    /// Override the git URL. Default is the repo cloned at provision time.
    #[arg(long, default_value = "https://github.com/agent-ix/ecaz.git")]
    pub git_url: String,

    /// SSM execution timeout (seconds). Builds on c7g.large run ~5–10
    /// min from a clean cargo cache; bump for cold runs at large scale.
    #[arg(long, default_value = "1800")]
    pub timeout: u64,

    /// Extra Cargo features for the extension install, repeatable.
    ///
    /// Defaults still include the package default `pg18`; this only appends
    /// opt-in features such as `rabitq-bf16`.
    #[arg(long = "extension-feature")]
    pub extension_features: Vec<String>,

    /// Skip `DROP EXTENSION` / `CREATE EXTENSION` after installing files.
    ///
    /// Use this when retaining benchmark tables that depend on extension-owned
    /// types; the install still copies the new shared library, installs the
    /// CLI, and restarts PostgreSQL.
    #[arg(long)]
    pub skip_extension_recreate: bool,

    /// Skip rebuilding and reinstalling `/usr/local/bin/ecaz`.
    ///
    /// Use this when only extension build features are being changed and the
    /// existing remote CLI is already suitable for the follow-up command.
    #[arg(long)]
    pub skip_cli_build: bool,

    /// Run `cargo clean` in the remote checkout before building.
    ///
    /// Use this when a retained build cache has filled the benchmark host's
    /// disk and an extension rebuild needs temporary space.
    #[arg(long)]
    pub clean_cargo_target: bool,
}

impl InstallArgs {
    pub async fn run(self, repo_root: PathBuf) -> Result<()> {
        aws::ensure_credentials().await?;
        let tf = Terraform::new(self.profile, &repo_root)?;
        if !tf.state_exists() {
            return Err(eyre!(
                "no stack for profile {}; run `ecaz cloud up` first",
                self.profile
            ));
        }
        let out = tf.outputs().await?;

        let script = build_script_with_options(
            &self.git_url,
            &self.git_ref,
            &self.extension_features,
            self.skip_extension_recreate,
            self.skip_cli_build,
            self.clean_cargo_target,
        );
        tracing::info!(profile = %self.profile, instance = %out.db_instance_id, "ssm: ecaz install");
        let stdout =
            ssm::run_shell(&out.region, &out.db_instance_id, &script, self.timeout).await?;
        tracing::info!(stdout = %stdout.lines().take(5).collect::<Vec<_>>().join(" / "), "install ok");

        println!(
            "install: profile={} db={} ref={} ok",
            self.profile, out.db_private_ip, self.git_ref
        );
        Ok(())
    }
}

fn build_script_with_options(
    git_url: &str,
    git_ref: &str,
    extension_features: &[String],
    skip_extension_recreate: bool,
    skip_cli_build: bool,
    clean_cargo_target: bool,
) -> String {
    // Mirror the cloud-init build path so the same install command works
    // before and after the host's first boot. Shell-escaping is intentionally
    // strict — the only caller-supplied strings are the URL and ref.
    let url = shell_escape(git_url);
    let r = shell_escape(git_ref);
    let origin_ref = shell_escape(&format!("origin/{git_ref}"));
    let extension_features_arg = if extension_features.is_empty() {
        String::new()
    } else {
        let features = shell_escape(&extension_features.join(" "));
        format!(" --features {features}")
    };
    let extension_sql = if skip_extension_recreate {
        "sudo -u postgres psql -c \"SELECT extname, extversion FROM pg_extension WHERE extname = 'ecaz';\""
            .to_owned()
    } else {
        r#"sudo -u postgres psql -c 'DROP EXTENSION IF EXISTS ecaz;'
sudo -u postgres psql -c 'CREATE EXTENSION ecaz;'
sudo -u postgres psql -c "SELECT extname, extversion FROM pg_extension WHERE extname = 'ecaz';""#
            .to_owned()
    };
    let cli_build = if skip_cli_build {
        String::new()
    } else {
        "  cargo build --release -p ecaz-cli\n".to_owned()
    };
    let cli_install = if skip_cli_build {
        String::new()
    } else {
        "sudo install -Dm755 /var/lib/pgsql/build/ecaz/target/release/ecaz /usr/local/bin/ecaz\n"
            .to_owned()
    };
    let pre_git_clean_target = if clean_cargo_target {
        "  cargo clean\n".to_owned()
    } else {
        String::new()
    };
    format!(
        r#"#!/usr/bin/env bash
set -euxo pipefail
sudo mkdir -p /var/lib/pgsql/build
sudo chown -R postgres:postgres /var/lib/pgsql/build
sudo -u postgres bash -lc '
  set -eux
  export PATH=$HOME/.cargo/bin:$PATH
  if [ ! -d /var/lib/pgsql/build/ecaz/.git ]; then
    rm -rf /var/lib/pgsql/build
    mkdir -p /var/lib/pgsql/build
    git clone {url} /var/lib/pgsql/build/ecaz
  fi
  cd /var/lib/pgsql/build/ecaz
{pre_git_clean_target}
  git reset --hard
  git clean -fd
  git fetch --all --tags
  git checkout --force {r} || git checkout --force {origin_ref}
  if git rev-parse --verify {origin_ref} >/dev/null 2>&1; then
    git reset --hard {origin_ref}
  else
    git reset --hard {r}
  fi
  cargo pgrx install --sudo --release --pg-config /usr/bin/pg_config{extension_features_arg}
{cli_build}
'
{cli_install}sudo systemctl restart postgresql
{extension_sql}
"#
    )
}

fn shell_escape(s: &str) -> String {
    // Single-quote and escape any single quotes inside.
    format!("'{}'", s.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_script_omits_feature_flag_by_default() {
        let script = build_script_with_options(
            "https://github.com/agent-ix/ecaz.git",
            "main",
            &[],
            true,
            false,
            false,
        );

        assert!(
            script.contains("cargo pgrx install --sudo --release --pg-config /usr/bin/pg_config\n")
        );
        assert!(!script.contains("--features"));
        assert!(script.contains("cargo build --release -p ecaz-cli"));
        assert!(script.contains(
            "sudo install -Dm755 /var/lib/pgsql/build/ecaz/target/release/ecaz /usr/local/bin/ecaz"
        ));
    }

    #[test]
    fn install_script_adds_extension_features() {
        let script = build_script_with_options(
            "https://github.com/agent-ix/ecaz.git",
            "main",
            &[String::from("rabitq-bf16")],
            true,
            false,
            false,
        );

        assert!(script.contains(
            "cargo pgrx install --sudo --release --pg-config /usr/bin/pg_config --features 'rabitq-bf16'"
        ));
    }

    #[test]
    fn install_script_can_skip_cli_build() {
        let script = build_script_with_options(
            "https://github.com/agent-ix/ecaz.git",
            "main",
            &[String::from("rabitq-bf16")],
            true,
            true,
            false,
        );

        assert!(script.contains(
            "cargo pgrx install --sudo --release --pg-config /usr/bin/pg_config --features 'rabitq-bf16'"
        ));
        assert!(!script.contains("cargo build --release -p ecaz-cli"));
        assert!(!script.contains(
            "sudo install -Dm755 /var/lib/pgsql/build/ecaz/target/release/ecaz /usr/local/bin/ecaz"
        ));
        assert!(script.contains("sudo systemctl restart postgresql"));
        assert!(
            script.contains("SELECT extname, extversion FROM pg_extension WHERE extname = 'ecaz';")
        );
    }

    #[test]
    fn install_script_can_clean_cargo_target_before_build() {
        let script = build_script_with_options(
            "https://github.com/agent-ix/ecaz.git",
            "main",
            &[],
            true,
            false,
            true,
        );

        let clean_pos = script.find("cargo clean").expect("cargo clean present");
        let reset_pos = script.find("git reset --hard").expect("git reset present");
        let build_pos = script
            .find("cargo pgrx install --sudo --release --pg-config /usr/bin/pg_config")
            .expect("pgrx install present");
        assert!(clean_pos < reset_pos);
        assert!(clean_pos < build_pos);
    }
}
