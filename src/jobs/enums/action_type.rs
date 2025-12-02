use crate::logger::logger::Logger;

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
                Logger::warn(format!("Unknown action type '{}', fallback to 'test'", s).as_str());
                ActionType::Test
            }
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            ActionType::Test => String::from("test"),
            ActionType::Stop => String::from("stop"),
            ActionType::Delete => String::from("delete"),
        }
    }
}
