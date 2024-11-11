use std::{collections::HashMap, env, sync::{Arc, Mutex}, time::Duration};
use database::start_db;
use poise::serenity_prelude as serenity;
use dotenvy::dotenv;
use warp::Filter;

mod graphql_queries;
mod anime_guessing_game;
mod types;
mod helpers;
mod group_scores;
mod database;
mod anime_guessing_helpers; 
mod commands;
mod team_swapping;
mod api_routes;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data {
    votes: Mutex<HashMap<String, u32>>,
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    // This is our custom error handler
    // They are many errors that can occur, so we only handle the ones we want to customize
    // and forward the rest to the default handler
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e)
            }
        }
    }
}

#[tokio::main]
async fn main() {
    //env_logger::init();
    // FrameworkOptions contains all of poise's configuration option in one struct
    // Every option can be omitted to use its default value
    dotenv().ok();

    match env::var("SWAPPER_ROLE_ID") {
        Ok(id) => println!("Swapper role ID: {}", id),
        Err(_) => println!("Swapper role ID not properly set"),
    }
    match env::var("HOST_ROLE_ID") {
        Ok(id) => println!("Host role ID: {}", id),
        Err(_) => println!("Host role ID not properly set"),
    }
    match env::var("GUILD_ID") {
        Ok(id) => println!("Guild ID: {}", id),
        Err(_) => println!("Guild ID not properly set"),
    }
    match env::var("PHASE") {
        Ok(id) => println!("PHASE: {}", id),
        Err(_) => println!("Guild ID not properly set"),
    }

    let options = poise::FrameworkOptions {
        commands: vec![
                    commands::help::help(), 
                    commands::anime_guessing::animeguess::animeguess(),
                    commands::anime_guessing::hint::hint(),
                    commands::anime_guessing::guess::guess(),
                    commands::anime_guessing::giveup::giveup(), 
                    commands::teamswap::create_team::create_team(),
                    commands::teamswap::submit_anime::submit_anime(),
                    commands::teamswap::remove_submission::remove_submission(),
                    commands::teamswap::claim::claim(),
                    commands::teamswap::delete_team::delete_team(),
                    commands::teamswap::edit_team_name::edit_team_name(),
                    commands::teamswap::edit_team_image::edit_team_image(),
                    commands::teamswap::status::status(),
                    commands::teamswap::unclaim::unclaim(),
                    commands::teamswap::set_phase::set_phase(),
                    commands::teamswap::leave::leave(),
                    commands::teamswap::set_max_claims::set_max_claims(),
                    ],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("~".into()),
            edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
                Duration::from_secs(3600),
            ))),
            additional_prefixes: vec![
                poise::Prefix::Literal("hey bot"),
                poise::Prefix::Literal("hey bot,"),
            ],
            ..Default::default()
        },
        // The global error handler for all error cases that may occur
        on_error: |error| Box::pin(on_error(error)),
        // This code is run before every command
        pre_command: |ctx| {
            Box::pin(async move {
                println!("Executing command {}...", ctx.command().qualified_name);
            })
        },
        // This code is run after a command if it was successful (returned Ok)
        post_command: |ctx| {
            Box::pin(async move {
                println!("Executed command {}!", ctx.command().qualified_name);
            })
        },
        // Every command invocation must pass this check to continue execution
        command_check: Some(|ctx| {
            Box::pin(async move {
                if ctx.author().id == 123456789 {
                    return Ok(false);
                }
                Ok(true)
            })
        }),
        // Enforce command checks even for owners (enforced by default)
        // Set to true to bypass checks, which is useful for testing
        skip_checks_for_owners: false,
        event_handler: |_ctx, event, _framework, _data| {
            Box::pin(async move {
                println!(
                    "Got an event in event handler: {:?}",
                    event.snake_case_name()
                );
                Ok(())
            })
        },
        ..Default::default()
    };
    let bot_task = tokio::spawn(async move {
        let framework = poise::Framework::builder()
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                println!("Logged in as {}", _ready.user.name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    votes: Mutex::new(HashMap::new()),
                })
            })
        })
        .options(options)
        .build();

    let token = env::var("TOKEN")
        .expect("Missing `TOKEN` env var, see README for more information.");
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap()
    });

    let api_task = tokio::spawn(async {
        let anime_route = api_routes::get_anime::get_anime();
        let teams_route = api_routes::get_teams::get_teams();
        let routes = anime_route.or(teams_route);
        warp::serve(routes)
            .run(([127, 0, 0, 1], 3030))
            .await;
    });

    let _ = tokio::join!(bot_task, api_task);
}