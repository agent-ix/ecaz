//! `ecaz dev worktree-prune` — report and reclaim stale git worktrees.
//!
//! Agent lanes create one worktree per task and nothing ever reaps them. On
//! 2026-07-27 a 1TB dev host filled to 100%: 23 live checkouts, each carrying
//! its own multi-GB build tree and bench fixtures.
//!
//! This is deliberately conservative, because the checkout is shared with
//! other agents and a worktree may hold unpushed work:
//!
//! - report-only by default; `--apply` is required to remove anything
//! - only worktrees whose branch is fully merged into the upstream base are
//!   eligible, unless `--include-unmerged` is passed
//! - only worktrees idle for `--idle-days` are eligible
//! - the primary working tree and the current worktree are never eligible
//! - removal never passes `git worktree remove --force`, so a worktree with
//!   uncommitted changes refuses to be removed rather than losing work

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, SystemTime};

use clap::Args;
use color_eyre::eyre::{bail, Context, Result};
use tokio::process::Command;

use super::support::repo_root;

#[derive(Args, Debug)]
pub struct WorktreePruneArgs {
    /// Only consider worktrees whose branch has had no commit for this many days.
    #[arg(long, default_value_t = 5)]
    idle_days: u64,

    /// Upstream base branch that a worktree branch must be merged into.
    #[arg(long, default_value = "origin/main")]
    base: String,

    /// Actually remove eligible worktrees. Without this the command only reports.
    #[arg(long)]
    apply: bool,

    /// Also treat worktrees whose branch is NOT merged into --base as eligible.
    /// Unpushed commits on such a branch are lost when its worktree is removed,
    /// so this still refuses any worktree with uncommitted changes.
    #[arg(long)]
    include_unmerged: bool,
}

#[derive(Debug)]
struct Worktree {
    path: PathBuf,
    branch: Option<String>,
    is_primary: bool,
}

#[derive(Debug)]
struct Candidate {
    worktree: Worktree,
    idle_days: u64,
    merged: bool,
    dirty: bool,
    size_bytes: u64,
}

impl Candidate {
    /// Eligible for removal under the requested policy.
    fn eligible(&self, args: &WorktreePruneArgs) -> bool {
        if self.dirty {
            return false;
        }
        if self.idle_days < args.idle_days {
            return false;
        }
        self.merged || args.include_unmerged
    }

    fn skip_reason(&self, args: &WorktreePruneArgs) -> &'static str {
        if self.dirty {
            "uncommitted changes"
        } else if self.idle_days < args.idle_days {
            "recently active"
        } else if !self.merged && !args.include_unmerged {
            "not merged into base"
        } else {
            "eligible"
        }
    }
}

