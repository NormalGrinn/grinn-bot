use crate::{Context, Error};
use rusqlite::Result;

use crate::database;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn edit_team_image(
    ctx: Context<'_>,
    #[description = "The new image for your team"]
    new_image: String,
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
    match database::update_team_image(team_id, new_image) {
        Ok(_) => {
            ctx.say("Updated team image").await?;
            return Ok(())
        },
        Err(_) => {
            ctx.say("Error updating team image").await?;
            return Ok(())
        },
        }
}