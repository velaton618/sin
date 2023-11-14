mod command;
mod database;
mod models;
mod state;
mod user_state;

use database::Database;
use log::debug;
use models::{gender::Gender, user::User};
use state::State;
use std::env;
use teloxide::{
    dispatching::{
        dialogue::{self, InMemStorage},
        UpdateHandler,
    },
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, InputFile},
};
use tokio::sync::Mutex as TokioMutex;
use user_state::UserState;

use crate::{command::Command, models::chat_type::ChatType};

use once_cell::sync::OnceCell;

static DATABASE: OnceCell<TokioMutex<Database>> = OnceCell::new();

type Dialog = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

async fn initilize() {
    dotenv::dotenv().ok();

    let token = env::var("TELOXIDE_TOKEN").unwrap();
    env::set_var("TELOXIDE_TOKEN", token);
    env::set_var("RUST_LOG", "debug");

    pretty_env_logger::init();
    log::info!("Starting bot...");
}

#[tokio::main]
async fn main() {
    initilize().await;

    let db = DATABASE.get_or_init(|| TokioMutex::new(Database::new("db.db").unwrap()));
    let users = db.lock().await.get_all_users().unwrap();

    let bot = Bot::from_env();

    for user in users {
        bot.send_message(ChatId(user.id), "Наш бот перезагрузился")
            .await
            .unwrap();
    }

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
        .branch(case![Command::Cancel].endpoint(cancel))
        .branch(
            case![State::Dialog { interlocutor }]
                .branch(case![Command::Search].endpoint(dialog_search)),
        )
        .branch(case![Command::SetName].endpoint(set_name))
        .branch(case![Command::SetAge].endpoint(set_age));

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
        .branch(dptree::case![State::SearchChoose])
        .branch(dptree::case![State::Search])
        .endpoint(search_callback);

    dialogue::enter::<Update, InMemStorage<State>, State, _>()
        .branch(message_handler)
        .branch(callback_query_handler)
}

async fn dialog_search(bot: Bot, dialog: Dialog, _: Message) -> HandlerResult {
    bot.send_message(dialog.chat_id(), "Ты уже в диалоге!")
        .await
        .unwrap();

    Ok(())
}

async fn idle(bot: Bot, dialog: Dialog, msg: Message) -> HandlerResult {
    if let Some(txt) = msg.text() {
        if txt.contains("search") {
            let db = DATABASE
                .get_or_init(|| TokioMutex::new(Database::new("db.db").unwrap()))
                .lock()
                .await;

            let user = db.get_user(dialog.chat_id().0);

            if user.is_ok() && user.as_ref().unwrap().is_some() {
                let user = user.unwrap().unwrap();
                if user.state == UserState::Dialog {
                    bot.send_message(dialog.chat_id(), "Ты не готов к поиску! Останови диалог")
                        .await?;

                    return Ok(());
                } else if user.state == UserState::Search {
                    bot.send_message(dialog.chat_id(), "Не мешай! Я ищу")
                        .await?;

                    return Ok(());
                }
            } else {
                bot.send_message(
                    dialog.chat_id(),
                    "Ты не готов к поиску! Зарегестрируйся!\n\n/start",
                )
                .await?;

                return Ok(());
            }

            let genders =
                ["🍌", "🍑"].map(|product| InlineKeyboardButton::callback(product, product));
            bot.send_message(dialog.chat_id(), "Теперь выбери пол собеседника")
                .reply_markup(InlineKeyboardMarkup::new([genders]))
                .await
                .unwrap();
            dialog.update(State::SearchChoose).await.unwrap();
        } else {
            receive_message(bot, dialog, msg).await?;
        }
    } else {
        receive_message(bot, dialog, msg).await?;
    }

    Ok(())
}

async fn start(bot: Bot, dialog: Dialog, msg: Message) -> HandlerResult {
    let db = DATABASE
        .get_or_init(|| TokioMutex::new(Database::new("db.db").unwrap()))
        .lock()
        .await;

    let user = db.get_user(dialog.chat_id().0);

    if user.is_ok() && user.as_ref().unwrap().is_some() {
        idle(bot, dialog, msg).await?;
    } else {
        bot.send_message(msg.chat.id, "Добро пожаловать в анонимный чат Sin!")
            .await?;
        bot.send_message(
            msg.chat.id,
            "Нужно зарегестрироваться! Введи свой возраст: ",
        )
        .await?;
        dialog.update(State::ReceiveAge).await?;
    }

    Ok(())
}

async fn set_name(bot: Bot, dialog: Dialog, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "Введите свой новый никнейм(он будет публичным): ",
    )
    .await?;
    dialog.update(State::SetNickname).await?;

    Ok(())
}

async fn set_age(bot: Bot, dialog: Dialog, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Введите свой возраст(он будет публичным): ")
        .await?;

    dialog.update(State::SetAge).await?;

    Ok(())
}

