use crate::{database, server_list::get_user_list, Context, Error};
use rusqlite::Result;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn remove_user(
    ctx: Context<'_>,
    #[description = "AL username"] 
    username: String,
) -> Result<(), Error> {
    let res = database::remove_user(&username).await;
    match res {
        Ok(_) => {
            let message = format!("Successfully removed {}", username);
            ctx.say(message).await?;
        },
        Err(e) => {
            eprintln!("{}", e);
            let message = format!("Error removing {}", username);
            ctx.say(message).await?;
        },
    }
    Ok(())
}