#[derive(Clone)]
pub enum StrikeType {
    HandleUnlinked,
    HandleNotWorking,
    HandleOrphaned,
}

impl StrikeType {
    pub fn to_string(&self) -> String {
        match self {
            StrikeType::HandleUnlinked => String::from("handle_unlinked"),
            StrikeType::HandleNotWorking => String::from("handle_not_working"),
            StrikeType::HandleOrphaned => String::from("handle_orphaned"),
        }
    }
}
