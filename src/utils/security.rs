//! Security and validation utilities for the Sprite multi-agent workflow toolkit.
//!
//! This module provides functions for path validation, input sanitization,
//! and security checks needed for safe agent operations.

use anyhow::{Context, Result};
use crate::error::{SpriteError, SecurityViolationType};
use std::path::{Path, PathBuf};

/// Maximum allowed path length to prevent path traversal attacks
const MAX_PATH_LENGTH: usize = 4096;

/// List of dangerous characters that should not be in file paths
const DANGEROUS_CHARS: &[char] = &['\0', '<', '>', '|', '"', ';', '&', '$', '`', '(', ')', '[', ']', '{', '}'];

/// Validate that a path is safe for agent operations.
pub fn validate_agent_path(path: &Path) -> Result<()> {
    // Check path length
    let path_str = path.to_string_lossy();
    if path_str.len() > MAX_PATH_LENGTH {
        return Err(SpriteError::security(format!(
            "Path too long: {} characters (max: {})",
            path_str.len(),
            MAX_PATH_LENGTH
        ), SecurityViolationType::InvalidPath).into());
    }

    // Check for dangerous characters
    for char in DANGEROUS_CHARS.iter() {
        if path_str.contains(*char) {
            return Err(SpriteError::security(format!(
                "Path contains dangerous character '{}': {}",
                char, path_str
            ), SecurityViolationType::InvalidPath).into());
        }
    }

    // Check for path traversal attempts
    if path_str.contains("..") {
        return Err(SpriteError::security(format!(
            "Path traversal detected: {}",
            path_str
        ), SecurityViolationType::PathTraversal).into());
    }

    // Check if path is absolute (should be relative to workspace)
    if path.is_absolute() {
        return Err(SpriteError::security(format!(
            "Absolute paths not allowed: {}",
            path_str
        ), SecurityViolationType::InvalidPath).into());
    }

    Ok(())
}

/// Validate that a directory path is safe and within allowed bounds.
pub fn validate_directory_path(path: &Path, base_dir: &Path) -> Result<()> {
    validate_agent_path(path)?;

    // Canonicalize both paths to resolve relative components
    let canonical_path = std::fs::canonicalize(path)
        .with_context(|| format!("Failed to canonicalize path: {}", path.display()))?;

    let canonical_base = std::fs::canonicalize(base_dir)
        .with_context(|| format!("Failed to canonicalize base directory: {}", base_dir.display()))?;

    // Check that the path is within the base directory
    if !canonical_path.starts_with(&canonical_base) {
        return Err(SpriteError::security(format!(
            "Path '{}' is outside of allowed base directory '{}'",
            canonical_path.display(),
            canonical_base.display()
        ), SecurityViolationType::PathTraversal).into());
    }

    Ok(())
}

/// Validate agent name for security and proper formatting.
pub fn validate_agent_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(SpriteError::security("Agent name cannot be empty", SecurityViolationType::InvalidInput).into());
    }

    if name.len() > 64 {
        return Err(SpriteError::security(format!(
            "Agent name too long: {} characters (max: 64)",
            name.len()
        ), SecurityViolationType::InvalidInput).into());
    }

    // Check for valid characters (alphanumeric, hyphen, underscore)
    if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(SpriteError::security(format!(
            "Agent name contains invalid characters: {}",
            name
        ), SecurityViolationType::InvalidInput).into());
    }

    // Name should not start with a hyphen or underscore
    if let Some(first_char) = name.chars().next() {
        if first_char == '-' || first_char == '_' {
            return Err(SpriteError::security(format!(
                "Agent name cannot start with '{}': {}",
                first_char, name
            ), SecurityViolationType::InvalidInput).into());
        }
    }

    // Name should not be a reserved system name
    let reserved_names = &[
        "root", "admin", "system", "daemon", "nobody", "www-data",
        "sprite", "control", "master", "main", "default"
    ];

    if reserved_names.contains(&name.to_lowercase().as_str()) {
        return Err(SpriteError::security(format!(
            "Agent name '{}' is reserved and cannot be used",
            name
        ), SecurityViolationType::InvalidInput).into());
    }

    Ok(())
}

