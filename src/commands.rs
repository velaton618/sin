use std::{ env, time::Duration };

use chrono::{ DateTime, Datelike };
use teloxide::{
    payloads::SendMessageSetters,
    requests::Requester,
    types::{ ChatId, InlineKeyboardButton, InlineKeyboardMarkup, Message },
    Bot,
};
use tokio::sync::Mutex as TokioMutex;

use crate::{
    database::Database,
    messages::receive_message,
    models::{ chat_type::ChatType, gender::Gender },
    state::State,
    user_state::{ self, UserState },
    Dialog,
    HandlerResult,
    DATABASE,
};

pub async fn admin_message(bot: Bot, _: Dialog, msg: Message) -> HandlerResult {
    let admin = env::var("ADMIN").unwrap();
    if msg.chat.id.0.to_string() == admin {
    }

    let db = DATABASE.get_or_init(|| TokioMutex::new(Database::new("db.db").unwrap()));
    let users = db.lock().await.get_all_users().unwrap();

    for user in users {
        let _ = bot.send_message(
            ChatId(user.id),
            format!("--- SinChat ---\n\n{}", msg.text().unwrap().split("/message").nth(1).unwrap())
        ).await;
    }

    Ok(())
}

pub async fn rules(bot: Bot, _: Dialog, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "Что ЗАПРЕЩЕННО в SinChat\n\n💬Общие\nРеклама\nПопрошайничество\nСпам\nНацизм / фашизм / расизм\nБулинг\n\n💬 Обычный чат\nРазговор на темы 18+ \nВыпрашивание интимных фотографий\n\n🔞 Пошлый чат\nОбщаться на НЕ пошлые темы\nИскать друзей\n\nЗа любое нарушение правил ваша репутация снижается, если ваша репутация иже 20, вы будете заблокированы.\n\n⚠️НЕ ЗНАНИЕ ПРАВИЛ, НЕ УБИРАЕТ С ВАС ОТВЕТСВЕННОСТИ⚠️"
    ).await?;

    Ok(())
}

pub async fn ban(bot: Bot, _: Dialog, msg: Message) -> HandlerResult {
    if let Some(txt) = msg.text() {
        if txt.split("/ban").nth(1).is_none() {
            bot.send_message(msg.chat.id, format!("Что-то не так")).await?;
            return Ok(());
        }
    }

    let admin = env::var("ADMIN").unwrap();

    if msg.chat.id.0.to_string() == admin {
        let db = Database::new("db.db").unwrap();
        let user = db.get_user(
            msg.text().unwrap().split("/ban").nth(1).unwrap().trim().parse::<i64>().unwrap()
        );

        if user.is_ok() {
            let user = user.unwrap();

            if user.is_some() {
                let user = user.unwrap();

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
                    bot.send_message(msg.chat.id, format!("Готово\n\n{:#?}", user)).await?;
                } else {
                    bot.send_message(msg.chat.id, format!("Что-то не так")).await?;
                }
            }
        }
    }

    Ok(())
}

pub async fn unban(bot: Bot, _: Dialog, msg: Message) -> HandlerResult {
    if let Some(txt) = msg.text() {
        if txt.split("/unban").nth(1).is_none() {
            bot.send_message(msg.chat.id, format!("Что-то не так")).await?;
            return Ok(());
        }
    }

    let admin = env::var("ADMIN").unwrap();

    if msg.chat.id.0.to_string() == admin {
        let db = Database::new("db.db").unwrap();
        let user = db.get_user(
            msg.text().unwrap().split("/unban").nth(1).unwrap().trim().parse::<i64>().unwrap()
        );

        if user.is_ok() {
            let user = user.unwrap();

            if user.is_some() {
                let user = user.unwrap();

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
                    bot.send_message(msg.chat.id, format!("Готово\n\n{:#?}", user)).await?;
                } else {
                    bot.send_message(msg.chat.id, format!("Что-то не так")).await?;
                }
            }
        }
    }

    Ok(())
}

