use crate::{Context, Error};
use rusqlite::Result;
use crate::database;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn giveup(
    ctx: Context<'_>
) -> Result<(), Error> {
    let channel_id = ctx.channel_id().get();
    let channel_check = database::get_anime_id_by_channel_id(ctx.channel_id().get()).await;
    match channel_check {
        Ok(anime_id) => {
            let res = database::give_up(channel_id).await;
            match res {
                Ok(x) => { if x == 0 {
                    ctx.say("No game to give up!").await?;
                } else {
                    let resp = format!("You've given up, the anime was: https://anilist.co/anime/{}", anime_id);
                    ctx.say(resp).await?;
                }
                return Ok(());
            },
                Err(_) => {
                    ctx.say("An error occured").await?;
                    return Ok(());
                }
            }
        },
        Err(_) => {
            ctx.say("No game to give up!").await?;
            return Ok(());
        }
    };
}