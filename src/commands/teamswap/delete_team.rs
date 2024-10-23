use crate::{Context, Error};
use poise::CreateReply;
use rusqlite::Result;

use crate::database;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn delete_team(
    ctx: Context<'_>,
) -> Result<(), Error> {
    let user_id = ctx.author().id.get();
    let team_id: u64;
    match  database::check_if_user_in_team(user_id) {
        Ok(id) => {
            match id {
                Some(id) => team_id = id,
                None => {
                    ctx.send(CreateReply::default().content("You are not in a team and thus cannot delete it").ephemeral(true)).await?;
                    return Ok(());
                },
            }
        },
        Err(_) => {
            ctx.send(CreateReply::default().content("Error checking if you are in a team").ephemeral(true)).await?;
            return Ok(())
        },
    }
    match database::delete_team_by_team_id(team_id) {
        Ok(_) => {
            ctx.send(CreateReply::default().content("Team has been deleted").ephemeral(true)).await?;
            return Ok(());
        },
        Err(_) => {
            ctx.send(CreateReply::default().content("Error deleting the team").ephemeral(true)).await?;
            return Ok(());
        },
    }
}