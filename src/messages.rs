use std::env;

use teloxide::{
    payloads::{ SendMessageSetters, SendPhotoSetters, SendVideoSetters },
    requests::Requester,
    types::{ ChatId, InlineKeyboardButton, InlineKeyboardMarkup, InputFile, Message, MessageId },
    Bot,
};
use tokio::sync::Mutex as TokioMutex;

use crate::commands::{ idle, stop };
use crate::{ database::Database, state::State, Dialog, HandlerResult, DATABASE };

pub async fn receive_set_age(bot: Bot, dialog: Dialog, msg: Message) -> HandlerResult {
    match msg.text().map(|text| text.parse::<u8>()) {
        Some(Ok(age)) => {
            if age < 12 {
                bot.send_message(msg.chat.id, "Эй, ты ещё ребенок!").await?;
                dialog.update(State::Idle).await?;
            } else {
                let db = DATABASE.get_or_init(||
                    TokioMutex::new(Database::new("db.db").unwrap())
                ).lock().await;
                db.update_user_age(msg.chat.id.0, age).unwrap();

                bot.send_message(msg.chat.id, "Готово").await?;

                dialog.update(State::Idle).await?;
            }
        }
        _ => {
            bot.send_message(msg.chat.id, "Пытаешься найти баг? Давай заново!").await?;
        }
    }

    Ok(())
}

pub async fn receive_set_nickname(bot: Bot, dialog: Dialog, msg: Message) -> HandlerResult {
    match msg.text().map(ToOwned::to_owned) {
        Some(nickname) => {
            let db = DATABASE.get_or_init(||
                TokioMutex::new(Database::new("db.db").unwrap())
            ).lock().await;

            db.update_user_nickname(msg.chat.id.0, &nickname).unwrap();
            bot.send_message(msg.chat.id, "Готово").await?;

            dialog.update(State::Idle).await?;
        }
        _ => {
            bot.send_message(msg.chat.id, "Пытаешься найти баг? Давай заново!").await?;
        }
    }

    Ok(())
}

pub async fn receive_message(bot: Bot, dialog: Dialog, msg: Message) -> HandlerResult {
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

            dialog.update(State::Dialog {
                interlocutor: chat as u64,
            }).await?;

            if let Some(voice) = msg.voice() {
                bot.send_audio(ChatId(chat), InputFile::file_id(&voice.file.id)).await?;
            } else if let Some(sticker) = msg.sticker() {
                bot.send_sticker(ChatId(chat), InputFile::file_id(&sticker.file.id)).await?;
            } else if let Some(photo) = msg.photo() {
                if let Some(txt) = msg.caption() {
                    bot
                        .send_photo(
                            ChatId(chat),
                            InputFile::file_id(&photo.last().unwrap().file.id)
                        )
                        .caption(txt).await?;
                } else {
                    bot.send_photo(
                        ChatId(chat),
                        InputFile::file_id(&photo.last().unwrap().file.id)
                    ).await?;
                }
            } else if let Some(video) = msg.video() {
                if let Some(txt) = msg.caption() {
                    bot
                        .send_video(ChatId(chat), InputFile::file_id(&video.file.id))
                        .caption(txt).await?;
                } else {
                    bot.send_video(ChatId(chat), InputFile::file_id(&video.file.id)).await?;
                }
            } else if let Some(animation) = msg.animation() {
                bot.send_animation(ChatId(chat), InputFile::file_id(&animation.file.id)).await?;
            } else if let Some(sticker) = msg.sticker() {
                bot.send_sticker(ChatId(chat), InputFile::file_id(&sticker.file.id)).await?;
            } else if let Some(video_note) = msg.video_note() {
                bot.send_video_note(ChatId(chat), InputFile::file_id(&video_note.file.id)).await?;
            } else if let Some(txt) = msg.text() {
                if
                    txt.to_lowercase().contains("http") ||
                    txt.to_lowercase().contains("цп") ||
                    txt.to_lowercase().contains("детское") ||
                    txt.to_lowercase().contains("продаю") ||
                    txt.to_lowercase().contains("продам")
                {
                    let admin = env::var("ADMIN").unwrap();

                    bot.send_message(
                        ChatId(admin.parse::<i64>().unwrap()),
                        format!("{} отправил что-то подозрительное!\n\n{}", msg.chat.id.0, txt)
                    ).await?;
                }
                if let Some(rpmsg) = msg.reply_to_message() {
                    let res = bot
                        .send_message(ChatId(chat), txt)
                        .reply_to_message_id(MessageId(rpmsg.id.0 - 1)).await;

                    if res.is_err() {
                        bot
                            .send_message(ChatId(chat), txt)
                            .reply_to_message_id(MessageId(rpmsg.id.0 + 1)).await?;
                    }
                } else {
                    bot.send_message(ChatId(chat), txt).await?;
                }
            } else {
                bot.send_message(
                    msg.chat.id,
                    "Такой формат сообщения пока что не поддерживается"
                ).await?;
            }
        } else {
            bot.send_message(msg.chat.id, "Ты не в диалоге! /search чтобы попасть туда!").await?;
            dialog.update(State::Idle).await?;
        }
    } else {
        bot.send_message(msg.chat.id, "Ты не в диалоге! /search чтобы попасть туда!").await?;
        dialog.update(State::Idle).await?;
    }

    Ok(())
}

