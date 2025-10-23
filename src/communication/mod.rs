//! Communication module for multi-agent system.
//!
//! Provides reliable message delivery, response handling, and priority-based
//! communication between agents in the multi-agent workflow toolkit.

pub mod delivery;
// TODO: Implement T047 and T048
// pub mod responses;
// pub mod priority;

// Re-export main types for convenience
pub use delivery::{
    DeliveryAttempt, DeliveryConfig, DeliveryConfirmation, DeliveryReceipt, DeliveryStats,
    DeliveryStatus, DeliveryTracking,
};
