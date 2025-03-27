use core::time;
use std::thread;

use serde::{Deserialize};
use reqwest::Client;
use serde_json::{json, Value};

use crate::graphql_queries;

#[derive(Debug, Deserialize)]
struct PageInfo {
    hasNextPage: bool,
}

#[derive(Debug, Deserialize)]
struct Node {
    id: u64,
}

#[derive(Debug, Deserialize)]
struct Favourites {
    anime: AnimeData,
}

#[derive(Debug, Deserialize)]
struct AnimeData {
    nodes: Vec<Node>,
    pageInfo: PageInfo,
}

#[derive(Debug, Deserialize)]
struct User {
    favourites: Favourites,
}

#[derive(Debug, Deserialize)]
struct Data {
    User: User,
}

#[derive(Debug, Deserialize)]
struct Response {
    data: Data,
}

pub async fn get_favs(username: &str) -> Vec<u64> {
    let mut favs_ids: Vec<u64> = Vec::new();
    let mut has_next_page = true;
    let mut page_num = 1;
    let client = Client::new();
    while has_next_page {
        let json = json! (
            {
                "query": graphql_queries::FAVSQUERY,
                "variables": {
                    "userName": username,
                    "page": page_num,
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
        let result: Response = serde_json::from_str(&resp.unwrap()).unwrap();
        has_next_page = result.data.User.favourites.anime.pageInfo.hasNextPage;
        page_num += 1;
        for fav in result.data.User.favourites.anime.nodes {
            favs_ids.push(fav.id);
        }
        thread::sleep(time::Duration::from_millis(50));
    }
    favs_ids
}