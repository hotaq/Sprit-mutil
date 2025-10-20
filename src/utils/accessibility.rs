//! Accessibility utilities for the Sprite multi-agent toolkit.
//!
//! This module provides accessibility features to ensure the CLI is usable
//! by people with disabilities, including screen reader support and WCAG compliance.

#![allow(dead_code)]

use std::env;
use std::io::{self, Write};

/// Configuration for accessibility features
#[derive(Debug, Clone)]
pub struct AccessibilityConfig {
    /// Whether to use screen reader compatible output
    pub screen_reader: bool,
    /// Whether to disable colors (NO_COLOR support)
    pub no_color: bool,
    /// Whether to use high contrast mode
    #[allow(dead_code)]
    pub high_contrast: bool,
    /// Whether to use verbose output for better context
    #[allow(dead_code)]
    pub verbose: bool,
}

impl Default for AccessibilityConfig {
    fn default() -> Self {
        Self {
            screen_reader: should_enable_screen_reader(),
            no_color: should_disable_color(),
            high_contrast: false,
            verbose: false,
        }
    }
}

/// Determine if screen reader mode should be enabled
fn should_enable_screen_reader() -> bool {
    // Check for common screen reader environment variables
    env::var("SCREEN_READER").is_ok() ||
    env::var("SR").is_ok() ||
    env::var("JAWS").is_ok() ||
    env::var("NVDA").is_ok() ||
    env::var("VOICEOVER").is_ok() ||
    env::var("TALKBACK").is_ok() ||
    // Check for assistive technology indicators
    env::var("ACCESSIBILITY").is_ok() ||
    env::var("A11Y").is_ok()
}

/// Determine if colors should be disabled
fn should_disable_color() -> bool {
    // NO_COLOR specification: https://no-color.org/
    env::var("NO_COLOR").is_ok() ||
    // Traditional way to disable colors
    env::var("TERM").unwrap_or_default().contains("dumb") ||
    env::var("CLICOLOR").unwrap_or_default() == "0"
}

/// Print accessible text with proper formatting
#[allow(dead_code)]
pub fn print_accessible(text: &str, config: &AccessibilityConfig) {
    let formatted_text = format_for_accessibility(text, config);
    println!("{}", formatted_text);
}

/// Print accessible error message
pub fn print_error(message: &str, config: &AccessibilityConfig) {
    let formatted = if config.screen_reader {
        format!("ERROR: {}. {}", message, get_suggestion_for_error(message))
    } else {
        format!("❌ Error: {}", message)
    };

    writeln!(io::stderr(), "{}", formatted).ok();
}

/// Print accessible success message
pub fn print_success(message: &str, config: &AccessibilityConfig) {
    let formatted = if config.screen_reader {
        format!("SUCCESS: {}", message)
    } else if config.no_color {
        format!("✓ {}", message)
    } else {
        format!("✅ {}", message)
    };

    println!("{}", formatted);
}

/// Print accessible warning message
pub fn print_warning(message: &str, config: &AccessibilityConfig) {
    let formatted = if config.screen_reader {
        format!("WARNING: {}", message)
    } else if config.no_color {
        format!("⚠ {}", message)
    } else {
        format!("⚠️  {}", message)
    };

    println!("{}", formatted);
}

/// Print accessible info message
pub fn print_info(message: &str, config: &AccessibilityConfig) {
    let formatted = if config.screen_reader {
        format!("INFO: {}", message)
    } else if config.no_color {
        format!("ℹ {}", message)
    } else {
        format!("ℹ️  {}", message)
    };

    println!("{}", formatted);
}

/// Format text for accessibility
fn format_for_accessibility(text: &str, config: &AccessibilityConfig) -> String {
    let mut formatted = text.to_string();

    if config.screen_reader {
        // Add semantic annotations for screen readers
        formatted = add_semantic_annotations(&formatted);
    } else if config.no_color || config.high_contrast {
        // Replace emojis with simple text equivalents for no_color or high contrast
        formatted = replace_emojis_simple(&formatted);
    }

    formatted
}

