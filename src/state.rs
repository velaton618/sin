use crate::models::gender::Gender;

#[derive(Clone, Default, Debug)]
pub enum State {
    #[default]
    Start,
    ReceiveAge,
    ReceiveNickname {
        age: u8,
    },
    ReceiveGender {
        age: u8,
        nickname: String,
    },
    Completed {
        age: u8,
        nickname: String,
        gender: Gender,
    },
}
