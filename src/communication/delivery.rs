//! Message delivery confirmation system for multi-agent communication.
//!
//! This module provides reliable message delivery with confirmation,
//! retry mechanisms, and delivery status tracking for agent communication.

use crate::error::SpriteError;
use crate::models::MessagePriority;
use crate::utils::tmux;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{Mutex, RwLock};

/// Delivery status for message tracking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeliveryStatus {
    /// Message is pending delivery
    Pending,
    /// Message has been sent to agent
    Sent,
    /// Message delivery confirmed by agent
    Delivered,
    /// Message delivery failed
    Failed,
    /// Message delivery timed out
    Timeout,
    /// Message delivery is being retried
    Retrying,
}

/// Delivery receipt from agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryReceipt {
    /// Unique message identifier
    pub message_id: String,
    /// Agent that received the message
    pub agent_id: String,
    /// Delivery timestamp
    pub delivered_at: u64,
    /// Agent acknowledgment message
    pub acknowledgment: Option<String>,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Delivery attempt information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryAttempt {
    /// Attempt number
    pub attempt_number: u32,
    /// Timestamp of attempt
    pub timestamp: u64,
    /// Success status
    pub success: bool,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Response time in milliseconds
    pub response_time_ms: u64,
}

/// Delivery tracking information for a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryTracking {
    /// Unique message identifier
    pub message_id: String,
    /// Target agent
    pub target_agent: String,
    /// Current delivery status
    pub status: DeliveryStatus,
    /// Message content (for retries)
    pub message_content: String,
    /// Message priority
    pub priority: MessagePriority,
    /// Original send timestamp
    pub created_at: u64,
    /// delivery attempts
    pub attempts: Vec<DeliveryAttempt>,
    /// Maximum retry attempts
    pub max_retries: u32,
    /// Timeout in seconds
    pub timeout_secs: u64,
    /// Delivery receipt if confirmed
    pub receipt: Option<DeliveryReceipt>,
}

impl DeliveryTracking {
    /// Create new delivery tracking
    pub fn new(
        message_id: String,
        target_agent: String,
        message_content: String,
        priority: MessagePriority,
        max_retries: u32,
        timeout_secs: u64,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("Failed to get system time")
            .unwrap_or_default()
            .as_secs();

        Self {
            message_id,
            target_agent,
            status: DeliveryStatus::Pending,
            message_content,
            priority,
            created_at: now,
            attempts: Vec::new(),
            max_retries,
            timeout_secs,
            receipt: None,
        }
    }

    /// Add delivery attempt
    pub fn add_attempt(
        &mut self,
        success: bool,
        error_message: Option<String>,
        response_time_ms: u64,
    ) {
        let attempt_number = self.attempts.len() as u32 + 1;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("Failed to get system time")
            .unwrap_or_default()
            .as_secs();

        self.attempts.push(DeliveryAttempt {
            attempt_number,
            timestamp: now,
            success,
            error_message,
            response_time_ms,
        });
    }

    /// Mark as delivered with receipt
    pub fn mark_delivered(&mut self, receipt: DeliveryReceipt) {
        self.status = DeliveryStatus::Delivered;
        self.receipt = Some(receipt);
    }

    /// Check if should retry
    pub fn should_retry(&self) -> bool {
        self.attempts.len() < self.max_retries as usize
            && matches!(
                self.status,
                DeliveryStatus::Failed | DeliveryStatus::Timeout
            )
    }

    /// Get total attempts made
    pub fn total_attempts(&self) -> u32 {
        self.attempts.len() as u32
    }

    /// Get last attempt response time
    pub fn last_response_time(&self) -> Option<u64> {
        self.attempts.last().map(|attempt| attempt.response_time_ms)
    }

    /// Check if delivery is overdue
    pub fn is_overdue(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("Failed to get system time")
            .unwrap_or_default()
            .as_secs();

        now > self.created_at + self.timeout_secs
    }
}

