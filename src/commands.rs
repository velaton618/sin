use std::env;

use teloxide::{
    payloads::SendMessageSetters,
    requests::Requester,
    types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup, Message},
    Bot,
};
use tokio::sync::Mutex as TokioMutex;

use crate::{
    database::Database, messages::receive_message, models::gender::Gender, state::State,
    user_state::UserState, Dialog, HandlerResult, DATABASE,
};

pub async fn admin_message(bot: Bot, _: Dialog, msg: Message) -> HandlerResult {
    let admin = env::var("ADMIN").unwrap();
    if msg.chat.id.0.to_string() == admin {}

    let db = DATABASE.get_or_init(|| TokioMutex::new(Database::new("db.db").unwrap()));
    let users = db.lock().await.get_all_users().unwrap();

    for user in users {
        let _ = bot
            .send_message(
                ChatId(user.id),
                format!(
                    "--- SinChat ---\n\n{}",
                    msg.text().unwrap().split("/message").nth(1).unwrap()
                ),
            )
            .await;
    }

    Ok(())
}

pub async fn rules(bot: Bot, _: Dialog, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Что ЗАПРЕЩЕННО в SinChat\n\n💬Общие\nРеклама\nПопрошайничество\nСпам\nНацизм / фашизм / расизм\nБулинг\n\n💬 Обычний чат\nРазговор на темы 18+ \nВыпрашивание интимных фотографий\n\n🔞 Пошлый чат\nОбщаться на НЕ пошлые темы\nИскать друзей\n\nЗа любое нарушение правил ваша репутация снижается, если ваша репутация иже 20, вы будете заблокированы.\n\n⚠️НЕ ЗНАНИЕ ПРАВИЛ, НЕ УБИРАЕТ С ВАС ОТВЕТСВЕННОСТИ⚠️")

        .await?;

    Ok(())
}

pub async fn ban(bot: Bot, _: Dialog, msg: Message) -> HandlerResult {
    if let Some(txt) = msg.text() {
        if txt.split("/ban").nth(1).is_none() {
            bot.send_message(msg.chat.id, format!("Что-то не так"))
                .await?;
            return Ok(());
        }
    }

    let admin = env::var("ADMIN").unwrap();

    if msg.chat.id.0.to_string() == admin {
        let db = Database::new("db.db").unwrap();
        let user = db
            .get_user(
                msg.text()
                    .unwrap()
                    .split("/ban")
                    .nth(1)
                    .unwrap()
                    .trim()
                    .parse::<i64>()
                    .unwrap(),
            )
            .unwrap()
            .unwrap();

        let id = msg
            .text()
            .unwrap_or("/ban")
            .split("/ban")
            .nth(1)
            .unwrap_or("")
            .trim()
            .parse::<i64>()
            .unwrap_or(0);
        if id != 0 {
            db.ban_user(id).unwrap();
            bot.send_message(msg.chat.id, format!("Готово\n\n{:#?}", user))
                .await?;
        } else {
            bot.send_message(msg.chat.id, format!("Что-то не так"))
                .await?;
        }
    }

    Ok(())
}

pub async fn unban(bot: Bot, _: Dialog, msg: Message) -> HandlerResult {
    if let Some(txt) = msg.text() {
        if txt.split("/unban").nth(1).is_none() {
            bot.send_message(msg.chat.id, format!("Что-то не так"))
                .await?;
            return Ok(());
        }
    }

    let admin = env::var("ADMIN").unwrap();

    if msg.chat.id.0.to_string() == admin {
        let db = Database::new("db.db").unwrap();
        let user = db
            .get_user(
                msg.text()
                    .unwrap()
                    .split("/unban")
                    .nth(1)
                    .unwrap()
                    .trim()
                    .parse::<i64>()
                    .unwrap(),
            )
            .unwrap()
            .unwrap();

        let id = msg
            .text()
            .unwrap_or("/unban")
            .split("/unban")
            .nth(1)
            .unwrap_or("")
            .trim()
            .parse::<i64>()
            .unwrap_or(0);
        if id != 0 {
            db.unban_user(id).unwrap();
            bot.send_message(msg.chat.id, format!("Готово\n\n{:#?}", user))
                .await?;
        } else {
            bot.send_message(msg.chat.id, format!("Что-то не так"))
                .await?;
        }
    }

    Ok(())
}

