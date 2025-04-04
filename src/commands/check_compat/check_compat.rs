use crate::{compat_check::{self, calculate_cosine_sim::calculate_cosine_sim, mean_abs_diff::calculate_mad, mean_abs_diff_norm::calculate_mad_norm}, types::{self, SimilarityMeasure}, Context, Error};
use poise::CreateReply;
use rusqlite::Result;
use serenity::{all::CreateAttachment, futures::{self, Stream}};
use strsim::jaro_winkler;
use tokio::fs::OpenOptions;
use tokio::fs;
use tokio::io::AsyncWriteExt;

const PATH: &str = "comp.txt";

async fn autocomplete_sim<'a>(
    ctx: Context<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    let types: Vec<String> = vec!("CosineSim".to_string(), "MeanAbsoluteDifferenceNorm".to_string(), "MeanAbsoluteDifference".to_string());
    let mut similarity_tuples: Vec<(String, f64)> = types
    .iter()
    .map(|s| (s.clone(), jaro_winkler(&partial, s)))
    .collect();
    similarity_tuples.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let sorted: Vec<String> = similarity_tuples.into_iter().map(|(s, _)| s).collect();
    futures::stream::iter(sorted)
}

async fn cos_sim(mainlist: &Vec<types::AnimeScored>, friends: Vec<String>) -> Vec<(String, f64, usize)> {
    let mut results: Vec<(String, f64, usize)> = Vec::new(); 
    for friend in friends {
        let friend_list = compat_check::get_anime_list::get_anime_list(&friend).await;
        let (z_score, shared) = calculate_mad_norm(mainlist, friend_list);
        if z_score.is_nan() {
            continue
        }
        results.push((friend, z_score, shared));
    }
    results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    results
}

async fn mad_sim_norm(mainlist: &Vec<types::AnimeScored>, friends: Vec<String>) -> Vec<(String, f64, usize)> {
    let mut results: Vec<(String, f64, usize)> = Vec::new(); 
    for friend in friends {
        let friend_list = compat_check::get_anime_list::get_anime_list(&friend).await;
        let (z_score, shared) = calculate_mad_norm(mainlist, friend_list);
        if z_score.is_nan() {
            continue
        }
        results.push((friend, z_score, shared));
    }
    results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    results
}

async fn mad_sim(mainlist: &Vec<types::AnimeScored>, friends: Vec<String>) -> Vec<(String, f64, usize)> {
    let mut results: Vec<(String, f64, usize)> = Vec::new(); 
    for friend in friends {
        let friend_list = compat_check::get_anime_list::get_anime_list(&friend).await;
        let (z_score, shared) = calculate_mad(mainlist, friend_list);
        if z_score.is_nan() {
            continue
        }
        results.push((friend, z_score, shared));
    }
    results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    results
}

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn check_compat(
    ctx: Context<'_>,
    #[description = "Your AL username"] 
    username: String,
    #[description = "Similairty measure"]
    #[autocomplete = "autocomplete_sim"]
    sim_measure: Option<types::SimilarityMeasure> 
) -> Result<(), Error> {
    ctx.defer().await?;
    let mut list_main = compat_check::get_anime_list::get_anime_list(&username).await;
    list_main.sort_by_key(|f| f.id);
    let friends = compat_check::get_friends::get_friends_names(&username).await;
    
    // Name, compat, shared
    let mut results: Vec<(String, f64, usize)>; 
    
    match sim_measure {
        Some(sim) => {
            match sim {
                SimilarityMeasure::CosineSim => results =  cos_sim(&list_main, friends).await,
                SimilarityMeasure::MeanAbsoluteDifferenceNorm => results = mad_sim_norm(&list_main, friends).await,
                SimilarityMeasure::MeanAbsoluteDifference => results = mad_sim(&list_main, friends).await,
            }       
        },
        None => results = mad_sim_norm(&list_main, friends).await,
    }

    let mut buffer = String::new();

    for res in results {
        buffer.push_str(&format!("{}, {}, shared entries: {}\n", res.0, res.1, res.2));
    }

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(PATH)
        .await?;
    fs::write(PATH, buffer.as_bytes()).await?;
    file.flush().await?;

    let attachment = CreateAttachment::path(PATH).await?;
    let reply = CreateReply::default().attachment(attachment);

    let message = format!("<@{}> compat checking is done!", ctx.author().id.get());
    ctx.say(message).await?;
    ctx.send(reply).await?;

    let _ = tokio::fs::remove_file(PATH).await; // Ignore error if the file doesn't exist

    Ok(())
}