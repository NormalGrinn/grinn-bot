use crate::{team_swapping::team_swap_utils, Context, Error};
use rusqlite::Result;
use poise::serenity_prelude as serenity;

use crate::database;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn create_team(
    ctx: Context<'_>,
    #[description = "The first team member, this should be you"] 
    member1: serenity::User,
    #[description = "The name of your team"] 
    team_name: String,
    #[description = "The second team member"] 
    member2: Option<serenity::User>,
    #[description = "The third team member"] 
    member3: Option<serenity::User>,
) -> Result<(), Error> {
    let mut members = vec![member1];
    match member2 {
        Some(x) => members.push(x),
        None => (),
    }
    match member3 {
        Some(x) => members.push(x),
        None => (),
    }
    for m in &members {
        match team_swap_utils::check_swapper_role(&m, &ctx).await {
            Ok(b) => {
                if !b {
                    let message = format!("{} does not have the swapper role, and therefore a team cannot be created", m.global_name.clone().unwrap());
                    ctx.say(message).await?;
                    return  Ok(())
                }
            },
            Err(_) => {
                ctx.say("An error has occured").await?;
                return Ok(())
            },
        }
        match team_swap_utils::check_if_already_participating(m).await {
            Ok(b) => {
                if b {
                    let message = format!("{} already is in another team therefore a team cannot be created", m.global_name.clone().unwrap());
                    ctx.say(message).await?;
                    return  Ok(())
                }
            },
            Err(_) => {
                ctx.say("An error has occured").await?;
                return Ok(())
            },
        }
    }
    match team_swap_utils::check_if_team_exists(&team_name).await {
        Ok(b) => if b {
            ctx.say("A team with this name already exists, please pick another one").await?;
            return Ok(());
        },
        Err(_) =>  {
            ctx.say("An error has occured").await?;
            return Ok(())
        },
    }
    let res = database::create_team(members, team_name).await;
    match res {
        Ok(_) => {
            ctx.say("Team has been created succesfully").await?;
            return Ok(())
        },
        Err(_) => {
            ctx.say("An error occured").await?;
            return Ok(())
        },
    }
}