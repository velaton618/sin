#[derive(Debug, Clone, PartialEq)]
pub enum ChatType {
    Regular,
    Vulgar,
}

impl From<i32> for ChatType {
    fn from(value: i32) -> Self {
        match value {
            0 => ChatType::Regular,
            1 => ChatType::Vulgar,
            _ => panic!("Invalid value for ChatType: {}", value),
        }
    }
}
