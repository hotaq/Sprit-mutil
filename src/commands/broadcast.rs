//! Broadcast command - Send commands to all active agents simultaneously

use crate::commands::send;
use crate::error::SpriteError;
use anyhow::Result;

#[allow(dead_code)]
/// Execute the broadcast command with the given parameters.
pub fn execute(
    command: &str,
    args: &[String],
    timeout: u64,
    work_dir: Option<&str>,
    env_vars: &[String],
    priority: Option<&str>,
    dry_run: bool,
) -> Result<()> {
    println!("📡 Broadcasting to all agents...");

    // Validate priority if provided
    if let Some(p) = priority {
        if !["low", "normal", "high", "critical"].contains(&p.to_lowercase().as_str()) {
            return Err(SpriteError::config(format!(
                "Invalid priority '{}'. Must be one of: low, normal, high, critical",
                p
            ))
            .into());
        }
        println!("⚡ Priority: {}", p);
    }

    if dry_run {
        println!("🔍 DRY RUN - Would broadcast: {}", command);
        if !args.is_empty() {
            println!("   Arguments: {}", args.join(" "));
        }
        if let Some(dir) = work_dir {
            println!("   Work directory: {}", dir);
        }
        if !env_vars.is_empty() {
            println!("   Environment variables: {}", env_vars.join(", "));
        }
        return Ok(());
    }

    // Use existing send command functionality (sequential=false for parallel execution)
    send::execute(command, args, timeout, work_dir, env_vars, false)
}
