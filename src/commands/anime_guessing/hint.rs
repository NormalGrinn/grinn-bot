use crate::{Context, Error};
use rusqlite::Result;
use serenity::futures;
use poise::serenity_prelude as serenity;
use ::serenity::futures::Stream;
use strsim::jaro_winkler;

use crate::database;
use crate::anime_guessing_game;
use crate::helpers;

async fn autocomplete_hint_type<'a>(
    ctx: Context<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    let types: Vec<String> = vec!("Season".to_string(), "Year".to_string(), "Format".to_string(), "Genre".to_string(), "Studio".to_string(),
                                 "Voice Actor".to_string(), "Tag".to_string(), "Staff".to_string(), "AL Score".to_string(), "Source".to_string(), "User Score".to_string());
    let mut similarity_tuples: Vec<(String, f64)> = types
    .iter()
    .map(|s| (s.clone(), jaro_winkler(&partial, s)))
    .collect();
    similarity_tuples.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let sorted: Vec<String> = similarity_tuples.into_iter().map(|(s, _)| s).collect();
    futures::stream::iter(sorted)
}

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn hint(
    ctx: Context<'_>,
    #[description = "Gives you a hint on the anime"] 
    mut number_of_hints: u64,
    #[description = "Select what type of hint you want"]
    #[autocomplete = "autocomplete_hint_type"]
    hint_type: Option<String>,
) -> Result<(), Error> {
    if number_of_hints > 10 {
        number_of_hints = 1;
    }
    let channel_id = ctx.channel_id().get();
    let channel_check = database::get_anime_id_by_channel_id(ctx.channel_id().get()).await;
    match channel_check {
        Ok(anime_id) => {
        let mut disp_hints: Vec<String> = Vec::new();
            match database::get_hints(channel_id).await {
                Ok((mut rem_hints, mut cur_hints)) => {
                    let mut hint = anime_guessing_game::process_hint(&mut rem_hints, hint_type.clone(), number_of_hints);
                    cur_hints.append(&mut hint);
                    let _set_response = database::set_hints(channel_id, rem_hints, &cur_hints).await;
                    disp_hints = cur_hints;
                },
                Err(_) => {
                    ctx.say("An error occured!").await?;
                    return Ok(());
                },
            }
            ctx.say(helpers::display_str_vec(&disp_hints)).await?;
        },
        Err(_) => {
            ctx.say("No anime to give hints for!").await?;
            return Ok(());
        },
    }
    Ok(())
}