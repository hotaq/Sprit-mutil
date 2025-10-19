//! Logging infrastructure for the Sprite multi-agent workflow toolkit.
//!
//! This module provides logging functionality with support for different
//! log levels, file output, and rotation.

use crate::error::SpriteError;
use anyhow::Result;
use chrono::{DateTime, Local, Utc};
use log::{Level, LevelFilter, Log, Metadata, Record};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Logger configuration.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LoggerConfig {
    /// Log file path (None for stdout only)
    pub log_file: Option<PathBuf>,
    /// Log level
    pub level: Level,
    /// Whether to log to stdout
    pub log_to_stdout: bool,
    /// Whether to include timestamps
    pub include_timestamp: bool,
    /// Whether to include module names
    pub include_module: bool,
    /// Whether to use colors (for stdout)
    pub use_colors: bool,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            log_file: Some(PathBuf::from("logs/sprite.log")),
            level: Level::Info,
            log_to_stdout: true,
            include_timestamp: true,
            include_module: true,
            use_colors: true,
        }
    }
}

/// A simple file and console logger for Sprite.
#[allow(dead_code)]
pub struct SpriteLogger {
    config: LoggerConfig,
    log_file: Option<Mutex<BufWriter<File>>>,
    #[allow(dead_code)]
    start_time: DateTime<Utc>,
}

impl SpriteLogger {
    /// Create a new logger with the given configuration.
    #[allow(dead_code)]
    pub fn new(config: LoggerConfig) -> Result<Self> {
        let log_file = if let Some(ref log_path) = config.log_file {
            // Create parent directory if it doesn't exist
            if let Some(parent) = log_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    SpriteError::filesystem_with_source(
                        format!("Failed to create log directory: {}", e),
                        parent.display().to_string(),
                        e,
                    )
                })?;
            }

            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_path)
                .map_err(|e| {
                    SpriteError::filesystem_with_source(
                        format!("Failed to open log file: {}", e),
                        log_path.display().to_string(),
                        e,
                    )
                })?;

            Some(Mutex::new(BufWriter::new(file)))
        } else {
            None
        };

        Ok(Self {
            config,
            log_file,
            start_time: Utc::now(),
        })
    }

    /// Write a log entry.
    #[allow(dead_code)]
    fn write_log(&self, level: Level, record: &Record) {
        let message = record.args().to_string();

        let formatted = self.format_message(
            level.to_level_filter(),
            &message,
            record.file(),
            record.line(),
        );

        // Write to file if configured
        if let Some(ref file_mutex) = self.log_file {
            if let Ok(mut file) = file_mutex.lock() {
                let _ = writeln!(file, "{}", formatted);
                let _ = file.flush();
            }
        }

        // Write to stdout if configured
        if self.config.log_to_stdout {
            eprintln!("{}", formatted);
        }
    }

    /// Format a log message.
    #[allow(dead_code)]
    fn format_message(
        &self,
        level: LevelFilter,
        message: &str,
        file: Option<&str>,
        line: Option<u32>,
    ) -> String {
        let mut parts = Vec::new();

        // Add timestamp if configured
        if self.config.include_timestamp {
            parts.push(Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
        }

        // Add log level with colors if configured
        if self.config.use_colors {
            let level_str = match level {
                LevelFilter::Error => "\x1b[31mERROR\x1b[0m".to_string(),
                LevelFilter::Warn => "\x1b[33mWARN \x1b[0m".to_string(),
                LevelFilter::Info => "\x1b[32mINFO \x1b[0m".to_string(),
                LevelFilter::Debug => "\x1b[36mDEBUG\x1b[0m".to_string(),
                LevelFilter::Trace => "\x1b[37mTRACE\x1b[0m".to_string(),
                _ => "UNKNOWN".to_string(),
            };
            parts.push(level_str);
        } else {
            let level_str = match level {
                LevelFilter::Error => "ERROR".to_string(),
                LevelFilter::Warn => "WARN ".to_string(),
                LevelFilter::Info => "INFO ".to_string(),
                LevelFilter::Debug => "DEBUG".to_string(),
                LevelFilter::Trace => "TRACE".to_string(),
                _ => "UNKNOWN".to_string(),
            };
            parts.push(level_str);
        }

        // Add module name if configured
        if self.config.include_module {
            if let Some(file) = file {
                let module = file.rfind('/').map(|i| &file[i + 1..]).unwrap_or(file);
                parts.push(format!("[{}]", module));
            }
        }

        // Add line number if available
        if let Some(line) = line {
            parts.push(format!(":{}", line));
        }

        // Add the actual message
        parts.push(message.to_string());

        parts.join(" ")
    }

    /// Check if the given log level should be logged.
    #[allow(dead_code)]
    fn should_log(&self, level: LevelFilter) -> bool {
        level <= self.config.level
    }

    /// Flush any buffered log data.
    #[allow(dead_code)]
    pub fn flush(&self) -> Result<()> {
        if let Some(ref file_mutex) = self.log_file {
            if let Ok(mut file) = file_mutex.lock() {
                file.flush()?;
            }
        }
        Ok(())
    }

    /// Get the logger start time.
    #[allow(dead_code)]
    pub fn start_time(&self) -> DateTime<Utc> {
        self.start_time
    }

    /// Get the uptime since logger start.
    #[allow(dead_code)]
    pub fn uptime(&self) -> chrono::Duration {
        Utc::now() - self.start_time
    }
}

