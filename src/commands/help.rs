use crate::{Context, Error};
use rusqlite::Result;
use serenity::futures;
use strsim::jaro_winkler;
use poise::serenity_prelude as serenity;

use crate::database;
use crate::anime_guessing_game;
use crate::helpers;

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