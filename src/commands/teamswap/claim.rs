use crate::{team_swapping::team_swap_utils, Context, Error};
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
    let user_id = ctx.author().id.get();
    match team_swap_utils::check_swapper_role(&ctx.author(), &ctx).await {
        Ok(b) => {
            if !b {
                let message = format!("{} does not have the swapper role, and therefore cannot remove submissions", ctx.author().global_name.clone().unwrap());
                ctx.say(message).await?;
                return  Ok(())
            }
        },
        Err(_) => {
            ctx.say("An error has occured checking roles").await?;
            return Ok(())
        },
    }
    match database::check_if_user_in_team(user_id) {
        Ok(b) => {
            if !b {
                ctx.say("This user is not in a team and thus can't claim an anime").await?;
                return Ok(())
            }
        },
        Err(_) => {
            ctx.say("An error has occured checking if the user is already in a team").await?;
            return Ok(())
        },
    }
    let anime_id;
    match database::get_anime_id_by_name(&anime_name) {
        Ok(id) => {
            match id {
                Some(id) => anime_id = id,
                None => {
                    ctx.say("This anime was not submitted and thus cannot be claimed").await?;
                    return Ok(());
                },
            }
        },
        Err(_) => {
            ctx.say("Error checking fetching the anime id").await?;
            return Ok(());
        },
    }
    match database::check_if_anime_is_claimed(&anime_name) {
        Ok(b) => if b {
            ctx.say("This anime has already been claimed, please pick another one").await?;
            return Ok(());
        },
        Err(_) => {
            ctx.say("An error has occured checking if the anime has been claimed").await?;
            return Ok(())
        },
    }
    let (_, team_id) = database::get_member_with_team(user_id)?;
    match database::create_claimed_anime(anime_id, team_id, user_id) {
        Ok(_) => {
            ctx.say(format!("You claimed {}", anime_name)).await?;
            return Ok(());
        },
        Err(_) => {
            ctx.say("Error claiming anime").await?;
            return Ok(());
        },
    }
}