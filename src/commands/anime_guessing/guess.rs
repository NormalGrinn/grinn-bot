use crate::{Context, Error};
use rusqlite::Result;
use rust_fuzzy_search::fuzzy_compare;
use serenity::futures;
use ::serenity::futures::Stream;
use poise::serenity_prelude as serenity;

use crate::database;
use crate::anime_guessing_game;

async fn autocomplete_guess<'a>(
    ctx: Context<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    let mut names = database::get_filtered_names(ctx.channel_id().get()).await;
    names.sort();
    let mut similarity_tuples: Vec<(String, f32)> = names
        .iter()
        .map(|s| (s.clone(), fuzzy_compare(&partial.to_lowercase(), &s.to_lowercase())))
        .collect();
    similarity_tuples.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let guesses: Vec<String> = similarity_tuples.into_iter().map(|(s, _)| s).collect();
    futures::stream::iter(guesses)
}

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn guess(
    ctx: Context<'_>,
    #[description = "Gives the anime"] 
    #[autocomplete = "autocomplete_guess"]
    anime_guess: String,
) -> Result<(), Error> {
    let channel_id = ctx.channel_id().get();
    let channel_check = database::get_anime_id_by_channel_id(ctx.channel_id().get()).await;
    match channel_check {
        Ok(anime_id) => {
            let resp = database::get_anime_synonyms(channel_id).await;
            match resp {
                Ok(wrapped_synonyms) => {
                    let correct = anime_guessing_game::process_guess(&anime_guess, &wrapped_synonyms.stringvec).await;
                    if correct {
                        let win_message = format!("You guessed right! <:happy:1210688855011762196> The anime was: https://anilist.co/anime/{}", anime_id);
                        ctx.say(win_message).await?;
                        let _ = database::give_up(channel_id).await;
                        return Ok(());
                    } else {
                        let wrong_message = format!("Your guess of {} is wrong!", anime_guess);
                        ctx.say(wrong_message).await?;
                        return Ok(());
                    }
                },
                Err(_) => {
                    ctx.say("An error occured").await?;
                    return Ok(());
                },      
            }
        },
        Err(_) => {
            ctx.say("No anime to guess!").await?;
            return Ok(());
        },        
    }
}