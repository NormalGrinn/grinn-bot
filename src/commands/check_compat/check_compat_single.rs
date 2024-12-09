use crate::{compat_check::{self, mean_abs_diff::calculate_mad, calculate_cosine_sim::calculate_cosine_sim, mean_abs_diff_norm::calculate_mad_norm}, types, Context, Error};
use rusqlite::Result;
use serenity::futures;
use strsim::jaro_winkler;

const PATH: &str = "comp.txt";

async fn autocomplete_sim<'a>(
    ctx: Context<'_>,
    partial: &'a str,
) -> impl serenity::futures::Stream<Item = String> + 'a {
    let types: Vec<String> = vec!("CosineSim".to_string(), "MeanAbsoluteDifferenceNorm".to_string(), "MeanAbsoluteDifference".to_string());
    let mut similarity_tuples: Vec<(String, f64)> = types
    .iter()
    .map(|s| (s.clone(), jaro_winkler(&partial, s)))
    .collect();
    similarity_tuples.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let sorted: Vec<String> = similarity_tuples.into_iter().map(|(s, _)| s).collect();
    futures::stream::iter(sorted)
}

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn check_compat_single(
    ctx: Context<'_>,
    #[description = "AL username 1"] 
    username1: String,
    #[description = "AL username 2"] 
    username2: String,
    #[description = "Similairty measure"]
    #[autocomplete = "autocomplete_sim"]
    sim_measure: Option<types::SimilarityMeasure> 
) -> Result<(), Error> {
    ctx.defer().await?;
    let mut list_main = compat_check::get_anime_list::get_anime_list(&username1).await;
    list_main.sort_by_key(|f| f.id);
    let list2 = compat_check::get_anime_list::get_anime_list(&username2).await;
    let sim_score: f64;
    let entries: usize;
    match sim_measure {
        Some(sim) => {
            match sim {
                types::SimilarityMeasure::CosineSim => (sim_score, entries) = calculate_cosine_sim(&list_main, list2),
                types::SimilarityMeasure::MeanAbsoluteDifferenceNorm => (sim_score, entries) = calculate_mad_norm(&list_main, list2),
                types::SimilarityMeasure::MeanAbsoluteDifference => (sim_score, entries) = calculate_mad(&list_main, list2),
            }
        },
        None => (sim_score, entries) = calculate_mad_norm(&list_main, list2),
    }
    let message = format!("{} has a compatibility score of {} with {}, and shares {} completed entries with them!", username1, sim_score, username2, entries);
    ctx.say(message).await?;
    Ok(())
}