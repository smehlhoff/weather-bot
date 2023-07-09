use serenity::model::channel::Message;
use sqlx::sqlite::SqlitePool;

use crate::error::Error;

struct LogEntry {
    username: String,
    content: String,
    timestamp: String,
}

pub async fn create_log_table(pool: &SqlitePool) -> Result<(), Error> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS logs (
            id INTEGER PRIMARY KEY,
            username VARCHAR,
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
        username: msg.author.name,
        content: msg.content,
        timestamp: msg.timestamp.to_string(),
    };

    sqlx::query("INSERT INTO logs (username, content, timestamp) VALUES (?, ?, ?)")
        .bind(data.username)
        .bind(data.content)
        .bind(data.timestamp)
        .execute(pool)
        .await?;

    Ok(())
}
