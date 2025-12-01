use std::sync::atomic::{AtomicI32, Ordering};

use chrono::Local;

use crate::logger::enums::log_level::LogLevel;

pub struct Logger;

static LOG_LEVEL: AtomicI32 = AtomicI32::new(0);

impl Logger {
    pub fn set_log_level(log_level: LogLevel) {
        LOG_LEVEL.store(log_level.as_int(), Ordering::Relaxed);
    }

    fn log(log_level: LogLevel, message: &str) {
        let current_log_level = LOG_LEVEL.load(Ordering::Relaxed);
        if log_level.as_int() >= current_log_level {
            let date = Local::now();
            println!(
                "{} | {}{} | {}",
                date.format("%Y-%m-%d %H:%M:%S.%3f"),
                log_level.as_colored_string(),
                " ".repeat(5 - log_level.as_string().len()),
                message,
            );
        }
    }

    pub fn trace(message: &str) {
        Self::log(LogLevel::Trace, message);
    }

    pub fn debug(message: &str) {
        Self::log(LogLevel::Debug, message);
    }

    pub fn info(message: &str) {
        Self::log(LogLevel::Info, message);
    }

    pub fn warn(message: &str) {
        Self::log(LogLevel::Warn, message);
    }

    pub fn error(message: &str) {
        Self::log(LogLevel::Error, message);
    }
}
