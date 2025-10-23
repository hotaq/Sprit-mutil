use crate::config::SpriteConfig;
use crate::error::SpriteError;
use crate::utils::tmux;
use crate::communication::{DeliveryConfirmation, DeliveryConfig, DeliveryTracking};
use crate::models::MessagePriority;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::process::Command as TokioCommand;
use tokio::time::timeout;

/// Represents a command sent to an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandMessage {
    /// Unique message identifier
    pub id: String,
    /// Message timestamp
    pub timestamp: u64,
    /// Target agent name (empty for broadcast)
    pub target_agent: Option<String>,
    /// Command to execute
    pub command: String,
    /// Command arguments
    pub args: Vec<String>,
    /// Working directory for command
    pub work_dir: Option<PathBuf>,
    /// Environment variables for this command
    pub env_vars: HashMap<String, String>,
    /// Timeout for this command in seconds
    pub timeout_secs: Option<u64>,
    /// Message priority
    pub priority: MessagePriority,
}

/// Represents the result of command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Corresponding command message ID
    pub command_id: String,
    /// Agent that executed the command
    pub agent_name: String,
    /// Execution status
    pub status: ExecutionStatus,
    /// Command output (stdout)
    pub output: String,
    /// Error output (stderr)
    pub error_output: String,
    /// Exit code
    pub exit_code: i32,
    /// Execution start time
    pub start_time: u64,
    /// Execution end time
    pub end_time: u64,
    /// Resource usage during execution
    pub resource_usage: ResourceUsage,
    /// Agent responsiveness information
    pub responsiveness: ResponsivenessInfo,
}

// Message priority is now defined in models/mod.rs

/// Execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Success,
    Failed,
    Timeout,
    Cancelled,
}

/// Resource usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// CPU usage percentage
    pub cpu_percent: f64,
    /// Memory usage in MB
    pub memory_mb: u64,
    /// Disk usage in MB
    pub disk_mb: u64,
}

/// Agent responsiveness information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponsivenessInfo {
    /// Time to first response in milliseconds
    pub response_time_ms: u64,
    /// Whether the agent was responsive
    pub was_responsive: bool,
    /// Number of ping attempts made
    pub ping_attempts: u32,
}

/// Agent targeting information
#[derive(Debug, Clone)]
pub struct AgentTarget {
    /// Agent identifier
    pub id: String,
    /// Agent name/alias
    pub name: String,
    /// Tmux pane identifier
    pub tmux_pane: String,
    /// Agent workspace path
    pub workspace_path: PathBuf,
    /// Current agent status
    pub status: AgentStatus,
    /// Last activity timestamp
    pub last_activity: Option<u64>,
}

/// Agent status for communication purposes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentStatus {
    /// Agent is ready to receive commands
    Ready,
    /// Agent is currently executing a command
    Busy,
    /// Agent is unresponsive
    Unresponsive,
    /// Agent encountered an error
    Error(String),
    /// Agent is offline/not found
    Offline,
}

/// Broadcast execution configuration
#[derive(Debug, Clone)]
pub struct BroadcastConfig {
    /// Execute commands in parallel
    pub parallel: bool,
    /// Maximum concurrent executions
    pub max_concurrent: usize,
    /// Default timeout for all commands
    pub default_timeout: Duration,
    /// Whether to continue on individual failures
    pub continue_on_failure: bool,
    /// Performance monitoring enabled
    pub monitor_performance: bool,
}

/// Default broadcast configuration
impl Default for BroadcastConfig {
    fn default() -> Self {
        Self {
            parallel: true,
            max_concurrent: 10,
            default_timeout: Duration::from_secs(30),
            continue_on_failure: true,
            monitor_performance: true,
        }
    }
}

/// Agent targeting and validation logic
pub struct AgentResolver;

