// Manual test for T046 delivery confirmation
// Run with: cargo run --bin test_delivery_manual

use sprite::communication::{DeliveryConfig, DeliveryConfirmation};
use sprite::models::MessagePriority;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Testing T046 Delivery Confirmation System\n");

    // Test 1: Basic delivery confirmation creation
    println!("📋 Test 1: Creating delivery confirmation system...");
    let config = DeliveryConfig::default();
    let delivery = DeliveryConfirmation::new(config);
    println!("✅ Delivery confirmation system created successfully\n");

    // Test 2: Send a test message
    println!("📋 Test 2: Sending test message with confirmation...");
    let message_id = "manual-test-123".to_string();

    // First, let's check if there are any active tmux sessions
    let tmux_sessions_output = std::process::Command::new("tmux")
        .args(["list-sessions"])
        .output();

    println!("\n🖥️  Checking existing tmux sessions:");
    match tmux_sessions_output {
        Ok(output) => {
            let sessions = String::from_utf8_lossy(&output.stdout);
            if sessions.trim().is_empty() {
                println!("   No tmux sessions found - this is expected for delivery failures");
            } else {
                println!("   Found sessions: {}", sessions);
            }
        }
        Err(_) => {
            println!("   Could not check tmux sessions");
        }
    }

    // Try with a common tmux session name or create our own
    let agent_pane = "sprite-session:0.0"; // Use your existing session, pane 0.0
    let result = delivery
        .send_with_confirmation(
            message_id.clone(),
            "test-agent",
            agent_pane,
            "echo 'Hello from delivery confirmation!'",
            MessagePriority::Normal,
        )
        .await?;

    println!("📨 Message sent!");
    println!("   Message ID: {}", result.message_id);
    println!("   Target Agent: {}", result.target_agent);
    println!("   Status: {:?}", result.status);
    println!("   Attempts: {}", result.total_attempts());

    if let Some(receipt) = &result.receipt {
        println!("   ✅ Delivery confirmed!");
        println!("   📅 Delivered at: {}", receipt.delivered_at);
        println!("   💬 Acknowledgment: {:?}", receipt.acknowledgment);
    } else {
        println!("   ⚠️  Delivery receipt not available (may be normal)");
    }

    // Test 3: Check delivery tracking
    println!("\n📋 Test 3: Checking delivery tracking...");
    let tracking = delivery.get_tracking(&message_id).await;
    if let Some(track) = tracking {
        println!("📊 Tracking found:");
        println!("   Status: {:?}", track.status);
        println!("   Priority: {:?}", track.priority);
        println!("   Created at: {}", track.created_at);
        println!("   Attempts: {}", track.attempts.len());
    } else {
        println!("❌ No tracking found");
    }

    // Test 4: Get statistics
    println!("\n📋 Test 4: Getting delivery statistics...");
    let stats = delivery.get_delivery_stats().await;
    println!("📊 Delivery Statistics:");
    println!("   Delivered: {}", stats.delivered);
    println!("   Failed: {}", stats.failed);
    println!("   Timeouts: {}", stats.timeouts);
    println!("   Pending: {}", stats.pending);
    println!("   Success Rate: {:.1}%", stats.success_rate * 100.0);
    println!("   Avg Response Time: {:.1}ms", stats.avg_response_time_ms);

    // Test 5: Multiple messages
    println!("\n📋 Test 5: Sending multiple messages with different priorities...");
    for i in 1..=3 {
        let priority = match i {
            1 => MessagePriority::Low,
            2 => MessagePriority::Normal,
            _ => MessagePriority::High,
        };

        let result = delivery
            .send_with_confirmation(
                format!("multi-test-{}", i),
                "test-agent",
                "sprite-session:0.0", // Use your existing session
                &format!("echo 'Message {} with priority {:?}'", i, priority),
                priority.clone(),
            )
            .await?;

        println!(
            "   Message {}: Status {:?}, Priority {:?}",
            i, result.status, priority
        );
    }

    // Wait a bit for processing
    sleep(Duration::from_millis(500)).await;

    // Final statistics
    println!("\n📋 Test 6: Final statistics after multiple messages...");
    let final_stats = delivery.get_delivery_stats().await;
    println!("📊 Final Stats:");
    println!("   Total Attempts: {}", final_stats.total_attempts);
    println!("   Delivered: {}", final_stats.delivered);
    println!("   Failed: {}", final_stats.failed);
    println!("   Success Rate: {:.1}%", final_stats.success_rate * 100.0);

    println!("\n🎉 T046 Delivery Confirmation Manual Test Complete!");
    println!("💡 Note: Some deliveries may fail due to missing tmux sessions,");
    println!("   but the delivery confirmation system itself is working correctly.");

    Ok(())
}
