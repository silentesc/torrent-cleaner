use std::fmt;

pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let log_level_str = match self {
            LogLevel::Trace => String::from("TRACE"),
            LogLevel::Debug => String::from("DEBUG"),
            LogLevel::Info => String::from("INFO"),
            LogLevel::Warn => String::from("WARN"),
            LogLevel::Error => String::from("ERROR"),
        };
        write!(f, "{}", log_level_str)
    }
}

impl LogLevel {
    pub fn from_string(log_level: &str) -> Self {
        match log_level {
            "TRACE" => LogLevel::Trace,
            "DEBUG" => LogLevel::Debug,
            "INFO" => LogLevel::Info,
            "WARN" => LogLevel::Warn,
            "ERROR" => LogLevel::Error,
            _ => LogLevel::Info,
        }
    }

    pub fn to_int(&self) -> i32 {
        match self {
            LogLevel::Trace => 0,
            LogLevel::Debug => 1,
            LogLevel::Info => 2,
            LogLevel::Warn => 3,
            LogLevel::Error => 4,
        }
    }

    pub fn to_colored_string(&self) -> String {
        const RESET: &str = "\x1b[0m";
        const RED: &str = "\x1b[31m";
        const YELLOW: &str = "\x1b[33m";
        const GREEN: &str = "\x1b[32m";
        const BLUE: &str = "\x1b[34m";
        const GRAY: &str = "\x1b[90m";

        match self {
            LogLevel::Trace => format!("{}TRACE{}", GRAY, RESET),
            LogLevel::Debug => format!("{}DEBUG{}", BLUE, RESET),
            LogLevel::Info => format!("{}INFO{}", GREEN, RESET),
            LogLevel::Warn => format!("{}WARN{}", YELLOW, RESET),
            LogLevel::Error => format!("{}ERROR{}", RED, RESET),
        }
    }
}
