use crate::{database, server_list::get_user_list, Context, Error};
use rusqlite::Result;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn add_user(
    ctx: Context<'_>,
    #[description = "AL username"] 
    username: String,
) -> Result<(), Error> {
    let user_list = get_user_list::get_user_list(&username).await;
    let res = database::upsert_user(user_list).await;
    let _ = match res {
        Ok(_) => {
            let message = format!("{} was successfully added", username);
            ctx.say(message).await?
        },
        Err(_) => ctx.say("A problem occured").await?,
    };
    Ok(())
}