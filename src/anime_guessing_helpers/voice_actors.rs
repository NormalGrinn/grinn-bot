use reqwest::Client;
use serde_json::json;
use serde::Deserialize;

use crate::graphql_queries;

#[derive(Debug, Deserialize)]
pub struct Response {
    pub data: Data,
}

#[derive(Debug, Deserialize)]
pub struct Data {
    pub Media: Media,
}

#[derive(Debug, Deserialize)]
pub struct Media {
    pub characters: Characters,
}

#[derive(Debug, Deserialize)]
pub struct Characters {
    pub edges: Vec<Edge>,
}

#[derive(Debug, Deserialize)]
pub struct Edge {
    pub node: Node,
    pub voiceActors: Vec<VoiceActor>,
}

#[derive(Debug, Deserialize)]
pub struct Node {
    pub id: u64,
}

#[derive(Debug, Deserialize)]
pub struct VoiceActor {
    pub name: Name,
}

#[derive(Debug, Deserialize)]
pub struct Name {
    pub full: String,
}

pub async fn get_voice_actors(anime_id: u64) -> Vec<String> {
    let mut voice_actors: Vec<String> = Vec::new(); 
    let client = Client::new();
    let json = json! (
        {
            "query": graphql_queries::MAINVAQUERY,
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
                    if !result.data.Media.characters.edges.is_empty() {
                        for s in result.data.Media.characters.edges.iter() {
                            if !s.voiceActors.is_empty() {
                                voice_actors.push(s.voiceActors[0].name.full.clone());
                            }
                        }
                    }
    voice_actors
}