/// Validate session name for security and proper formatting.
pub fn validate_session_name(name: &str) -> Result<()> {
    // Use same validation as agent names for consistency
    validate_agent_name(name)?;

    // Additional session-specific validation could be added here
    if name.to_lowercase().contains("control") {
        return Err(SpriteError::security(format!(
            "Session name containing 'control' is reserved: {}",
            name
        ), SecurityViolationType::InvalidInput).into());
    }

    Ok(())
}

/// Validate tmux session name for compatibility with tmux requirements.
pub fn validate_tmux_session_name(name: &str) -> Result<()> {
    validate_session_name(name)?;

    // Tmux has additional restrictions on session names
    if name.len() > 32 {
        return Err(SpriteError::security(format!(
            "Tmux session name too long: {} characters (max: 32)",
            name.len()
        ), SecurityViolationType::InvalidInput).into());
    }

    // Tmux doesn't allow certain characters in session names
    if name.contains('.') || name.contains('@') || name.contains(':') {
        return Err(SpriteError::security(format!(
            "Tmux session name contains invalid characters: {}",
            name
        ), SecurityViolationType::InvalidInput).into());
    }

    Ok(())
}

/// Validate git branch name for compatibility with git requirements.
pub fn validate_git_branch_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(SpriteError::security("Git branch name cannot be empty", SecurityViolationType::InvalidInput).into());
    }

    // Git branch name restrictions
    if name.len() > 255 {
        return Err(SpriteError::security(format!(
            "Git branch name too long: {} characters (max: 255)",
            name.len()
        ), SecurityViolationType::InvalidInput).into());
    }

    // Cannot start with a dot or slash
    if let Some(first_char) = name.chars().next() {
        if first_char == '.' || first_char == '/' {
            return Err(SpriteError::security(format!(
                "Git branch name cannot start with '{}': {}",
                first_char, name
            ), SecurityViolationType::InvalidInput).into());
        }
    }

    // Cannot end with a slash
    if let Some(last_char) = name.chars().last() {
        if last_char == '/' {
            return Err(SpriteError::security(format!(
                "Git branch name cannot end with '{}': {}",
                last_char, name
            ), SecurityViolationType::InvalidInput).into());
        }
    }

    // Cannot contain consecutive slashes
    if name.contains("//") {
        return Err(SpriteError::security(format!(
            "Git branch name cannot contain consecutive slashes: {}",
            name
        ), SecurityViolationType::InvalidInput).into());
    }

    // Cannot contain certain characters
    let invalid_patterns = &["..", "~", "^", ":", "?", "*", "[", " ", "\t", "\n", "\r"];
    for invalid in invalid_patterns {
        if name.contains(invalid) {
            return Err(SpriteError::security(format!(
                "Git branch name contains invalid sequence '{}': {}",
                invalid, name
            ), SecurityViolationType::InvalidInput).into());
        }
    }

    // Cannot be a special git reference
    let reserved_refs = &[
        "HEAD", "ORIG_HEAD", "FETCH_HEAD", "MERGE_HEAD",
        "main", "master", "develop", "release", "hotfix"
    ];

    if reserved_refs.contains(&name) {
        return Err(SpriteError::security(format!(
            "Git branch name '{}' is reserved",
            name
        ), SecurityViolationType::InvalidInput).into());
    }

    Ok(())
}

/// Sanitize a string input to remove potentially dangerous content.
pub fn sanitize_input(input: &str) -> String {
    input
        .chars()
        .filter(|c| {
            // Allow only printable ASCII characters and some Unicode
            c.is_ascii_graphic() || *c == ' ' || *c == '\t' || *c == '\n' || *c == '\r'
        })
        .collect::<String>()
        .trim()
        .to_string()
}

