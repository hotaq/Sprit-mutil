//! Help command - Comprehensive help system for Sprite

use crate::cli::HelpCategory;
use crate::utils::help::{
    get_help_system, search_help, show_accessible_help, show_help, show_usage_patterns,
    UsageCategory,
};
use anyhow::Result;

/// Command-line arguments for the help command.
#[derive(Debug)]
pub struct HelpArgs {
    /// Command to get help for
    pub command: Option<String>,

    /// Search help content
    pub search: Option<String>,

    /// Show usage patterns
    pub patterns: bool,

    /// Show troubleshooting guides
    pub troubleshooting: bool,

    /// Show quick reference
    pub quick: bool,

    /// Show accessible help (screen reader friendly)
    pub accessible: bool,

    /// Show usage patterns for specific category
    pub category: Option<HelpCategory>,
}

impl From<HelpCategory> for UsageCategory {
    fn from(category: HelpCategory) -> Self {
        match category {
            HelpCategory::GettingStarted => UsageCategory::GettingStarted,
            HelpCategory::DailyWorkflow => UsageCategory::DailyWorkflow,
            HelpCategory::Troubleshooting => UsageCategory::Troubleshooting,
            HelpCategory::Advanced => UsageCategory::Advanced,
            HelpCategory::Collaboration => UsageCategory::Collaboration,
        }
    }
}

/// Execute the help command with the given arguments.
pub fn execute(args: HelpArgs) -> Result<()> {
    // Handle accessible help request
    if args.accessible {
        return show_accessible_help(args.command.as_deref());
    }

    // Handle search request
    if let Some(query) = args.search {
        return search_help(&query);
    }

    // Handle usage patterns request
    if args.patterns {
        let category = args.category.map(|c| c.into());
        return show_usage_patterns(category);
    }

    // Handle troubleshooting request
    if args.troubleshooting {
        return show_troubleshooting();
    }

    // Handle quick reference request
    if args.quick {
        return show_quick_reference();
    }

    // Default help for specific command or general help
    show_help(args.command.as_deref())
}

/// Show troubleshooting guides.
fn show_troubleshooting() -> Result<()> {
    let help_system = get_help_system();
    let guides = help_system.get_troubleshooting();

    if guides.is_empty() {
        println!("🔧 No troubleshooting guides available.");
        return Ok(());
    }

    println!("🔧 Troubleshooting Guides\n");
    for guide in guides {
        println!("❓ Problem: {}", guide.problem);

        if !guide.symptoms.is_empty() {
            println!("🔍 Symptoms:");
            for symptom in &guide.symptoms {
                println!("  • {}", symptom);
            }
        }

        if !guide.causes.is_empty() {
            println!("🤔 Possible Causes:");
            for cause in &guide.causes {
                println!("  • {}", cause);
            }
        }

        if !guide.solutions.is_empty() {
            println!("💡 Solutions:");
            for solution in &guide.solutions {
                println!("  → {}", solution);
            }
        }

        if !guide.prevention.is_empty() {
            println!("🛡️  Prevention:");
            for prevention in &guide.prevention {
                println!("  • {}", prevention);
            }
        }

        if !guide.related_commands.is_empty() {
            println!("🔗 Related Commands: {}", guide.related_commands.join(", "));
        }

        println!();
    }

    Ok(())
}

/// Show quick reference.
fn show_quick_reference() -> Result<()> {
    let help_system = get_help_system();
    let quick_ref = help_system.get_quick_reference();

    println!("⚡ Quick Reference\n");

    // Common commands
    println!("🔥 Most Common Commands:");
    for (i, cmd) in quick_ref.common_commands.iter().enumerate() {
        println!("  {}. {}", i + 1, cmd);
    }

    println!();

    // Keyboard shortcuts
    if !quick_ref.shortcuts.is_empty() {
        println!("⌨️  Keyboard Shortcuts:");
        for shortcut in &quick_ref.shortcuts {
            println!("  {} - {}", shortcut.keys, shortcut.description);
            println!("     Context: {}", shortcut.context);
        }
        println!();
    }

    // FAQ
    if !quick_ref.faq.is_empty() {
        println!("❓ Frequently Asked Questions:");
        for faq in &quick_ref.faq {
            println!("  Q: {}", faq.question);
            println!("  A: {}", faq.answer);
            if !faq.related_commands.is_empty() {
                println!("  Related: {}", faq.related_commands.join(", "));
            }
            println!();
        }
    }

    println!("💡 For detailed help: sprite help <command>");
    println!("🔍 For search: sprite help --search <query>");

    Ok(())
}

