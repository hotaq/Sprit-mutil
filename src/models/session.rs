//! Session and tmux profile data structures for the Sprite multi-agent workflow toolkit.

use super::{ConflictResolution, LogLevel};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents the tmux session that hosts all agent panes and provides supervision interface.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session identifier
    pub name: String,

    /// Current session status
    pub status: SessionStatus,

    /// Tmux profile being used
    pub profile: String,

    /// Active profile configuration
    pub profile_config: TmuxProfile,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last activity timestamp
    pub last_activity: Option<DateTime<Utc>>,

    /// Session-specific settings
    pub settings: SessionSettings,
}

#[allow(dead_code)]
impl Session {
    /// Create a new session with basic configuration.
    pub fn new(name: impl Into<String>, profile: TmuxProfile) -> Self {
        let name_str = name.into();
        Self {
            name: name_str.clone(),
            status: SessionStatus::Creating,
            profile: profile.name.clone(),
            profile_config: profile,
            created_at: Utc::now(),
            last_activity: None,
            settings: SessionSettings::default(),
        }
    }

    /// Update the session's last activity timestamp.
    pub fn update_activity(&mut self) {
        self.last_activity = Some(Utc::now());
    }

    /// Check if the session is currently active.
    pub fn is_active(&self) -> bool {
        matches!(self.status, SessionStatus::Active | SessionStatus::Detached)
    }

    /// Check if the session is in an error state.
    pub fn has_error(&self) -> bool {
        matches!(self.status, SessionStatus::Error(_))
    }

    /// Get the error message if the session is in an error state.
    pub fn error_message(&self) -> Option<&str> {
        match &self.status {
            SessionStatus::Error(msg) => Some(msg),
            _ => None,
        }
    }

    /// Get the duration since session creation.
    pub fn duration(&self) -> chrono::Duration {
        Utc::now() - self.created_at
    }

    /// Validate the session configuration.
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Session name cannot be empty".to_string());
        }

        // Validate session name format (alphanumeric with hyphens/underscores)
        if !self
            .name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(
                "Session name can only contain alphanumeric characters, hyphens, and underscores"
                    .to_string(),
            );
        }

        // Validate profile configuration
        self.profile_config.validate()?;

        Ok(())
    }
}

/// Current status of a session.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionStatus {
    /// Session is being created
    Creating,
    /// Session is active and running
    Active,
    /// Session is detached but still running
    Detached,
    /// Session is being terminated
    Terminating,
    /// Session encountered an error
    Error(String),
}

impl Default for SessionStatus {
    fn default() -> Self {
        Self::Creating
    }
}

/// Session-specific settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSettings {
    /// Whether to auto-sync agents
    pub auto_sync: bool,

    /// Sync interval in seconds
    pub sync_interval_secs: Option<u64>,

    /// Conflict resolution strategy
    pub conflict_resolution: ConflictResolution,

    /// Log level
    pub log_level: LogLevel,

    /// Whether to enable activity monitoring
    pub enable_monitoring: bool,

    /// Maximum session duration in seconds (None for unlimited)
    pub max_duration_secs: Option<u64>,

    /// Whether to automatically cleanup on termination
    pub auto_cleanup: bool,
}

impl Default for SessionSettings {
    fn default() -> Self {
        Self {
            auto_sync: false,
            sync_interval_secs: Some(300), // 5 minutes
            conflict_resolution: ConflictResolution::default(),
            log_level: LogLevel::default(),
            enable_monitoring: true,
            max_duration_secs: None, // Unlimited
            auto_cleanup: true,
        }
    }
}

/// Represents pre-configured tmux layout profiles for different agent arrangements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmuxProfile {
    /// Profile identifier (profile0 - profile5)
    pub name: String,

    /// Profile description and use case
    pub description: String,

    /// Best for number of agents
    pub best_for_agent_count: Option<u32>,

    /// Layout type and configuration
    pub layout: ProfileLayout,

    /// Shell script path for this profile
    pub script_path: PathBuf,

    /// Whether this profile includes a supervisor pane
    pub includes_supervisor: bool,

    /// Supervisor configuration (if applicable)
    pub supervisor_config: Option<SupervisorConfig>,
}

