use serenity::model::channel::Message;
use sqlx::sqlite::SqlitePool;

use crate::error::Error;

struct LogEntry {
    user_id: String,
    username: String,
    discriminator: u16,
    bot: bool,
    content: String,
    timestamp: String,
}

pub async fn create_log_table(pool: &SqlitePool) -> Result<(), Error> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS logs (
            id INTEGER PRIMARY KEY,
            user_id VARCHAR,
            username VARCHAR,
            discriminator INTEGER,
            bot BOOLEAN,
            content TEXT,
            timestamp TIMESTAMP WITH TIME ZONE
        )",
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn insert_log(pool: &SqlitePool, msg: Message) -> Result<(), Error> {
    let data = LogEntry {
        user_id: msg.author.id.0.to_string(),
        username: msg.author.name,
        discriminator: msg.author.discriminator,
        bot: msg.author.bot,
        content: msg.content,
        timestamp: msg.timestamp.to_string(),
    };

    sqlx::query("INSERT INTO logs (user_id, username, discriminator, bot, content, timestamp) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(data.user_id)
        .bind(data.username)
        .bind(data.discriminator)
        .bind(data.bot)
        .bind(data.content)
        .bind(data.timestamp)
        .execute(pool)
        .await?;

    Ok(())
}
