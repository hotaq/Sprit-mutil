//! Error handling for the Sprite multi-agent workflow toolkit.
//!
//! This module defines custom error types for clear, actionable
//! error messages while using anyhow for error propagation throughout the application.

/// Main error type for the Sprite application.
#[derive(Debug)]
#[allow(dead_code)]
pub enum SpriteError {
    /// Configuration-related errors
    Config {
        message: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Git operation errors
    Git {
        operation: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Tmux operation errors
    Tmux {
        operation: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Agent management errors
    Agent {
        message: String,
        agent_id: Option<String>,
    },

    /// Session management errors
    Session {
        message: String,
        session_name: Option<String>,
    },

    /// File system errors
    FileSystem {
        operation: String,
        path: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Validation errors
    Validation {
        message: String,
        field: Option<String>,
        value: Option<String>,
    },

    /// Process execution errors
    Process {
        command: String,
        exit_code: Option<i32>,
        stdout: String,
        stderr: String,
    },

    /// I/O errors
    Io {
        operation: String,
        source: std::io::Error,
    },

    /// YAML parsing errors
    Yaml {
        message: String,
        line: Option<usize>,
        column: Option<usize>,
    },

    /// Security errors
    Security {
        message: String,
        violation_type: SecurityViolationType,
    },

    /// Network/communication errors
    Communication {
        message: String,
        target: Option<String>,
    },
}

/// Types of security violations for better error categorization.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum SecurityViolationType {
    PathTraversal,
    InvalidPath,
    UnauthorizedAccess,
    CommandInjection,
    UnsafeCommand,
    InvalidInput,
    MalformedSessionName,
    WorkspaceValidation,
    InsecurePermissions,
    InsecureEnvironment,
}

#[allow(dead_code)]
impl SpriteError {
    /// Create a configuration error with a message.
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
            source: None,
        }
    }

    /// Create a configuration error with a message and source error.
    pub fn config_with_source(
        message: impl Into<String>,
        source: impl Into<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self::Config {
            message: message.into(),
            source: Some(source.into()),
        }
    }

    /// Create a git operation error.
    pub fn git(operation: impl Into<String>) -> Self {
        Self::Git {
            operation: operation.into(),
            source: None,
        }
    }

    /// Create a git operation error with a source error.
    pub fn git_with_source(
        operation: impl Into<String>,
        source: impl Into<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self::Git {
            operation: operation.into(),
            source: Some(source.into()),
        }
    }

    /// Create a tmux operation error.
    pub fn tmux(operation: impl Into<String>) -> Self {
        Self::Tmux {
            operation: operation.into(),
            source: None,
        }
    }

    /// Create a tmux operation error with a source error.
    pub fn tmux_with_source(
        operation: impl Into<String>,
        source: impl Into<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self::Tmux {
            operation: operation.into(),
            source: Some(source.into()),
        }
    }

    /// Create an agent error.
    pub fn agent(message: impl Into<String>, agent_id: Option<impl Into<String>>) -> Self {
        Self::Agent {
            message: message.into(),
            agent_id: agent_id.map(|id| id.into()),
        }
    }

    /// Create a session error.
    pub fn session(message: impl Into<String>, session_name: Option<impl Into<String>>) -> Self {
        Self::Session {
            message: message.into(),
            session_name: session_name.map(|name| name.into()),
        }
    }

    /// Create a file system error.
    pub fn filesystem(operation: impl Into<String>, path: impl Into<String>) -> Self {
        Self::FileSystem {
            operation: operation.into(),
            path: path.into(),
            source: None,
        }
    }

    /// Create a file system error with a source error.
    pub fn filesystem_with_source(
        operation: impl Into<String>,
        path: impl Into<String>,
        source: impl Into<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self::FileSystem {
            operation: operation.into(),
            path: path.into(),
            source: Some(source.into()),
        }
    }

    /// Create a validation error.
    pub fn validation(
        message: impl Into<String>,
        field: Option<impl Into<String>>,
        value: Option<impl Into<String>>,
    ) -> Self {
        Self::Validation {
            message: message.into(),
            field: field.map(|f| f.into()),
            value: value.map(|v| v.into()),
        }
    }

    /// Create a process execution error.
    pub fn process(
        command: impl Into<String>,
        exit_code: Option<i32>,
        stdout: impl Into<String>,
        stderr: impl Into<String>,
    ) -> Self {
        Self::Process {
            command: command.into(),
            exit_code,
            stdout: stdout.into(),
            stderr: stderr.into(),
        }
    }

    /// Create an I/O error.
    pub fn io(operation: impl Into<String>, source: std::io::Error) -> Self {
        Self::Io {
            operation: operation.into(),
            source,
        }
    }

    /// Create a YAML parsing error.
    pub fn yaml(message: impl Into<String>, line: Option<usize>, column: Option<usize>) -> Self {
        Self::Yaml {
            message: message.into(),
            line,
            column,
        }
    }

    /// Create a security error.
    pub fn security(message: impl Into<String>, violation_type: SecurityViolationType) -> Self {
        Self::Security {
            message: message.into(),
            violation_type,
        }
    }

    /// Create a communication error.
    pub fn communication(message: impl Into<String>, target: Option<impl Into<String>>) -> Self {
        Self::Communication {
            message: message.into(),
            target: target.map(|t| t.into()),
        }
    }

    /// Get a user-friendly suggestion for resolving this error.
    pub fn suggestion(&self) -> Option<String> {
        match self {
            Self::Config { message, .. } => {
                if message.contains("not found") {
                    Some("Run 'sprite init' to create a default configuration.".to_string())
                } else if message.contains("invalid") {
                    Some("Check your configuration file syntax and required fields.".to_string())
                } else {
                    None
                }
            }
            Self::Git { operation, .. } => {
                if operation.contains("worktree") {
                    Some("Make sure you're in a git repository and the branch exists.".to_string())
                } else {
                    Some("Check your git repository status and permissions.".to_string())
                }
            }
            Self::Tmux { operation, .. } => {
                if operation.contains("session") {
                    Some("Use 'sprite start' to create a new session.".to_string())
                } else {
                    Some("Make sure tmux is installed and running.".to_string())
                }
            }
            Self::Agent { agent_id, .. } => {
                if let Some(id) = agent_id {
                    Some(format!(
                        "Check if agent '{}' exists and is properly configured.",
                        id
                    ))
                } else {
                    Some("Verify your agent configuration in agents.yaml.".to_string())
                }
            }
            Self::Session { session_name, .. } => {
                if let Some(name) = session_name {
                    Some(format!(
                        "Use 'sprite attach {}' to connect to the session.",
                        name
                    ))
                } else {
                    Some("Use 'sprite start' to create a new session.".to_string())
                }
            }
            Self::FileSystem {
                operation, path, ..
            } => {
                if operation.contains("create") || operation.contains("write") {
                    Some(format!(
                        "Check permissions for path '{}' and parent directories.",
                        path
                    ))
                } else {
                    Some(format!("Verify that '{}' exists and is accessible.", path))
                }
            }
            Self::Validation { field, .. } => {
                if let Some(f) = field {
                    Some(format!(
                        "Check the value of field '{}' in your configuration.",
                        f
                    ))
                } else {
                    Some("Review your configuration for invalid values.".to_string())
                }
            }
            Self::Process { command, .. } => Some(format!(
                "Check if '{}' is installed and in your PATH.",
                command
            )),
            Self::Security { violation_type, .. } => match violation_type {
                SecurityViolationType::PathTraversal => {
                    Some("Use relative paths within the project directory only.".to_string())
                }
                SecurityViolationType::InvalidPath => {
                    Some("Check that the path is valid and within allowed boundaries.".to_string())
                }
                SecurityViolationType::UnauthorizedAccess => {
                    Some("Check file permissions and access rights.".to_string())
                }
                SecurityViolationType::CommandInjection => {
                    Some("Avoid special shell characters in agent names and commands.".to_string())
                }
                SecurityViolationType::UnsafeCommand => Some(
                    "Use only safe, approved commands. Avoid shell metacharacters.".to_string(),
                ),
                SecurityViolationType::InvalidInput => Some(
                    "Check that input contains only allowed characters and formats.".to_string(),
                ),
                SecurityViolationType::MalformedSessionName => {
                    Some("Use alphanumeric session names with hyphens only.".to_string())
                }
                SecurityViolationType::WorkspaceValidation => {
                    Some("Ensure all workspaces are within the agents/ directory.".to_string())
                }
                SecurityViolationType::InsecurePermissions => {
                    Some("Restrict file permissions to prevent unauthorized access.".to_string())
                }
                SecurityViolationType::InsecureEnvironment => Some(
                    "Run with appropriate user permissions and secure environment settings."
                        .to_string(),
                ),
            },
            Self::Communication { target, .. } => {
                if let Some(t) = target {
                    Some(format!("Check if agent '{}' is running and accessible.", t))
                } else {
                    Some("Verify that all agents are running and responsive.".to_string())
                }
            }
            Self::Io { operation, .. } => Some(format!(
                "Check file permissions and disk space for operation: {}",
                operation
            )),
            Self::Yaml { line, column, .. } => {
                let mut suggestion = "Check your YAML syntax.".to_string();
                if let Some(l) = line {
                    suggestion.push_str(&format!(" Error at line {}.", l));
                    if let Some(c) = column {
                        suggestion.push_str(&format!(" Column {}.", c));
                    }
                }
                Some(suggestion)
            }
        }
    }

    /// Get the exit code for this error type.
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Config { .. } => 2,
            Self::Git { .. } => 3,
            Self::Tmux { .. } => 4,
            Self::Agent { .. } => 5,
            Self::Session { .. } => 6,
            Self::FileSystem { .. } => 7,
            Self::Validation { .. } => 8,
            Self::Process { exit_code, .. } => exit_code.unwrap_or(9),
            Self::Io { .. } => 10,
            Self::Yaml { .. } => 11,
            Self::Security { .. } => 12,
            Self::Communication { .. } => 13,
        }
    }
}

