use std::{env, fmt::format};
use reqwest::Client;
use serde_json::json;
use serde::Deserialize;
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};
use tokio::time::{sleep, Duration};
use rand::Rng;
use strsim::damerau_levenshtein;

use crate::{graphql_queries, types::AnimeGuess};
use crate::types;
use crate::helpers;

#[derive(Deserialize, Debug)]
struct Response {
    data: Data,
}

#[derive(Deserialize, Debug)]
struct Data {
    MediaListCollection: MediaListCollection,
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
    score: u64,
    media: Media,
}

#[derive(Deserialize, Debug)]
struct Title {
    romaji: Option<String>,
    english: Option<String>,
    native: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Media {
    id: u64,
    season: Option<String>,
    seasonYear: Option<u64>,
    format: Option<String>,
    genres: Vec<String>,
    tags: Vec<types::Tag>,
    averageScore: Option<u64>,
    source: Option<String>,
    title: Title,
    synonyms: Vec<String>,
}

fn generate_hint(entry_info: &Entry) -> types::AnimeGuess {
    let mut hints: Vec<types::Hint> = Vec::new();
    if let Some(season_year) = entry_info.media.seasonYear {
        hints.push(types::Hint::SeasonYear(season_year));
    }
    hints.push(types::Hint::UserScore(entry_info.score));
    if !entry_info.media.genres.is_empty() {
        hints.push(types::Hint::Genres(entry_info.media.genres.clone()));
    }
    if !entry_info.media.tags.is_empty() {
        hints.push(types::Hint::Tag(entry_info.media.tags.clone()));
    }
    if let Some(ref season) = entry_info.media.season {
        hints.push(types::Hint::Season(season.clone()));
    }
    if let Some(ref format) = entry_info.media.format {
        hints.push(types::Hint::Format(format.clone()));
    }
    if let Some(average_score) = entry_info.media.averageScore {
        hints.push(types::Hint::AverageScore(average_score));
    }
    if let Some(ref source) = entry_info.media.source {
        hints.push(types::Hint::Source(source.clone()));
    }
    let mut titles: Vec<String> = Vec::new();
    match entry_info.media.title.romaji {
        Some(ref r) => titles.push(r.clone()),
        None => (),
    }
    match entry_info.media.title.english {
        Some(ref r) => titles.push(r.clone()),
        None => (),
    }
    
    match entry_info.media.title.native {
        Some(ref r) => titles.push(r.clone()),
        None => (),
    }
    for e in entry_info.media.synonyms.iter() {
        titles.push(e.to_string());
    }
    let anime_guess = types::AnimeGuess {
        id: entry_info.media.id,
        synonyms: titles,
        hints: hints,
    };
    anime_guess
}

pub async fn anime_guessing_setup(userName: &str) -> AnimeGuess {
    let client = Client::new();
    let json = json!(
        {
            "query": graphql_queries::USERLISTGUESSINGQUERY,
            "variables": {
                "userName": userName,
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
    let mut rng = rand::thread_rng();
    let chosen_entry: usize = rng.gen_range(0..result.data.MediaListCollection.lists[0].entries.len());
    let anime_hints = generate_hint(&result.data.MediaListCollection.lists[0].entries[chosen_entry]);
    anime_hints
}

// Process the guess via a damerau_levenshtein similairty.
// It goes trough all of the titles and synonyms of an anime
pub async fn process_guess(guess: &str, titles: &Vec<String>) -> bool {
    for e in titles.iter() {
        if damerau_levenshtein(&e.to_lowercase(), &guess.to_lowercase()) <= 3 {
            return true;
        }
    }
    false
}

// The ranks are:
// 0: unscored
// 0 < x < 20 very low 
// 20 < x =< 50 low
// 50 < x =< 65 middling 
// 65 < x =< 75 high
// 75 < x =< 90 very high
// 90 < x =< 100 extremly high
fn rank_weight(number: u64) -> String {
    if number == 0 {
        "unscored".to_string()
    } else if number > 0 && number < 20 {
        "very low".to_string()
    } else if number >= 20 && number <= 50 {
        "low".to_string()
    } else if number > 50 && number <= 65 {
        "middling".to_string()
    } else if number > 65 && number <= 75 {
        "high".to_string()
    } else if number > 75 && number <= 90 {
        "very high".to_string()
    } else if number > 90 && number <= 100 {
        "extremely high".to_string()
    } else {
        "out of range".to_string() // Optional: to handle cases where number > 100
    }
}

pub fn process_hint(remaining_hints: &mut Vec<types::Hint>) -> String {
    let potential_hint = helpers::get_random_element_from_vec(remaining_hints);
    let hint: String;
    match potential_hint {
        None => hint = format!("No hints are left!"),
        Some(chosen_hint) =>     
        match chosen_hint {
            types::Hint::SeasonYear(x) => hint = format!("This anime started airing in the year {}", x),
            types::Hint::UserScore(x) => { hint = format!("You gave this anime a {} score", rank_weight(x))},
            types::Hint::AverageScore(x) => hint = format!("On AL this anime has a {} average score", rank_weight(x)),
            types::Hint::Format(s) => hint = format!("The format of this anime is: {}", s),
            types::Hint::Season(s) => hint = format!("This anime aired in the {} season", s),
            types::Hint::Source(s) => hint = format!("The source of this anime is: {}", s),
            types::Hint::Genres(mut vs) => {
                let potential_genre = helpers::get_random_element_from_vec(&mut vs);
                match potential_genre {
                    None => hint = format!("weh"),
                    Some(genre) => hint = format!("{} is one of this anime's genres", genre),
                }
                if !vs.is_empty() { remaining_hints.push(types::Hint::Genres(vs)); }
            },
            types::Hint::Tag(mut vt) => {
                let potential_tag = helpers::get_random_element_from_vec(&mut vt);
                match potential_tag {
                    None => hint = format!("weh"),
                    Some(tag) => hint = format!("{} is one of this anime's tags and it has a {} rating", tag.name, rank_weight(tag.rank)),
                }
                if !vt.is_empty() { remaining_hints.push(types::Hint::Tag(vt)); }
            }
        }
    }
    hint
}