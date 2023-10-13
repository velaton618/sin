use super::gender::Gender;

#[derive(Debug)]
pub struct User {
    pub id: i64,
    pub nickname: String,
    pub age: u8,
    pub gender: Gender,
}
impl User {
    pub fn new(id: i64, age: u8, nickname: String, gender: Gender) -> Self {
        Self {
            id: id,
            nickname: nickname,
            age: age,
            gender: gender,
        }
    }
}
