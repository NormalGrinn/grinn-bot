use crate::{team_swapping::team_swap_utils, types, Context, Error};
use poise::CreateReply;
use rusqlite::Result;
use serenity::all::{CreateAttachment, CreateMessage};
use tokio::fs::OpenOptions;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const PATH: &str = "status.txt";

use crate::database;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn status(
    ctx: Context<'_>,
) -> Result<(), Error> {
    match team_swap_utils::check_host_role(&ctx.author(), &ctx).await {
        Ok(b) => {
            if !b {
                let message = format!("{} does not have the host role, and therefore you can't check the status", ctx.author().name.clone());
                ctx.send(CreateReply::default().content(message).ephemeral(true)).await?;
                return  Ok(())
            }
        },
        Err(_) => {
            ctx.send(CreateReply::default().content("An error has occured checking roles").ephemeral(true)).await?;
            return Ok(())
        },
    }
    let mut teams: Vec<types::TeamMembers>;
    let mut counted_submissions: Vec<(u64, String, u64)>;
    let lonely_users: Vec<(u64, String)>;
    match database::get_teams() {
        Ok(t) => teams = t,
        Err(_) => {
            ctx.send(CreateReply::default().content("Error fetching teams").ephemeral(true)).await?;
            return Ok(())
        },    
    }
    match database::count_submissions_by_user() {
        Ok(s) => counted_submissions = s,
        Err(_) => {
            ctx.send(CreateReply::default().content("Error fetching submissions").ephemeral(true)).await?;
            return Ok(())
        },    
    }
    match database::get_lonely_users() {
        Ok(u) => lonely_users = u,
        Err(_) => {
            ctx.send(CreateReply::default().content("Error fetching users").ephemeral(true)).await?;
            return Ok(())
        },    
    }
    counted_submissions.sort_by_key(|k| k.2);
    teams.sort_by_key(|k| k.members.len());

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(PATH)
        .await?;
    let mut buffer = String::new();

    buffer.push_str("Users who are not yet in a team:\n");
    for (_, name) in lonely_users {
        buffer.push_str(&format!("{}\n", name));
    }
    buffer.push_str("\nTeams sorted by amount of members:\n");
    for team in &teams {
        buffer.push_str(&format!("{} number of members: {}\n", team.team.team_name, team.members.len()));
    }
    buffer.push_str("\nNumber of submissions per member:\n");
    for (_, name, count) in counted_submissions {
        buffer.push_str(&format!("{} has submitted ({}/12) anime\n", name, count));
    }

    fs::write(PATH, buffer.as_bytes()).await?;
    file.flush().await?;

    let attachment = CreateAttachment::path(PATH).await?;
    let builder = CreateMessage::new().add_file(attachment);

    match ctx.author().direct_message(ctx.http(), builder).await {
        Ok(_) => {
            ctx.send(CreateReply::default().content("Successfully sent status").ephemeral(true)).await?;
        },
        Err(_) => {
            ctx.send(CreateReply::default().content("Error sending status").ephemeral(true)).await?;
        },
    };

    let _ = tokio::fs::remove_file(PATH).await; // Ignore error if the file doesn't exist

    Ok(())
}