async fn receive_set_age(bot: Bot, dialog: Dialog, msg: Message) -> HandlerResult {
    match msg.text().map(|text| text.parse::<u8>()) {
        Some(Ok(age)) => {
            if age < 12 {
                bot.send_message(msg.chat.id, "Эй, ты ещё ребенок!").await?;
                dialog.update(State::Idle).await?;
            } else {
                let db = DATABASE
                    .get_or_init(|| TokioMutex::new(Database::new("db.db").unwrap()))
                    .lock()
                    .await;
                db.update_user_age(msg.chat.id.0, age).unwrap();

                bot.send_message(msg.chat.id, "Готово").await?;

                dialog.update(State::Idle).await?;
            }
        }
        _ => {
            bot.send_message(msg.chat.id, "Пытаешься найти баг? Давай заново!")
                .await?;
        }
    }

    Ok(())
}

async fn receive_set_nickname(bot: Bot, dialog: Dialog, msg: Message) -> HandlerResult {
    match msg.text().map(ToOwned::to_owned) {
        Some(nickname) => {
            let db = DATABASE
                .get_or_init(|| TokioMutex::new(Database::new("db.db").unwrap()))
                .lock()
                .await;

            db.update_user_nickname(msg.chat.id.0, &nickname).unwrap();
            bot.send_message(msg.chat.id, "Готово").await?;

            dialog.update(State::Idle).await?;
        }
        _ => {
            bot.send_message(msg.chat.id, "Пытаешься найти баг? Давай заново!")
                .await?;
        }
    }

    Ok(())
}

async fn stop(bot: Bot, dialog: Dialog, msg: Message) -> HandlerResult {
    let db = DATABASE
        .get_or_init(|| TokioMutex::new(Database::new("db.db").unwrap()))
        .lock()
        .await;

    let intr = db.delete_chat(dialog.chat_id().0);
    dialog.update(State::Idle).await?;

    if intr.is_ok() {
        let intr = intr.unwrap();

        if intr.is_some() {
            let intr = intr.unwrap();

            db.set_user_state(msg.chat.id.0, UserState::Idle).unwrap();
            db.set_user_state(intr, UserState::Idle).unwrap();

            bot.send_message(
                msg.chat.id,
                " Диалог остановлен!\n\n/search - найти нового собеседника",
            )
            .await?;
            bot.send_message(ChatId(intr), "Твой собеседник остановил диалог!!")
                .await?;
        } else {
            bot.send_message(msg.chat.id, "Ты не находишься в диалоге!")
                .await?;
        }
    } else {
        bot.send_message(msg.chat.id, "Ты не находишься в диалоге!")
            .await?;
    }
    dialog.update(State::Idle).await?;

    Ok(())
}

async fn cancel(bot: Bot, dialog: Dialog, msg: Message) -> HandlerResult {
    let db = DATABASE
        .get_or_init(|| TokioMutex::new(Database::new("db.db").unwrap()))
        .lock()
        .await;
    db.dequeue_user(msg.chat.id.0).unwrap();
    bot.send_message(msg.chat.id, "Поиск отменён!").await?;
    dialog.update(State::Idle).await?;
    db.set_user_state(msg.chat.id.0, UserState::Idle).unwrap();

    Ok(())
}

