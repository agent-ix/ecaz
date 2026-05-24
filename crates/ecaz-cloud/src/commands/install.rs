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

        let script = build_script(&self.git_url, &self.git_ref);
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

fn build_script(git_url: &str, git_ref: &str) -> String {
    // Mirror the cloud-init build path so the same install command works
    // before and after the host's first boot. Shell-escaping is intentionally
    // strict — the only caller-supplied strings are the URL and ref.
    let url = shell_escape(git_url);
    let r = shell_escape(git_ref);
    let origin_ref = shell_escape(&format!("origin/{git_ref}"));
    format!(
        r#"#!/usr/bin/env bash
set -euxo pipefail
sudo -u postgres bash -lc '
  set -eux
  export PATH=$HOME/.cargo/bin:$PATH
  if [ ! -d /var/lib/pgsql/build/ecaz/.git ]; then
    rm -rf /var/lib/pgsql/build
    mkdir -p /var/lib/pgsql/build
    git clone {url} /var/lib/pgsql/build/ecaz
  fi
  cd /var/lib/pgsql/build/ecaz
  git reset --hard
  git clean -fd
  git fetch --all --tags
  git checkout --force {r} || git checkout --force {origin_ref}
  if git rev-parse --verify {origin_ref} >/dev/null 2>&1; then
    git reset --hard {origin_ref}
  else
    git reset --hard {r}
  fi
  cargo pgrx install --sudo --release --pg-config /usr/bin/pg_config
  cargo build --release -p ecaz-cli
'
sudo install -Dm755 /var/lib/pgsql/build/ecaz/target/release/ecaz /usr/local/bin/ecaz
sudo systemctl restart postgresql
sudo -u postgres psql -c 'DROP EXTENSION IF EXISTS ecaz;'
sudo -u postgres psql -c 'CREATE EXTENSION ecaz;'
sudo -u postgres psql -c "SELECT extname, extversion FROM pg_extension WHERE extname = 'ecaz';"
"#
    )
}

fn shell_escape(s: &str) -> String {
    // Single-quote and escape any single quotes inside.
    format!("'{}'", s.replace('\'', "'\\''"))
}
