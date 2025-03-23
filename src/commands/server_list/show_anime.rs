use crate::{database, Context, Error};
use rusqlite::Result;
use serenity::futures;
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
        Ok(info) => {

        },
        Err(e) => {
            eprintln!("{}", e);
            ctx.say("An error occured getting the anime info");
        },
    }
    Ok(())
}