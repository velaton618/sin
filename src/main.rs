mod callbacks;
mod command;
mod commands;
mod database;
mod messages;
mod models;
mod state;
mod user_state;

use database::Database;
use state::State;
use std::env;
use teloxide::{
    dispatching::{
        dialogue::{self, InMemStorage},
        UpdateHandler,
    },
    prelude::*,
};
use tokio::sync::Mutex as TokioMutex;

use crate::{
    callbacks::{
        chat_type_callback, reactions_callback, receive_gender, receive_set_gender, search_callback,
    },
    command::Command,
    commands::{
        admin, admin_message, ban, cancel, delete_user, idle, next, referral, rules, start, stop,
        top, top_rep, unban, user_info,
    },
    messages::{
        dialog_search, receive_age, receive_message, receive_nickname, receive_set_age,
        receive_set_nickname, set_age, set_gender, set_name,
    },
};

use once_cell::sync::OnceCell;

static DATABASE: OnceCell<TokioMutex<Database>> = OnceCell::new();

type Dialog = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

async fn initilize() {
    dotenv::dotenv().ok();

    let token = env::var("TELOXIDE_TOKEN").unwrap();
    env::set_var("TELOXIDE_TOKEN", token);
    env::set_var("RUST_LOG", "warn");

    pretty_env_logger::init();
    log::info!("Starting bot...");
}

#[tokio::main]
async fn main() {
    initilize().await;

    let db = DATABASE.get_or_init(|| TokioMutex::new(Database::new("db.db").unwrap()));
    let users = db.lock().await.get_all_users().unwrap();

    let bot = Bot::from_env();

    // for user in users {
    //     let _ = bot
    //         .send_message(ChatId(user.id), "Наш бот перезагрузился")
    //         .await;
    // }

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![InMemStorage::<State>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use dptree::case;

    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(case![Command::Start].endpoint(start))
        .branch(case![State::Dialog { interlocutor }].branch(case![Command::Stop].endpoint(stop)))
        .branch(case![Command::Search].endpoint(idle))
        .branch(case![Command::Next].endpoint(next))
        .branch(case![Command::Cancel].endpoint(cancel))
        .branch(case![Command::Referral].endpoint(referral))
        .branch(case![Command::Top].endpoint(top))
        .branch(case![Command::TopRep].endpoint(top_rep))
        .branch(
            case![State::Dialog { interlocutor }]
                .branch(case![Command::Search].endpoint(dialog_search)),
        )
        .branch(case![Command::SetName].endpoint(set_name))
        .branch(case![Command::Message].endpoint(admin_message))
        .branch(case![Command::Delete].endpoint(delete_user))
        .branch(case![Command::Admin].endpoint(admin))
        .branch(case![Command::Rules].endpoint(rules))
        .branch(case![Command::Unban].endpoint(unban))
        .branch(case![Command::Ban].endpoint(ban))
        .branch(case![Command::UserInfo].endpoint(user_info))
        .branch(case![Command::SetAge].endpoint(set_age))
        .branch(case![Command::SetGender].endpoint(set_gender));

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .enter_dialogue::<Message, InMemStorage<State>, State>()
        .branch(dptree::case![State::Idle].endpoint(idle))
        .branch(dptree::case![State::Start].endpoint(start))
        .branch(dptree::case![State::SetAge].endpoint(receive_set_age))
        .branch(dptree::case![State::SetNickname].endpoint(receive_set_nickname))
        .branch(dptree::case![State::ReceiveAge].endpoint(receive_age))
        .branch(dptree::case![State::ReceiveNickname { age }].endpoint(receive_nickname))
        .branch(dptree::case![State::Search].endpoint(receive_message))
        .branch(dptree::case![State::Dialog { interlocutor }].endpoint(receive_message));

    let callback_query_handler = Update::filter_callback_query()
        .branch(case![State::ReceiveGender { age, nickname }].endpoint(receive_gender))
        .branch(case![State::SearchChooseChatType { gender }].endpoint(chat_type_callback))
        .branch(dptree::case![State::SearchChooseGender])
        .branch(dptree::case![State::Search])
        .branch(dptree::case![State::SetGender].endpoint(receive_set_gender))
        .branch(case![State::Idle].endpoint(reactions_callback))
        .endpoint(search_callback);

    dialogue::enter::<Update, InMemStorage<State>, State, _>()
        .branch(message_handler)
        .branch(callback_query_handler)
}