/// Validate that a command is safe to execute.
pub fn validate_command(command: &str) -> Result<()> {
    if command.is_empty() {
        return Err(SpriteError::security("Command cannot be empty", SecurityViolationType::InvalidInput).into());
    }

    // Check for dangerous command patterns
    let dangerous_patterns = &[
        "rm -rf /", "sudo rm", "chmod 777", "chown root",
        "dd if=", "mkfs", "format", "fdisk", "shutdown", "reboot",
        "iptables", "ufw", "firewall", "passwd", "su -", "sudo su",
        "curl | sh", "wget | sh", "eval $(curl", "eval $(wget",
        "> /dev/", "< /dev/", "&& rm", "; rm", "|| rm",
    ];

    let lower_command = command.to_lowercase();
    for pattern in dangerous_patterns.iter() {
        if lower_command.contains(&pattern.to_lowercase()) {
            return Err(SpriteError::security(format!(
                "Command contains dangerous pattern '{}': {}",
                pattern, command
            ), SecurityViolationType::UnsafeCommand).into());
        }
    }

    // Check for shell injection attempts
    if lower_command.contains("$(") || lower_command.contains("`") || lower_command.contains("${") {
        return Err(SpriteError::security(format!(
            "Command contains potential shell injection: {}",
            command
        ), SecurityViolationType::UnsafeCommand).into());
    }

    Ok(())
}

/// Validate that a URL is safe and uses allowed protocols.
pub fn validate_url(url: &str) -> Result<()> {
    if url.is_empty() {
        return Err(SpriteError::security("URL cannot be empty", SecurityViolationType::InvalidInput).into());
    }

    // Check URL length
    if url.len() > 2048 {
        return Err(SpriteError::security(format!(
            "URL too long: {} characters (max: 2048)",
            url.len()
        ), SecurityViolationType::InvalidInput).into());
    }

    // Check for allowed protocols
    let allowed_protocols = &["http://", "https://", "git://", "ssh://"];
    let has_allowed_protocol = allowed_protocols.iter().any(|protocol| {
        url.to_lowercase().starts_with(protocol)
    });

    if !has_allowed_protocol {
        return Err(SpriteError::security(format!(
            "URL must use allowed protocol (http, https, git, ssh): {}",
            url
        ), SecurityViolationType::InvalidInput).into());
    }

    // Check for dangerous URL patterns
    if url.contains("file://") || url.contains("ftp://") || url.contains("data:") {
        return Err(SpriteError::security(format!(
            "URL uses disallowed protocol: {}",
            url
        ), SecurityViolationType::InvalidInput).into());
    }

    Ok(())
}

/// Validate file permissions to ensure they're not too permissive.
pub fn validate_file_permissions(path: &Path) -> Result<()> {
    if !path.exists() {
        return Err(SpriteError::security(format!(
            "File does not exist: {}",
            path.display()
        ), SecurityViolationType::InvalidPath).into());
    }

    let metadata = std::fs::metadata(path)
        .with_context(|| format!("Failed to read file metadata: {}", path.display()))?;

    let permissions = metadata.permissions();

    // Check if file is readable/writable by others (security risk)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = permissions.mode();

        // Check if readable or writable by others
        if mode & 0o044 != 0 {
            return Err(SpriteError::security(format!(
                "File has overly permissive permissions (readable by others): {}",
                path.display()
            ), SecurityViolationType::InsecurePermissions).into());
        }
    }

    Ok(())
}

/// Create a secure temporary directory with appropriate permissions.
pub fn create_secure_temp_dir(prefix: &str) -> Result<PathBuf> {
    let temp_dir = std::env::temp_dir();
    let secure_name = format!("{}_{}", sanitize_input(prefix), uuid::Uuid::new_v4());
    let temp_path = temp_dir.join(secure_name);

    std::fs::create_dir(&temp_path)
        .with_context(|| format!("Failed to create temporary directory: {}", temp_path.display()))?;

    // Set secure permissions (only owner can read/write/execute)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(&temp_path)?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o700);
        std::fs::set_permissions(&temp_path, permissions)?;
    }

    Ok(temp_path)
}