/// Type alias for Result with our error type.
#[allow(dead_code)]
pub type Result<T> = anyhow::Result<T, SpriteError>;

/// Convert std::io::Error to our error type.
impl From<std::io::Error> for SpriteError {
    fn from(err: std::io::Error) -> Self {
        Self::io("I/O operation", err)
    }
}

/// Convert serde_yaml::Error to our error type.
impl From<serde_yaml::Error> for SpriteError {
    fn from(err: serde_yaml::Error) -> Self {
        let location = err.location();
        let line = location.as_ref().map(|loc| loc.line());
        let column = location.as_ref().map(|loc| loc.column());
        Self::yaml(err.to_string(), line, column)
    }
}

/// Display implementation that includes suggestions.
impl std::fmt::Display for SpriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Config { message, .. } => write!(f, "Configuration error: {}", message),
            Self::Git { operation, .. } => write!(f, "Git operation failed: {}", operation),
            Self::Tmux { operation, .. } => write!(f, "Tmux operation failed: {}", operation),
            Self::Agent { message, .. } => write!(f, "Agent error: {}", message),
            Self::Session { message, .. } => write!(f, "Session error: {}", message),
            Self::FileSystem { operation, path, .. } => {
                write!(f, "File system error: {} on '{}' failed", operation, path)
            }
            Self::Validation { message, .. } => write!(f, "Validation error: {}", message),
            Self::Process { command, .. } => write!(f, "Command execution failed: {}", command),
            Self::Io { operation, .. } => write!(f, "I/O error: {}", operation),
            Self::Yaml { message, .. } => write!(f, "YAML parsing error: {}", message),
            Self::Security { message, .. } => write!(f, "Security error: {}", message),
            Self::Communication { message, .. } => write!(f, "Communication error: {}", message),
        }?;

        if let Some(suggestion) = self.suggestion() {
            write!(f, "\n\nSuggestion: {}", suggestion)?;
        }
        Ok(())
    }
}

