use crate::{Context, Error};
use poise::CreateReply;
use regex::Regex;
use rusqlite::Result;

use crate::database;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn edit_team_image(
    ctx: Context<'_>,
    #[description = "An imgur link to the image for your team"]
    new_image: String,
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
    let re = Regex::new(r"^https:\/\/i\.imgur\.com\/[a-zA-Z0-9]+\.[a-zA-Z0-9]+$").unwrap();
    if !re.is_match(&new_image) {
        ctx.send(CreateReply::default().content("You did not submit a valid imgur link").ephemeral(true)).await?;
        return Ok(())
    }
    match database::update_team_image(team_id, new_image) {
        Ok(_) => {
            ctx.send(CreateReply::default().content("Updated team image").ephemeral(true)).await?;
            return Ok(())
        },
        Err(_) => {
            ctx.send(CreateReply::default().content("Error updating team image").ephemeral(true)).await?;
            return Ok(())
        },
        }
}