pub async fn user_info(bot: Bot, _: Dialog, msg: Message) -> HandlerResult {
    let admin = env::var("ADMIN").unwrap();

    if msg.chat.id.0.to_string() == admin {
        let db = Database::new("db.db").unwrap();
        let user = db
            .get_user(
                msg.text()
                    .unwrap_or("/userinfo")
                    .split("/userinfo")
                    .nth(1)
                    .unwrap_or("")
                    .trim()
                    .parse::<i64>()
                    .unwrap_or(msg.chat.id.0),
            )
            .unwrap()
            .unwrap();

        bot.send_message(msg.chat.id, format!("{:#?}", user))
            .await?;
    } else {
        let db = Database::new("db.db").unwrap();
        let user = db.get_user(msg.chat.id.0).unwrap().unwrap();

        bot.send_message(
            msg.chat.id,
            format!(
                "{}\n\nНикнейм: {}\nПол: {}\nВозраст: {}\nРепутация: {}",
                user.id,
                user.nickname,
                if user.gender == Gender::Male {
                    "🍌"
                } else {
                    "🍑"
                },
                user.age,
                user.reputation
            ),
        )
        .await?;
    }

    Ok(())
}

pub async fn admin(bot: Bot, _: Dialog, msg: Message) -> HandlerResult {
    let admin = env::var("ADMIN").unwrap();
    let db = Database::new("db.db").unwrap();
    let total_users = db.get_total_users()?;
    let female_count = db.get_female_count()?;
    let male_count = db.get_male_count()?;
    let total_chats = db.get_total_chats()?;
    let total_queue = db.get_queue_count()?;
    let total_male_queue = db.get_male_queue_count()?;
    let total_female_queue = db.get_female_queue_count()?;

    if msg.chat.id.0.to_string() == admin {
        bot.send_message(
            msg.chat.id,
            format!(
                "Users: {}\n🍌 Males: {}\n🍑 Females: {}\n\n💬 Chats: {}\n\nQueue: {}\n\n\n🍌 Queue Males: {}\n🍑 Queue Females: {}",
                total_users, male_count, female_count, total_chats, total_queue, total_male_queue, total_female_queue
            ),
        )
        .await?;
    }

    Ok(())
}

pub async fn stop(bot: Bot, dialog: Dialog, msg: Message) -> HandlerResult {
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

            let reactions = [
                InlineKeyboardButton::callback("👍", format!("like_{}", intr)),
                InlineKeyboardButton::callback("👎", format!("dislike_{}", intr)),
            ];
            bot.send_message(
                dialog.chat_id(),
                "Диалог остановлен!\n\n/search - найти нового собеседника",
            )
            .reply_markup(InlineKeyboardMarkup::new([reactions]))
            .await?;

            let reactions = [
                InlineKeyboardButton::callback("👍", format!("like_{}", msg.chat.id)),
                InlineKeyboardButton::callback("👎", format!("dislike_{}", msg.chat.id)),
            ];
            bot.send_message(ChatId(intr), "Твой собеседник остановил диалог!!")
                .reply_markup(InlineKeyboardMarkup::new([reactions]))
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

pub async fn cancel(bot: Bot, dialog: Dialog, msg: Message) -> HandlerResult {
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

pub async fn idle(bot: Bot, dialog: Dialog, msg: Message) -> HandlerResult {
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
            dialog.update(State::SearchChooseGender).await.unwrap();
        } else {
            receive_message(bot, dialog, msg).await?;
        }
    } else {
        receive_message(bot, dialog, msg).await?;
    }

    Ok(())
}
