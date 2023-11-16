use crate::user_state::UserState;

use super::{chat_type::ChatType, gender::Gender};

#[derive(Debug)]
pub struct User {
    pub id: i64,
    pub nickname: String,
    pub age: u8,
    pub gender: Gender,
    pub search_gender: Option<Gender>,
    pub chat_type: Option<ChatType>,
    pub state: UserState,
    pub reputation: i32,
    pub is_banned: bool,
    pub referrals: u32,
}
impl User {
    pub fn new(id: i64, age: u8, nickname: String, gender: Gender) -> Self {
        Self {
            id: id,
            nickname: nickname,
            age: age,
            gender: gender,
            search_gender: None,
            chat_type: None,
            state: UserState::Default,
            reputation: 0,
            is_banned: false,
            referrals: 0,
        }
    }
}
