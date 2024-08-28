use reqwest::Client;
use serde_json::json;
use serde::Deserialize;

use crate::graphql_queries;
use crate::types;

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
    pub staff: Staff,
}

#[derive(Debug, Deserialize)]
pub struct Staff {
    pub edges: Vec<Edge>,
}

#[derive(Debug, Deserialize)]
pub struct Edge {
    pub node: Node,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct Node {
    pub name: Name,
}

#[derive(Debug, Deserialize)]
pub struct Name {
    pub full: String,
}

pub async fn get_staff(anime_id: u64) -> Vec<types::Staff> {
    let mut staff: Vec<types::Staff> = Vec::new(); 
    let client = Client::new();
    let json = json! (
        {
            "query": graphql_queries::STAFFQUERY,
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
    let num_of_staff = 5;
    if !result.data.Media.staff.edges.is_empty() {
        for s in result.data.Media.staff.edges.iter() {
            if staff.len() < num_of_staff {
                let added_staff = types::Staff {
                    name: s.node.name.full.clone(),
                    role: s.role.clone(),
                };
                staff.push(added_staff);
            } else {
                break;
            }
        }
    }
    staff
}