use serenity::model::channel::Message;
use sqlx::sqlite::SqlitePool;
use sqlx::Row;

use crate::error::Error;

#[derive(Debug, Serialize)]
pub struct LogEntry {
    pub user_id: String,
    pub username: String,
    pub bot: bool,
    pub content: String,
    pub timestamp: String,
}

pub async fn create_log_table(pool: &SqlitePool) -> Result<(), Error> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS logs (
            id INTEGER PRIMARY KEY,
            user_id VARCHAR,
            username VARCHAR,
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
        bot: msg.author.bot,
        content: msg.content.trim_matches('`').to_string(),
        timestamp: msg.timestamp.to_string(),
    };

    sqlx::query(
        "INSERT INTO logs (user_id, username, bot, content, timestamp) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(data.user_id)
    .bind(data.username)
    .bind(data.bot)
    .bind(data.content)
    .bind(data.timestamp)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn fetch_log(pool: &SqlitePool) -> Result<Vec<LogEntry>, Error> {
    let logs = sqlx::query("SELECT * FROM logs").fetch_all(pool).await?;
    let mut v = Vec::new();

    for log in logs {
        let user_id: String = log.get("user_id");
        let username: String = log.get("username");
        let bot: bool = log.get("bot");
        let content: String = log.get("content");
        let timestamp: String = log.get("timestamp");
        let obj = LogEntry { user_id, username, bot, content, timestamp };

        v.push(obj);
    }

    Ok(v)
}