pub async fn referral(bot: Bot, _: Dialog, msg: Message) -> HandlerResult {
    let link = format!("https://t.me/s1nchat_bot?start={}", msg.chat.id.0);
    bot.send_message(
        msg.chat.id,
        format!("Пригласи 10 человек и получи бесплатный премиум на неделю!\n\nТвоя реферальная ссылка: {}\n\nПример использования:", link)
    ).await?;

    bot.send_message(
        msg.chat.id,
        format!("💫 Анонимный чат с бесплатным поиском по полу, и разделением чатов!\n\n👻Скорее регистрируйся по этой ссылке чтобы найти хорошего собеседника!\n\n{}", link)
    ).await?;

    Ok(())
}

pub async fn top(bot: Bot, _: Dialog, msg: Message) -> HandlerResult {
    let db = DATABASE.get().unwrap().lock().await;
    let users = db.get_top_referral_users(10);
    if users.is_ok() {
        let users = users.unwrap();
        let mut response = String::new();
        response.push_str("💫ТОП 10 ПО РЕФЕРАЛАМ\n\n");

        for user in users {
            response.push_str(
                &format!(
                    "{} {} » {}\n",
                    if user.gender == Gender::Male {
                        "Мужской ♂"
                    } else {
                        "Женский ♀"
                    },
                    user.nickname,
                    user.referrals
                )
            );
        }
        bot.send_message(msg.chat.id, &response).await?;
        bot.send_message(msg.chat.id, "/referral - чтобы попасть в этот топ").await?;
    } else {
        bot.send_message(msg.chat.id, "Что-то пошло не так... Обратитесь в администрацию").await?;
    }

    Ok(())
}

pub async fn premium(bot: Bot, _: Dialog, msg: Message) -> HandlerResult {
    bot.send_message(
        msg.chat.id,
        "Пригласи 10 человек и получи бесплатный премиум на неделю!\n\n💎 Что даёт премиум?\n\nПолучив премиум вы можете:\n\n1. Иметь полную информацию о собеседнике\n2. разделение пошлого и обычного чата\n3. Первее получите доступ к новым функциям чата\n4. Все видят ваш премиум"
    ).await.unwrap();

    Ok(())
}
pub async fn top_rep(bot: Bot, _: Dialog, msg: Message) -> HandlerResult {
    let db = DATABASE.get().unwrap().lock().await;
    let users = db.get_top_reputation_users(10);
    if users.is_ok() {
        let users = users.unwrap();
        let mut response = String::new();
        response.push_str("💫ТОП 10 ПО РЕПУТАЦИИ\n\n");

        for user in users {
            response.push_str(
                &format!(
                    "{} {} {} » {}\n",
                    if user.is_premium {
                        "💎 Премиум"
                    } else {
                        ""
                    },
                    if user.gender == Gender::Male {
                        "Мужской ♂"
                    } else {
                        "Женский ♀"
                    },
                    user.nickname,
                    user.reputation
                )
            );
        }
        bot.send_message(msg.chat.id, &response).await?;
    } else {
        bot.send_message(msg.chat.id, "Что-то пошло не так... Обратитесь в администрацию").await?;
    }

    Ok(())
}

pub async fn user_info(bot: Bot, _: Dialog, msg: Message) -> HandlerResult {
    let admin = env::var("ADMIN").unwrap();

    if msg.chat.id.0.to_string() == admin {
        let db = Database::new("db.db").unwrap();
        let user = db.get_user(
            msg
                .text()
                .unwrap_or("/userinfo")
                .split("/userinfo")
                .nth(1)
                .unwrap_or("")
                .trim()
                .parse::<i64>()
                .unwrap_or(msg.chat.id.0)
        );

        if user.is_ok() {
            let user = user.unwrap();

            if user.is_some() {
                let user = user.unwrap();

                bot.send_message(
                    msg.chat.id,
                    format!(
                        "{}\n\n🆔: {}\nНикнейм: {}\nПол: {}\nВозраст: {}\nРепутация: {}\nКоличество приглашенных людей: {}",
                        if user.is_premium {
                            "💎 Премиум"
                        } else {
                            ""
                        },
                        user.id,
                        user.nickname,
                        if user.gender == Gender::Male {
                            "Мужской ♂"
                        } else {
                            "Женский ♀"
                        },
                        user.age,
                        user.reputation,
                        user.referrals
                    )
                ).await?;
                bot.send_message(msg.chat.id, format!("{:#?}", user)).await?;
            }
        }
    } else {
        let db = Database::new("db.db").unwrap();
        let user = db.get_user(msg.chat.id.0).unwrap().unwrap();

        bot.send_message(
            msg.chat.id,
            format!(
                "{}\n\nНикнейм: {}\nПол: {}\nВозраст: {}\nРепутация: {}\nКоличество приглашенных людей: {}",
                user.id,
                user.nickname,
                if user.gender == Gender::Male {
                    "Мужской ♂"
                } else {
                    "Женский ♀"
                },
                user.age,
                user.reputation,
                user.referrals
            )
        ).await?;
    }

    Ok(())
}