pub async fn run(args: WorktreePruneArgs) -> Result<()> {
    let repo_root = repo_root()?;
    let current = std::env::current_dir().wrap_err("resolving current working directory")?;

    let worktrees = list_worktrees(&repo_root).await?;
    let mut candidates = Vec::new();
    for worktree in worktrees {
        // Never propose removing the primary working tree or the worktree the
        // operator is standing in.
        if worktree.is_primary || current.starts_with(&worktree.path) {
            continue;
        }
        let idle_days = branch_idle_days(&repo_root, &worktree).await?;
        let merged = match &worktree.branch {
            Some(branch) => is_merged(&repo_root, branch, &args.base).await?,
            // A detached HEAD has no branch to check for merge status; treat it
            // as unmerged so it needs --include-unmerged to be reclaimed.
            None => false,
        };
        let dirty = is_dirty(&worktree.path).await?;
        let size_bytes = dir_size_bytes(&worktree.path).await.unwrap_or(0);
        candidates.push(Candidate {
            worktree,
            idle_days,
            merged,
            dirty,
            size_bytes,
        });
    }

    candidates.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));

    if candidates.is_empty() {
        crate::ecaz_println!("[worktree-prune] no secondary worktrees found");
        return Ok(());
    }

    let mut reclaimable = 0_u64;
    crate::ecaz_println!(
        "[worktree-prune] base={} idle_days={} apply={}",
        args.base,
        args.idle_days,
        args.apply
    );
    for candidate in &candidates {
        let eligible = candidate.eligible(&args);
        if eligible {
            reclaimable += candidate.size_bytes;
        }
        crate::ecaz_println!(
            "[worktree-prune] {:<8} {:>8}  idle={:>3}d  {:<24} {}",
            if eligible { "ELIGIBLE" } else { "keep" },
            human_bytes(candidate.size_bytes),
            candidate.idle_days,
            candidate.skip_reason(&args),
            candidate.worktree.path.display()
        );
    }
    crate::ecaz_println!(
        "[worktree-prune] reclaimable={} across {} worktree(s)",
        human_bytes(reclaimable),
        candidates.iter().filter(|c| c.eligible(&args)).count()
    );

    if !args.apply {
        crate::ecaz_println!("[worktree-prune] report only; pass --apply to remove");
        return Ok(());
    }

    for candidate in candidates.iter().filter(|c| c.eligible(&args)) {
        // No --force: a worktree that turned dirty between the scan above and
        // now must fail loudly rather than discard the change.
        let mut command = Command::new("git");
        command
            .arg("-C")
            .arg(&repo_root)
            .arg("worktree")
            .arg("remove")
            .arg(&candidate.worktree.path)
            .stdin(Stdio::null());
        let output = command
            .output()
            .await
            .wrap_err_with(|| format!("removing worktree {}", candidate.worktree.path.display()))?;
        if output.status.success() {
            crate::ecaz_println!(
                "[worktree-prune] removed {} ({})",
                candidate.worktree.path.display(),
                human_bytes(candidate.size_bytes)
            );
        } else {
            crate::ecaz_println!(
                "[worktree-prune] FAILED {}: {}",
                candidate.worktree.path.display(),
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
    }

    Ok(())
}

async fn list_worktrees(repo_root: &Path) -> Result<Vec<Worktree>> {
    let output = git(repo_root, &["worktree", "list", "--porcelain"]).await?;
    let mut worktrees = Vec::new();
    let mut path: Option<PathBuf> = None;
    let mut branch: Option<String> = None;
    let mut first = true;
    for line in output.lines() {
        if let Some(value) = line.strip_prefix("worktree ") {
            if let Some(previous) = path.take() {
                worktrees.push(Worktree {
                    path: previous,
                    branch: branch.take(),
                    is_primary: first,
                });
                first = false;
            }
            path = Some(PathBuf::from(value));
        } else if let Some(value) = line.strip_prefix("branch ") {
            branch = Some(
                value
                    .strip_prefix("refs/heads/")
                    .unwrap_or(value)
                    .to_string(),
            );
        }
    }
    if let Some(previous) = path {
        worktrees.push(Worktree {
            path: previous,
            branch,
            is_primary: first,
        });
    }
    Ok(worktrees)
}

/// Days since the worktree's branch tip was committed. Detached or unreadable
/// worktrees fall back to the directory mtime.
async fn branch_idle_days(repo_root: &Path, worktree: &Worktree) -> Result<u64> {
    if let Some(branch) = &worktree.branch {
        if let Ok(output) = git(repo_root, &["log", "-1", "--format=%ct", branch]).await {
            if let Ok(committed) = output.trim().parse::<u64>() {
                let now = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or(Duration::ZERO)
                    .as_secs();
                return Ok(now.saturating_sub(committed) / 86_400);
            }
        }
    }
    let metadata = std::fs::metadata(&worktree.path)
        .wrap_err_with(|| format!("stat {}", worktree.path.display()))?;
    let modified = metadata.modified().wrap_err("reading worktree mtime")?;
    Ok(SystemTime::now()
        .duration_since(modified)
        .unwrap_or(Duration::ZERO)
        .as_secs()
        / 86_400)
}

async fn is_merged(repo_root: &Path, branch: &str, base: &str) -> Result<bool> {
    let status = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .arg("merge-base")
        .arg("--is-ancestor")
        .arg(branch)
        .arg(base)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .wrap_err_with(|| format!("checking whether {branch} is merged into {base}"))?;
    Ok(status.success())
}

async fn is_dirty(worktree: &Path) -> Result<bool> {
    // Tracked-file changes and staged content only. Untracked build/bench
    // output is expected in these trees and must not pin a worktree forever.
    let output = git(worktree, &["status", "--porcelain", "--untracked-files=no"]).await?;
    Ok(!output.trim().is_empty())
}

async fn dir_size_bytes(path: &Path) -> Result<u64> {
    // -B1 reports allocated blocks in bytes, matching df. -b would report
    // apparent size, which overstates sparse files (PGDATA is full of them).
    let output = Command::new("du")
        .arg("-sB1")
        .arg("--one-file-system")
        .arg(path)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .await
        .wrap_err_with(|| format!("sizing {}", path.display()))?;
    let text = String::from_utf8_lossy(&output.stdout);
    let bytes = text
        .split_whitespace()
        .next()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0);
    Ok(bytes)
}

async fn git(cwd: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(cwd)
        .args(args)
        .stdin(Stdio::null())
        .output()
        .await
        .wrap_err_with(|| format!("running git {}", args.join(" ")))?;
    if !output.status.success() {
        bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "K", "M", "G", "T"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes}{}", UNITS[unit])
    } else {
        format!("{value:.1}{}", UNITS[unit])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(idle_days: u64, include_unmerged: bool) -> WorktreePruneArgs {
        WorktreePruneArgs {
            idle_days,
            base: "origin/main".to_string(),
            apply: false,
            include_unmerged,
        }
    }

    fn candidate(idle_days: u64, merged: bool, dirty: bool) -> Candidate {
        Candidate {
            worktree: Worktree {
                path: PathBuf::from("/tmp/wt"),
                branch: Some("task-1".to_string()),
                is_primary: false,
            },
            idle_days,
            merged,
            dirty,
            size_bytes: 0,
        }
    }

    #[test]
    fn dirty_worktree_is_never_eligible() {
        // Even merged, idle, and with --include-unmerged, uncommitted work wins.
        let candidate = candidate(365, true, true);
        assert!(!candidate.eligible(&args(5, true)));
        assert_eq!(candidate.skip_reason(&args(5, true)), "uncommitted changes");
    }

    #[test]
    fn recently_active_worktree_is_not_eligible() {
        let candidate = candidate(1, true, false);
        assert!(!candidate.eligible(&args(5, false)));
        assert_eq!(candidate.skip_reason(&args(5, false)), "recently active");
    }

    #[test]
    fn unmerged_worktree_requires_opt_in() {
        let candidate = candidate(30, false, false);
        assert!(!candidate.eligible(&args(5, false)));
        assert!(candidate.eligible(&args(5, true)));
    }

    #[test]
    fn merged_and_idle_worktree_is_eligible() {
        assert!(candidate(30, true, false).eligible(&args(5, false)));
    }

    #[test]
    fn human_bytes_scales() {
        assert_eq!(human_bytes(512), "512B");
        assert_eq!(human_bytes(1536), "1.5K");
        assert_eq!(human_bytes(50 * 1024 * 1024 * 1024), "50.0G");
    }
}