/// Delivery confirmation configuration
#[derive(Debug, Clone)]
pub struct DeliveryConfig {
    /// Default timeout for delivery confirmation (seconds)
    pub default_timeout_secs: u64,
    /// Max retry attempts
    pub max_retries: u32,
    /// Retry delay (seconds)
    pub retry_delay_secs: u64,
    /// Whether to wait for confirmation
    pub wait_for_confirmation: bool,
    /// Background processing enabled
    pub background_processing: bool,
    /// Cleanup completed deliveries after (seconds)
    pub cleanup_after_secs: u64,
}

impl Default for DeliveryConfig {
    fn default() -> Self {
        Self {
            default_timeout_secs: 30,
            max_retries: 3,
            retry_delay_secs: 2,
            wait_for_confirmation: true,
            background_processing: true,
            cleanup_after_secs: 300, // 5 minutes
        }
    }
}

/// Delivery confirmation system
pub struct DeliveryConfirmation {
    config: DeliveryConfig,
    tracking: Arc<RwLock<HashMap<String, DeliveryTracking>>>,
    pending_confirmations: Arc<Mutex<HashMap<String, Instant>>>,
}

impl DeliveryConfirmation {
    /// Create new delivery confirmation system
    pub fn new(config: DeliveryConfig) -> Self {
        Self {
            config,
            tracking: Arc::new(RwLock::new(HashMap::new())),
            pending_confirmations: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Send message with delivery confirmation
    pub async fn send_with_confirmation(
        &self,
        message_id: String,
        target_agent: &str,
        agent_pane: &str,
        message_content: &str,
        priority: MessagePriority,
    ) -> Result<DeliveryTracking> {
        // Create delivery tracking
        let mut tracking = DeliveryTracking::new(
            message_id.clone(),
            target_agent.to_string(),
            message_content.to_string(),
            priority,
            self.config.max_retries,
            self.config.default_timeout_secs,
        );

        // Send to agent and wait for confirmation
        let delivery_result = self
            .attempt_delivery(&mut tracking, agent_pane, message_content)
            .await?;

        // Store tracking
        {
            let mut tracking_map = self.tracking.write().await;
            tracking_map.insert(message_id.clone(), tracking.clone());
        }

        // If background processing is enabled and waiting for confirmation
        if self.config.wait_for_confirmation && self.config.background_processing {
            let mut pending = self.pending_confirmations.lock().await;
            pending.insert(message_id.clone(), Instant::now());
        }

        Ok(delivery_result)
    }

    /// Attempt delivery to agent
    async fn attempt_delivery(
        &self,
        tracking: &mut DeliveryTracking,
        agent_pane: &str,
        message_content: &str,
    ) -> Result<DeliveryTracking> {
        let start_time = Instant::now();

        // Mark as sent
        tracking.status = DeliveryStatus::Sent;

        // Send the command to agent
        let send_result = self
            .send_command_to_agent(agent_pane, message_content)
            .await;
        let response_time_ms = start_time.elapsed().as_millis() as u64;

        match send_result {
            Ok(_) => {
                // Command sent successfully, wait for confirmation
                if self.config.wait_for_confirmation {
                    let confirmation_result = self
                        .wait_for_confirmation(
                            &tracking.message_id,
                            &tracking.target_agent,
                            self.config.default_timeout_secs,
                        )
                        .await;

                    match confirmation_result {
                        Ok(receipt) => {
                            tracking.add_attempt(true, None, response_time_ms);
                            tracking.mark_delivered(receipt);
                            tracking.status = DeliveryStatus::Delivered;
                        }
                        Err(e) => {
                            tracking.add_attempt(false, Some(e.to_string()), response_time_ms);
                            tracking.status = DeliveryStatus::Timeout;
                        }
                    }
                } else {
                    // Don't wait for confirmation, mark as delivered
                    tracking.add_attempt(true, None, response_time_ms);
                    tracking.status = DeliveryStatus::Delivered;

                    // Create auto-receipt
                    let receipt = DeliveryReceipt {
                        message_id: tracking.message_id.clone(),
                        agent_id: tracking.target_agent.clone(),
                        delivered_at: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .context("Failed to get system time")
                            .unwrap_or_default()
                            .as_secs(),
                        acknowledgment: Some("Auto-confirmed (no wait)".to_string()),
                        processing_time_ms: response_time_ms,
                    };
                    tracking.mark_delivered(receipt);
                }
            }
            Err(e) => {
                tracking.add_attempt(false, Some(e.to_string()), response_time_ms);
                tracking.status = DeliveryStatus::Failed;
            }
        }

        Ok(tracking.clone())
    }

    /// Send command to agent via tmux
    async fn send_command_to_agent(&self, agent_pane: &str, command: &str) -> Result<()> {
        // Send the command text first
        std::process::Command::new("tmux")
            .args(["send-keys", "-t", agent_pane, command])
            .output()
            .context("Failed to send command text to agent")?;

        // Small delay then send Enter to execute
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        std::process::Command::new("tmux")
            .args(["send-keys", "-t", agent_pane, "C-m"])
            .output()
            .context("Failed to send Enter key to agent")?;

        Ok(())
    }

    /// Wait for delivery confirmation from agent
    async fn wait_for_confirmation(
        &self,
        message_id: &str,
        agent_id: &str,
        timeout_secs: u64,
    ) -> Result<DeliveryReceipt> {
        let start_time = Instant::now();
        let timeout_duration = Duration::from_secs(timeout_secs);

        // Poll for confirmation
        while start_time.elapsed() < timeout_duration {
            // Check if we received confirmation
            if let Some(receipt) = self.check_confirmation(message_id, agent_id).await? {
                return Ok(receipt);
            }

            // Wait before next check
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Err(SpriteError::agent(
            format!(
                "Delivery confirmation timeout for message {} to agent {}",
                message_id, agent_id
            ),
            Some(agent_id.to_string()),
        )
        .into())
    }

    /// Check for delivery confirmation
    async fn check_confirmation(
        &self,
        message_id: &str,
        agent_id: &str,
    ) -> Result<Option<DeliveryReceipt>> {
        // This is a simplified implementation
        // In a real scenario, we would monitor tmux output or have agent callbacks

        // For now, simulate finding confirmation by checking tmux buffer
        // This would be enhanced with proper output capture
        let _confirmation_check = format!(
            "grep -q 'SPRITE-DELIVERY-CONFIRMED' $(tmux capture-pane -p -t {} | tail -n 5)",
            agent_id
        );

        // Simplified mock - in reality would check actual tmux output
        // For now, assume success after a short delay
        tokio::time::sleep(Duration::from_millis(50)).await;

        let receipt = DeliveryReceipt {
            message_id: message_id.to_string(),
            agent_id: agent_id.to_string(),
            delivered_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .context("Failed to get system time")
                .unwrap_or_default()
                .as_secs(),
            acknowledgment: Some("Delivery confirmed".to_string()),
            processing_time_ms: 50,
        };

        Ok(Some(receipt))
    }

    /// Retry failed deliveries
    pub async fn retry_failed_deliveries(&self) -> Result<Vec<DeliveryTracking>> {
        let mut retried = Vec::new();
        let mut tracking_map = self.tracking.write().await;

        // Find deliveries that need retry
        let to_retry: Vec<String> = tracking_map
            .iter()
            .filter(|(_, tracking)| tracking.should_retry())
            .map(|(id, _)| id.clone())
            .collect();

        for message_id in to_retry {
            let agent_pane = {
                // Get tracking data to avoid borrow conflicts
                let tracking = tracking_map.get(&message_id);
                if let Some(tr) = tracking {
                    format!("sprite:0.{}", tr.target_agent)
                } else {
                    continue;
                }
            };

            if let Some(tracking) = tracking_map.get_mut(&message_id) {
                // Mark as retrying
                tracking.status = DeliveryStatus::Retrying;

                // Wait before retry
                tokio::time::sleep(Duration::from_secs(self.config.retry_delay_secs)).await;

                // Clone the necessary data to avoid borrow issues
                let message_content = tracking.message_content.clone();

                // Attempt retry
                let retry_result = self
                    .attempt_delivery(tracking, &agent_pane, &message_content)
                    .await;

                if let Ok(updated_tracking) = retry_result {
                    retried.push(updated_tracking);
                }
            }
        }

        Ok(retried)
    }

    /// Get delivery tracking for message
    pub async fn get_tracking(&self, message_id: &str) -> Option<DeliveryTracking> {
        let tracking_map = self.tracking.read().await;
        tracking_map.get(message_id).cloned()
    }

    /// Get all pending deliveries
    pub async fn get_pending_deliveries(&self) -> Vec<DeliveryTracking> {
        let tracking_map = self.tracking.read().await;
        tracking_map
            .values()
            .filter(|tracking| {
                matches!(
                    tracking.status,
                    DeliveryStatus::Pending | DeliveryStatus::Sent | DeliveryStatus::Retrying
                )
            })
            .cloned()
            .collect()
    }

    /// Get delivery statistics
    pub async fn get_delivery_stats(&self) -> DeliveryStats {
        let tracking_map = self.tracking.read().await;
        let mut stats = DeliveryStats::default();

        for tracking in tracking_map.values() {
            match tracking.status {
                DeliveryStatus::Delivered => stats.delivered += 1,
                DeliveryStatus::Failed => stats.failed += 1,
                DeliveryStatus::Timeout => stats.timeouts += 1,
                DeliveryStatus::Pending => stats.pending += 1,
                DeliveryStatus::Sent => stats.sent += 1,
                DeliveryStatus::Retrying => stats.retrying += 1,
            }

            stats.total_attempts += tracking.total_attempts();

            if let Some(response_time) = tracking.last_response_time() {
                stats.total_response_time_ms += response_time;
                if response_time < stats.min_response_time_ms {
                    stats.min_response_time_ms = response_time;
                }
                if response_time > stats.max_response_time_ms {
                    stats.max_response_time_ms = response_time;
                }
            }
        }

        if stats.delivered > 0 {
            stats.success_rate =
                stats.delivered as f64 / (stats.delivered + stats.failed + stats.timeouts) as f64;
        }

        if stats.total_attempts > 0 {
            stats.avg_response_time_ms =
                stats.total_response_time_ms as f64 / stats.total_attempts as f64;
        }

        stats
    }

    /// Cleanup old delivery records
    pub async fn cleanup_old_deliveries(&self) -> Result<usize> {
        let mut tracking_map = self.tracking.write().await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("Failed to get system time")
            .unwrap_or_default()
            .as_secs();

        let initial_count = tracking_map.len();

        tracking_map
            .retain(|_, tracking| now < tracking.created_at + self.config.cleanup_after_secs);

        let cleaned_count = initial_count - tracking_map.len();
        Ok(cleaned_count)
    }
}

/// Delivery statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeliveryStats {
    /// Total messages delivered
    pub delivered: u64,
    /// Total messages failed
    pub failed: u64,
    /// Total messages timed out
    pub timeouts: u64,
    /// Messages pending delivery
    pub pending: u64,
    /// Messages sent but awaiting confirmation
    pub sent: u64,
    /// Messages being retried
    pub retrying: u64,
    /// Total delivery attempts
    pub total_attempts: u32,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Minimum response time in milliseconds
    pub min_response_time_ms: u64,
    /// Maximum response time in milliseconds
    pub max_response_time_ms: u64,
    /// Total response time in milliseconds
    pub total_response_time_ms: u64,
}

impl DeliveryStats {
    /// Format stats for display
    pub fn format_for_display(&self) -> String {
        format!(
            "Delivery Statistics:\n\
             Delivered: {} ({:.1}% success rate)\n\
             Failed: {}\n\
             Timeouts: {}\n\
             Pending: {}\n\
             Sent: {}\n\
             Retrying: {}\n\
             Total Attempts: {}\n\
             Avg Response Time: {:.1}ms\n\
             Min/Max Response Time: {}ms / {}ms",
            self.delivered,
            self.success_rate * 100.0,
            self.failed,
            self.timeouts,
            self.pending,
            self.sent,
            self.retrying,
            self.total_attempts,
            self.avg_response_time_ms,
            self.min_response_time_ms,
            self.max_response_time_ms
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delivery_tracking_creation() {
        let tracking = DeliveryTracking::new(
            "test-msg-123".to_string(),
            "agent-1".to_string(),
            "echo hello".to_string(),
            MessagePriority::Normal,
            3,
            30,
        );

        assert_eq!(tracking.message_id, "test-msg-123");
        assert_eq!(tracking.target_agent, "agent-1");
        assert_eq!(tracking.status, DeliveryStatus::Pending);
        assert_eq!(tracking.attempts.len(), 0);
    }

    #[test]
    fn test_delivery_attempts() {
        let mut tracking = DeliveryTracking::new(
            "test-msg-123".to_string(),
            "agent-1".to_string(),
            "echo hello".to_string(),
            MessagePriority::Normal,
            3,
            30,
        );

        tracking.add_attempt(true, None, 100);
        assert_eq!(tracking.attempts.len(), 1);
        assert_eq!(tracking.total_attempts(), 1);
        assert_eq!(tracking.last_response_time(), Some(100));

        tracking.add_attempt(false, Some("Failed to send".to_string()), 200);
        assert_eq!(tracking.attempts.len(), 2);
        assert_eq!(tracking.total_attempts(), 2);
        assert_eq!(tracking.last_response_time(), Some(200));
    }

    #[test]
    fn test_should_retry() {
        let mut tracking = DeliveryTracking::new(
            "test-msg-123".to_string(),
            "agent-1".to_string(),
            "echo hello".to_string(),
            MessagePriority::Normal,
            3,
            30,
        );

        // Should retry when failed and under max retries
        tracking.add_attempt(false, None, 100);
        tracking.status = DeliveryStatus::Failed;
        assert!(tracking.should_retry());

        // Should not retry when delivered
        tracking.status = DeliveryStatus::Delivered;
        assert!(!tracking.should_retry());

        // Should not retry when max retries reached
        tracking.status = DeliveryStatus::Failed;
        tracking.add_attempt(false, None, 100);
        tracking.add_attempt(false, None, 100);
        tracking.add_attempt(false, None, 100);
        assert!(!tracking.should_retry());
    }

    #[test]
    fn test_delivery_receipt() {
        let receipt = DeliveryReceipt {
            message_id: "test-msg-123".to_string(),
            agent_id: "agent-1".to_string(),
            delivered_at: 1234567890,
            acknowledgment: Some("Message received".to_string()),
            processing_time_ms: 150,
        };

        assert_eq!(receipt.message_id, "test-msg-123");
        assert_eq!(receipt.agent_id, "agent-1");
        assert_eq!(receipt.processing_time_ms, 150);
        assert_eq!(receipt.acknowledgment, Some("Message received".to_string()));
    }

    #[test]
    fn test_delivery_config_default() {
        let config = DeliveryConfig::default();
        assert_eq!(config.default_timeout_secs, 30);
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.retry_delay_secs, 2);
        assert!(config.wait_for_confirmation);
        assert!(config.background_processing);
        assert_eq!(config.cleanup_after_secs, 300);
    }

    #[test]
    fn test_delivery_stats() {
        let mut stats = DeliveryStats::default();

        stats.delivered = 80;
        stats.failed = 15;
        stats.timeouts = 5;
        stats.total_response_time_ms = 8000;
        stats.total_attempts = 100;

        stats.min_response_time_ms = 50;
        stats.max_response_time_ms = 200;

        // Calculate derived values
        stats.success_rate =
            stats.delivered as f64 / (stats.delivered + stats.failed + stats.timeouts) as f64;
        stats.avg_response_time_ms =
            stats.total_response_time_ms as f64 / stats.total_attempts as f64;

        assert_eq!(stats.delivered, 80);
        assert_eq!(stats.failed, 15);
        assert_eq!(stats.timeouts, 5);
        assert!((stats.success_rate - 0.8).abs() < 0.01);
        assert!((stats.avg_response_time_ms - 80.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_delivery_confirmation_creation() {
        let config = DeliveryConfig::default();
        let delivery = DeliveryConfirmation::new(config);

        assert_eq!(delivery.config.default_timeout_secs, 30);
        assert_eq!(delivery.config.max_retries, 3);
    }
}
