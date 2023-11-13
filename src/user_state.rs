#[derive(Debug, PartialEq)]
pub enum UserState {
    Default,
    Idle,
    Search,
    Dialog,
}

impl From<i32> for UserState {
    fn from(value: i32) -> Self {
        match value {
            0 => UserState::Default,
            1 => UserState::Idle,
            2 => UserState::Search,
            3 => UserState::Dialog,
            _ => UserState::Default,
        }
    }
}