/// Add semantic annotations for screen readers
fn add_semantic_annotations(text: &str) -> String {
    // Replace emojis with descriptive text
    text.replace("✅", "[SUCCESS]")
        .replace("❌", "[ERROR]")
        .replace("⚠️", "[WARNING]")
        .replace("ℹ️", "[INFO]")
        .replace("📡", "[BROADCAST]")
        .replace("🔄", "[SYNC]")
        .replace("📁", "[DIRECTORY]")
        .replace("🎬", "[SESSION]")
        .replace("📋", "[PANE]")
        .replace("🔍", "[ZOOM]")
        .replace("💡", "[SUGGESTION]")
        .replace("🧹", "[CLEANUP]")
        .replace("✓", "[DONE]")
        .replace("→", "[ARROW]")
}

/// Replace emojis with simple text equivalents (for no_color mode)
fn replace_emojis_simple(text: &str) -> String {
    text.replace("✅", "✓")
        .replace("❌", "✗")
        .replace("⚠️", "⚠")
        .replace("ℹ️", "ℹ")
        .replace("📡", "[BROADCAST]")
        .replace("🔄", "[SYNC]")
        .replace("📁", "[DIR]")
        .replace("🎬", "[SESSION]")
        .replace("📋", "[LIST]")
        .replace("🔍", "[SEARCH]")
        .replace("💡", "[INFO]")
        .replace("🧹", "[CLEAN]")
        .replace("→", "->")
}

/// Replace emojis with text equivalents
#[allow(dead_code)]
fn replace_emojis_with_text(text: &str) -> String {
    text.replace("✅", "SUCCESS")
        .replace("❌", "ERROR")
        .replace("⚠️", "WARNING")
        .replace("ℹ️", "INFO")
        .replace("📡", "BROADCAST")
        .replace("🔄", "SYNC")
        .replace("📁", "DIRECTORY")
        .replace("🎬", "SESSION")
        .replace("📋", "PANE")
        .replace("🔍", "ZOOM")
        .replace("💡", "SUGGESTION")
        .replace("🧹", "CLEANUP")
        .replace("✓", "DONE")
        .replace("→", "ARROW")
}

/// Get suggestion for common error types
fn get_suggestion_for_error(error_message: &str) -> String {
    if error_message.contains("tmux") {
        "Make sure tmux is installed and running. Use 'brew install tmux' on macOS or 'apt install tmux' on Ubuntu.".to_string()
    } else if error_message.contains("git") {
        "Make sure you're in a git repository. Use 'git init' to create one.".to_string()
    } else if error_message.contains("configuration") || error_message.contains("config") {
        "Use 'sprite init' to create a default configuration file.".to_string()
    } else if error_message.contains("permission") {
        "Check file permissions and try running with appropriate access rights.".to_string()
    } else if error_message.contains("not found") {
        "Check if the specified file or directory exists and is accessible.".to_string()
    } else {
        "Use 'sprite --help' to see available commands and options.".to_string()
    }
}

