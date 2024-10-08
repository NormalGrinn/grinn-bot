use crate::{Context, Error};
use rusqlite::Result;
use serenity::futures;
use futures::{Stream};
use strsim::jaro_winkler;
use poise::serenity_prelude as serenity;

use crate::database;
use crate::anime_guessing_game;
use crate::helpers;

async fn autocomplete_guess<'a>(
    ctx: Context<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    let names = database::get_filtered_names(&partial, ctx.channel_id().get()).await;
    futures::stream::iter(names)
}

/*
                    let mut similarity_tuples: Vec<(String, f64)> = names.stringvec
                    .iter()
                    .map(|s| (s.clone(), jaro_winkler(partial, s)))
                    .collect();
                    similarity_tuples.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                    let filtered_names: Vec<String> = similarity_tuples.into_iter().map(|(s, _)| s).collect();
                    return filtered_names
*/

async fn autocomplete_hint_type<'a>(
    ctx: Context<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    let types: Vec<String> = vec!("Season".to_string(), "Year".to_string(), "Format".to_string(), "Genre".to_string(), "Studio".to_string(),
                                 "Voice Actor".to_string(), "Tag".to_string(), "Staff".to_string(), "AL Score".to_string(), "Source".to_string(), "User Score".to_string());
    let mut similarity_tuples: Vec<(String, f64)> = types
    .iter()
    .map(|s| (s.clone(), jaro_winkler(&partial, s)))
    .collect();
    similarity_tuples.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let sorted: Vec<String> = similarity_tuples.into_iter().map(|(s, _)| s).collect();
    futures::stream::iter(sorted)
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

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn hint(
    ctx: Context<'_>,
    #[description = "Gives you a hint on the anime"] 
    mut number_of_hints: u64,
    #[description = "Select what type of hint you want"]
    #[autocomplete = "autocomplete_hint_type"]
    hint_type: Option<String>,
) -> Result<(), Error> {
    if number_of_hints > 10 {
        number_of_hints = 1;
    }
    let channel_id = ctx.channel_id().get();
    let channel_check = database::get_anime_id_by_channel_id(ctx.channel_id().get()).await;
    match channel_check {
        Ok(anime_id) => {
        let mut disp_hints: Vec<String> = Vec::new();
            match database::get_hints(channel_id).await {
                Ok((mut rem_hints, mut cur_hints)) => {
                    let mut hint = anime_guessing_game::process_hint(&mut rem_hints, hint_type.clone(), number_of_hints);
                    cur_hints.append(&mut hint);
                    let _set_response = database::set_hints(channel_id, rem_hints, &cur_hints).await;
                    disp_hints = cur_hints;
                },
                Err(_) => {
                    ctx.say("An error occured!").await?;
                    return Ok(());
                },
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
                        let win_message = format!("You guessed right! <:happy:1210688855011762196> The anime was: https://anilist.co/anime/{}", anime_id);
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

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn display_teams(
    ctx: Context<'_>
) -> Result<(), Error> {
    let members = database::get_teams().await;
    match members {
        Ok(res) => {
            let mut message: Vec<String> = Vec::new();
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