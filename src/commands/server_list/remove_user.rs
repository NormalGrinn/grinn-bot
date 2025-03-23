use crate::{database, server_list::get_user_list, Context, Error};
use rusqlite::Result;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn remove_user(
    ctx: Context<'_>,
    #[description = "AL username"] 
    username: String,
) -> Result<(), Error> {
    Ok(())
}