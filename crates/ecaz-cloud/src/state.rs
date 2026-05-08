//! Local per-profile state file. Tracks the things terraform's own state
//! does not capture: recorded EBS snapshot ids, the last connect string,
//! and pause/resume timestamps used by `status`.

use color_eyre::eyre::{eyre, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::profiles::Profile;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ProfileState {
    /// Most recent successful EBS snapshot id, if any.
    pub last_snapshot_id: Option<String>,
    /// Timestamp of the most recent `pause`, if currently paused.
    pub paused_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Last DSN handed out by `up`/`resume` (host:port and database).
    pub last_dsn: Option<String>,
}

pub fn state_dir(profile: Profile) -> Result<PathBuf> {
    let proj = directories::ProjectDirs::from("ai", "agent-ix", "ecaz")
        .ok_or_else(|| eyre!("could not resolve XDG state dir for ecaz"))?;
    Ok(proj.data_local_dir().join("cloud").join(profile.name()))
}

pub fn state_file(profile: Profile) -> Result<PathBuf> {
    Ok(state_dir(profile)?.join("state.json"))
}

pub async fn load(profile: Profile) -> Result<ProfileState> {
    let path = state_file(profile)?;
    if !path.exists() {
        return Ok(ProfileState::default());
    }
    let bytes = tokio::fs::read(&path)
        .await
        .wrap_err_with(|| format!("read state file {}", path.display()))?;
    Ok(serde_json::from_slice(&bytes).unwrap_or_default())
}

pub async fn save(profile: Profile, state: &ProfileState) -> Result<()> {
    let path = state_file(profile)?;
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await.ok();
    }
    let bytes = serde_json::to_vec_pretty(state)?;
    tokio::fs::write(&path, &bytes)
        .await
        .wrap_err_with(|| format!("write state file {}", path.display()))?;
    Ok(())
}

pub fn terraform_state_path(profile: Profile) -> Result<PathBuf> {
    Ok(state_dir(profile)?.join("terraform.tfstate"))
}

pub fn ensure_dir(path: &Path) -> Result<()> {
    std::fs::create_dir_all(path)
        .wrap_err_with(|| format!("create {}", path.display()))?;
    Ok(())
}