impl AgentResolver {
    /// Resolve agent identifier to targeting information
    pub fn resolve_agent(
        agent_id: &str,
        config: &SpriteConfig,
    ) -> Result<AgentTarget> {
        // Try to find agent by ID first
        if let Some(agent_config) = config.agents.get(agent_id) {
            return Ok(AgentTarget {
                id: agent_id.to_string(),
                name: agent_config.description.clone().unwrap_or_else(|| agent_id.to_string()),
                tmux_pane: format!("{}:0.{}", config.session.name, agent_id),
                workspace_path: agent_config.worktree_path.clone()
                    .unwrap_or_else(|| PathBuf::from(format!("agents/{}", agent_id))),
                status: AgentStatus::Ready,
                last_activity: None,
            });
        }

        // Try to find agent by description/name
        for (id, agent_config) in &config.agents {
            if let Some(desc) = &agent_config.description {
                if desc.to_lowercase().contains(&agent_id.to_lowercase()) {
                    return Ok(AgentTarget {
                        id: id.clone(),
                        name: desc.clone(),
                        tmux_pane: format!("{}:0.{}", config.session.name, id),
                        workspace_path: agent_config.worktree_path.clone()
                            .unwrap_or_else(|| PathBuf::from(format!("agents/{}", id))),
                        status: AgentStatus::Ready,
                        last_activity: None,
                    });
                }
            }
        }

        Err(SpriteError::agent(
            format!("Agent '{}' not found. Available agents: {}",
                    agent_id,
                    config.agents.keys().cloned().collect::<Vec<_>>().join(", ")
            ),
            agent_id.to_string(),
        ).into())
    }

    /// Validate that agent is ready for communication
    pub fn validate_agent_readiness(target: &AgentTarget) -> Result<()> {
        match target.status {
            AgentStatus::Ready => Ok(()),
            AgentStatus::Busy => Err(SpriteError::agent(
                format!("Agent '{}' is currently busy and cannot accept commands", target.name),
                target.id.clone(),
            ).into()),
            AgentStatus::Unresponsive => Err(SpriteError::agent(
                format!("Agent '{}' is unresponsive", target.name),
                target.id.clone(),
            ).into()),
            AgentStatus::Error(ref msg) => Err(SpriteError::agent(
                format!("Agent '{}' has an error: {}", target.name, msg),
                target.id.clone(),
            ).into()),
            AgentStatus::Offline => Err(SpriteError::agent(
                format!("Agent '{}' is offline", target.name),
                target.id.clone(),
            ).into()),
        }
    }

    /// Get all available agents for broadcast
    pub fn get_all_agents(config: &SpriteConfig) -> Result<Vec<AgentTarget>> {
        let mut agents = Vec::new();

        for (id, agent_config) in &config.agents {
            agents.push(AgentTarget {
                id: id.clone(),
                name: agent_config.description.clone().unwrap_or_else(|| id.to_string()),
                tmux_pane: format!("{}:0.{}", config.session.name, id),
                workspace_path: agent_config.worktree_path.clone()
                    .unwrap_or_else(|| PathBuf::from(format!("agents/{}", id))),
                status: AgentStatus::Ready,
                last_activity: None,
            });
        }

        if agents.is_empty() {
            return Err(SpriteError::agent(
                "No agents configured".to_string(),
                "none".to_string(),
            ).into());
        }

        Ok(agents)
    }
}

/// Command execution engine
pub struct CommandExecutor {
    /// Delivery confirmation system
    delivery_confirmation: Arc<DeliveryConfirmation>,
}

impl CommandExecutor {
    /// Create new command executor with delivery confirmation
    pub fn new() -> Self {
        let delivery_config = DeliveryConfig::default();
        let delivery_confirmation = Arc::new(DeliveryConfirmation::new(delivery_config));
        
        Self {
            delivery_confirmation,
        }
    }

    /// Create command executor with custom delivery configuration
    pub fn with_delivery_config(config: DeliveryConfig) -> Self {
        let delivery_confirmation = Arc::new(DeliveryConfirmation::new(config));
        
        Self {
            delivery_confirmation,
        }
    }

    /// Get reference to delivery confirmation system
    pub fn delivery_confirmation(&self) -> &Arc<DeliveryConfirmation> {
        &self.delivery_confirmation
    }
}