pub async fn receive_age(bot: Bot, dialog: Dialog, msg: Message) -> HandlerResult {
    match msg.text().map(|text| text.parse::<u8>()) {
        Some(Ok(age)) => {
            if age < 12 {
                bot.send_message(msg.chat.id, "Эй, ты ещё ребенок!").await?;
                dialog.update(State::Start).await?;
            } else {
                bot.send_message(
                    msg.chat.id,
                    "Теперь введи псевдоним который будет публичным (его можно будет изменить!)"
                ).await?;

                dialog.update(State::ReceiveNickname { age: age }).await?;
            }
        }
        _ => {
            bot.send_message(msg.chat.id, "Пытаешься найти баг? Давай заново!").await?;
        }
    }

    Ok(())
}

pub async fn receive_nickname(bot: Bot, dialog: Dialog, msg: Message, age: u8) -> HandlerResult {
    match msg.text().map(ToOwned::to_owned) {
        Some(nickname) => {
            let genders = ["♂", "♀"].map(|product|
                InlineKeyboardButton::callback(product, product)
            );
            bot
                .send_message(msg.chat.id, "Теперь выбери свой пол")
                .reply_markup(InlineKeyboardMarkup::new([genders])).await?;
            dialog.update(State::ReceiveGender { age, nickname }).await?;
        }
        _ => {
            bot.send_message(msg.chat.id, "Пытаешься найти баг? Давай заново!").await?;
        }
    }

    Ok(())
}

pub async fn dialog_search(bot: Bot, dialog: Dialog, _: Message) -> HandlerResult {
    bot.send_message(dialog.chat_id(), "Ты уже в диалоге!").await.unwrap();

    Ok(())
}

pub async fn set_name(bot: Bot, dialog: Dialog, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Введите свой новый никнейм(он будет публичным): ").await?;
    dialog.update(State::SetNickname).await?;

    Ok(())
}

pub async fn set_age(bot: Bot, dialog: Dialog, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Введите свой возраст(он будет публичным): ").await?;

    dialog.update(State::SetAge).await?;

    Ok(())
}

pub async fn set_gender(bot: Bot, dialog: Dialog, msg: Message) -> HandlerResult {
    let genders = ["♂", "♀"].map(|product| InlineKeyboardButton::callback(product, product));
    bot
        .send_message(msg.chat.id, "Выбери свой пол")
        .reply_markup(InlineKeyboardMarkup::new([genders])).await?;

    dialog.update(State::SetGender).await?;

    Ok(())
}
