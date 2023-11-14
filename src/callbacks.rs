use log::debug;
use teloxide::{
    payloads::SendMessageSetters,
    requests::Requester,
    types::{CallbackQuery, ChatId, InlineKeyboardButton, InlineKeyboardMarkup},
    Bot,
};
use tokio::sync::Mutex as TokioMutex;

use crate::{
    database::Database,
    messages,
    models::{chat_type::ChatType, gender::Gender, user::User},
    state::State,
    user_state::{self, UserState},
    Dialog, HandlerResult, DATABASE,
};

pub async fn receive_gender(
    bot: Bot,
    dialog: Dialog,
    (age, nickname): (u8, String),
    q: CallbackQuery,
) -> HandlerResult {
    bot.delete_message(dialog.chat_id(), q.message.unwrap().id)
        .await?;

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
        dialog.update(State::SearchChooseGender).await?;
    }

    Ok(())
}

pub async fn search_callback(bot: Bot, dialog: Dialog, q: CallbackQuery) -> HandlerResult {
    bot.delete_message(dialog.chat_id(), q.message.clone().unwrap().id)
        .await?;

    if let Some(g) = &q.data {
        if g.contains("like") {
            reactions_callback(bot, dialog, q).await?;
            return Ok(());
        }
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
        let cancel = [
            InlineKeyboardButton::callback("💬", "regular"),
            InlineKeyboardButton::callback("🔞", "vulgar"),
        ];
        bot.send_message(dialog.chat_id(), "Теперь выбери тип разговора")
            .reply_markup(InlineKeyboardMarkup::new([cancel]))
            .await?;

        dialog
            .update(State::SearchChooseChatType { gender })
            .await?;
    }

    Ok(())
}

pub async fn reactions_callback(bot: Bot, dialog: Dialog, q: CallbackQuery) -> HandlerResult {
    if let Some(msg) = q.message {
        let _ = bot.delete_message(dialog.chat_id(), msg.id).await;
    }

    dialog.update(State::Idle).await?;

    if let Some(g) = &q.data {
        println!("G:::{}", g);
        let db = DATABASE
            .get_or_init(|| TokioMutex::new(Database::new("db.db").unwrap()))
            .lock()
            .await;

        if g.contains("dislike") {
            if let Some(id) = g.split("_").nth(1) {
                db.decrease_reputation(id.parse::<i64>().unwrap(), 1)
                    .unwrap();
            }
        } else {
            if let Some(id) = g.split("_").nth(1) {
                db.increase_reputation(id.parse::<i64>().unwrap(), 1)
                    .unwrap();
            }
        }
    }
    Ok(())
}

pub async fn chat_type_callback(
    bot: Bot,
    dialog: Dialog,
    q: CallbackQuery,
    gender: Gender,
) -> HandlerResult {
    bot.delete_message(dialog.chat_id(), q.message.unwrap().id)
        .await?;

    if let Some(g) = &q.data {
        let mut chat_type = ChatType::Regular;
        if g == "regular" {
            chat_type = ChatType::Regular;
        } else if g == "vulgar" {
            chat_type = ChatType::Vulgar;
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
                if user.is_banned {
                    bot.send_message(ChatId(user.id), "Вы заблокаированы!")
                        .await?;
                    return Ok(());
                }
                let result =
                    db.enqueue_user(dialog.chat_id().0, gender, user.gender, chat_type.clone());

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
                            "{}\n\n{} {} ({})\n\nСобеседник найден!\n\n/stop - чтобы остановить диалог",
                            if chat_type == ChatType::Regular {
                                "💬"
                            } else {
                                "🔞"
                            },
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
                            "{}\n\n{} {} ({})\n\nСобеседник найден!\n\n/stop - чтобы остановить диалог",
                             if chat_type == ChatType::Regular {
                                "💬"
                            } else {
                                "🔞"
                            },
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