/// Check if running in a secure environment.
pub fn check_environment_security() -> Result<()> {
    // Check if running as root (not recommended)
    #[cfg(unix)]
    {
        if unsafe { libc::getuid() } == 0 {
            return Err(SpriteError::security(
                "Running as root is not recommended for security reasons",
                SecurityViolationType::InsecureEnvironment
            ).into());
        }
    }

    // Check if HOME directory is writable by others
    if let Ok(home) = std::env::var("HOME") {
        let home_path = Path::new(&home);
        if let Ok(metadata) = std::fs::metadata(home_path) {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = metadata.permissions().mode();
                if mode & 0o022 != 0 {
                    return Err(SpriteError::security(format!(
                        "Home directory has overly permissive permissions: {:o}",
                        mode
                    ), SecurityViolationType::InsecureEnvironment).into());
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_validate_agent_name() {
        assert!(validate_agent_name("agent-1").is_ok());
        assert!(validate_agent_name("agent_test").is_ok());

        assert!(validate_agent_name("").is_err());
        assert!(validate_agent_name("agent name").is_err());
        assert!(validate_agent_name("-agent").is_err());
        assert!(validate_agent_name("root").is_err());
        assert!(validate_agent_name(&"a".repeat(65)).is_err());
    }

    #[test]
    fn test_validate_agent_path() {
        assert!(validate_agent_path(Path::new("valid/path")).is_ok());
        assert!(validate_agent_path(Path::new("valid-file.txt")).is_ok());

        assert!(validate_agent_path(Path::new("../dangerous")).is_err());
        assert!(validate_agent_path(Path::new("/absolute/path")).is_err());
        assert!(validate_agent_path(Path::new("path/with<danger")).is_err());
    }

    #[test]
    fn test_validate_command() {
        assert!(validate_command("ls -la").is_ok());
        assert!(validate_command("cargo build").is_ok());

        assert!(validate_command("rm -rf /").is_err());
        assert!(validate_command("curl | sh").is_err());
        assert!(validate_command("eval $(curl url)").is_err());
    }

    #[test]
    fn test_validate_url() {
        assert!(validate_url("https://github.com/user/repo").is_ok());
        assert!(validate_url("git@github.com:user/repo.git").is_ok());

        assert!(validate_url("file:///etc/passwd").is_err());
        assert!(validate_url("ftp://example.com").is_err());
        assert!(validate_url("").is_err());
    }

    #[test]
    fn test_sanitize_input() {
        assert_eq!(sanitize_input("hello world"), "hello world");
        assert_eq!(sanitize_input("hello\x00world"), "helloworld");
        assert_eq!(sanitize_input("  trimmed  "), "trimmed");
        assert_eq!(sanitize_input("multiple\nlines"), "multiple lines");
    }

    #[test]
    fn test_validate_directory_path() {
        let temp_dir = TempDir::new().unwrap();
        let base_dir = temp_dir.path();

        // Create a subdirectory for testing
        let sub_dir = base_dir.join("test");
        std::fs::create_dir(&sub_dir).unwrap();

        assert!(validate_directory_path(&sub_dir, base_dir).is_ok());

        // Test path outside base directory (should fail)
        let outside_path = temp_dir.path().parent().unwrap().join("outside");
        assert!(validate_directory_path(&outside_path, base_dir).is_err());
    }

    #[test]
    fn test_create_secure_temp_dir() {
        let temp_dir = create_secure_temp_dir("test").unwrap();
        assert!(temp_dir.exists());
        assert!(temp_dir.is_dir());

        // Check permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = std::fs::metadata(&temp_dir).unwrap();
            let mode = metadata.permissions().mode();
            assert_eq!(mode & 0o777, 0o700);
        }
    }
}