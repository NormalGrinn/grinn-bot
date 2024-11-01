use std::env;

use crate::{team_swapping::team_swap_utils, Context, Error};
use poise::CreateReply;
use rusqlite::Result;

use crate::database;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn set_phase(
    ctx: Context<'_>,
    #[description = "Set the current phase"] 
    phase: u64,
) -> Result<(), Error> {
    match team_swap_utils::check_host_role(&ctx.author(), &ctx).await {
        Ok(b) => {
            if !b {
                let message = format!("{} does not have the host role, and therefore you can't set the phase", ctx.author().name.clone());
                ctx.send(CreateReply::default().content(message).ephemeral(true)).await?;
                return  Ok(())
            }
        },
        Err(_) => {
            ctx.send(CreateReply::default().content("An error has occured checking roles").ephemeral(true)).await?;
            return Ok(())
        },
    }
    if phase > 3 {
        ctx.send(CreateReply::default().content("trying to set invalid phase").ephemeral(true)).await?;
        return Ok(())
    }
    env::set_var("PHASE", format!("{}", phase));
    let message = format!("Set phase to: {}", phase);
    ctx.send(CreateReply::default().content(message).ephemeral(true)).await?;
    Ok(())
}