use std::env;

use crate::{team_swapping::team_swap_utils, Context, Error};
use poise::CreateReply;
use rusqlite::Result;
use serenity::futures::Stream;
use serenity::futures;
use strsim::jaro_winkler;

use crate::{database, Data};

async fn autocomplete_claim<'a>(
    ctx: Context<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    let names = database::get_unclaimed_anime_names().unwrap();
    let mut scored_names: Vec<(f64, String)> = names
    .iter()
    .map(|name| {
        let score = jaro_winkler(partial, name);
        (score, name.clone())
    })
    .collect();

    scored_names.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

    let sorted_names: Vec<String> = scored_names.into_iter().map(|(_, name)| name).collect();
    futures::stream::iter(sorted_names)
}

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn claim(
    ctx: Context<'_>,
    #[description = "The name of the anime you want to claim"]
    #[autocomplete = "autocomplete_claim"]
    anime_name: String,
) -> Result<(), Error> {
    match team_swap_utils::check_phase(vec![3]) {
        Ok(b) => {
            if !b {
                ctx.send(CreateReply::default().content("Command is not allowed in current phase").ephemeral(true)).await?;
                return Ok(());
            }
        },
        Err(_) => {
            ctx.send(CreateReply::default().content("Error checking phases").ephemeral(true)).await?;
            return Ok(())
        },
    }
    let user_id = ctx.author().id.get();
    match team_swap_utils::check_swapper_role(&ctx.author(), &ctx).await {
        Ok(b) => {
            if !b {
                let message = format!("{} does not have the swapper role, and therefore cannot claim an anime", ctx.author().global_name.clone().unwrap());
                ctx.send(CreateReply::default().content(message).ephemeral(true)).await?;
                return  Ok(())
            }
        },
        Err(_) => {
            ctx.send(CreateReply::default().content("An error has occured checking roles").ephemeral(true)).await?;
            return Ok(())
        },
    }
    let user_team_id: u64;
    match database::check_if_user_in_team(user_id) {
        Ok(team_id) => {
            match team_id {
                Some(id) => user_team_id = id,
                None => {
                    ctx.send(CreateReply::default().content("You are not in a team and thus cannot claim an anime").ephemeral(true)).await?;
                    return Ok(());
                },
            }
        },
        Err(_) => {
            ctx.send(CreateReply::default().content("An error has occured checking if the user is already in a team").ephemeral(true)).await?;
            return Ok(())
        },
    }
    let anime_id;
    let member_id;
    match database::get_anime_id_by_name(&anime_name) {
        Ok(id) => {
            match id {
                Some((a_id, m_id)) => {
                    anime_id = a_id;
                    member_id = m_id;
                },
                None => {
                    ctx.send(CreateReply::default().content("This anime was not submitted and thus cannot be claimed").ephemeral(true)).await?;
                    return Ok(());
                },
            }
        },
        Err(_) => {
            ctx.send(CreateReply::default().content("Error checking fetching the anime id").ephemeral(true)).await?;
            return Ok(());
        },
    }
    match database::check_if_anime_is_claimed(&anime_name) {
        Ok(b) => if b {
            ctx.send(CreateReply::default().content("This anime has already been claimed, please pick another one").ephemeral(true)).await?;
            return Ok(());
        },
        Err(_) => {
            ctx.send(CreateReply::default().content("An error has occured checking if the anime has been claimed").ephemeral(true)).await?;
            return Ok(())
        },
    }
    match database::get_teammembers_id_by_team_id(user_team_id) {
    Ok(ids) => {
        if ids.contains(&member_id) {
            ctx.send(CreateReply::default().content("Someone on your team submitted this anime, thus you cannot claim it").ephemeral(true)).await?;
            return Ok(())
        }
    },
    Err(_) => {
        ctx.send(CreateReply::default().content("An error has occured checking team members").ephemeral(true)).await?;
        return Ok(())
    },
    }

    match database::get_claimed_anime_by_user(user_id) {
        Ok(n) => {
            let max_claims: u64 = env::var("ALLOWED_CLAIMS")?.parse()?;
            if n.len() >= max_claims.try_into().unwrap() {
                ctx.send(CreateReply::default().content("You currently can't claim any more anime, wait until the maximun is raised").ephemeral(true)).await?;
                return Ok(())
            }
        },
        Err(_) => {
            ctx.send(CreateReply::default().content("An error has occured checking the claimed anime").ephemeral(true)).await?;
            return Ok(())
        },
    }

    let (_, team_id) = database::get_member_with_team(user_id)?;
    match database::create_claimed_anime(anime_id, team_id, user_id) {
        Ok(_) => {
            let message = format!("You claimed {}", anime_name);
            ctx.send(CreateReply::default().content(message).ephemeral(true)).await?;
            return Ok(());
        },
        Err(_) => {
            ctx.send(CreateReply::default().content("Error claiming anime").ephemeral(true)).await?;
            return Ok(());
        },
    }
}