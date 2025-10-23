// Debug test to see exactly what tmux commands are being sent
use sprite::utils::tmux;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Debug: Testing tmux send_keys behavior\n");

    // Get your active session
    let sessions = tmux::list_sessions()?;
    println!("📋 Available sessions:");
    for session in &sessions {
        println!(
            "   - {} (windows: {}, attached: {})",
            session.name, session.windows, session.attached
        );
    }

    // Find the sprite-session
    let sprite_session = sessions
        .iter()
        .find(|s| s.name.contains("sprite"))
        .ok_or("No sprite session found")?;

    println!("\n🎯 Using session: {}", sprite_session.name);

    // Get panes
    let panes = tmux::get_session_panes(&sprite_session.name)?;
    println!("📋 Panes in session:");
    for pane in &panes {
        println!("   - {}", pane.pane_id);
    }

    // Test different ways to send commands
    let agent_pane = &panes[0].pane_id; // Use first pane

    println!("\n🧪 Testing different command sending methods:");

    // Test 1: Standard send_keys (what hey command uses)
    println!("1. Testing standard send_keys (from hey command):");
    tmux::send_keys(
        &sprite_session.name,
        agent_pane,
        "echo 'Test from send_keys'",
    )?;
    println!("   ✅ Sent command with C-m");

    std::thread::sleep(std::time::Duration::from_secs(2));

    // Test 2: Direct tmux command
    println!("2. Testing direct tmux send-keys:");
    let output = std::process::Command::new("tmux")
        .args([
            "send-keys",
            "-t",
            &format!("{}.{}", sprite_session.name, agent_pane),
            "echo 'Test from direct tmux'",
            "C-m",
        ])
        .output()?;

    if output.status.success() {
        println!("   ✅ Direct tmux command successful");
    } else {
        println!(
            "   ❌ Direct tmux command failed: {:?}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    std::thread::sleep(std::time::Duration::from_secs(2));

    // Test 3: Try without C-m (just the text)
    println!("3. Testing without C-m (just text):");
    let output = std::process::Command::new("tmux")
        .args([
            "send-keys",
            "-t",
            &format!("{}.{}", sprite_session.name, agent_pane),
            "echo 'Test without Enter key'",
        ])
        .output()?;

    if output.status.success() {
        println!("   ✅ Text-only command sent");
    } else {
        println!("   ❌ Text-only command failed");
    }

    std::thread::sleep(std::time::Duration::from_secs(2));

    // Test 4: Try with Enter separately
    println!("4. Testing with separate Enter:");
    std::process::Command::new("tmux")
        .args([
            "send-keys",
            "-t",
            &format!("{}.{}", sprite_session.name, agent_pane),
            "echo 'Test with separate Enter'",
        ])
        .output()?;

    std::process::Command::new("tmux")
        .args([
            "send-keys",
            "-t",
            &format!("{}.{}", sprite_session.name, agent_pane),
            "C-m",
        ])
        .output()?;

    println!("   ✅ Command + Enter sent separately");

    println!("\n🔍 Check your tmux pane to see which approach works!");
    println!(
        "💡 You can check with: tmux capture-pane -p -t {}:{}",
        sprite_session.name, agent_pane
    );

    Ok(())
}
