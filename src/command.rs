use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Список команд:")]
pub enum Command {
    #[command(description = "Список команд")]
    Help,

    #[command(description = "Старт")]
    Start,

    #[command(description = "Поиск")]
    Search,

    #[command(description = "Найти следующего собеседника")]
    Next,

    #[command(description = "Отменить поиск")]
    Cancel,

    #[command(description = "Остановить диалог")]
    Stop,

    #[command(description = "Изменить имя")]
    SetName,

    #[command(description = "Изменить возраст")]
    SetAge,

    #[command(description = "Изменить пол")]
    SetGender,

    #[command(description = "Админ команда чтобы узнать количество пользователей")]
    Admin,

    #[command(description = "Админ команда чтобы отправить сообщение всем пользователям")]
    Message,

    #[command(description = "Правила")]
    Rules,

    #[command(description = "Забанить пользователя")]
    Ban,

    #[command(description = "Разбанить пользователя")]
    Unban,

    #[command(description = "Получить информацию о пользователе")]
    UserInfo,

    #[command(description = "Получить реферальную ссылку")]
    Referral,

    #[command(description = "Топ по рефералам")]
    Top,

    #[command(description = "Топ по репутации")]
    TopRep,
}
