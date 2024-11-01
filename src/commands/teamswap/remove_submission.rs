use crate::{team_swapping::team_swap_utils, Context, Error};
use poise::CreateReply;
use rusqlite::Result;
use serenity::futures::Stream;
use serenity::futures;
use strsim::jaro_winkler;

use crate::database;

async fn autocomplete_removal<'a>(
    ctx: Context<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    let names = database::get_submitted_anime(ctx.author().id.get()).unwrap();
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
pub async fn remove_submission(
    ctx: Context<'_>,
    #[description = "Remove one of the anime you submitted"]
    #[autocomplete = "autocomplete_removal"]
    anime_name: String,
) -> Result<(), Error> {
    match team_swap_utils::check_phase(vec![1,2]) {
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
                let message = format!("{} does not have the swapper role, and therefore cannot remove submissions", ctx.author().global_name.clone().unwrap());
                ctx.send(poise::CreateReply::default().content(message).ephemeral(true)).await?;
                return  Ok(())
            }
        },
        Err(_) => {
            ctx.send(poise::CreateReply::default().content("An error has occured checking roles").ephemeral(true)).await?;
            return Ok(())
        },
    }
    match database::get_anime_submitter(&anime_name) {
        Ok(submitter_id) => if submitter_id != user_id {
            ctx.send(poise::CreateReply::default().content("You are trying to remove an anime you did not submit").ephemeral(true)).await?;
            return Ok(())
        },
        Err(_) => {
            ctx.send(poise::CreateReply::default().content("Error trying to find the anime submitter").ephemeral(true)).await?;
            return Ok(())
        },
    }
    match database::delete_anime(&anime_name) {
    Ok(_) => {
        ctx.send(poise::CreateReply::default().content("Anime removed successfully").ephemeral(true)).await?;
        return Ok(());
    },
    Err(_) => {
        ctx.send(poise::CreateReply::default().content("Error trying to remove anime").ephemeral(true)).await?;
        return Ok(());
    },
    }
}