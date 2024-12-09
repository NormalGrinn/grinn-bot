use crate::{compat_check::{self, calculate_z_score::calculate_z}, Context, Error};
use poise::CreateReply;
use rusqlite::Result;
use serenity::all::CreateAttachment;
use tokio::fs::OpenOptions;
use tokio::fs;
use tokio::io::AsyncWriteExt;

const PATH: &str = "comp.txt";

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn check_compat(
    ctx: Context<'_>,
    #[description = "Your AL username"] 
    username: String,
) -> Result<(), Error> {
    ctx.defer().await?;
    let mut list_main = compat_check::get_anime_list::get_anime_list(&username).await;
    list_main.sort_by_key(|f| f.id);
    let friends = compat_check::get_friends::get_friends_names(&username).await;
    
    // Name, compat, shared
    let mut results: Vec<(String, f64, usize)> = Vec::new(); 
    
    for friend in friends {
        let friend_list = compat_check::get_anime_list::get_anime_list(&friend).await;
        let (z_score, shared) = calculate_z(&list_main, friend_list);
        if z_score.is_nan() {
            continue
        }
        results.push((friend, z_score, shared));
    }

    let mut buffer = String::new();

    results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

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