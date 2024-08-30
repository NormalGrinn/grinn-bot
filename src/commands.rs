use crate::types;
use crate::{Context, Error};
use rusqlite::Result;
use serenity::futures;
use futures::{Stream, StreamExt};

use crate::database;
use crate::anime_guessing_game;
use crate::helpers;

async fn autocomplete_guess<'a>(
    ctx: Context<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    let names = database::get_filtered_names(&partial, ctx.channel_id().get()).await;
    println!("{:?}", names);
    futures::stream::iter(names)
}

/// Show this help menu
#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            extra_text_at_bottom: "These are the commands you can do",
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn animeguess(
    ctx: Context<'_>,
    #[description = "Start a round of the anime guessing game. Usage: /animeguess (AL username)"] username: String,
    //command: Option<String>,
) -> Result<(), Error> {
    let channel_check = database::get_anime_id_by_channel_id(ctx.channel_id().get()).await;
    match channel_check {
        Ok(x) => if x != 0 {
            ctx.say("There already is a game in progress!").await?;
            return Ok(());
        },
        Err(_) => (),
    };
    let (mut entry_info, names) = anime_guessing_game::anime_guessing_setup(&username).await;
    let mut starting_hint_wrapper = types::AnimeGuess {
        id: entry_info.id,
        synonyms: entry_info.synonyms.clone(),
        hints: vec!(entry_info.hints.remove(0)),
    };
    let starting_hint = anime_guessing_game::process_hint(&mut starting_hint_wrapper.hints);
    let gotten_hints: Vec<String> = vec!(starting_hint);
    let starting_message = format!("The anime guessing game has started for {}\n{}", username, gotten_hints[0]);
    let resp = database::set_anime_info(ctx.channel_id().get(), entry_info, gotten_hints, names).await;
    match resp {
        Ok(_) => {
            ctx.say(starting_message).await?;
            return Ok(())
        },
        Err(_) => (),
    }
    ctx.say("An error has occured").await?;
    Ok(())
}

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn giveup(
    ctx: Context<'_>
    //command: Option<String>,
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

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn hint(
    ctx: Context<'_>,
    #[description = "Gives you a hint on the anime"] mut number_of_hints: u64,
) -> Result<(), Error> {
    if number_of_hints > 10 {
        number_of_hints = 1;
    }
    let channel_id = ctx.channel_id().get();
    let channel_check = database::get_anime_id_by_channel_id(ctx.channel_id().get()).await;
    match channel_check {
        Ok(anime_id) => {
            let mut disp_hints: Vec<String> = Vec::new();
            for i in 0..number_of_hints {
                match database::get_hints(channel_id).await {
                    Ok((mut rem_hints, mut cur_hints)) => {
                        let hint = anime_guessing_game::process_hint(&mut rem_hints);
                        cur_hints.push(hint);
                        let set_response = database::set_hints(channel_id, rem_hints, &cur_hints).await;
                        disp_hints = cur_hints;
                    },
                    Err(_) => {
                        ctx.say("An error occured!").await?;
                        return Ok(());
                    },
                }
            }
            ctx.say(helpers::display_str_vec(&disp_hints)).await?;
        },
        Err(_) => {
            ctx.say("No anime to give hints for!").await?;
            return Ok(());
        },
    }
    Ok(())
}

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn guess(
    ctx: Context<'_>,
    #[description = "Gives the anime"] 
    #[autocomplete = "autocomplete_guess"]
    anime_guess: String,
) -> Result<(), Error> {
    let channel_id = ctx.channel_id().get();
    let channel_check = database::get_anime_id_by_channel_id(ctx.channel_id().get()).await;
    match channel_check {
        Ok(anime_id) => {
            let resp = database::get_anime_synonyms(channel_id).await;
            match resp {
                Ok(wrapped_synonyms) => {
                    let correct = anime_guessing_game::process_guess(&anime_guess, &wrapped_synonyms.stringvec).await;
                    if correct {
                        let win_message = format!("You guessed right! The anime was: https://anilist.co/anime/{}", anime_id);
                        ctx.say(win_message).await?;
                        let _ = database::give_up(channel_id).await;
                        return Ok(());
                    } else {
                        let wrong_message = format!("Your guess of {} is wrong!", anime_guess);
                        ctx.say(wrong_message).await?;
                        return Ok(());
                    }
                },
                Err(_) => {
                    ctx.say("An error occured").await?;
                    return Ok(());
                },      
            }
        },
        Err(_) => {
            ctx.say("No anime to guess!").await?;
            return Ok(());
        },        
    }
}
