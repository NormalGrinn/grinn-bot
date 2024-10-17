use std::clone;

use crate::{Context, Error};
use rusqlite::Result;
use serenity::utils;

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
            for team in res {
                let names: Vec<String> = team.members.iter().map(|member| member.member_name.clone()).collect();
                let members = helpers::display_str_vec(&names);
                message.push(format!("Team: {} has members: {}", team.team.team_name, members))
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