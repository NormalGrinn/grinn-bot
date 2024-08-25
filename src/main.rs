use std::env;
use reqwest::Client as ReqestClient;
use serde_json::json;
use serde::Deserialize;
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};
use tokio::time::{sleep, Duration};

pub mod graphql_queries;

struct Handler;

#[derive(Deserialize, Debug)]
struct MediaTitle {
    romaji: String,
}

#[derive(Deserialize, Debug)]
struct Media {
    title: MediaTitle,
    episodes: Option<u64>,
    duration: Option<u64>,
}

#[derive(Deserialize, Debug)]
struct Page {
    media: Vec<Media>,
}

#[derive(Deserialize, Debug)]
struct Data {
    Page: Page,
}

#[derive(Deserialize, Debug)]
struct Root {
    data: Data,
}

#[derive(Debug, PartialEq)]
struct AnimeLength {
    anime_name: String,
    anime_episode_count: Option<u64>,
    anime_episode_length: Option<u64>,
}

impl From<Media> for AnimeLength {
    fn from(media: Media) -> Self {
        AnimeLength {
            anime_name: media.title.romaji,
            anime_episode_count: media.episodes,
            anime_episode_length: media.duration,
        }
    }
}

fn count_minutes_of_anime(anime: &AnimeLength) -> u64 {
    let episode_count = match anime.anime_episode_count {
        Some(count) => count, 
        None => 0,           
    };
    let episode_length = match anime.anime_episode_length {
        Some(length) => length, 
        None => 0,              
    };

    episode_count * episode_length
}


async fn anime_in_year(msg_content: &Vec<String>) -> String {
    let season_year = &msg_content[1];
    let mut anime_in_year: Vec<AnimeLength> = Vec::new();
    let mut page_number: i64 = 1;
    let client = ReqestClient::new();
    loop {
        sleep(Duration::from_millis(667)).await;
        let json = json!(
            {
                "query": graphql_queries::YEARQUERY,
                "variables": {
                    "seasonYear": season_year,
                    "page": page_number,
                }
            }
        );
        let resp = client.post("https://graphql.anilist.co/")
                    .header("Content-Type", "application/json")
                    .header("Accept", "application/json")
                    .body(json.to_string())
                    .send()
                    .await
                    .unwrap()
                    .text()
                    .await;
        // TODO: HANDLE RATE LIMITING
        let result: Root = serde_json::from_str(&resp.unwrap()).unwrap();
        let mut anime_lengths: Vec<AnimeLength> = result
        .data
        .Page
        .media
        .into_iter()
        .map(AnimeLength::from)
        .collect();
        if anime_lengths.len() == 0 {
            break;
        }
        anime_in_year.append(&mut anime_lengths);
        page_number += 1;
    }
    // Get the minute count of each anime, get the total minutes of anime in a year, then convert that into hours and minutes
    let anime_durations: Vec<u64> = anime_in_year.iter().map(count_minutes_of_anime).collect();
    let year_duration = anime_durations.iter().fold(0, |acc, x| acc + x);
    let year_hours = year_duration/60;
    let year_minutes = year_duration%60;
    let result_message = format!("The year {} has {} hours and {} minutes of anime!", season_year, year_hours, year_minutes);
    result_message
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let split = msg.content.split(" ");
        let msg_content: Vec<String> = split.map(String::from).collect();
        if msg.content == "!test" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "sugoi").await {
                println!("Error sending message: {:?}", why);
            }
        }
        if msg_content[0] == "!yearDuration" {
        let result_message: String = anime_in_year(&msg_content).await;
        println!("{}", result_message);
            if let Err(why) = msg.channel_id.say(&ctx.http, result_message).await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
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