/// Implement std::error::Error trait for SpriteError.
impl std::error::Error for SpriteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Config { source, .. } => source.as_ref().map(|e| e.as_ref() as &dyn std::error::Error),
            Self::Git { source, .. } => source.as_ref().map(|e| e.as_ref() as &dyn std::error::Error),
            Self::Tmux { source, .. } => source.as_ref().map(|e| e.as_ref() as &dyn std::error::Error),
            Self::FileSystem { source, .. } => source.as_ref().map(|e| e.as_ref() as &dyn std::error::Error),
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = SpriteError::config("Test config error");
        assert!(matches!(err, SpriteError::Config { .. }));
        assert_eq!(err.exit_code(), 2);
    }

    #[test]
    fn test_error_with_agent_id() {
        let err = SpriteError::agent("Agent not found", Some("agent-1"));
        match err {
            SpriteError::Agent { agent_id, .. } => {
                assert_eq!(agent_id.as_deref(), Some("agent-1"));
            }
            _ => panic!("Expected Agent error"),
        }
    }

    #[test]
    fn test_error_suggestion() {
        let err = SpriteError::config("Configuration file not found");
        let suggestion = err.suggestion();
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("sprite init"));
    }

    #[test]
    fn test_error_display() {
        let err = SpriteError::git("failed to create worktree");
        let display = format!("{}", err);
        assert!(display.contains("Git operation failed"));
        assert!(display.contains("Suggestion:"));
    }

    #[test]
    fn test_security_violation_types() {
        let err = SpriteError::security("Path contains '..'", SecurityViolationType::PathTraversal);
        assert!(matches!(err, SpriteError::Security { .. }));
        assert_eq!(err.exit_code(), 12);

        let suggestion = err.suggestion();
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("relative paths"));
    }

    #[test]
    fn test_yaml_error_with_location() {
        let err = SpriteError::yaml("Invalid mapping", Some(10), Some(5));
        assert!(matches!(
            err,
            SpriteError::Yaml {
                line: Some(10),
                column: Some(5),
                ..
            }
        ));

        let suggestion = err.suggestion();
        assert!(suggestion.is_some());
        assert!(suggestion.clone().unwrap().contains("line 10"));
        assert!(suggestion.clone().unwrap().contains("Column 5"));
    }
}
