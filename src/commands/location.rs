use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::lib::{db, utils};
use crate::Database;

#[command]
#[aliases("set", "add")]
pub async fn location_set(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let zip_code = args.message();
    let pool = {
        let data = ctx.data.read().await;
        data.get::<Database>().expect("Error retrieving database pool").clone()
    };

    match db::fetch_location(&pool, msg).await {
        Ok(val) => {
            msg.channel_id
                .say(
                    &ctx.http,
                    format!("`You can only have one location set. Your current location is {val}`"),
                )
                .await?;
        }
        Err(_) => {
            match utils::check_zip_code(zip_code) {
                Ok(zip_code) => match db::insert_location(&pool, msg, zip_code).await {
                    Ok(()) => msg.channel_id.say(&ctx.http, "`Your location has been set`").await?,
                    Err(e) => msg.channel_id.say(&ctx.http, format!("`{e}`")).await?,
                },
                Err(e) => msg.channel_id.say(&ctx.http, format!("`{e}`")).await?,
            };
        }
    }

    Ok(())
}

#[command]
#[aliases("list", "show")]
pub async fn location_list(ctx: &Context, msg: &Message) -> CommandResult {
    let pool = {
        let data = ctx.data.read().await;
        data.get::<Database>().expect("Error retrieving database pool").clone()
    };

    match db::fetch_location(&pool, msg).await {
        Ok(val) => {
            msg.channel_id.say(&ctx.http, format!("`Your current location is {val}`")).await?;
        }
        Err(_) => {
            msg.channel_id.say(&ctx.http, "`You don't have a location set`").await?;
        }
    }

    Ok(())
}

#[command]
#[aliases("delete", "del")]
pub async fn location_delete(ctx: &Context, msg: &Message) -> CommandResult {
    let pool = {
        let data = ctx.data.read().await;
        data.get::<Database>().expect("Error retrieving database pool").clone()
    };

    match db::fetch_location(&pool, msg).await {
        Ok(_) => {
            match db::delete_location(&pool, msg).await {
                Ok(()) => msg.channel_id.say(&ctx.http, "`Your location has been deleted`").await?,
                Err(e) => msg.channel_id.say(&ctx.http, format!("`{e}`")).await?,
            };
        }
        Err(_) => {
            msg.channel_id.say(&ctx.http, "`You don't have a location set`").await?;
        }
    }

    Ok(())
}
