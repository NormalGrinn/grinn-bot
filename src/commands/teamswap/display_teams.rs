use crate::{Context, Error};
use rusqlite::Result;

use crate::database;
use crate::helpers;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn display_teams(
    ctx: Context<'_>
) -> Result<(), Error> {
    let members = database::get_teams();
    match members {
        Ok(res) => {
            let mut message: Vec<String> = Vec::new();
            if res.is_empty() {
                message.push("There are no teams to be displayed".to_string());
            }
            for (name, team) in res {
                message.push(format!("Member: {} in team: {}", name, team))
            }
            ctx.say(helpers::display_str_vec(&message)).await?;
            return Ok(())
        },
        Err(_) => {
            ctx.say("An error occured").await?;
            return Ok(())
        },
    }
}