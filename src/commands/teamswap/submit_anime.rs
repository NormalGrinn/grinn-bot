use crate::{team_swapping::team_swap_utils, Context, Error};
use poise::CreateReply;
use rusqlite::Result;
use regex::Regex;

use crate::database;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn submit_anime(
    ctx: Context<'_>,
    #[description = "The anime you want to submit, this should be an AL URL to the anime"] 
    link: String,
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
    let user = ctx.author();
    match team_swap_utils::check_swapper_role(&user, &ctx).await {
        Ok(b) => {
            if !b {
                let message = format!("{} does not have the swapper role, and therefore you cannot submit an anime", user.name);
                ctx.send(CreateReply::default().content(message).ephemeral(true)).await?;
                return  Ok(())
            }
        },
        Err(_) => {
            ctx.send(CreateReply::default().content("An error has occured").ephemeral(true)).await?;
            return Ok(())
        },
    }
    match database::check_if_user_exists(user.id.get()) {
        Ok(b) => { if !b { database::create_member(ctx.author().clone())?; } },
        Err(_) => {
            ctx.send(CreateReply::default().content("Error chekcing if users exists").ephemeral(true)).await?;
            return Ok(());
        },
    }
    match database::count_submitted_anime(user.id.get()) {
        Ok(count) => {
            if count >= 12 {
                ctx.send(CreateReply::default().content("You have already submitted 12 anime, if you want to submit a different anime you should remove a submission").ephemeral(true)).await?;
                return Ok(());
            }
        },
        Err(_) => {
            ctx.send(CreateReply::default().content("Error counting the submitted anime").ephemeral(true)).await?;
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
                    let cleaned_title = anime_title.replace("-", " ");
                    database::create_anime(anime_id, &cleaned_title, user.id.get())?;
                    ctx.send(CreateReply::default().content("Anime was submitted successfully").ephemeral(true)).await?;
                    return Ok(());
                } else {
                    ctx.send(CreateReply::default().content("Anime has already been submitted").ephemeral(true)).await?;
                    return Ok(());
                }

            },
            Err(_) => {
                ctx.send(CreateReply::default().content("An error has occured checking if the anime exists").ephemeral(true)).await?;
                return Ok(());
            },
        }
    } else {
        ctx.send(CreateReply::default().content("You did not provide a valid AL link").ephemeral(true)).await?;
        return Ok(());
    }
}