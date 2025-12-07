pub enum ActionType {
    Test,
    Stop,
    Delete,
}

impl ActionType {
    pub fn from_str(s: &str) -> Result<Self, anyhow::Error> {
        match s.to_lowercase().as_str() {
            "test" => Ok(ActionType::Test),
            "stop" => Ok(ActionType::Stop),
            "delete" => Ok(ActionType::Delete),
            _ => Err(anyhow::anyhow!("Unknown action type '{}'", s)),
        }
    }
}