impl Log for SpriteLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.should_log(metadata.level().to_level_filter())
    }

    fn log(&self, record: &Record) {
        if self.should_log(record.level().to_level_filter()) {
            self.write_log(record.level(), record);
        }
    }

    fn flush(&self) {
        let _ = self.flush();
    }
}

/// Initialize logging for the application.
#[allow(dead_code)]
pub fn init_logging(config: LoggerConfig) -> Result<()> {
    let logger = SpriteLogger::new(config)?;
    let _ = log::set_boxed_logger(Box::new(logger));
    Ok(())
}

/// Initialize logging with default configuration.
#[allow(dead_code)]
pub fn init_default_logging() -> Result<()> {
    init_logging(LoggerConfig::default())
}

/// Initialize logging with a log file.
#[allow(dead_code)]
pub fn init_file_logging<P: AsRef<Path>>(log_file: P, level: Level) -> Result<()> {
    let config = LoggerConfig {
        log_file: Some(log_file.as_ref().to_path_buf()),
        level,
        log_to_stdout: true,
        include_timestamp: true,
        include_module: true,
        use_colors: true,
    };
    init_logging(config)
}

/// Initialize logging without colors (for scripts).
#[allow(dead_code)]
pub fn init_script_logging<P: AsRef<Path>>(log_file: P, level: Level) -> Result<()> {
    let config = LoggerConfig {
        log_file: Some(log_file.as_ref().to_path_buf()),
        level,
        log_to_stdout: false,
        include_timestamp: true,
        include_module: false,
        use_colors: false,
    };
    init_logging(config)
}

/// Get the current log level.
#[allow(dead_code)]
pub fn get_log_level() -> LevelFilter {
    log::max_level()
}

/// Set the log level.
#[allow(dead_code)]
pub fn set_log_level(level: LevelFilter) {
    log::set_max_level(level);
}

/// Create a log file with rotation support.
#[allow(dead_code)]
pub struct RotatingLogger {
    config: LoggerConfig,
    #[allow(dead_code)]
    base_path: PathBuf,
    #[allow(dead_code)]
    current_file: PathBuf,
    max_size: u64,
    current_size: std::sync::atomic::AtomicU64,
    file: Option<Mutex<BufWriter<File>>>,
    #[allow(dead_code)]
    start_time: DateTime<Utc>,
}

impl RotatingLogger {
    /// Create a new rotating logger.
    #[allow(dead_code)]
    pub fn new<P: AsRef<Path>>(base_path: P, max_size: u64, level: Level) -> Result<Self> {
        let base_path = base_path.as_ref().to_path_buf();
        let current_file = base_path.join(format!("{}.log", Local::now().format("%Y%m%d_%H%M%S")));

        let config = LoggerConfig {
            log_file: Some(current_file.clone()),
            level,
            log_to_stdout: true,
            include_timestamp: true,
            include_module: true,
            use_colors: true,
        };

        let file = Self::create_log_file(&current_file)?;
        let current_size = Self::get_file_size(&current_file)?;

        Ok(Self {
            config,
            base_path,
            current_file,
            max_size,
            current_size: std::sync::atomic::AtomicU64::new(current_size),
            file: Some(Mutex::new(BufWriter::new(file))),
            start_time: Utc::now(),
        })
    }

