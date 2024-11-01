use crate::{team_swapping::team_swap_utils, Context, Error};
use poise::CreateReply;
use rusqlite::Result;

use crate::database;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn leave(
    ctx: Context<'_>,
) -> Result<(), Error> {
    match team_swap_utils::check_phase(vec![1,2]) {
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
    match database::check_if_user_in_team(ctx.author().id.get()) {
        Ok(team_id) => {
            match team_id {
                Some(_) => {
                    let message = format!("If you want to leave the event, first leave your team");
                    ctx.send(CreateReply::default().content(message).ephemeral(true)).await?;
                    return Ok(());
                },
                None => (),
            }
        },
        Err(_) => {
            ctx.send(CreateReply::default().content("An error has occured checking if the user is already in a team").ephemeral(true)).await?;
            return Ok(())
        },
    }
    match database::delete_user(&ctx.author().id.get()) {
        Ok(_) => {
            ctx.send(CreateReply::default().content("You have now left the event, and all your anime are removed").ephemeral(true)).await?;
        },
        Err(_) => {
            ctx.send(CreateReply::default().content("Error removing you from the event").ephemeral(true)).await?;
        },
    }
    Ok(())
}