pub async fn delete_user(bot: Bot, _: Dialog, msg: Message) -> HandlerResult {
    if let Some(txt) = msg.text() {
        if txt.split("/delete").nth(1).is_none() {
            bot.send_message(msg.chat.id, format!("Что-то не так")).await?;
            return Ok(());
        }
    }

    let admin = env::var("ADMIN").unwrap();

    if msg.chat.id.0.to_string() == admin {
        let db = Database::new("db.db").unwrap();
        let user = db.get_user(
            msg.text().unwrap().split("/delete").nth(1).unwrap().trim().parse::<i64>().unwrap()
        );

        if user.is_ok() {
            let user = user.unwrap();

            if user.is_some() {
                let user = user.unwrap();

                let id = msg
                    .text()
                    .unwrap_or("/delete")
                    .split("/delete")
                    .nth(1)
                    .unwrap_or("")
                    .trim()
                    .parse::<i64>()
                    .unwrap_or(0);
                if id != 0 {
                    db.delete_user_by_id(id).unwrap();
                    bot.send_message(msg.chat.id, format!("Готово\n\n{:#?}", user)).await?;
                } else {
                    bot.send_message(msg.chat.id, format!("Что-то не так")).await?;
                }
            }
        }
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
                "Users: {}\nМужской ♂ Males: {}\nЖенский ♀ Females: {}\n\n💬 Chats: {}\nQueue: {}\n\n\nМужской ♂ Queue Males: {}\nЖенский ♀ Queue Females: {}",
                total_users,
                male_count,
                female_count,
                total_chats,
                total_queue,
                total_male_queue,
                total_female_queue
            )
        ).await?;
    }

    Ok(())
}

pub async fn stop(bot: Bot, dialog: Dialog, msg: Message) -> HandlerResult {
    let db = DATABASE.get_or_init(|| TokioMutex::new(Database::new("db.db").unwrap())).lock().await;

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
            bot
                .send_message(
                    dialog.chat_id(),
                    "Диалог остановлен!\n\n/next - найти нового собеседника"
                )
                .reply_markup(InlineKeyboardMarkup::new([reactions])).await?;

            let reactions = [
                InlineKeyboardButton::callback("👍", format!("like_{}", msg.chat.id)),
                InlineKeyboardButton::callback("👎", format!("dislike_{}", msg.chat.id)),
            ];
            bot
                .send_message(ChatId(intr), "Твой собеседник остановил диалог!!")
                .reply_markup(InlineKeyboardMarkup::new([reactions])).await?;
        } else {
            bot.send_message(msg.chat.id, "Ты не находишься в диалоге!").await?;
        }
    } else {
        bot.send_message(msg.chat.id, "Ты не находишься в диалоге!").await?;
    }
    dialog.update(State::Idle).await?;

    Ok(())
}

pub async fn cancel(bot: Bot, dialog: Dialog, msg: Message) -> HandlerResult {
    let db = DATABASE.get_or_init(|| TokioMutex::new(Database::new("db.db").unwrap())).lock().await;
    db.dequeue_user(msg.chat.id.0).unwrap();
    bot.send_message(msg.chat.id, "Поиск отменён!").await?;
    dialog.update(State::Idle).await?;
    db.set_user_state(msg.chat.id.0, UserState::Idle).unwrap();

    Ok(())
}

