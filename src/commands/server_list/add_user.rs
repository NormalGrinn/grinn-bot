use crate::{server_list::get_user_list, Context, Error};
use rusqlite::Result;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn add_user(
    ctx: Context<'_>,
    #[description = "AL username"] 
    username: String,
) -> Result<(), Error> {
    let (anime_info_list, user_id, score_format) = get_user_list::get_user_list(&username).await;
    
    Ok(())
}