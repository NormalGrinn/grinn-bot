use std::time::Duration;

use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{graphql_queries, types};

#[derive(Deserialize, Debug)]
struct Response {
    data: Data,
}

#[derive(Deserialize, Debug)]
struct Data {
    MediaListCollection: MediaListCollection,
}

#[derive(Serialize, Deserialize, Debug)]
struct MediaListCollection {
    pub lists: Vec<MediaList>,
}

#[derive(Serialize, Deserialize, Debug)]
struct MediaList {
    pub entries: Vec<Entry>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Entry {
    score: Option<u64>,
    media: Media,
}

#[derive(Serialize, Deserialize, Debug)]
struct Media {
    id: u64,
}

pub async fn get_anime_list(username: &String) -> Vec<types::AnimeScored> {
    println!("{}", username);
    let client = Client::new();
    let json = json! (
        {
            "query": graphql_queries::USERANIMELISTQUERY,
            "variables": {
                "userName": username,
            }
        }
    );

    let mut retries = 0;
    let max_retries = 60;

    loop {
        let resp = client.post("https://graphql.anilist.co/")
                .header("Content-Type", "application/json")
                .header("Accept", "application/json")
                .body(json.to_string())
                .send()
                .await;
        match resp {
            Ok(response) => {
                if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                    println!("Rate limit exceeded (weh). Retrying...");
                    if retries >= max_retries {
                        println!("Max retries reached. Exiting...");
                        return Vec::new();
                    }
                    retries += 1;
                    tokio::time::sleep(Duration::from_secs(2 * retries)).await;
                    continue;
                }
                let result: Response;
                // let result: Response = serde_json::from_str(&response.text().await.expect("Error parsing message")).expect("Error parsing Json to string");
                let res: Result<Response, serde_json::Error> = serde_json::from_str(&response.text().await.expect("Error parsing message"));
                match res {
                    Ok(r) => result = r,
                    Err(_) => {
                        println!("Error fetching list, may be private");
                        return Vec::new();
                    },
                }
                let mut anime: Vec<types::AnimeScored> = Vec::new();
                if result.data.MediaListCollection.lists.is_empty() {
                    return Vec::new();
                }
                let list = &result.data.MediaListCollection.lists[0];
                for e in &list.entries {
                    match e.score {
                        Some(s) => {
                            let scored_anime = types::AnimeScored {
                                id: e.media.id,
                                score: s,
                            };
                            anime.push(scored_anime);
                        },
                        None => continue,
                        }
                    }
                return anime   
            },
            Err(_) => {
                println!("Error with the request.");
                return Vec::new();
            },
        }
    }
}