pub async fn next(bot: Bot, dialog: Dialog, msg: Message) -> HandlerResult {
    let db = DATABASE.get().unwrap().lock().await;
    let user = db.get_user(msg.chat.id.0);

    if user.is_ok() {
        let user = user.unwrap();
        if user.is_some() {
            let user = user.unwrap();

            if user.is_banned {
                bot.send_message(ChatId(user.id), "Вы заблокаированы!").await?;
                return Ok(());
            }
            if user.is_premium {
                let now = chrono::Utc::now();

                if now.timestamp() > user.premium_until {
                    db.set_premium(user.id, false).unwrap();
                    db.set_premium_until(user.id, 0).unwrap();

                    bot.send_message(ChatId(user.id), "Ваша подписка закончилась!").await?;
                }
            }

            if user.search_gender.is_none() || user.chat_type.is_none() {
                bot.send_message(
                    ChatId(user.id),
                    "Не могу найти прошлые фильтры! \n\n/search - чтобы искать"
                ).await?;
                return Ok(());
            }

            let chat = db.get_chat(msg.chat.id.0);
            if chat.is_ok() {
                let chat = chat.unwrap();
                if chat.is_some() {
                    let chat = chat.unwrap();
                    let _ = db.delete_chat(msg.chat.id.0);
                    db.set_user_state(msg.chat.id.0, UserState::Idle).unwrap();
                    db.set_user_state(chat, UserState::Idle).unwrap();

                    let reactions = [
                        InlineKeyboardButton::callback("👍", format!("like_{}", chat)),
                        InlineKeyboardButton::callback("👎", format!("dislike_{}", chat)),
                    ];
                    bot
                        .send_message(
                            dialog.chat_id(),
                            "Диалог остановлен!\n\n/next - найти нового собеседника"
                        )
                        .reply_markup(InlineKeyboardMarkup::new([reactions])).await?;

                    let reactions = [
                        InlineKeyboardButton::callback("👍", format!("like_{}", msg.chat.id)),
                        InlineKeyboardButton::callback("👎", format!("dislike_{}", msg.chat.id)),
                    ];
                    bot
                        .send_message(ChatId(chat), "Твой собеседник остановил диалог!!")
                        .reply_markup(InlineKeyboardMarkup::new([reactions])).await?;
                }
            }
            let result = db.enqueue_user(
                dialog.chat_id().0,
                user.search_gender.unwrap(),
                user.gender,
                user.chat_type.as_ref().unwrap().clone()
            );

            if result.is_ok() {
                let result = result.unwrap();
                let cancel = [InlineKeyboardButton::callback("❌ Отменить", "cancel")];

                if result != 0 {
                    dialog.update(State::Dialog {
                        interlocutor: result as u64,
                    }).await?;
                    let interlocutor = db.get_user(result).unwrap().unwrap();

                    if user.is_premium {
                        bot.send_message(
                            dialog.chat_id(),
                            format!(
                                "🆔: {}\nПол: {}\nПсевдоним: {} \nВозраст: {}\n\nСобеседник найден!\n\n/next - чтобы найти нового собеседника\n/stop - чтобы остановить диалог",

                                interlocutor.id,
                                if interlocutor.gender == Gender::Male {
                                    "Мужской ♂"
                                } else {
                                    "Женский ♀"
                                },
                                interlocutor.nickname,
                                interlocutor.age
                            )
                        ).await?;
                    } else {
                        bot.send_message(
                            dialog.chat_id(),
                            format!(
                                "Собеседник найден!\n\n/next - чтобы найти нового собеседника\n/stop - чтобы остановить диалог"
                            )
                        ).await?;
                    }

                    if interlocutor.is_premium {
                        bot.send_message(
                            ChatId(result),
                            format!(
                                "{} \n\nСобеседник найден!\n\n🆔: {}\nПол: {}\nПсевдоним: {} \nВозраст: {}\n\n/next - чтобы найти нового собеседника\n/stop - чтобы остановить диалог",
                                if user.chat_type == Some(ChatType::Regular) {
                                    "💬"
                                } else {
                                    "🔞"
                                },
                                user.id,
                                if user.gender == Gender::Male {
                                    "Мужской ♂"
                                } else {
                                    "Женский ♀"
                                },
                                user.nickname,
                                user.age
                            )
                        ).await?;
                    } else {
                        bot.send_message(
                            ChatId(result),
                            format!(
                                "Собеседник найден!\n\n/next - чтобы найти нового собеседника\n/stop - чтобы остановить диалог"
                            )
                        ).await?;
                    }

                    db.set_user_state(user.id, user_state::UserState::Dialog).unwrap();
                    db.set_user_state(result, user_state::UserState::Dialog).unwrap();
                } else {
                    bot
                        .send_message(dialog.chat_id(), "Ищу...")
                        .reply_markup(InlineKeyboardMarkup::new([cancel])).await?;
                    dialog.update(State::Search).await?;

                    db.set_user_state(user.id, user_state::UserState::Search).unwrap();
                }
            } else {
                bot.send_message(dialog.chat_id(), format!("Ой! Голова кружится...")).await?;
            }
        }
    }

    Ok(())
}

