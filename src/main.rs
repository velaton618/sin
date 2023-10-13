mod command;
mod database;
mod models;
mod state;

use models::{gender::Gender, user::User};
use state::State;
use std::env;
use teloxide::{
    dispatching::{
        dialogue::{self, InMemStorage},
        UpdateHandler,
    },
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
};

use crate::command::Command;

type Dialog = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

async fn initilize() {
    dotenv::dotenv().ok();

    let token = env::var("TELOXIDE_TOKEN").unwrap();
    env::set_var("TELOXIDE_TOKEN", token);
    env::set_var("RUST_LOG", "info");

    pretty_env_logger::init();
    log::info!("Starting bot...");
}

#[tokio::main]
async fn main() {
    initilize().await;

    let bot = Bot::from_env();

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
        .branch(case![State::Start].branch(case![Command::Help].endpoint(help)));

    let message_handler = Update::filter_message()
        .enter_dialogue::<Message, InMemStorage<State>, State>()
        .branch(dptree::case![State::Start].endpoint(start))
        .branch(dptree::case![State::ReceiveAge].endpoint(receive_age))
        .branch(dptree::case![State::ReceiveNickname { age }].endpoint(receive_nickname));

    let callback_query_handler = Update::filter_callback_query()
        .branch(case![State::ReceiveGender { age, nickname }].endpoint(receive_gender));

    dialogue::enter::<Update, InMemStorage<State>, State, _>()
        .branch(message_handler)
        .branch(callback_query_handler)
}

async fn help(bot: Bot, dialog: Dialog, msg: Message) {}

async fn start(bot: Bot, dialog: Dialog, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "Добро пожаловать в анонимный чат Sin! Для общения, введите ваш возраст: ",
    )
    .await?;
    dialog.update(State::ReceiveAge).await?;
    Ok(())
}

async fn receive_age(bot: Bot, dialog: Dialog, msg: Message) -> HandlerResult {
    match msg.text().map(|text| text.parse::<u8>()) {
        Some(Ok(age)) => {
            if age < 12 {
                bot.send_message(msg.chat.id, "Эй, ты ещё ребенок!").await?;
                dialog.update(State::Start).await?;
            } else {
                bot.send_message(
                    msg.chat.id,
                    "Теперь введи псевдоним который будет публичным (его можно будет изменить!)",
                )
                .await?;

                dialog.update(State::ReceiveNickname { age: age }).await?;
            }
        }
        _ => {
            bot.send_message(msg.chat.id, "Пытаешься найти баг? Давай заново!")
                .await?;
        }
    }

    Ok(())
}

async fn receive_nickname(bot: Bot, dialog: Dialog, msg: Message, age: u8) -> HandlerResult {
    match msg.text().map(ToOwned::to_owned) {
        Some(nickname) => {
            let genders = ["Мужской", "Женский"]
                .map(|product| InlineKeyboardButton::callback(product, product));
            bot.send_message(msg.chat.id, "Теперь выбери свой тип письки")
                .reply_markup(InlineKeyboardMarkup::new([genders]))
                .await?;
            dialog
                .update(State::ReceiveGender { age, nickname })
                .await?;
        }
        _ => {
            bot.send_message(msg.chat.id, "Пытаешься найти баг? Давай заново!")
                .await?;
        }
    }

    Ok(())
}

async fn receive_gender(
    bot: Bot,
    dialog: Dialog,
    (age, nickname): (u8, String),
    q: CallbackQuery,
) -> HandlerResult {
    if let Some(g) = &q.data {
        let gender;
        if g == "Мужской" {
            gender = Gender::Male;
        } else {
            gender = Gender::Female;
        }
        let user = User::new(dialog.chat_id().0, age, nickname, gender);
    }

    Ok(())
}