#[allow(dead_code)]
impl TmuxProfile {
    /// Validate the profile configuration.
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Profile name cannot be empty".to_string());
        }

        if self.script_path.as_os_str().is_empty() {
            return Err("Profile script path cannot be empty".to_string());
        }

        self.layout.validate()?;

        if let Some(ref supervisor) = self.supervisor_config {
            supervisor.validate()?;
        }

        Ok(())
    }

    /// Check if this profile is suitable for the given number of agents.
    pub fn is_suitable_for(&self, agent_count: u32) -> bool {
        self.best_for_agent_count
            .map(|best| agent_count <= best)
            .unwrap_or(true)
    }

    /// Get the total number of panes this profile creates.
    pub fn total_panes(&self) -> u32 {
        let mut count = self.layout.pane_sizes.len() as u32;
        if self.includes_supervisor {
            count += 1;
        }
        count
    }
}

/// Layout configuration for tmux profiles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileLayout {
    /// Layout pattern: main-horizontal, main-vertical, even-horizontal, tiled
    pub layout_type: String,

    /// Pane size specifications
    pub pane_sizes: Vec<PaneSize>,

    /// Root pane configuration (main repository)
    pub root_pane: Option<RootPaneConfig>,
}

#[allow(dead_code)]
impl ProfileLayout {
    /// Validate the layout configuration.
    pub fn validate(&self) -> Result<(), String> {
        if self.layout_type.is_empty() {
            return Err("Layout type cannot be empty".to_string());
        }

        if self.pane_sizes.is_empty() {
            return Err("Layout must have at least one pane size specification".to_string());
        }

        // Validate layout type
        let valid_layouts = [
            "main-horizontal",
            "main-vertical",
            "even-horizontal",
            "tiled",
        ];
        if !valid_layouts.contains(&self.layout_type.as_str()) {
            return Err(format!(
                "Invalid layout type '{}'. Valid options: {}",
                self.layout_type,
                valid_layouts.join(", ")
            ));
        }

        for (i, pane_size) in self.pane_sizes.iter().enumerate() {
            if let Err(e) = pane_size.validate() {
                return Err(format!("Pane size {} validation failed: {}", i + 1, e));
            }
        }

        Ok(())
    }
}

/// Size and position specification for a pane.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaneSize {
    /// Agent identifier
    pub agent_id: String,

    /// Percentage or size specification
    pub size_spec: String,

    /// Position in layout
    pub position: LayoutPosition,
}

#[allow(dead_code)]
impl PaneSize {
    /// Validate the pane size specification.
    pub fn validate(&self) -> Result<(), String> {
        if self.agent_id.is_empty() {
            return Err("Agent ID cannot be empty".to_string());
        }

        if self.size_spec.is_empty() {
            return Err("Size specification cannot be empty".to_string());
        }

        // Validate size spec format (e.g., "50%", "80px", etc.)
        if !self.size_spec.ends_with('%') && !self.size_spec.ends_with("px") {
            return Err("Size specification must end with '%' or 'px'".to_string());
        }

        Ok(())
    }
}

/// Position of a pane in the layout.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LayoutPosition {
    Top,
    Bottom,
    Left,
    Right,
    Center,
}

/// Configuration for the root pane (main repository).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootPaneConfig {
    /// Whether root pane is enabled
    pub enabled: bool,

    /// Root pane position
    pub position: LayoutPosition,

    /// Root pane size
    pub size: String,

    /// Default working directory
    pub work_dir: PathBuf,
}

/// Supervisor configuration for session management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisorConfig {
    /// Supervisor pane position
    pub position: SupervisorPosition,

    /// Supervisor pane size
    pub size: SupervisorSize,

    /// Default working directory for supervisor
    pub work_dir: PathBuf,

    /// Commands to run in supervisor pane on startup
    pub startup_commands: Vec<String>,

    /// Whether supervisor pane should be focused by default
    pub focus_by_default: bool,
}

#[allow(dead_code)]
impl SupervisorConfig {
    /// Validate the supervisor configuration.
    pub fn validate(&self) -> Result<(), String> {
        if self.work_dir.as_os_str().is_empty() {
            return Err("Supervisor work directory cannot be empty".to_string());
        }

        Ok(())
    }
}

/// Position of the supervisor pane.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SupervisorPosition {
    Top,
    Bottom,
    Left,
    Right,
    Hidden,
}

/// Size specification for supervisor pane.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SupervisorSize {
    /// Fixed size in pixels
    Fixed(u32),
    /// Percentage of available space
    Percentage(u8),
    /// Auto-size based on content
    Auto,
}

/// Session layout configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct SessionLayout {
    /// Layout type: tiled, main-vertical, main-horizontal, custom
    pub layout_type: LayoutType,

    /// Pane arrangement specifications
    pub pane_config: Vec<PaneConfig>,

    /// Session dimensions
    pub dimensions: Option<SessionDimensions>,
}

