use std::fmt;

#[derive(Clone)]
pub enum StrikeType {
    HandleUnlinked,
    HandleUnregistered,
    HandleOrphaned,
}

impl fmt::Display for StrikeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let strike_type_str = match self {
            StrikeType::HandleUnlinked => String::from("handle_unlinked"),
            StrikeType::HandleUnregistered => String::from("handle_unregistered"),
            StrikeType::HandleOrphaned => String::from("handle_orphaned"),
        };
        write!(f, "{}", strike_type_str)
    }
}