async fn receive_message(bot: Bot, dialog: Dialog, msg: Message) -> HandlerResult {
    if let Some(txt) = msg.text() {
        if txt.contains("search") {
            bot.send_message(msg.chat.id, "Не мешай! Я ищу").await?;
            return Ok(());
        }

        if txt.contains("stop") {
            stop(bot, dialog, msg).await?;
            return Ok(());
        }
    }
    let db = Database::new("db.db").unwrap();

    let chat = db.get_chat(dialog.chat_id().0);

    if chat.is_ok() {
        let chat = chat.unwrap();

        if chat.is_some() {
            let chat = chat.unwrap();

            dialog
                .update(State::Dialog {
                    interlocutor: chat as u64,
                })
                .await?;

            if let Some(voice) = msg.voice() {
                bot.send_audio(ChatId(chat), InputFile::file_id(&voice.file.id))
                    .await?;
            } else if let Some(sticker) = msg.sticker() {
                bot.send_sticker(ChatId(chat), InputFile::file_id(&sticker.file.id))
                    .await?;
            } else if let Some(photo) = msg.photo() {
                if let Some(txt) = msg.caption() {
                    bot.send_photo(
                        ChatId(chat),
                        InputFile::file_id(&photo.last().unwrap().file.id),
                    )
                    .caption(txt)
                    .await?;
                } else {
                    bot.send_photo(
                        ChatId(chat),
                        InputFile::file_id(&photo.last().unwrap().file.id),
                    )
                    .await?;
                }
            } else if let Some(video) = msg.video() {
                if let Some(txt) = msg.caption() {
                    bot.send_video(ChatId(chat), InputFile::file_id(&video.file.id))
                        .caption(txt)
                        .await?;
                } else {
                    bot.send_video(ChatId(chat), InputFile::file_id(&video.file.id))
                        .await?;
                }
            } else if let Some(sticker) = msg.sticker() {
                bot.send_sticker(ChatId(chat), InputFile::file_id(&sticker.file.id))
                    .await?;
            } else if let Some(txt) = msg.text() {
                bot.send_message(ChatId(chat), txt).await?;
            } else {
                bot.send_message(
                    ChatId(chat),
                    "Такой формат сообщения пока что не поддерживается",
                )
                .await?;
            }
        } else {
            bot.send_message(msg.chat.id, "Ты не в диалоге! /search чтобы попасть туда!")
                .await?;
            dialog.update(State::Idle).await?;
        }
    } else {
        bot.send_message(msg.chat.id, "Ты не в диалоге! /search чтобы попасть туда!")
            .await?;
        dialog.update(State::Idle).await?;
    }

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
            let genders =
                ["🍌", "🍑"].map(|product| InlineKeyboardButton::callback(product, product));
            bot.send_message(msg.chat.id, "Теперь выбери свой пол")
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

async fn search_callback(bot: Bot, dialog: Dialog, q: CallbackQuery) -> HandlerResult {
    if let Some(g) = &q.data {
        if g == "cancel" {
            let db = DATABASE
                .get_or_init(|| TokioMutex::new(Database::new("db.db").unwrap()))
                .lock()
                .await;
            db.dequeue_user(dialog.chat_id().0).unwrap();
            bot.send_message(dialog.chat_id(), "Поиск отменён!").await?;
            dialog.update(State::Idle).await?;
            db.set_user_state(dialog.chat_id().0, UserState::Idle)
                .unwrap();

            return Ok(());
        }

        let gender;
        if g == "🍌" {
            gender = Gender::Male;
        } else {
            gender = Gender::Female;
        }
        let db = DATABASE
            .get_or_init(|| TokioMutex::new(Database::new("db.db").unwrap()))
            .lock()
            .await;

        let user = db.get_user(dialog.chat_id().0);
        debug!("{:?}", user);
        if user.is_ok() {
            let user = user.unwrap();

            if user.is_some() {
                let user = user.unwrap();
                let result =
                    db.enqueue_user(dialog.chat_id().0, gender, user.gender, ChatType::Vulgar);

                println!("{:?}", result);

                if result.is_ok() {
                    let result = result.unwrap();
                    let cancel = [InlineKeyboardButton::callback("❌ Отменить", "cancel")];
                    bot.send_message(dialog.chat_id(), "Ищу...")
                        .reply_markup(InlineKeyboardMarkup::new([cancel]))
                        .await?;
                    dialog.update(State::Search).await?;

                    db.set_user_state(user.id, user_state::UserState::Search)
                        .unwrap();

                    if result != 0 {
                        dialog
                            .update(State::Dialog {
                                interlocutor: result as u64,
                            })
                            .await?;
                        let interlocutor = db.get_user(result).unwrap().unwrap();
                        bot.send_message(
                            dialog.chat_id(),
                            format!(
                            "{} {} ({})\n\nСобеседник найден!\n\n/stop - чтобы оставноить диалог",
                            if interlocutor.gender == Gender::Male {
                                "🍌"
                            } else {
                                "🍑"
                            },
                            interlocutor.nickname,
                            interlocutor.age
                        ),
                        )
                        .await?;
                        bot.send_message(
                            ChatId(result),
                            format!(
                            "{} {} ({})\n\nСобеседник найден!\n\n/stop - чтобы оставноить диалог",
                            if user.gender.clone() == Gender::Male {
                                "🍌"
                            } else {
                                "🍑"
                            },
                            user.nickname,
                            user.age
                        ),
                        )
                        .await?;
                        db.set_user_state(user.id, user_state::UserState::Dialog)
                            .unwrap();
                        db.set_user_state(result, user_state::UserState::Dialog)
                            .unwrap();
                    }
                } else {
                    bot.send_message(dialog.chat_id(), format!("Ой! Голова кружится...",))
                        .await?;
                }
            } else {
                bot.send_message(dialog.chat_id(), format!("Ой! Голова кружится...",))
                    .await?;
            }
        } else {
            bot.send_message(dialog.chat_id(), format!("Ой! Голова кружится...",))
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
        if g == "🍌" {
            gender = Gender::Male;
        } else {
            gender = Gender::Female;
        }

        let user = User::new(dialog.chat_id().0, age, nickname.clone(), gender.clone());
        let db = DATABASE
            .get_or_init(|| TokioMutex::new(Database::new("db.db").unwrap()))
            .lock()
            .await;

        db.add_user(&user).unwrap();

        bot.send_message(
            dialog.chat_id(),
            format!(
                "Готово!\n\n{} {} {}",
                nickname,
                age,
                (if gender == Gender::Male {
                    "🍌"
                } else {
                    "🍑"
                })
            ),
        )
        .await?;

        let genders = ["🍌", "🍑"].map(|product| InlineKeyboardButton::callback(product, product));
        bot.send_message(dialog.chat_id(), "Теперь выбери пол собеседника")
            .reply_markup(InlineKeyboardMarkup::new([genders]))
            .await?;
        dialog.update(State::SearchChoose).await?;
    }

    Ok(())
}
