use crate::{compat_check::{self, calculate_cosine_sim::calculate_cosine_sim, calculate_z_score::calculate_z}, types, Context, Error};
use poise::CreateReply;
use rusqlite::Result;
use serenity::all::CreateAttachment;
use tokio::fs::OpenOptions;
use tokio::fs;
use tokio::io::AsyncWriteExt;

const PATH: &str = "comp.txt";

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn check_compat_single(
    ctx: Context<'_>,
    #[description = "AL username 1"] 
    username1: String,
    #[description = "AL username 2"] 
    username2: String,
    #[description = "Similairty measure"]
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
                types::SimilarityMeasure::ZScore => (sim_score, entries) = calculate_z(&list_main, list2),
                types::SimilarityMeasure::CosineSim => (sim_score, entries) = calculate_cosine_sim(&list_main, list2),
            }
        },
        None => (sim_score, entries) = calculate_z(&list_main, list2),
    }
    let message = format!("{} has a compatibility score of {} with {}, and shares {} completed entries with them!", username1, sim_score, username2, entries);
    ctx.say(message).await?;
    Ok(())
}