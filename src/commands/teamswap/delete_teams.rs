use crate::{Context, Error};
use rusqlite::Result;

use crate::database;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn delete_teams(
    ctx: Context<'_>
) -> Result<(), Error> {
    match database::delete_teams() {
        Ok(_) => {
            ctx.say("Teams were deleted succesfully").await?;
            return Ok(())
        },
        Err(_) => {
            ctx.say("An error occured deleting the teams").await?;
            return Ok(())
        },
    }
}