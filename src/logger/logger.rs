use std::{
    fmt::Arguments,
    sync::atomic::{AtomicI32, Ordering},
};

use chrono::Local;

use crate::logger::enums::{category::Category, log_level::LogLevel};

static LOG_LEVEL: AtomicI32 = AtomicI32::new(0);

#[macro_export]
macro_rules! trace {
    ($category:expr, $($arg:tt)+) => {
        $crate::logger::logger::Logger::log($category, $crate::logger::enums::log_level::LogLevel::Trace, format_args!($($arg)+))
    };
}

#[macro_export]
macro_rules! debug {
    ($category:expr, $($arg:tt)+) => {
        $crate::logger::logger::Logger::log($category, $crate::logger::enums::log_level::LogLevel::Debug, format_args!($($arg)+))
    };
}

#[macro_export]
macro_rules! info {
    ($category:expr, $($arg:tt)+) => {
        $crate::logger::logger::Logger::log($category, $crate::logger::enums::log_level::LogLevel::Info, format_args!($($arg)+))
    };
}

#[macro_export]
macro_rules! warn {
    ($category:expr, $($arg:tt)+) => {
        $crate::logger::logger::Logger::log($category, $crate::logger::enums::log_level::LogLevel::Warn, format_args!($($arg)+))
    };
}

#[macro_export]
macro_rules! error {
    ($category:expr, $($arg:tt)+) => {
        $crate::logger::logger::Logger::log($category, $crate::logger::enums::log_level::LogLevel::Error, format_args!($($arg)+))
    };
}

pub struct Logger;

impl Logger {
    pub fn set_log_level(log_level: LogLevel) {
        LOG_LEVEL.store(log_level.to_int(), Ordering::Relaxed);
    }

    pub fn log(category: Category, log_level: LogLevel, args: Arguments) {
        let current_log_level = LOG_LEVEL.load(Ordering::Relaxed);
        if log_level.to_int() >= current_log_level {
            let date = Local::now();
            println!(
                "{} | {}{} | [{}] {}",
                date.format("%Y-%m-%d %H:%M:%S.%3f"),
                log_level.to_colored_string(),
                " ".repeat(5 - log_level.to_string().len()),
                category,
                args,
            );
        }
    }
}
