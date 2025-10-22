//! Cross-platform timeout utilities for Sprite operations
//!
//! This module provides cross-platform timeout functionality that works
//! on macOS, Linux, and Windows without external dependencies.

use anyhow::{Context, Result};
use std::io;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

/// Cross-platform timeout for executing commands
pub fn run_with_timeout(mut cmd: Command, timeout_secs: u64) -> Result<String> {
    let timeout_duration = Duration::from_secs(timeout_secs);

    // Redirect stdout and stderr to capture output
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let child = cmd.spawn().context("Failed to spawn command")?;

    // Wait for completion with timeout
    let result = wait_with_timeout(child, timeout_duration);

    match result {
        TimeoutResult::Completed(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            if output.status.success() {
                Ok(stdout)
            } else {
                Err(anyhow::anyhow!(
                    "Command failed with exit code {:?}. Stderr: {}",
                    output.status.code(),
                    stderr
                ))
            }
        }
        TimeoutResult::Timeout => Err(anyhow::anyhow!(
            "Command timed out after {} seconds",
            timeout_secs
        )),
        TimeoutResult::Error(e) => Err(anyhow::anyhow!("Error waiting for command: {}", e)),
    }
}

/// Result of a timed wait operation
enum TimeoutResult {
    Completed(std::process::Output),
    Timeout,
    Error(io::Error),
}

/// Wait for a child process with timeout
fn wait_with_timeout(mut child: Child, timeout: Duration) -> TimeoutResult {
    let start_time = Instant::now();

    loop {
        match child.try_wait() {
            Ok(Some(_)) => {
                // Process has exited, get the output
                match child.wait_with_output() {
                    Ok(output) => return TimeoutResult::Completed(output),
                    Err(e) => return TimeoutResult::Error(e),
                }
            }
            Ok(None) => {
                // Process is still running
                if start_time.elapsed() >= timeout {
                    return TimeoutResult::Timeout;
                }
                // Sleep for a short time before checking again
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => return TimeoutResult::Error(e),
        }
    }
}

/// Force kill a child process and its children
#[cfg(unix)]
#[allow(dead_code)]
fn kill_child_process(child: &mut Child) -> Result<()> {
    // Check if process is still running first
    if child.try_wait().ok().is_some_and(|status| status.is_none()) {
        let pid = child.id();
        // Send SIGTERM first
        unsafe {
            libc::kill(pid as libc::pid_t, libc::SIGTERM);
        }

        // Give it a moment to exit gracefully
        thread::sleep(Duration::from_millis(1000));

        // If still running, send SIGKILL
        match child.try_wait() {
            Ok(Some(_)) => {
                // Process has exited
                Ok(())
            }
            Ok(None) => {
                // Still running, force kill
                unsafe {
                    libc::kill(pid as libc::pid_t, libc::SIGKILL);
                }
                child.wait()?;
                Ok(())
            }
            Err(e) => Err(anyhow::anyhow!("Failed to kill process: {}", e)),
        }
    } else {
        Ok(())
    }
}

/// Force kill a child process on Windows
#[cfg(windows)]
#[allow(dead_code)]
fn kill_child_process(child: &mut Child) -> Result<()> {
    // On Windows, we can use a simpler approach
    // Check if process is still running first
    if child.try_wait().ok().is_some_and(|status| status.is_none()) {
        let pid = child.id();
        // Use taskkill to terminate the process
        let mut kill_cmd = Command::new("taskkill");
        kill_cmd.args(["/F", "/PID", &pid.to_string()]);
        let _ = kill_cmd.output();

        // Wait for the process to exit
        let _ = child.wait();
    } else {
        // Process already exited
    }
    Ok(())
}

/// Execute a command string with timeout (for shell commands)
pub fn execute_with_timeout(shell_cmd: &str, timeout_secs: u64) -> Result<String> {
    let (shell, shell_arg) = if cfg!(target_os = "windows") {
        ("cmd", "/C")
    } else {
        ("sh", "-c")
    };

    let mut cmd = Command::new(shell);
    cmd.arg(shell_arg);
    cmd.arg(shell_cmd);

    run_with_timeout(cmd, timeout_secs)
}

/// Simple timeout function for blocking operations
pub fn with_timeout<F, T>(timeout_secs: u64, operation: F) -> Result<T>
where
    F: FnOnce() -> Result<T> + std::marker::Send + 'static,
    T: Send + 'static,
{
    let timeout_duration = Duration::from_secs(timeout_secs);

    // Use a channel to implement timeout
    let (sender, receiver) = std::sync::mpsc::channel();

    // Spawn the operation in a separate thread
    thread::spawn(move || match operation() {
        Ok(result) => {
            let _ = sender.send(Ok(result));
        }
        Err(e) => {
            let _ = sender.send(Err(e));
        }
    });

    // Wait for completion with timeout
    match receiver.recv_timeout(timeout_duration) {
        Ok(result) => result,
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => Err(anyhow::anyhow!(
            "Operation timed out after {} seconds",
            timeout_secs
        )),
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
            Err(anyhow::anyhow!("Operation thread panicked"))
        }
    }
}
