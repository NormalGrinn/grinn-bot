use crate::{Context, Error};
use rusqlite::Result;

use crate::database;
use crate::anime_guessing_game;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn animeguess(
    ctx: Context<'_>,
    #[description = "Start a round of the anime guessing game. Usage: /animeguess (AL username)"] 
    username: String,
    #[description = "Optional parameter for selecting which list you want to use"] 
    list_number: Option<usize>,
) -> Result<(), Error> {
    let mut list = 0;
    match list_number {
        Some(x) => list = x - 1,
        None => (),
    };
    let channel_check = database::get_anime_id_by_channel_id(ctx.channel_id().get()).await;
    match channel_check {
        Ok(x) => if x != 0 {
            ctx.say("There already is a game in progress!").await?;
            return Ok(());
        },
        Err(_) => (),
    };
    ctx.defer().await?;
    let (mut entry_info, names) = anime_guessing_game::anime_guessing_setup(&username, list).await;
    let gotten_hints: Vec<String> = Vec::new();
    let starting_message = format!("The anime guessing game has started for {}", username);
    match database::set_anime_info(ctx.channel_id().get(), entry_info, gotten_hints, names).await {
        Ok(_) => {
            ctx.say(starting_message).await?;
            return Ok(())
        },
        Err(e) => eprintln!("Error setting anime info: {:?}", e)
    }
    ctx.say("An error has occured").await?;
    Ok(())
}