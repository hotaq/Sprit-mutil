// Test Claude Code command format

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Testing Claude Code command format\n");

    // Use the active Claude Code pane (pane 2 based on capture)
    let session = "sprite-session";
    let claude_pane = "2"; // Claude Code is running in pane 2

    println!(
        "🎯 Sending commands to Claude Code in session {}:{}",
        session, claude_pane
    );

    // Test 1: Send a proper Claude Code command (starts with \)
    println!("\n🧪 Test 1: Sending a Claude Code slash command");
    std::process::Command::new("tmux")
        .args([
            "send-keys",
            "-t",
            &format!("{}.{}", session, claude_pane),
            "/help",
            "C-m",
        ])
        .output()?;
    println!("   ✅ Sent: /help");

    std::thread::sleep(std::time::Duration::from_secs(2));

    // Test 2: Send another slash command
    println!("\n🧪 Test 2: Sending a slash command with parameters");
    std::process::Command::new("tmux")
        .args([
            "send-keys",
            "-t",
            &format!("{}.{}", session, claude_pane),
            "/workspace",
            "C-m",
        ])
        .output()?;
    println!("   ✅ Sent: /workspace");

    std::thread::sleep(std::time::Duration::from_secs(2));

    // Test 3: Send a user message (not a command)
    println!("\n🧪 Test 3: Sending a regular message");
    std::process::Command::new("tmux")
        .args([
            "send-keys",
            "-t",
            &format!("{}.{}", session, claude_pane),
            "Hello Claude, this is a test message from tmux",
            "C-m",
        ])
        .output()?;
    println!("   ✅ Sent regular message");

    std::thread::sleep(std::time::Duration::from_secs(2));

    // Test 4: Try clearing and sending hey command like scenario
    println!("\n🧪 Test 4: Simulating /hey with proper Claude format");

    // Clear current input first
    std::process::Command::new("tmux")
        .args([
            "send-keys",
            "-t",
            &format!("{}.{}", session, claude_pane),
            "C-c",
        ])
        .output()?;

    std::thread::sleep(std::time::Duration::from_millis(100));

    // Send the command Claude should actually execute
    std::process::Command::new("tmux")
        .args([
            "send-keys",
            "-t",
            &format!("{}.{}", session, claude_pane),
            "echo 'This command should execute in shell'",
            "C-m",
        ])
        .output()?;
    println!("   ✅ Sent shell command");

    std::thread::sleep(std::time::Duration::from_secs(1));

    // Send another test that Claude can analyze
    std::process::Command::new("tmux")
        .args([
            "send-keys",
            "-t",
            &format!("{}.{}", session, claude_pane),
            "ls -la",
            "C-m",
        ])
        .output()?;
    println!("   ✅ Sent: ls -la");

    println!(
        "\n🔍 Check pane {} to see which commands Claude actually executes!",
        claude_pane
    );
    println!("💡 Claude should execute shell commands but not messages.");

    Ok(())
}
