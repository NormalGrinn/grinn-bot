use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

use crate::graphql_queries;


#[derive(Debug, Deserialize)]
struct Media {
    id: u32,
    id_mal: u32,
}

#[derive(Debug, Deserialize)]
struct Entry {
    score: u32,
    media: Media,
}

#[derive(Debug, Deserialize)]
struct MediaList {
    entries: Vec<Entry>,
}

#[derive(Debug, Deserialize)]
struct MediaListOptions {
    scoreFormat: String,
}

#[derive(Debug, Deserialize)]
struct User {
    mediaListOptions: MediaListOptions,
}

#[derive(Debug, Deserialize)]
struct MediaListCollection {
    lists: Vec<MediaList>,
    user: User,
}

#[derive(Debug, Deserialize)]
struct Data {
    mediaListCollection: MediaListCollection,
}

#[derive(Debug, Deserialize)]
struct Response {
    data: Data,
}


pub async fn add_user(user_name: &str) {
    let client = Client::new();
    let json = json!(
        {
            "query": graphql_queries::USERANIMELISTQUERY,
            "variables": {
                "userName": user_name,
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
}