pub async fn start(bot: Bot, dialog: Dialog, msg: Message) -> HandlerResult {
    let db = DATABASE.get_or_init(|| TokioMutex::new(Database::new("db.db").unwrap())).lock().await;

    if let Some(txt) = msg.text() {
        if let Some(id) = txt.split("/start").nth(1) {
            let id = id.trim().parse::<i64>();

            if id.is_ok() {
                let id = id.unwrap();

                let user = db.get_user(id);
                if user.is_ok() {
                    let user = user.unwrap();

                    if user.is_some() {
                        let user = user.unwrap();
                        let u = db.get_user(msg.chat.id.0);

                        if u.is_err() || u.unwrap().is_none() {
                            if user.referrals + 1 >= 10 {
                                db.set_premium(user.id, true).unwrap();

                                let new_premium;
                                if user.is_premium {
                                    let old_premium = match
                                        DateTime::from_timestamp(user.premium_until, 0)
                                    {
                                        Some(o) => o,
                                        None => chrono::Utc::now(),
                                    };
                                    new_premium =
                                        old_premium + chrono::Duration::try_days(7).unwrap();
                                } else {
                                    let now = chrono::Utc::now();
                                    new_premium = now + chrono::Duration::try_seconds(1).unwrap();
                                }
                                db.set_premium_until(user.id, new_premium.timestamp()).unwrap();

                                bot.send_message(ChatId(user.id), "Вы получили премиум 💎").await?;
                                let _ = bot.send_message(
                                    ChatId(user.id),
                                    format!(
                                        "Ваш премиум действует до: {}",
                                        new_premium.format("%d.%m.%Y")
                                    )
                                ).await;
                            }
                            let _ = db.increase_referral_count(user.id);
                            let _ = bot.send_message(
                                ChatId(user.id),
                                format!(
                                    "По вашей реферальной ссылке перешёл 1 человек! \n\nДо получения премиум 💎 осталось: {} человек",
                                    10 - (user.referrals + 1)
                                )
                            ).await;
                        }
                    }
                }
            }
        }
    }

    let user = db.get_user(dialog.chat_id().0);

    if user.is_ok() && user.as_ref().unwrap().is_some() {
        idle(bot, dialog, msg).await?;
    } else {
        bot.send_message(msg.chat.id, "Добро пожаловать в анонимный чат Sin!").await?;
        bot.send_message(msg.chat.id, "Нужно зарегестрироваться! Введи свой возраст: ").await?;
        dialog.update(State::ReceiveAge).await?;
    }

    Ok(())
}
pub async fn idle(bot: Bot, dialog: Dialog, msg: Message) -> HandlerResult {
    if let Some(txt) = msg.text() {
        if txt.contains("search") {
            let db = DATABASE.get_or_init(||
                TokioMutex::new(Database::new("db.db").unwrap())
            ).lock().await;

            let user = db.get_user(dialog.chat_id().0);

            if user.is_ok() && user.as_ref().unwrap().is_some() {
                let user = user.unwrap().unwrap();
                if user.state == UserState::Dialog {
                    bot.send_message(
                        dialog.chat_id(),
                        "Ты не готов к поиску! Останови диалог"
                    ).await?;

                    return Ok(());
                } else if user.state == UserState::Search {
                    bot.send_message(dialog.chat_id(), "Не мешай! Я ищу").await?;

                    return Ok(());
                }
            } else {
                bot.send_message(
                    dialog.chat_id(),
                    "Ты не готов к поиску! Зарегестрируйся!\n\n/start"
                ).await?;

                return Ok(());
            }

            let genders = ["Мужской ♂", "Женский ♀"].map(|product|
                InlineKeyboardButton::callback(product, product)
            );
            bot.send_message(dialog.chat_id(), "Теперь выбери пол собеседника")
                .reply_markup(InlineKeyboardMarkup::new([genders])).await
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
