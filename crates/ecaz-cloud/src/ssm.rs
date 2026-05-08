//! AWS SSM `send-command` + poll wrapper.
//!
//! All in-instance work (re-install ecaz, fan out corpus load, take a
//! Postgres dump, …) runs as `AWS-RunShellScript` invocations. Operators
//! can SSM-shell into the same hosts manually with `aws ssm
//! start-session --target <id>`.

use color_eyre::eyre::{eyre, Context, Result};
use serde::Deserialize;
use tokio::process::Command;
use tokio::time::{sleep, Duration};

/// Sends a shell script to one instance and polls until it terminates.
/// Returns the captured stdout. The full per-instance result document
/// (stdout, stderr, exit code, timings) is emitted to tracing for any
/// step that fails so the operator does not have to re-invoke
/// `get-command-invocation` by hand.
pub async fn run_shell(
    region: &str,
    instance_id: &str,
    script: &str,
    timeout_secs: u64,
) -> Result<String> {
    let command_id = send(region, instance_id, script, timeout_secs).await?;
    poll(region, instance_id, &command_id, timeout_secs).await
}

async fn send(
    region: &str,
    instance_id: &str,
    script: &str,
    timeout_secs: u64,
) -> Result<String> {
    // Pass the script as a single quoted JSON string for the
    // `commands` parameter. Indirection via a heredoc would be cleaner
    // but this keeps the call shape compatible with `aws ssm` defaults.
    let payload = serde_json::json!({
        "commands": [script],
        "executionTimeout": [timeout_secs.to_string()]
    })
    .to_string();

    let output = Command::new("aws")
        .args([
            "ssm",
            "send-command",
            "--region",
            region,
            "--instance-ids",
            instance_id,
            "--document-name",
            "AWS-RunShellScript",
            "--parameters",
            &payload,
            "--query",
            "Command.CommandId",
            "--output",
            "text",
        ])
        .output()
        .await
        .wrap_err("invoke aws ssm send-command")?;
    if !output.status.success() {
        return Err(eyre!(
            "ssm send-command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[derive(Debug, Deserialize)]
struct Invocation {
    #[serde(rename = "Status")]
    status: String,
    #[serde(rename = "StandardOutputContent", default)]
    stdout: String,
    #[serde(rename = "StandardErrorContent", default)]
    stderr: String,
    #[serde(rename = "ResponseCode", default)]
    response_code: i64,
}

async fn poll(
    region: &str,
    instance_id: &str,
    command_id: &str,
    timeout_secs: u64,
) -> Result<String> {
    let deadline = std::time::Instant::now() + Duration::from_secs(timeout_secs + 60);
    loop {
        // Some delay between send and the invocation being observable.
        let output = Command::new("aws")
            .args([
                "ssm",
                "get-command-invocation",
                "--region",
                region,
                "--instance-id",
                instance_id,
                "--command-id",
                command_id,
                "--output",
                "json",
            ])
            .output()
            .await
            .wrap_err("invoke aws ssm get-command-invocation")?;
        if output.status.success() {
            let inv: Invocation = serde_json::from_slice(&output.stdout)
                .wrap_err("parse ssm invocation document")?;
            match inv.status.as_str() {
                "Success" => return Ok(inv.stdout),
                "Failed" | "Cancelled" | "TimedOut" => {
                    tracing::warn!(stderr = %inv.stderr, "ssm invocation failed");
                    return Err(eyre!(
                        "ssm command {command_id} on {instance_id} ended in {} (rc={}): {}",
                        inv.status,
                        inv.response_code,
                        inv.stderr.lines().take(5).collect::<Vec<_>>().join(" / ")
                    ));
                }
                _ => {} // Pending / InProgress / Delayed
            }
        }
        if std::time::Instant::now() >= deadline {
            return Err(eyre!(
                "ssm command {command_id} on {instance_id} did not finish within {}s",
                timeout_secs
            ));
        }
        sleep(Duration::from_secs(5)).await;
    }
}
