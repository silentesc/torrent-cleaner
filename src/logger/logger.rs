use std::sync::atomic::{AtomicI32, Ordering};

use chrono::Local;

use crate::logger::enums::{category::Category, log_level::LogLevel};

pub struct Logger;

static LOG_LEVEL: AtomicI32 = AtomicI32::new(0);

impl Logger {
    pub fn set_log_level(log_level: LogLevel) {
        LOG_LEVEL.store(log_level.to_int(), Ordering::Relaxed);
    }

    fn log(category: Category, log_level: LogLevel, message: &str) {
        let current_log_level = LOG_LEVEL.load(Ordering::Relaxed);
        if log_level.to_int() >= current_log_level {
            let date = Local::now();
            println!(
                "{} | {}{} | [{}] {}",
                date.format("%Y-%m-%d %H:%M:%S.%3f"),
                log_level.to_colored_string(),
                " ".repeat(5 - log_level.to_string().len()),
                category.to_string(),
                message,
            );
        }
    }

    pub fn trace(category: Category, message: &str) {
        Self::log(category, LogLevel::Trace, message);
    }

    pub fn debug(category: Category, message: &str) {
        Self::log(category, LogLevel::Debug, message);
    }

    pub fn info(category: Category, message: &str) {
        Self::log(category, LogLevel::Info, message);
    }

    pub fn warn(category: Category, message: &str) {
        Self::log(category, LogLevel::Warn, message);
    }

    pub fn error(category: Category, message: &str) {
        Self::log(category, LogLevel::Error, message);
    }
}