impl Default for CommandExecutor {
    fn default() -> Self {
        Self::new()
    }
}

    /// Send command to specific agent with delivery confirmation
    pub async fn send_to_agent_with_confirmation(
        &self,
        target: &AgentTarget,
        command: &str,
        args: &[String],
        env_vars: &HashMap<String, String>,
        timeout_secs: u64,
    ) -> Result<(ExecutionResult, DeliveryTracking)> {
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("Failed to get system time")?
            .as_secs();

        let command_id = uuid::Uuid::new_v4().to_string();

        // Prepare full command
        let full_command = if args.is_empty() {
            command.to_string()
        } else {
            format!("{} {}", command, args.join(" "))
        };

        // Send with delivery confirmation
        let delivery_tracking = self.delivery_confirmation
            .send_with_confirmation(
                command_id.clone(),
                &target.id,
                &target.tmux_pane,
                &full_command,
                MessagePriority::Normal, // Default to normal priority
            ).await?;

        // Create execution result based on delivery tracking
        let (status, output, error_output, exit_code) = match delivery_tracking.status {
            crate::communication::DeliveryStatus::Delivered => {
                (ExecutionStatus::Success, 
                 "Command delivered successfully".to_string(),
                 String::new(), 0)
            }
            crate::communication::DeliveryStatus::Failed => {
                (ExecutionStatus::Failed,
                 String::new(),
                 "Delivery failed".to_string(), 1)
            }
            crate::communication::DeliveryStatus::Timeout => {
                (ExecutionStatus::Timeout,
                 String::new(),
                 "Delivery timeout".to_string(), 124)
            }
            crate::communication::DeliveryStatus::Retrying => {
                (ExecutionStatus::Failed,
                 String::new(),
                 "Delivery is being retried".to_string(), 1)
            }
            _ => {
                (ExecutionStatus::Failed,
                 String::new(),
                 "Unknown delivery status".to_string(), 1)
            }
        };

        let end_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("Failed to get system time")?
            .as_secs();

        let execution_result = ExecutionResult {
            command_id,
            agent_name: target.name.clone(),
            status,
            output,
            error_output,
            exit_code,
            start_time,
            end_time,
            resource_usage: ResourceUsage {
                cpu_percent: 0.0,
                memory_mb: 0,
                disk_mb: 0,
            },
            responsiveness: ResponsivenessInfo {
                response_time_ms: delivery_tracking.last_response_time().unwrap_or(0),
                was_responsive: matches!(delivery_tracking.status, crate::communication::DeliveryStatus::Delivered),
                ping_attempts: delivery_tracking.total_attempts(),
            },
        };

        Ok((execution_result, delivery_tracking))
    }

    /// Send command to specific agent (legacy method without delivery confirmation)
    pub async fn send_to_agent(
        &self,
        target: &AgentTarget,
        command: &str,
        args: &[String],
        env_vars: &HashMap<String, String>,
        timeout_secs: u64,
    ) -> Result<ExecutionResult> {
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("Failed to get system time")?
            .as_secs();

        let command_id = uuid::Uuid::new_v4().to_string();

        // Check agent responsiveness
        let responsiveness = self.check_agent_responsiveness(target, timeout_secs).await?;

        if !responsiveness.was_responsive {
            return Ok(ExecutionResult {
                command_id,
                agent_name: target.name.clone(),
                status: ExecutionStatus::Timeout,
                output: String::new(),
                error_output: format!("Agent '{}' was unresponsive", target.name),
                exit_code: 124, // Standard timeout exit code
                start_time,
                end_time: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .context("Failed to get system time")?
                    .as_secs(),
                resource_usage: ResourceUsage {
                    cpu_percent: 0.0,
                    memory_mb: 0,
                    disk_mb: 0,
                },
                responsiveness,
            });
        }

        // Prepare full command with arguments
        let full_command = if args.is_empty() {
            command.to_string()
        } else {
            format!("{} {}", command, args.join(" "))
        };

        // Send command to tmux pane
        let mut tmux_args = vec![
            "send-keys",
            "-t",
            &target.tmux_pane,
            &full_command,
            "C-m",
        ];

        // Set working directory if specified
        if let Some(work_dir) = &target.workspace_path {
            let cd_command = format!("cd {}", work_dir.display());
            tmux::send_command_to_pane(&target.session_name(), &target.pane_index(), &cd_command)?;
        }

        // Set environment variables
        for (key, value) in env_vars {
            let export_command = format!("export {}={}", key, value);
            tmux::send_command_to_pane(&target.session_name(), &target.pane_index(), &export_command)?;
        }

        // Send the actual command
        let execution_result = timeout(
            Duration::from_secs(timeout_secs),
            self.execute_command_in_pane(&target.tmux_pane, &full_command)
        ).await;

        let end_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("Failed to get system time")?
            .as_secs();

        match execution_result {
            Ok(result) => result,
            Err(_) => Ok(ExecutionResult {
                command_id,
                agent_name: target.name.clone(),
                status: ExecutionStatus::Timeout,
                output: String::new(),
                error_output: format!("Command timed out after {} seconds", timeout_secs),
                exit_code: 124,
                start_time,
                end_time,
                resource_usage: ResourceUsage {
                    cpu_percent: 0.0,
                    memory_mb: 0,
                    disk_mb: 0,
                },
                responsiveness,
            }),
        }
    }

    /// Broadcast command to multiple agents
    pub async fn broadcast_command(
        &self,
        targets: &[AgentTarget],
        command: &str,
        args: &[String],
        env_vars: &HashMap<String, String>,
        config: &BroadcastConfig,
    ) -> Result<Vec<ExecutionResult>> {
        let command_id = uuid::Uuid::new_v4().to_string();
        let start_time = Instant::now();

        if config.parallel {
            // Execute commands in parallel with concurrency limit
            let semaphore = tokio::sync::Semaphore::new(config.max_concurrent);
            let mut tasks = Vec::new();

            for target in targets {
                let permit = semaphore.acquire().await?;
                let command = command.to_string();
                let args = args.to_vec();
                let env_vars = env_vars.clone();
                let timeout = config.default_timeout;
                let target = target.clone();

                let task = tokio::spawn(async move {
                    let _permit = permit;
                    // Create a temporary executor for this task
                    let executor = CommandExecutor::new();
                    executor.send_to_agent(
                        &target,
                        &command,
                        &args,
                        &env_vars,
                        timeout.as_secs(),
                    ).await
                });

                tasks.push(task);
            }

            // Wait for all tasks to complete
            let results = futures::future::join_all(tasks).await;
            let mut execution_results = Vec::new();

            for result in results {
                match result {
                    Ok(execution_result) => execution_results.push(execution_result?),
                    Err(e) => {
                        if config.continue_on_failure {
                            eprintln!("Warning: Failed to execute command: {}", e);
                        } else {
                            return Err(SpriteError::agent(
                                format!("Broadcast execution failed: {}", e),
                                "broadcast".to_string(),
                            ).into());
                        }
                    }
                }
            }

            Ok(execution_results)
        } else {
            // Execute commands sequentially
            let mut results = Vec::new();

            for target in targets {
                let result = Self::send_to_agent(
                    target,
                    command,
                    args,
                    env_vars,
                    config.default_timeout.as_secs(),
                ).await?;

                if !config.continue_on_failure
                    && matches!(result.status, ExecutionStatus::Failed | ExecutionStatus::Timeout) {
                    return Ok(results);
                }

                results.push(result);
            }

            Ok(results)
        }
    }

    /// Check if agent is responsive
    async fn check_agent_responsiveness(
        &self,
        target: &AgentTarget,
        timeout_secs: u64,
    ) -> Result<ResponsivenessInfo> {
        let start = Instant::now();
        let mut ping_attempts = 0;
        let max_attempts = 3;

        for attempt in 1..=max_attempts {
            ping_attempts = attempt;

            // Send a simple echo command to test responsiveness
            let test_command = "echo 'sprite-ping-test'";

            match timeout(
                Duration::from_secs(1),
                self.ping_agent(&target.tmux_pane, test_command)
            ).await {
                Ok(Ok(_)) => {
                    return Ok(ResponsivenessInfo {
                        response_time_ms: start.elapsed().as_millis() as u64,
                        was_responsive: true,
                        ping_attempts,
                    });
                }
                Ok(Err(_)) => {
                    // Try again
                }
                Err(_) => {
                    // Timeout, try again
                }
            }

            // Wait a bit before retrying
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Ok(ResponsivenessInfo {
            response_time_ms: timeout_secs * 1000,
            was_responsive: false,
            ping_attempts,
        })
    }

    /// Simple ping to check if agent pane is responsive
    async fn ping_agent(&self, pane: &str, command: &str) -> Result<()> {
        // This is a simplified implementation
        // In a real scenario, we'd capture output from the tmux pane
        tmux::send_command_to_pane("", pane, command)?;
        Ok(())
    }

    /// Execute command and capture output from tmux pane
    async fn execute_command_in_pane(
        &self,
        pane: &str,
        command: &str,
    ) -> Result<ExecutionResult> {
        // This is a simplified implementation
        // In practice, we'd need more sophisticated output capture
        tmux::send_command_to_pane("", pane, command)?;

        // For now, return a success result
        // TODO: Implement proper output capture
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("Failed to get system time")?
            .as_secs();

        Ok(ExecutionResult {
            command_id: uuid::Uuid::new_v4().to_string(),
            agent_name: pane.to_string(),
            status: ExecutionStatus::Success,
            output: "Command executed".to_string(),
            error_output: String::new(),
            exit_code: 0,
            start_time: now,
            end_time: now,
            resource_usage: ResourceUsage {
                cpu_percent: 0.0,
                memory_mb: 0,
                disk_mb: 0,
            },
            responsiveness: ResponsivenessInfo {
                response_time_ms: 100,
                was_responsive: true,
                ping_attempts: 1,
            },
        })
    }
}

