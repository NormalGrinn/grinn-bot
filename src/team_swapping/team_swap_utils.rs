use std::env::{self};
use crate::{database, Context, Error};
use poise::serenity_prelude as serenity;

pub async fn check_swapper_role(user: &serenity::User, ctx: &Context<'_>) -> Result<bool, Error> {
    let guild_id = serenity::GuildId::new(env::var("GUILD_ID")?.parse()?);
    let role_id = serenity::RoleId::new(env::var("SWAPPER_ROLE_ID")?.parse()?);
    //let role_id = serenity::RoleId::new(role_id_env);

    let res = user.has_role(ctx.http(), guild_id, role_id).await;
    return Ok(res?);
}

pub async fn check_host_role(user: &serenity::User, ctx: &Context<'_>) -> Result<bool, Error> {
    let guild_id = serenity::GuildId::new(env::var("HOST_ROLE_ID")?.parse()?);
    let role_id = serenity::RoleId::new(env::var("SWAPPER_ROLE_ID")?.parse()?);
    //let role_id = serenity::RoleId::new(role_id_env);

    let res = user.has_role(ctx.http(), guild_id, role_id).await;
    return Ok(res?);
}

pub async fn check_if_already_participating(user: &serenity::User) -> Result<bool, Error> {
    let user_id = user.id.get();
    match database::check_if_user_in_db(user_id).await {
        Ok(b) => {
            if b { return Ok(true) } else { return Ok(false) };
        },
        Err(_) => return Err("An error has occured fetching from the database".into()),
    }
}

pub async fn check_if_team_exists(team_name: &String) -> Result<bool, Error> {
    match database::check_if_team_exists(team_name).await {
        Ok(b) => {
            if b { return Ok(true) } else { return Ok(false) };
        },
        Err(_) => return Err("An error has occured fetching from the database".into()),
    }
}