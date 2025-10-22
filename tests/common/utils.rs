//! Common test utilities

use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

pub fn wait_for_condition<F>(condition: F, timeout: Duration, check_interval: Duration) -> bool
where
    F: Fn() -> bool,
{
    let start = Instant::now();

    while start.elapsed() < timeout {
        if condition() {
            return true;
        }
        thread::sleep(check_interval);
    }

    false
}

pub fn run_command_with_timeout(
    command: &mut Command,
    timeout: Duration,
) -> Result<std::process::Output, Box<dyn std::error::Error>> {
    let mut child = command.spawn()?;

    let start = Instant::now();
    loop {
        match child.try_wait()? {
            Some(status) => {
                return Ok(child.wait_with_output()?);
            }
            None => {
                if start.elapsed() > timeout {
                    child.kill()?;
                    return Err("Command timed out".into());
                }
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
}

pub fn cleanup_tmux_sessions(prefix: &str) {
    if let Ok(output) = Command::new("tmux").args(["list-sessions"]).output() {
        let sessions = String::from_utf8_lossy(&output.stdout);
        for line in sessions.lines() {
            if line.starts_with(prefix) {
                if let Some(session_name) = line.split(':').next() {
                    let _ = Command::new("tmux")
                        .args(["kill-session", "-t", session_name])
                        .output();
                }
            }
        }
    }
}
