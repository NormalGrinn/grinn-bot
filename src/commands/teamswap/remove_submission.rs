use crate::{team_swapping::team_swap_utils, Context, Error};
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
    #[description = "The anime you want to submit, this should be an AL URL to the anime"]
    #[autocomplete = "autocomplete_removal"]
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
    match database::get_anime_submitter(&anime_name) {
        Ok(submitter_id) => if submitter_id != user_id {
            ctx.say("You are trying to remove an anime you did not submit").await?;
            return Ok(())
        },
        Err(_) => {
            ctx.say("Error trying to find the anime submitter").await?;
            return Ok(())
        },
    }
    match database::delete_anime(&anime_name) {
    Ok(_) => {
        ctx.say("Anime removed successfully").await?;
        return Ok(());
    },
    Err(_) => {
        ctx.say("Error trying to remove anime").await?;
        return Ok(());
    },
    }
}