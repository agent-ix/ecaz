//! ecaz cloud — orchestration for AWS-hosted ecaz benchmark stacks.
//!
//! This crate is the implementation behind `ecaz cloud …`. It does not
//! reimplement corpus or bench logic — for those it shells out to the
//! existing `ecaz` binary against a remote DSN. Its job is provisioning
//! (terraform), instance lifecycle (aws CLI), and tracking enough local
//! state per profile to make every verb idempotent.
//!
//! Design notes:
//!
//! - AWS API access goes through the `aws` CLI rather than a Rust SDK
//!   client. Rationale: keeps the dependency footprint small, mirrors the
//!   ergonomics of `terraform` shell-outs already in the plan, and makes
//!   it easy for an operator to reproduce a failing call from the shell.
//! - Local per-profile state lives under
//!   `${XDG_STATE_HOME}/ecaz/cloud/<profile>/` (typically
//!   `~/.local/state/ecaz/cloud/<profile>/`). It records terraform state
//!   pointer, recorded snapshot ids, and the last seen DSN.
//! - Every command is a struct that implements `Run`. The CLI surface in
//!   `commands::CloudCommand` dispatches to them.

pub mod aws;
pub mod commands;
pub mod profiles;
pub mod state;
pub mod terraform;

pub use commands::CloudCommand;
