use crate::logger::{enums::category::Category, logger::Logger};

pub enum ActionType {
    Test,
    Stop,
    Delete,
}

impl ActionType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "test" => ActionType::Test,
            "stop" => ActionType::Stop,
            "delete" => ActionType::Delete,
            _ => {
                Logger::warn(Category::Setup, format!("Unknown action type '{}', fallback to 'test'", s).as_str()); // TODO change to proper error response
                ActionType::Test
            }
        }
    }
}
