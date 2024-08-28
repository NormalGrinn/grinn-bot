use std::{env, fmt::format};
use anime_guessing_game::process_hint;
use reqwest::Client as ReqestClient;
use serde_json::json;
use serde::Deserialize;
use serenity::{
    all::ChannelId, async_trait, model::{channel::Message, gateway::Ready}, prelude::*
};
use tokio::time::{sleep, Duration, timeout};

mod graphql_queries;
mod anime_guessing_game;
mod types;
mod helpers;
mod anime_year_duration;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let split = msg.content.split(" ");
        let msg_content: Vec<String> = split.map(String::from).collect();
        if msg_content[0] == "!yearDuration" {
        let result_message: String = anime_year_duration::anime_in_year(&msg_content).await;
        println!("{}", result_message);
            if let Err(why) = msg.channel_id.say(&ctx.http, result_message).await {
                println!("Error sending message: {:?}", why);
            }
        }
        if msg_content[0] == "!animeGuess" {
            let mut anime_info = anime_guessing_game::anime_guessing_setup(&msg_content[1]).await;
            let mut starting_hint_wrapper = types::AnimeGuess {
                id: anime_info.id,
                hints: vec!(anime_info.hints.remove(0)),
            };
            let starting_hint = anime_guessing_game::process_hint(&mut starting_hint_wrapper);
            let mut hints: Vec<String> = vec!(starting_hint);
            let start_message = format!("The anime guessing game has started for {}'s list!\n{}", &msg_content[1], hints[0]);
            if let Err(why) = msg.channel_id.say(&ctx.http, start_message).await {
                println!("Error sending message: {:?}", why);
            }
            println!("{:?}", anime_info.id);
            loop {
                match timeout(Duration::from_secs(300), read_next_message(&ctx, msg.channel_id)).await {
                    Ok(Some(new_msg)) => {
                        let (msg_head, msg_tail) = helpers::split_first_word(&new_msg.content);
                        if msg_head == "!hint" {
                            let hint_message = anime_guessing_game::process_hint(&mut anime_info);
                            hints.push(hint_message);
                            if let Err(why) = msg.channel_id.say(&ctx.http, helpers::display_str_vec(&hints)).await {
                                println!("Error sending message: {:?}", why);
                            }
                        }
                        if msg_head == "!guess" {
                            let result = anime_guessing_game::process_guess(&msg_tail, anime_info.id).await;
                            if result {
                                let correct_message = format!("You guessed right! The anime was https://anilist.co/anime/{}", anime_info.id);
                                if let Err(why) = msg.channel_id.say(&ctx.http, correct_message).await {
                                    println!("Error sending message: {:?}", why);
                                }
                                break;
                            } else {
                                if let Err(why) = msg.channel_id.say(&ctx.http, "You guessed wrong!").await {
                                    println!("Error sending message: {:?}", why);
                                }
                            }

                        }  
                        if msg_head == "!quit" {
                            if let Err(why) = msg.channel_id.say(&ctx.http, "You've quit the guessing game").await {
                                println!("Error sending message: {:?}", why);
                            }
                            break;
                        }
                        if msg_head == "!giveUp" {
                            let give_up_message = format!("You've given up, the anime was: https://anilist.co/anime/{}", anime_info.id);
                            if let Err(why) = msg.channel_id.say(&ctx.http, give_up_message).await {
                                println!("Error sending message: {:?}", why);
                            }
                            break;
                        } 
                    },
                    Ok(None) => {
                        println!("No message received within the timeout period");
                        if let Err(why) = msg.channel_id.say(&ctx.http, "The bot timed out").await {
                            println!("Error sending message: {:?}", why);
                        }
                        break;
                    },
                    Err(_) => {
                        println!("Timeout occurred, exiting loop");
                        break;
                    }
                }
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

async fn read_next_message(ctx: &Context, channel_id: ChannelId) -> Option<Message> {
    let http = &ctx.http;
    loop {
        match http.get_messages(channel_id, None, Some(1)).await {
            Ok(messages) => {
                if let Some(new_message) = messages.get(0) {
                    return Some(new_message.clone());
                }
            }
            Err(why) => {
                println!("Error fetching messages: {:?}", why);
                return None;
            }
        }
        sleep(Duration::from_millis(250)).await;
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");

    let intents = GatewayIntents::GUILD_MESSAGES
    | GatewayIntents::DIRECT_MESSAGES
    | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}