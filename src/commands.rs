use std::env;

use teloxide::{
    payloads::SendMessageSetters,
    requests::Requester,
    types::{ChatId, InlineKeyboardButton, InlineKeyboardMarkup, Message},
    Bot,
};
use tokio::sync::Mutex as TokioMutex;

use crate::{
    database::Database, messages::receive_message, state::State, user_state::UserState, Dialog,
    HandlerResult, DATABASE,
};

pub async fn admin(bot: Bot, _: Dialog, msg: Message) -> HandlerResult {
    let admin = env::var("ADMIN").unwrap();
    let db = Database::new("db.db").unwrap();
    let total_users = db.get_total_users()?;
    let female_count = db.get_female_count()?;
    let male_count = db.get_male_count()?;
    let total_chats = db.get_total_chats()?;
    let total_female_vulgar = db.get_female_count_in_vulgar_chats()?;
    let total_male_vulgar = db.get_male_count_in_vulgar_chats()?;

    let total_female_chats = db.get_female_count_in_chats()?;
    let total_male_chats = db.get_male_count_in_chats()?;

    if msg.chat.id.0.to_string() == admin {
        bot.send_message(
            msg.chat.id,
            format!(
                "Users: {}\n🍌 Males: {}\n🍑 Females: {}\n\n🗨️ Chats: {}\n\n🔞🍌: {}\n🔞🍑: {}\n\n🗨️🍌: {}\n🗨️🍑: {}",
                total_users,
                male_count,
                female_count,
                total_chats,
                total_male_vulgar,
                total_female_vulgar,
                total_male_chats,
                total_female_chats
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
