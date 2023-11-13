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
    SearchChoose,
    Search,
    Dialog {
        interlocutor: u64,
    },
    Idle,
}
