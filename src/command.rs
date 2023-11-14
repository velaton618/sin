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
}