/// Layout type enumeration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(dead_code)]
pub enum LayoutType {
    /// All panes arranged in a grid
    Tiled,
    /// One main vertical pane with smaller panes on the side
    MainVertical,
    /// One main horizontal pane with smaller panes below
    MainHorizontal,
    /// Custom layout configuration
    Custom(String),
}

/// Configuration for individual panes in a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct PaneConfig {
    /// Agent name for this pane
    pub agent_name: String,

    /// Pane size/position specifications
    pub size: PaneSize,

    /// Pane index in layout
    pub index: u32,
}

/// Session dimensions for layout calculations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct SessionDimensions {
    /// Width in characters
    pub width: u32,

    /// Height in characters
    pub height: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let profile = TmuxProfile {
            name: "test-profile".to_string(),
            description: "Test profile".to_string(),
            best_for_agent_count: Some(3),
            layout: ProfileLayout {
                layout_type: "tiled".to_string(),
                pane_sizes: vec![PaneSize {
                    agent_id: "agent-1".to_string(),
                    size_spec: "50%".to_string(),
                    position: LayoutPosition::Top,
                }],
                root_pane: None,
            },
            script_path: PathBuf::from("test.sh"),
            includes_supervisor: false,
            supervisor_config: None,
        };

        let session = Session::new("test-session", profile);
        assert_eq!(session.name, "test-session");
        assert_eq!(session.status, SessionStatus::Creating);
        assert_eq!(session.profile, "test-profile");
    }

    #[test]
    fn test_session_validation() {
        let profile = TmuxProfile {
            name: "test-profile".to_string(),
            description: "Test profile".to_string(),
            best_for_agent_count: Some(3),
            layout: ProfileLayout {
                layout_type: "tiled".to_string(),
                pane_sizes: vec![PaneSize {
                    agent_id: "agent-1".to_string(),
                    size_spec: "50%".to_string(),
                    position: LayoutPosition::Top,
                }],
                root_pane: None,
            },
            script_path: PathBuf::from("test.sh"),
            includes_supervisor: false,
            supervisor_config: None,
        };

        let session = Session::new("valid-session", profile.clone());
        assert!(session.validate().is_ok());

        // Test empty name
        let mut invalid_session = Session::new("", profile.clone());
        assert!(invalid_session.validate().is_err());

        // Test invalid name format
        invalid_session.name = "invalid session!".to_string();
        assert!(invalid_session.validate().is_err());
    }

    #[test]
    fn test_profile_validation() {
        let valid_profile = TmuxProfile {
            name: "profile0".to_string(),
            description: "Valid profile".to_string(),
            best_for_agent_count: Some(3),
            layout: ProfileLayout {
                layout_type: "tiled".to_string(),
                pane_sizes: vec![PaneSize {
                    agent_id: "agent-1".to_string(),
                    size_spec: "50%".to_string(),
                    position: LayoutPosition::Top,
                }],
                root_pane: None,
            },
            script_path: PathBuf::from("profile0.sh"),
            includes_supervisor: false,
            supervisor_config: None,
        };

        assert!(valid_profile.validate().is_ok());

        // Test empty layout type
        let mut invalid_profile = valid_profile.clone();
        invalid_profile.layout.layout_type = "".to_string();
        assert!(invalid_profile.validate().is_err());

        // Test invalid layout type
        invalid_profile.layout.layout_type = "invalid".to_string();
        assert!(invalid_profile.validate().is_err());
    }

    #[test]
    fn test_pane_size_validation() {
        let valid_pane = PaneSize {
            agent_id: "agent-1".to_string(),
            size_spec: "50%".to_string(),
            position: LayoutPosition::Top,
        };

        assert!(valid_pane.validate().is_ok());

        // Test empty agent ID
        let mut invalid_pane = valid_pane.clone();
        invalid_pane.agent_id = "".to_string();
        assert!(invalid_pane.validate().is_err());

        // Test invalid size spec
        invalid_pane.agent_id = "agent-1".to_string();
        invalid_pane.size_spec = "50".to_string(); // Missing % or px
        assert!(invalid_pane.validate().is_err());
    }

    #[test]
    fn test_session_settings() {
        let settings = SessionSettings::default();
        assert!(!settings.auto_sync);
        assert_eq!(settings.sync_interval_secs, Some(300));
        assert_eq!(settings.conflict_resolution, ConflictResolution::Manual);
        assert!(settings.enable_monitoring);
        assert!(settings.auto_cleanup);
    }
}
