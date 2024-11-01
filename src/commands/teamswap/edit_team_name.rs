use crate::{team_swapping::team_swap_utils, Context, Error};
use poise::CreateReply;
use rusqlite::Result;

use crate::database;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn edit_team_name(
    ctx: Context<'_>,
    #[description = "The new name of your team"] 
    new_name: String,
) -> Result<(), Error> {
    match team_swap_utils::check_phase(vec![2,3]) {
        Ok(b) => {
            if !b {
                ctx.send(CreateReply::default().content("Command is not allowed in current phase").ephemeral(true)).await?;
                return Ok(());
            }
        },
        Err(_) => {
            ctx.send(CreateReply::default().content("Error checking phases").ephemeral(true)).await?;
            return Ok(())
        },
    }
    let user_id = ctx.author().id.get();
    let team_id: u64;
    match  database::check_if_user_in_team(user_id) {
        Ok(id) => {
            match id {
                Some(id) => team_id = id,
                None => {
                    ctx.send(CreateReply::default().content("You are not in a team and thus cannot edit it").ephemeral(true)).await?;
                    return Ok(());
                },
            }
        },
        Err(_) => {
            ctx.send(CreateReply::default().content("Error checking if you are in a team").ephemeral(true)).await?;
            return Ok(())
        },
    }
    match database::update_team_name(team_id, new_name) {
    Ok(_) => {
        ctx.send(CreateReply::default().content("Updated team name").ephemeral(true)).await?;
        return Ok(())
    },
    Err(_) => {
        ctx.send(CreateReply::default().content("Error updating team names").ephemeral(true)).await?;
        return Ok(())
    },
    }
}