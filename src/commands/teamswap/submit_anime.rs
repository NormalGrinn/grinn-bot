use crate::{team_swapping::team_swap_utils, Context, Error};
use rusqlite::Result;
use regex::Regex;

use crate::database;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn submit_anime(
    ctx: Context<'_>,
    #[description = "The anime you want to submit, this should be an AL URL to the anime"] 
    link: String,
) -> Result<(), Error> {
    let user = ctx.author();
    match team_swap_utils::check_swapper_role(&user, &ctx).await {
        Ok(b) => {
            if !b {
                let message = format!("{} does not have the swapper role, and therefore a team cannot be created", user.name);
                ctx.say(message).await?;
                return  Ok(())
            }
        },
        Err(_) => {
            ctx.say("An error has occured").await?;
            return Ok(())
        },
    }
    match database::check_if_user_exists(user.id.get()) {
        Ok(b) => { if !b { database::create_member(ctx.author().clone())?; } },
        Err(_) => {
            ctx.say("Error chekcing if users exists").await?;
            return Ok(());
        },
    }
    match database::count_submitted_anime(user.id.get()) {
        Ok(count) => {
            if count > 7 {
                ctx.say("You have already submitted 7 anime, if you want to submit a different anime you should remove a submission").await?;
                return Ok(());
            }
        },
        Err(_) => {
            ctx.say("Error counting the submitted anime").await?;
            return Ok(())
        }
    }
    let re = Regex::new(r"^https://anilist\.co/anime/(\d+)/([a-zA-Z0-9-]+)/?$").unwrap();
    if let Some(captures) = re.captures(&link) {
        let anime_id = &captures[1].parse::<u64>().unwrap();
        let anime_title = &captures[2];
        match database::check_if_anime_exists(*anime_id) {
            Ok(b) => {
                if !b {
                    database::create_anime(anime_id, &anime_title.to_string(), user.id.get())?;
                    ctx.say("Anime was submitted successfully").await?;
                    return Ok(());
                } else {
                    ctx.say("Anime has already been submitted").await?;
                    return Ok(());
                }

            },
            Err(_) => {
                ctx.say("An error has occured checking if the anime exists").await?;
                return Ok(());
            },
        }
    } else {
        ctx.say("You did not provide a valid AL link").await?;
        return Ok(());
    }
}