use reqwest::Client;
use serde_json::json;
use serde::Deserialize;

use crate::graphql_queries;

#[derive(Debug, Deserialize)]
struct Response {
    data: Data,
}

#[derive(Debug, Deserialize)]
struct Data {
    Media: Media,
}

#[derive(Debug, Deserialize)]
struct Media {
    studios: Studios,
}

#[derive(Debug, Deserialize)]
struct Studios {
    edges: Vec<Edge>,
}

#[derive(Debug, Deserialize)]
struct Edge {
    node: Node,
    isMain: bool,
}

#[derive(Debug, Deserialize)]
struct Node {
    name: String,
}


pub async fn get_studios(anime_id: u64) -> Vec<String> {
    let mut studios: Vec<String> = Vec::new(); 
    let client = Client::new();
    let json = json! (
        {
            "query": graphql_queries::STUDIOQUERY,
            "variables": {
                "animeID": anime_id,
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
    if !result.data.Media.studios.edges.is_empty() {
        for s in result.data.Media.studios.edges.iter() {
            if s.isMain {
                studios.push(s.node.name.clone());
            }
        }
    }
    studios
}