impl AgentTarget {
    /// Extract session name from tmux_pane
    pub fn session_name(&self) -> String {
        self.tmux_pane.split(':').next().unwrap_or("").to_string()
    }

    /// Extract pane index from tmux_pane
    pub fn pane_index(&self) -> String {
        self.tmux_pane.split('.').last().unwrap_or("0").to_string()
    }
}

/// Command history tracking with accessibility compliance
pub struct CommandHistory {
    entries: Vec<CommandMessage>,
    max_entries: usize,
}

impl CommandHistory {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::with_capacity(max_entries),
            max_entries,
        }
    }

    /// Add command to history
    pub fn add_command(&mut self, command: CommandMessage) {
        if self.entries.len() >= self.max_entries {
            self.entries.remove(0);
        }
        self.entries.push(command);
    }

    /// Get recent commands
    pub fn get_recent(&self, count: usize) -> &[CommandMessage] {
        let start = if self.entries.len() > count {
            self.entries.len() - count
        } else {
            0
        };
        &self.entries[start..]
    }

    /// Get commands for specific agent
    pub fn get_for_agent(&self, agent_id: &str) -> Vec<&CommandMessage> {
        self.entries
            .iter()
            .filter(|cmd| cmd.target_agent.as_ref().map_or(false, |target| target == agent_id))
            .collect()
    }

    /// Format history for screen readers
    pub fn format_for_screen_reader(&self) -> String {
        let mut output = String::new();

        if self.entries.is_empty() {
            output.push_str("No commands in history.\n");
        } else {
            output.push_str(&format!("Command history: {} entries.\n", self.entries.len()));

            for (index, cmd) in self.entries.iter().enumerate() {
                output.push_str(&format!(
                    "Entry {}: Command '{}' sent to agent '{}' at time {}. Priority: {}. ",
                    index + 1,
                    cmd.command,
                    cmd.target_agent.as_deref().unwrap_or("all"),
                    cmd.timestamp,
                    match cmd.priority {
                        MessagePriority::Low => "low",
                        MessagePriority::Normal => "normal",
                        MessagePriority::High => "high",
                        MessagePriority::Critical => "critical",
                    }
                ));

                if !cmd.args.is_empty() {
                    output.push_str(&format!("Arguments: {}. ", cmd.args.join(" ")));
                }

                output.push('\n');
            }
        }

        output
    }

    /// Export history to accessible format
    pub fn export_accessible(&self, format: AccessibleFormat) -> Result<String> {
        match format {
            AccessibleFormat::PlainText => Ok(self.format_for_screen_reader()),
            AccessibleFormat::Markdown => self.export_markdown(),
            AccessibleFormat::Json => self.export_json(),
        }
    }

    fn export_markdown(&self) -> Result<String> {
        let mut output = String::new();
        output.push_str("# Command History\n\n");

        if self.entries.is_empty() {
            output.push_str("*No commands in history*\n");
        } else {
            output.push_str("| # | Command | Agent | Time | Priority |\n");
            output.push_str("|---|---------|-------|------|----------|\n");

            for (index, cmd) in self.entries.iter().enumerate() {
                output.push_str(&format!(
                    "| {} | {} | {} | {} | {} |\n",
                    index + 1,
                    cmd.command,
                    cmd.target_agent.as_deref().unwrap_or("all"),
                    cmd.timestamp,
                    match cmd.priority {
                        MessagePriority::Low => "low",
                        MessagePriority::Normal => "normal",
                        MessagePriority::High => "high",
                        MessagePriority::Critical => "critical",
                    }
                ));
            }
        }

        Ok(output)
    }

    fn export_json(&self) -> Result<String> {
        serde_json::to_string_pretty(&self.entries)
            .context("Failed to serialize command history")
    }
}

