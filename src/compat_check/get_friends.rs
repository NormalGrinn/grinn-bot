use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::graphql_queries;

#[derive(Deserialize, Debug)]
struct Response {
    data: Data,
}

#[derive(Deserialize, Debug)]
struct Data {
    Page: Page,
}

#[derive(Deserialize, Debug)]
struct Page {
    following: Vec<Follower>,
}

#[derive(Deserialize, Debug)]
struct Follower {
    id: u64,
    name: String,
}

async fn get_friend_page(id: u64, page: u64, client: Client) -> Vec<String> {
    let json = json! (
        {
            "query": graphql_queries::FOLLOWINGQUERY,
            "variables": {
                "userId": id,
                "page": page,
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
                        .await
                        .expect("Expected JSON");
    let result: Response = serde_json::from_str(&resp).expect("Error parsing JSON");
    let mut follwing_names: Vec<String> = Vec::new();
    for follower in result.data.Page.following {
        follwing_names.push(follower.name);
    }
    follwing_names
}

pub async fn get_friends_names(username: &String) -> Vec<String> {
    let client = Client::new();
    let json = json! (
        {
            "query": graphql_queries::USERIDQUERY,
            "variables": {
                "userName": username,
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
                        .await
                        .expect("Expected JSON");
    let parsed_resp: Value = serde_json::from_str(&resp).expect("Error parsing JSON");
    let id: u64 = parsed_resp["data"]["User"]["id"].as_u64().expect("Expected non signed int");
    let mut friends: Vec<String> = Vec::new();
    let mut i: u64 = 1;
    loop {
        let mut page = get_friend_page(id, i, client.clone()).await;
        i += 1;
        if page.len() < 50 {
            friends.append(&mut page);
            break;
        }
        friends.append(&mut page);
    }
    friends
}