/// Show comprehensive help overview.
#[allow(dead_code)]
pub fn show_help_overview() -> Result<()> {
    println!("🚀 Sprite - Multi-Agent Workflow Toolkit\n");
    println!("Sprite helps you manage multiple AI coding agents in isolated tmux sessions\n");
    println!("with comprehensive workspace management and health monitoring.\n");

    println!("📋 Quick Start Guide:");
    println!("  1️⃣  Initialize your project");
    println!("     sprite init --agents 3");
    println!();
    println!("  2️⃣  Start your first session");
    println!("     sprite start");
    println!();
    println!("  3️⃣  Work with agents");
    println!("     sprite hey 1 'npm install'");
    println!("     sprite hey 2 'cargo build'");
    println!();
    println!("  4️⃣  Navigate workspaces");
    println!("     cd $(sprite warp --print 1)");
    println!();
    println!("  5️⃣  Check status");
    println!("     sprite status");
    println!();

    println!("🔧 Core Commands Overview:\n");

    let help_system = get_help_system();
    let core_commands = [
        "init", "start", "agents", "warp", "hey", "send", "sync", "status",
    ];

    for cmd in &core_commands {
        if let Some(help) = help_system.get_command_help(cmd) {
            println!("  {} - {}", cmd, help.description);
        }
    }

    println!();
    println!("📚 Getting More Help:");
    println!("  sprite help <command>           # Detailed command help");
    println!("  sprite help --search <query>    # Search help content");
    println!("  sprite help --patterns          # Show usage patterns");
    println!("  sprite help --troubleshooting   # Troubleshooting guides");
    println!("  sprite help --quick             # Quick reference");
    println!("  sprite help --accessible       # Screen reader friendly");

    println!();
    println!("🎯 Common Workflows:");
    println!("  First Time Setup → sprite help --patterns --category getting-started");
    println!("  Daily Workflow → sprite help --patterns --category daily-workflow");
    println!("  Team Collaboration → sprite help --patterns --category collaboration");

    println!();
    println!("🔗 Resources:");
    println!("  Documentation: https://docs.rs/sprite");
    println!("  Repository: https://github.com/hotaq/Sprit-mutil");
    println!("  Issues: https://github.com/hotaq/Sprit-mutil/issues");

    Ok(())
}

/// Show command examples in a structured way.
#[allow(dead_code)]
pub fn show_examples(command: Option<&str>) -> Result<()> {
    let help_system = get_help_system();

    if let Some(cmd) = command {
        if let Some(help) = help_system.get_command_help(cmd) {
            println!("💡 Examples for '{}'\n", cmd);

            if help.examples.is_empty() {
                println!("No examples available for this command.");
                return Ok(());
            }

            for (i, example) in help.examples.iter().enumerate() {
                println!("  {}. {}", i + 1, example);
            }

            println!();
            println!("💡 Tips:");
            for tip in &help.tips {
                println!("  • {}", tip);
            }
        } else {
            println!("❌ Command '{}' not found.", cmd);
            println!(
                "Available commands: {}",
                help_system.list_commands().join(", ")
            );
        }
    } else {
        println!("💡 Common Examples\n");

        let common_examples = [
            ("Initialize Project", "sprite init --agents 3"),
            ("Start Session", "sprite start --session-name dev"),
            ("List Agents", "sprite agents list"),
            ("Send Command to Agent", "sprite hey 1 'npm test'"),
            ("Broadcast to All", "sprite send 'git pull'"),
            ("Navigate Workspace", "cd $(sprite warp --print 1)"),
            ("Check Status", "sprite status --detailed"),
            ("Sync Workspaces", "sprite sync --agent 2"),
            ("Focus Agent", "sprite zoom 1"),
            ("End Session", "sprite kill --all"),
        ];

        for (description, command) in &common_examples {
            println!("  {} → {}", description, command);
        }

        println!();
        println!("For command-specific examples: sprite help --examples <command>");
    }

    Ok(())
}