/// Accessible export formats
#[derive(Debug, Clone)]
pub enum AccessibleFormat {
    PlainText,
    Markdown,
    Json,
}

/// Performance monitoring for broadcasts
pub struct PerformanceMonitor {
    metrics: HashMap<String, PerformanceMetrics>,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
        }
    }

    /// Record execution metrics
    pub fn record_execution(&mut self, result: &ExecutionResult) {
        let metrics = self.metrics.entry(result.agent_name.clone()).or_insert_with(|| {
            PerformanceMetrics::new(result.agent_name.clone())
        });

        metrics.record_execution(result);
    }

    /// Get performance report
    pub fn get_report(&self) -> String {
        if self.metrics.is_empty() {
            return "No performance data available.".to_string();
        }

        let mut report = String::new();
        report.push_str("Performance Report:\n\n");

        for (agent_name, metrics) in &self.metrics {
            report.push_str(&metrics.format_report());
            report.push('\n');
        }

        report
    }

    /// Get performance summary for accessibility
    pub fn get_accessible_summary(&self) -> String {
        if self.metrics.is_empty() {
            return "No performance data available.".to_string();
        }

        let mut summary = String::new();
        summary.push_str(&format!("Performance summary for {} agents:\n\n", self.metrics.len()));

        for (agent_name, metrics) in &self.metrics {
            summary.push_str(&format!(
                "Agent {}: {} commands executed, average response time {} milliseconds, success rate {:.1} percent. ",
                agent_name,
                metrics.total_executions,
                metrics.average_response_time_ms(),
                metrics.success_rate() * 100.0
            ));
        }

        summary
    }
}

