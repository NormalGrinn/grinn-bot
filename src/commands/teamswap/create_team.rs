use crate::{team_swapping::team_swap_utils, Context, Error};
use rusqlite::Result;
use poise::{serenity_prelude as serenity, CreateReply};

use crate::database;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn create_team(
    ctx: Context<'_>,
    #[description = "The name of your team"] 
    team_name: String,
    #[description = "The first team member, this should be you"] 
    member1: serenity::User,
    #[description = "The second team member"] 
    member2: Option<serenity::User>,
    #[description = "The third team member"] 
    member3: Option<serenity::User>,
) -> Result<(), Error> {
    match team_swap_utils::check_phase(vec![2]) {
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
    if member1.id.get() != ctx.author().id.get() {
        ctx.send(CreateReply::default().content("You should be the first member of your team").ephemeral(true)).await?;
        return Ok(())
    }
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
                    ctx.send(CreateReply::default().content(message).ephemeral(true)).await?;
                    return  Ok(())
                }
            },
            Err(_) => {
                ctx.send(CreateReply::default().content("An error has occured checking roles").ephemeral(true)).await?;
                return Ok(())
            },
        }
        match database::check_if_user_exists(m.id.get()) {
            Ok(b) => {
                if !b {
                    let message = format!("{} has not joined yet", m.global_name.clone().unwrap());
                    ctx.send(CreateReply::default().content(message).ephemeral(true)).await?;
                    return  Ok(())
                }
            },
            Err(e) => {
                println!("{:?}", e);
                ctx.send(CreateReply::default().content("An error has occured checking if users exists").ephemeral(true)).await?;
                return Ok(())
            },
        }
        match database::check_if_user_in_team(m.id.get()) {
            Ok(team_id) => {
                match team_id {
                    Some(_) => {
                        let message = format!("{} is already in a team and thus cannot be added", m.name);
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
        match database::get_submitted_anime(m.id.get()) {
            Ok(n) => {
                if n.len() < 7 {
                    ctx.send(CreateReply::default().content("You should have at least submitted 7 anime before joining a team").ephemeral(true)).await?;
                    return Ok(())
                }
            },
            Err(_) => {
                ctx.send(CreateReply::default().content("Error checking submitted anime").ephemeral(true)).await?;
                return Ok(())
            },
        }
    }
    match team_swap_utils::check_if_team_exists(&team_name) {
        Ok(b) => if b {
            ctx.send(CreateReply::default().content("A team with this name already exists, please pick another one").ephemeral(true)).await?;
            return Ok(());
        },
        Err(_) =>  {
            ctx.send(CreateReply::default().content("An error has occured checking teams").ephemeral(true)).await?;
            return Ok(())
        },
    }
    let res = database::create_team(&members, &team_name);
    match res {
        Ok(_) => {
            let mut users: String = String::new();
            for user in members {
                users.push_str(&format!("<@{}> ", user.id.get()));
            }
            let message = format!("Team {} has been created with the members: {}", &team_name, users);
            ctx.say(message).await?;
            return Ok(())
        },
        Err(_) => {
            ctx.send(CreateReply::default().content("An error occured creating the team").ephemeral(true)).await?;
            return Ok(())
        },
    }
}