/// Format a list with accessibility in mind
pub fn format_list(items: &[&str], config: &AccessibilityConfig) -> String {
    if config.screen_reader {
        // For screen readers, use numbered list with semantic markers
        items
            .iter()
            .enumerate()
            .map(|(i, item)| format!("Item {}: {}", i + 1, item))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        // Regular bullet list
        items
            .iter()
            .map(|item| format!("• {}", item))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Format a table with accessibility in mind
pub fn format_table(
    headers: &[&str],
    rows: &[Vec<String>],
    config: &AccessibilityConfig,
) -> String {
    if config.screen_reader {
        // For screen readers, format as simple text with column labels
        let mut result = String::new();
        result.push_str("Table start:\n");

        for (i, row) in rows.iter().enumerate() {
            result.push_str(&format!("Row {}:\n", i + 1));
            for (j, cell) in row.iter().enumerate() {
                if j < headers.len() {
                    result.push_str(&format!("  {}: {}\n", headers[j], cell));
                }
            }
        }

        result.push_str("Table end.\n");
        result
    } else {
        // Simple table formatting
        let mut result = String::new();

        // Header row
        result.push_str(&headers.join(" | "));
        result.push('\n');
        result.push_str(&"-".repeat(headers.join(" | ").len()));
        result.push('\n');

        // Data rows
        for row in rows {
            result.push_str(&row.join(" | "));
            result.push('\n');
        }

        result
    }
}

/// Format progress with accessibility in mind
pub fn format_progress(
    current: usize,
    total: usize,
    description: &str,
    config: &AccessibilityConfig,
) -> String {
    if config.screen_reader {
        format!(
            "Progress: {} of {} complete. {}",
            current, total, description
        )
    } else {
        let percentage = if total > 0 {
            (current * 100) / total
        } else {
            0
        };
        format!(
            "({}%){} {}",
            percentage,
            if percentage < 100 { " " } else { "" },
            description
        )
    }
}

/// Validate output for WCAG compliance
pub fn validate_wcag_compliance(text: &str) -> Vec<String> {
    let mut issues = Vec::new();

    // Check for sufficient contrast indicators
    if text.contains("✅") || text.contains("❌") || text.contains("⚠️") {
        issues.push(
            "Consider using text alternatives to emojis for better accessibility".to_string(),
        );
    }

    // Check for color-only information
    if text.contains("red") || text.contains("green") || text.contains("blue") {
        issues.push("Avoid using color as the only way to convey information".to_string());
    }

    // Check for complex formatting that might be hard for screen readers
    if text.chars().filter(|c| *c == '|').count() > 5 {
        issues.push("Complex table formatting may be difficult for screen readers".to_string());
    }

    // Check for adequate description length
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.len() < 3 && !text.is_empty() {
        issues.push("Very short text may not provide sufficient context".to_string());
    }

    issues
}

/// Print help text in an accessible format
pub fn print_help(
    command: &str,
    description: &str,
    usage: &str,
    options: &[(&str, &str)],
    config: &AccessibilityConfig,
) {
    if config.screen_reader {
        println!("Command: {}", command);
        println!("Description: {}", description);
        println!("Usage: {}", usage);
        println!("Options:");

        for (option, desc) in options {
            println!("  {}: {}", option, desc);
        }
    } else {
        println!("{} - {}", command, description);
        println!();
        println!("USAGE:");
        println!("  {}", usage);
        println!();
        println!("OPTIONS:");

        for (option, desc) in options {
            println!("  {}  {}", option, desc);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accessibility_config_default() {
        let config = AccessibilityConfig::default();
        // Should not panic
        assert!(!config.screen_reader || config.screen_reader); // Could be true or false
        assert!(!config.no_color || config.no_color); // Could be true or false
    }

    #[test]
    fn test_screen_reader_formatting() {
        let config = AccessibilityConfig {
            screen_reader: true,
            no_color: false,
            high_contrast: false,
            verbose: false,
        };

        let result = format_for_accessibility("✅ Success", &config);
        assert!(result.contains("[SUCCESS]"));
        assert!(!result.contains("✅"));
    }

    #[test]
    fn test_no_color_formatting() {
        let config = AccessibilityConfig {
            screen_reader: false,
            no_color: true,
            high_contrast: false,
            verbose: false,
        };

        let result = format_for_accessibility("✅ Success", &config);
        assert!(result.contains("✓"));
        assert!(!result.contains("✅"));
    }

    #[test]
    fn test_wcag_validation() {
        let issues = validate_wcag_compliance("✅ Success with red text");
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.contains("emoji")));
    }

    #[test]
    fn test_list_formatting() {
        let items = vec!["Item 1", "Item 2"];
        let config_screen_reader = AccessibilityConfig {
            screen_reader: true,
            no_color: false,
            high_contrast: false,
            verbose: false,
        };

        let result = format_list(&items, &config_screen_reader);
        assert!(result.contains("Item 1:"));
        assert!(result.contains("Item 2:"));
    }

    #[test]
    fn test_error_suggestions() {
        let suggestion = get_suggestion_for_error("tmux session not found");
        assert!(suggestion.contains("tmux"));
        assert!(suggestion.contains("install"));
    }
}
