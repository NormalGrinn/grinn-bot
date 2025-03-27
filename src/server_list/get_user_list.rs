use std::collections::HashSet;

use crate::{graphql_queries, types};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use serenity::{all::EventHandler, async_trait};

use super::get_favs;

#[derive(Deserialize, Debug)]
struct Response {
    data: Data,
}

#[derive(Deserialize, Debug)]
struct Data {
    MediaListCollection: MediaListCollection,
    User: User,
}

#[derive(Deserialize, Debug)]
struct MediaListCollection {
    lists: Vec<List>,
}

#[derive(Deserialize, Debug)]
struct List {
    entries: Vec<Entry>,
}

#[derive(Deserialize, Debug)]
struct Entry {
    score: f64,
    media: Media,
    notes: Option<String>,
    repeat: u64,
    status: String,
}

#[derive(Deserialize, Debug)]
struct Media {
    id: u64,
    title: MediaTitle,
}

#[derive(Deserialize, Debug)]
struct MediaTitle {
    romaji: Option<String>,
    native: Option<String>,
    english: Option<String>
}

#[derive(Deserialize, Debug)]
struct User {
    id: u64,
    mediaListOptions: MediaListOptions,
}

#[derive(Deserialize, Debug)]
struct MediaListOptions {
    scoreFormat: String,
}

fn remove_duplicate_ids(input: Vec<Entry>) -> Vec<Entry> {
    let mut seen = HashSet::new();
    let mut unique = Vec::new();

    for anime in input {
        if seen.insert(anime.media.id) {
            unique.push(anime);
        }
    }
    unique
}

pub async fn get_user_list(username: &str) -> types::UserList {
    let client = Client::new();
    let json = json! (
        {
            "query": graphql_queries::USERLISTINFOQUERY,
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
        .await;
    let result: Response = serde_json::from_str(&resp.unwrap()).unwrap();
    let mut list: Vec<Entry> = Vec::new();
    for mut l in result.data.MediaListCollection.lists {
        list.append(&mut l.entries);
    }
    let uniques = remove_duplicate_ids(list);

    let mut info_list: Vec<types::UserAnimeInfo> = Vec::new();
    for anime in uniques {
        let title = types::Title {
            romaji: anime.media.title.romaji,
            native: anime.media.title.native,
            english: anime.media.title.english,
        };
        let anime_status: types::AnimeStatus;
        match anime.status.as_str() {
            "CURRENT" => anime_status = types::AnimeStatus::CURRENT,
            "PLANNING" => anime_status = types::AnimeStatus::PLANNING,
            "COMPLETED" => anime_status = types::AnimeStatus::COMPLETED,
            "DROPPED" => anime_status = types::AnimeStatus::DROPPED,
            "PAUSED" => anime_status = types::AnimeStatus::PAUSED,
            "REPEATING" => anime_status = types::AnimeStatus::REPEATING,
            _ => continue
        }
        let new_entry = types::UserAnimeInfo {
            anime_id: anime.media.id,
            titles: title,
            score: anime.score,
            favourite: false,
            notes: anime.notes,
            status: anime_status,
            repeats: anime.repeat,
        };
        info_list.push(new_entry);
    }
    let id_nodes: Vec<u64> = get_favs::get_favs(username).await;
    for entry in &mut info_list {
        if id_nodes.contains(&entry.anime_id) {entry.favourite = true;}
    }
    let score_format : types::ScoreType;
    match result.data.User.mediaListOptions.scoreFormat.as_str() {
        "POINT_100" => score_format = types::ScoreType::POINT_100,
        "POINT_10_DECIMAL" => score_format = types::ScoreType::POINT_10_DECIMAL,
        "POINT_10" => score_format = types::ScoreType::POINT_10,
        "POINT_5" => score_format = types::ScoreType::POINT_5,
        "POINT_3" => score_format = types::ScoreType::POINT_3,
        _ => score_format = types::ScoreType::POINT_10,
    }
    let result = types::UserList {
        user_id: result.data.User.id,
        user_name: username.to_owned(),
        user_score_type: score_format,
        anime: info_list,
    };
    result
}