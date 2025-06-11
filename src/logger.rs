//! Logging utilities for playlist manager operations.

use std::sync::OnceLock;

/// A logger that handles verbose output with optional counters and formatting.
#[derive(Debug)]
pub struct Logger {
    verbose: bool,
}

impl Logger {
    /// Create a new logger with the specified verbose flag.
    pub fn new(verbose: bool) -> Self {
        Logger { verbose }
    }

    /// Log a simple message if verbose mode is enabled.
    pub fn log(&self, message: &str) {
        if self.verbose {
            eprintln!("{}", message);
        }
    }

    /// Log a formatted message if verbose mode is enabled.
    pub fn log_formatted(&self, message_template: &str, args: &[&str]) {
        if !self.verbose {
            return;
        }

        let formatted_message = args.iter().fold(message_template.to_string(), |acc, arg| {
            acc.replacen("{}", arg, 1)
        });

        eprintln!("{}", formatted_message);
    }

    /// Log with counters and optional file type formatting.
    pub fn log_with_counters(
        &self,
        message_template: &str,
        args: &[&str],
        current_count: Option<usize>,
        total_count: Option<usize>,
        file_type: Option<&str>,
    ) {
        if !self.verbose {
            return;
        }

        let formatted_message = args.iter().fold(message_template.to_string(), |acc, arg| {
            acc.replacen("{}", arg, 1)
        });

        let message = if let (Some(current), Some(total)) = (current_count, total_count) {
            let counter_prefix = match file_type {
                Some("lyrics") => format!("({}-L/{})", current, total),
                Some("media") => format!("({}-M/{})", current, total),
                _ => format!("({}/{})", current, total),
            };
            format!("{} {}", counter_prefix, formatted_message)
        } else {
            formatted_message
        };

        eprintln!("{}", message);
    }
}

/// Static logger instance - will be initialized once in process_normal_operations
static LOGGER: OnceLock<Logger> = OnceLock::new();

/// Initialize the static logger (called once from process_normal_operations)
pub fn init_logger(verbose: bool) {
    LOGGER.set(Logger::new(verbose)).ok(); // Ignore error if already set
}

/// Get the static logger instance
pub fn get_logger() -> &'static Logger {
    LOGGER.get().expect("Logger not initialized - call init_logger first")
}