    /// Create the initial log file.
    #[allow(dead_code)]
    fn create_log_file(path: &Path) -> Result<File> {
        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                SpriteError::filesystem_with_source(
                    format!("Failed to create log directory: {}", e),
                    parent.display().to_string(),
                    e,
                )
            })?;
        }

        Ok(OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| {
                SpriteError::filesystem_with_source(
                    format!("Failed to create log file: {}", e),
                    path.display().to_string(),
                    e,
                )
            })?)
    }

    /// Get the size of a file.
    #[allow(dead_code)]
    fn get_file_size(path: &Path) -> Result<u64> {
        let metadata = std::fs::metadata(path).map_err(|e| {
            SpriteError::filesystem_with_source(
                format!("Failed to get file size: {}", e),
                path.display().to_string(),
                e,
            )
        })?;
        Ok(metadata.len())
    }

    /// Write a log entry with rotation.
    #[allow(dead_code)]
    fn write_log_with_rotation(&self, level: LevelFilter, record: &Record) {
        let message = record.args().to_string();
        let formatted = self.format_message(level, &message, record.file(), record.line());

        // Check if we need to rotate
        if self.current_size.load(std::sync::atomic::Ordering::Relaxed) >= self.max_size {
            eprintln!("Log rotation needed but not implemented in this context");
        }

        // Write to current file
        if let Some(ref file_mutex) = self.file {
            if let Ok(mut file) = file_mutex.lock() {
                let content = format!("{}\n", formatted);
                if file.write(content.as_bytes()).is_ok() {
                    // Note: actual size tracking would require getting the written bytes count
                }
                let _ = file.flush();
            }
        }

        // Also write to stdout if configured
        if self.config.log_to_stdout {
            eprintln!("{}", formatted);
        }
    }

    /// Rotate log files when they get too large.
    #[allow(dead_code)]
    fn rotate_files(&self) -> Result<()> {
        // For now, just log that rotation is needed
        eprintln!("Log rotation is needed but not fully implemented");
        Ok(())
    }

    /// Clean up old log files.
    #[allow(dead_code)]
    fn cleanup_old_files(&self) -> Result<()> {
        // List files matching the pattern
        if let Ok(entries) = std::fs::read_dir(&self.base_path) {
            let mut files: Vec<_> = entries
                .filter_map(|entry| entry.ok())
                .filter(|entry| {
                    let binding = entry.file_name();
                    let file_name = binding.to_string_lossy();
                    file_name.ends_with(".log")
                        && Some(file_name.as_ref())
                            != self
                                .current_file
                                .file_name()
                                .map(|n| n.to_string_lossy())
                                .as_deref()
                })
                .collect();

            // Sort by modification time (oldest first)
            files.sort_by_key(|entry| {
                entry
                    .metadata()
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
            });

            // Keep only the last 5 files
            if files.len() > 5 {
                for old_file in &files[..files.len() - 5] {
                    let _ = std::fs::remove_file(old_file.path());
                    eprintln!("Removed old log file: {}", old_file.path().display());
                }
            }
        }

        Ok(())
    }

    /// Format a log message.
    #[allow(dead_code)]
    fn format_message(
        &self,
        level: LevelFilter,
        message: &str,
        file: Option<&str>,
        line: Option<u32>,
    ) -> String {
        let mut parts = Vec::new();

        if self.config.include_timestamp {
            parts.push(Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
        }

        let level_str = match level {
            LevelFilter::Error => "\x1b[31mERROR\x1b[0m".to_string(),
            LevelFilter::Warn => "\x1b[33mWARN \x1b[0m".to_string(),
            LevelFilter::Info => "\x1b[32mINFO \x1b[0m".to_string(),
            LevelFilter::Debug => "\x1b[36mDEBUG\x1b[0m".to_string(),
            LevelFilter::Trace => "\x1b[37mTRACE\x1b[0m".to_string(),
            _ => "UNKNOWN".to_string(),
        };
        parts.push(level_str);

        if self.config.include_module {
            if let Some(file) = file {
                let module = file.rfind('/').map(|i| &file[i + 1..]).unwrap_or(file);
                parts.push(format!("[{}]", module));
            }
        }

        if let Some(line) = line {
            parts.push(format!(":{}", line));
        }

        parts.push(message.to_string());

        parts.join(" ")
    }

    /// Check if the given log level should be logged.
    #[allow(dead_code)]
    fn should_log(&self, level: LevelFilter) -> bool {
        level <= self.config.level
    }

    /// Get the current log file path.
    #[allow(dead_code)]
    pub fn current_file(&self) -> &PathBuf {
        &self.current_file
    }

    /// Get the logger start time.
    #[allow(dead_code)]
    pub fn start_time(&self) -> DateTime<Utc> {
        self.start_time
    }
}

impl Log for RotatingLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.should_log(metadata.level().to_level_filter())
    }

    fn log(&self, record: &Record) {
        if self.should_log(record.level().to_level_filter()) {
            self.write_log_with_rotation(record.level().to_level_filter(), record);
        }
    }

    fn flush(&self) {
        if let Some(ref file_mutex) = self.file {
            if let Ok(mut file) = file_mutex.lock() {
                let _ = file.flush();
            }
        }
    }
}

/// Initialize rotating logging.
#[allow(dead_code)]
pub fn init_rotating_logging<P: AsRef<Path>>(
    log_dir: P,
    max_size_mb: u64,
    level: Level,
) -> Result<()> {
    let max_size_bytes = max_size_mb * 1024 * 1024;
    let logger = RotatingLogger::new(log_dir, max_size_bytes, level)?;
    let _ = log::set_boxed_logger(Box::new(logger));
    Ok(())
}

/// Initialize rotating logging with default values.
#[allow(dead_code)]
pub fn init_default_rotating_logging() -> Result<()> {
    init_rotating_logging("logs", 10, Level::Info)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_logger_config_default() {
        let config = LoggerConfig::default();
        assert!(config.log_file.is_some());
        assert_eq!(config.level, Level::Info);
        assert!(config.log_to_stdout);
        assert!(config.include_timestamp);
        assert!(config.include_module);
        assert!(config.use_colors);
    }

    #[test]
    fn test_sprite_logger_creation() {
        let config = LoggerConfig {
            log_file: None,
            level: Level::Debug,
            log_to_stdout: false,
            include_timestamp: false,
            include_module: false,
            use_colors: false,
        };

        let logger = SpriteLogger::new(config);
        assert!(logger.is_ok());
    }

    #[test]
    fn test_rotating_logger() {
        let temp_dir = TempDir::new().unwrap();
        let logger = RotatingLogger::new(&temp_dir.path(), 100, Level::Info);
        assert!(logger.is_ok());
    }
}