/// Performance metrics for a single agent
#[derive(Debug)]
pub struct PerformanceMetrics {
    agent_name: String,
    total_executions: u64,
    successful_executions: u64,
    failed_executions: u64,
    timeout_executions: u64,
    total_response_time_ms: u64,
    min_response_time_ms: u64,
    max_response_time_ms: u64,
}

impl PerformanceMetrics {
    pub fn new(agent_name: String) -> Self {
        Self {
            agent_name,
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            timeout_executions: 0,
            total_response_time_ms: 0,
            min_response_time_ms: u64::MAX,
            max_response_time_ms: 0,
        }
    }

    pub fn record_execution(&mut self, result: &ExecutionResult) {
        self.total_executions += 1;

        match result.status {
            ExecutionStatus::Success => self.successful_executions += 1,
            ExecutionStatus::Failed => self.failed_executions += 1,
            ExecutionStatus::Timeout => self.timeout_executions += 1,
            ExecutionStatus::Cancelled => self.failed_executions += 1,
        }

        let response_time = result.responsiveness.response_time_ms;
        self.total_response_time_ms += response_time;
        self.min_response_time_ms = self.min_response_time_ms.min(response_time);
        self.max_response_time_ms = self.max_response_time_ms.max(response_time);
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_executions == 0 {
            0.0
        } else {
            self.successful_executions as f64 / self.total_executions as f64
        }
    }

    pub fn average_response_time_ms(&self) -> f64 {
        if self.total_executions == 0 {
            0.0
        } else {
            self.total_response_time_ms as f64 / self.total_executions as f64
        }
    }

