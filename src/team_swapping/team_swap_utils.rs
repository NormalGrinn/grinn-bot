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
    let guild_id = serenity::GuildId::new(env::var("GUILD_ID")?.parse()?);
    let role_id = serenity::RoleId::new(env::var("HOST_ROLE_ID")?.parse()?);
    //let role_id = serenity::RoleId::new(role_id_env);

    let res = user.has_role(ctx.http(), guild_id, role_id).await;
    return Ok(res?);
}

pub fn check_if_team_exists(team_name: &String) -> Result<bool, Error> {
    match database::check_if_team_exists(team_name) {
        Ok(b) => {
            if b { return Ok(true) } else { return Ok(false) };
        },
        Err(_) => return Err("An error has occured fetching from the database".into()),
    }
}

pub fn check_phase(allowed_phases: Vec<u64>) -> Result<bool, Error> {
    let current_phase: u64 = env::var("PHASE")?.parse()?;
    return Ok(allowed_phases.contains(&current_phase))
}