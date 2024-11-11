use crate::{team_swapping::team_swap_utils, Context, Error};
use poise::CreateReply;
use rusqlite::Result;
use serenity::futures::{self, Stream};
use strsim::jaro_winkler;
use std::time::SystemTime;
use chrono::{DateTime, FixedOffset, Utc};

use crate::database;

async fn autocomplete_unclaim<'a>(
    ctx: Context<'_>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a {
    let names = database::get_claimed_anime_by_user(ctx.author().id.get()).unwrap();
    let mut scored_names: Vec<(f64, String)> = names
    .iter()
    .map(|name| {
        let score = jaro_winkler(partial, name);
        (score, name.clone())
    })
    .collect();

    scored_names.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

    let sorted_names: Vec<String> = scored_names.into_iter().map(|(_, name)| name).collect();
    futures::stream::iter(sorted_names)
}

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn unclaim(
    ctx: Context<'_>,
    #[description = "The name of the anime you want to unclaim"]
    #[autocomplete = "autocomplete_unclaim"]
    anime_name: String,
) -> Result<(), Error> {
    match team_swap_utils::check_phase(vec![3]) {
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
    let user_id = ctx.author().id.get();
    match team_swap_utils::check_swapper_role(&ctx.author(), &ctx).await {
        Ok(b) => {
            if !b {
                let message = format!("{} does not have the swapper role, and therefore cannot remove submissions", ctx.author().global_name.clone().unwrap());
                ctx.send(CreateReply::default().content(message).ephemeral(true)).await?;
                return  Ok(())
            }
        },
        Err(_) => {
            ctx.send(CreateReply::default().content("An error has occured checking roles").ephemeral(true)).await?;
            return Ok(())
        },
    }
    let user_team_id: u64;
    match database::check_if_user_in_team(user_id) {
        Ok(team_id) => {
            match team_id {
                Some(id) => user_team_id = id,
                None => {
                    ctx.send(CreateReply::default().content("You are not in a team and thus cannot claim an anime").ephemeral(true)).await?;
                    return Ok(());
                },
            }
        },
        Err(_) => {
            ctx.send(CreateReply::default().content("An error has occured checking if the user is already in a team").ephemeral(true)).await?;
            return Ok(())
        },
    }
    let anime_id;
    let member_id;
    match database::get_anime_id_by_name(&anime_name) {
        Ok(id) => {
            match id {
                Some((a_id, m_id)) => {
                    anime_id = a_id;
                    member_id = m_id;
                },
                None => {
                    ctx.send(CreateReply::default().content("This anime was not submitted and thus cannot be claimed").ephemeral(true)).await?;
                    return Ok(());
                },
            }
        },
        Err(_) => {
            ctx.send(CreateReply::default().content("Error checking fetching the anime id").ephemeral(true)).await?;
            return Ok(());
        },
    }
    match database::get_team_and_time_claimed_anime(anime_id) {
        Ok((t_id, time)) => {
            if t_id != user_team_id {
                ctx.send(CreateReply::default().content("You are trying to unclaim an anime not claimed by your team").ephemeral(true)).await?;
                return Ok(())
            }
            let present_time: DateTime<Utc> = Utc::now();
            let claimed_time_fixed_res= DateTime::parse_from_rfc3339(&time).map_err(|e| {
                eprintln!("Erro parsing time: {}", e);
            });
            let claimed_time_fixed: DateTime<FixedOffset>;
            match claimed_time_fixed_res {
                Ok(t) => claimed_time_fixed = t,
                Err(_) => {
                    ctx.send(CreateReply::default().content("Error with parsing time").ephemeral(true)).await?;
                    return Ok(())
                },
            }
            let claimed_time_utc: DateTime<Utc> = claimed_time_fixed.with_timezone(&Utc);
            let duration_since_claimed = present_time.signed_duration_since(claimed_time_utc);
            if duration_since_claimed < chrono::Duration::hours(12) { 
                match database::delete_claim(anime_id) {
                    Ok(_) => {
                        ctx.send(CreateReply::default().content("Successfully unclaimed anime").ephemeral(true)).await?;
                        return Ok(());
                    },
                    Err(_) => {
                        ctx.send(CreateReply::default().content("Error unclaming anime").ephemeral(true)).await?;
                        return Ok(())
                    },
                }
            } else {
                ctx.send(CreateReply::default().content("It's been more than 12 hours, so this anime cannot be unclaimed").ephemeral(true)).await?;
                return Ok(());
            }
        },
        Err(_) => {
            ctx.send(CreateReply::default().content("Error checking fetching the anime info").ephemeral(true)).await?;
            return Ok(())
        },
    }
    todo!()
}