    pub fn format_report(&self) -> String {
        format!(
            "Agent: {}\n\
             Total Executions: {}\n\
             Successful: {} ({:.1}%)\n\
             Failed: {} ({:.1}%)\n\
             Timeout: {} ({:.1}%)\n\
             Avg Response Time: {:.1}ms\n\
             Min/Max Response Time: {}ms / {}ms\n",
            self.agent_name,
            self.total_executions,
            self.successful_executions,
            self.success_rate() * 100.0,
            self.failed_executions,
            (self.failed_executions as f64 / self.total_executions as f64) * 100.0,
            self.timeout_executions,
            (self.timeout_executions as f64 / self.total_executions as f64) * 100.0,
            self.average_response_time_ms(),
            if self.min_response_time_ms == u64::MAX { 0 } else { self.min_response_time_ms },
            self.max_response_time_ms
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_command_message_creation() {
        let message = CommandMessage {
            id: "test-123".to_string(),
            timestamp: 1234567890,
            target_agent: Some("agent-1".to_string()),
            command: "echo".to_string(),
            args: vec!["hello".to_string()],
            work_dir: Some(PathBuf::from("/tmp")),
            env_vars: HashMap::new(),
            timeout_secs: Some(30),
            priority: MessagePriority::Normal,
        };

        assert_eq!(message.id, "test-123");
        assert_eq!(message.target_agent, Some("agent-1".to_string()));
        assert_eq!(message.command, "echo");
        assert_eq!(message.args, vec!["hello"]);
    }

    #[test]
    fn test_execution_result_creation() {
        let result = ExecutionResult {
            command_id: "test-456".to_string(),
            agent_name: "agent-1".to_string(),
            status: ExecutionStatus::Success,
            output: "hello world".to_string(),
            error_output: String::new(),
            exit_code: 0,
            start_time: 1234567890,
            end_time: 1234567900,
            resource_usage: ResourceUsage {
                cpu_percent: 50.0,
                memory_mb: 128,
                disk_mb: 256,
            },
            responsiveness: ResponsivenessInfo {
                response_time_ms: 100,
                was_responsive: true,
                ping_attempts: 1,
            },
        };

        assert_eq!(result.command_id, "test-456");
        assert_eq!(result.status, ExecutionStatus::Success);
        assert_eq!(result.output, "hello world");
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_agent_target_parsing() {
        let target = AgentTarget {
            id: "1".to_string(),
            name: "Test Agent".to_string(),
            tmux_pane: "session-name:0.1".to_string(),
            workspace_path: PathBuf::from("agents/1"),
            status: AgentStatus::Ready,
            last_activity: None,
        };

        assert_eq!(target.session_name(), "session-name");
        assert_eq!(target.pane_index(), "1");
    }

    #[test]
    fn test_command_history() {
        let mut history = CommandHistory::new(3);

        let command1 = CommandMessage {
            id: "1".to_string(),
            timestamp: 1000,
            target_agent: Some("agent-1".to_string()),
            command: "ls".to_string(),
            args: vec![],
            work_dir: None,
            env_vars: HashMap::new(),
            timeout_secs: None,
            priority: MessagePriority::Normal,
        };

        history.add_command(command1);
        assert_eq!(history.get_recent(1).len(), 1);

        // Test screen reader formatting
        let formatted = history.format_for_screen_reader();
        assert!(formatted.contains("Command history: 1 entries"));
        assert!(formatted.contains("Command 'ls'"));
    }

    #[test]
    fn test_performance_metrics() {
        let mut metrics = PerformanceMetrics::new("test-agent".to_string());

        let result = ExecutionResult {
            command_id: "test".to_string(),
            agent_name: "test-agent".to_string(),
            status: ExecutionStatus::Success,
            output: "success".to_string(),
            error_output: String::new(),
            exit_code: 0,
            start_time: 1000,
            end_time: 1100,
            resource_usage: ResourceUsage {
                cpu_percent: 50.0,
                memory_mb: 128,
                disk_mb: 256,
            },
            responsiveness: ResponsivenessInfo {
                response_time_ms: 100,
                was_responsive: true,
                ping_attempts: 1,
            },
        };

        metrics.record_execution(&result);
        assert_eq!(metrics.total_executions, 1);
        assert_eq!(metrics.successful_executions, 1);
        assert_eq!(metrics.success_rate(), 1.0);
        assert_eq!(metrics.average_response_time_ms(), 100.0);
    }

    #[test]
    fn test_message_priority_ordering() {
        assert!(MessagePriority::Critical > MessagePriority::High);
        assert!(MessagePriority::High > MessagePriority::Normal);
        assert!(MessagePriority::Normal > MessagePriority::Low);
    }

    #[test]
    fn test_broadcast_config_default() {
        let config = BroadcastConfig::default();
        assert!(config.parallel);
        assert_eq!(config.max_concurrent, 10);
        assert_eq!(config.default_timeout, Duration::from_secs(30));
        assert!(config.continue_on_failure);
        assert!(config.monitor_performance);
    }
}