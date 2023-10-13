use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Список команд:")]
pub enum Command {
    #[command(description = "Список команд")]
    Help,

    #[command(description = "Поиск")]
    Search,
}
