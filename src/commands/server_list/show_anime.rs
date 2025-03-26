use crate::{database, Context, Error};
use poise::CreateReply;
use rusqlite::Result;
use serenity::{all::{CreateEmbed, CreateMessage}, futures};
use ::serenity::futures::Stream;
use rust_fuzzy_search::fuzzy_compare;

async fn autocomplete_anime<'a>(
    ctx: Context<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    let mut names = database::get_server_anime_titles().await;
    names.sort();
    names.dedup();
    let mut similarity_tuples: Vec<(String, f32)> = names
        .iter()
        .map(|s| (s.clone(), fuzzy_compare(partial, s)))
        .collect();
    similarity_tuples.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let titles: Vec<String> = similarity_tuples.into_iter().map(|(s, _)| s).collect();
    futures::stream::iter(titles)
}

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn show_anime(
    ctx: Context<'_>,
    #[description = "anime name"] 
    #[autocomplete = "autocomplete_anime"]
    anime_name: String,
) -> Result<(), Error> {
    let anime_info = database::get_anime_info(&anime_name).await;
    match anime_info {
        Ok(mut info) => {
            let mut num_of_scores = 0;
            let mut total_score: f64 = 0.0;
            for e in &info {
                if e.anime_score != 0.0 {
                    num_of_scores += 1;
                    total_score += e.anime_score;
                }
            }
            info.sort();
            info.reverse();
            let title = format!("Anime info for: {}", anime_name);
            let mut embed = CreateEmbed::new().title(title);
            for (_i, entry) in info.iter().enumerate().take(25) {
                embed = embed.field(&entry.user_name, entry.anime_score.to_string(), false);
            }
            let message = CreateReply::default().embed(embed);
            ctx.send(message).await?;
        },
        Err(e) => {
            eprintln!("{}", e);
            ctx.say("An error occured getting the anime info").await?;
        },
    }
    Ok(())
}