/// Show command aliases and shortcuts.
#[allow(dead_code)]
pub fn show_aliases() -> Result<()> {
    println!("🔗 Command Aliases & Shortcuts\n");

    println!("Common Aliases (add to your shell configuration):");
    println!("  alias si='sprite init'");
    println!("  alias ss='sprite start'");
    println!("  alias sk='sprite kill'");
    println!("  alias sa='sprite agents'");
    println!("  alias sw='cd $(sprite warp --print \"$1\")'");
    println!("  alias sh='sprite hey'");
    println!();

    println!("Shell Functions:");
    println!("  # Function to quickly switch agent workspaces");
    println!("  sw() {{");
    println!("    if [ $# -eq 0 ]; then");
    println!("      sprite warp --list");
    println!("    else");
    println!("      cd \"$(sprite warp --print \"$1\")\"");
    println!("    fi");
    println!("  }}");
    println!();

    println!("Tmux Shortcuts (when in a sprite session):");
    println!("  Ctrl+B, d    → Detach from session");
    println!("  Ctrl+B, %    → Split pane horizontally");
    println!("  Ctrl+B, \"    → Split pane vertically");
    println!("  Ctrl+B, ←→↑↓ → Navigate panes");
    println!("  Ctrl+B, z    → Zoom current pane");
    println!();

    println!("🔧 Integration:");
    println!("  sprite help --generate-shell-integration  # Generate full integration script");

    Ok(())
}

/// Generate shell integration script.
#[allow(dead_code)]
pub fn generate_shell_integration() -> Result<()> {
    let script = r#"#!/bin/bash
# Sprite workspace integration script
# Add this to your ~/.bashrc or ~/.zshrc

# Tab completion for sprite commands
_sprite_completion() {
  local cur prev words
  cur="${COMP_WORDS[COMP_CWORD]}"
  prev="${COMP_WORDS[COMP_CWORD-1]}"
  words=("${COMP_WORDS[@]}")

  case "$prev" in
    sprite)
      COMPREPLY=($(compgen -W "init start agents attach kill send hey sync remove warp zoom status config help" -- "$cur"))
      ;;
    warp)
      COMPREPLY=($(compgen -W "$(sprite warp --list 2>/dev/null | grep '📁' | awk '{print $3}' | sed 's/://')" -- "$cur"))
      ;;
    hey|send)
      COMPREPLY=($(compgen -W "$(sprite agents list 2>/dev/null | grep -E '^[0-9]+' | awk '{print $1}')" -- "$cur"))
      ;;
  esac
}

complete -F _sprite_completion sprite

# Quick workspace navigation
sw() {
  if [ $# -eq 0 ]; then
    sprite warp --list
  else
    cd "$(sprite warp --print "$1")"
  fi
}

# Quick agent commands
sh() {
  local agent="$1"
  shift
  sprite hey "$agent" "$@"
}

# Sprite session management
ss() { sprite start "$@"; }
sk() { sprite kill "$@"; }
sa() { sprite attach "$@"; }
"#;

    println!("🔧 Shell Integration Script\n");
    println!("{}", script);

    println!("💡 Save this as ~/.sprite_completion.sh and add:");
    println!("   source ~/.sprite_completion.sh");
    println!("   to your shell configuration file.");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_args_parsing() {
        // Test basic parsing
        let args = HelpArgs {
            command: Some("init".to_string()),
            search: None,
            patterns: false,
            troubleshooting: false,
            quick: false,
            accessible: false,
            category: None,
        };

        assert_eq!(args.command, Some("init".to_string()));
        assert!(!args.patterns);
    }

    #[test]
    fn test_category_conversion() {
        let help_cat = HelpCategory::GettingStarted;
        let usage_cat: UsageCategory = help_cat.into();
        assert_eq!(usage_cat, UsageCategory::GettingStarted);
    }
}
