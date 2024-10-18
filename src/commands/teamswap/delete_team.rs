use crate::{Context, Error};
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
                    ctx.say("You are not in a team and thus cannot delete it").await?;
                    return Ok(());
                },
            }
        },
        Err(_) => {
            ctx.say("Error checking if you are in a team").await?;
            return Ok(())
        },
    }
    println!("Team ID: {}", team_id);
    match database::delete_team_by_team_id(team_id) {
        Ok(_) => {
            ctx.say("Team has been deleted").await?;
            return Ok(());
        },
        Err(_) => {
            ctx.say("Error deleting the team").await?;
            return Ok(());
        },
    }
}