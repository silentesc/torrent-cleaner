#[derive(Clone)]
pub enum StrikeType {
    HandleForgotten,
    HandleNotWorking,
    HandleOrphaned,
}

impl StrikeType {
    pub fn to_string(&self) -> String {
        match self {
            StrikeType::HandleForgotten => String::from("handle_forgotten"),
            StrikeType::HandleNotWorking => String::from("handle_not_working"),
            StrikeType::HandleOrphaned => String::from("handle_orphaned"),
        }
    }
}
