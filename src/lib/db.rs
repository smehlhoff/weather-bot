use serenity::model::channel::Message;
use sqlx::sqlite::SqlitePool;
use sqlx::Row;

use crate::error::Error;

#[derive(Debug, Serialize)]
pub struct Log {
    pub user_id: String,
    pub username: String,
    pub bot: bool,
    pub content: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize)]
pub struct Location {
    pub user_id: String,
    pub zip_code: String,
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

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS locations (
            id INTEGER PRIMARY KEY,
            user_id VARCHAR,
            zip_code VARCHAR,
            timestamp TIMESTAMP WITH TIME ZONE
        )",
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn insert_log(pool: &SqlitePool, msg: Message) -> Result<(), Error> {
    let data = Log {
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

pub async fn fetch_log(pool: &SqlitePool) -> Result<Vec<Log>, Error> {
    let rows = sqlx::query("SELECT * FROM logs").fetch_all(pool).await?;
    let mut v = Vec::new();

    for log in rows {
        let user_id: String = log.get("user_id");
        let username: String = log.get("username");
        let bot: bool = log.get("bot");
        let content: String = log.get("content");
        let timestamp: String = log.get("timestamp");
        let obj = Log { user_id, username, bot, content, timestamp };

        v.push(obj);
    }

    Ok(v)
}

pub async fn insert_location(pool: &SqlitePool, msg: &Message, zip_code: i32) -> Result<(), Error> {
    let data = Location {
        user_id: msg.author.id.0.to_string(),
        zip_code: zip_code.to_string(),
        timestamp: msg.timestamp.to_string(),
    };

    sqlx::query("INSERT INTO locations (user_id, zip_code, timestamp) VALUES (?, ?, ?)")
        .bind(data.user_id)
        .bind(data.zip_code)
        .bind(data.timestamp)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn fetch_location(pool: &SqlitePool, msg: &Message) -> Result<String, Error> {
    let user_id = msg.author.id.0.to_string();
    let row = sqlx::query("SELECT zip_code FROM locations WHERE user_id = ?")
        .bind(user_id)
        .fetch_one(pool)
        .await?;
    let zip_code = row.get("zip_code");

    Ok(zip_code)
}

pub async fn delete_location(pool: &SqlitePool, msg: &Message) -> Result<(), Error> {
    let user_id = msg.author.id.0.to_string();

    sqlx::query("DELETE FROM locations WHERE user_id = ?").bind(user_id).execute(pool).await?;

    Ok(())
}
