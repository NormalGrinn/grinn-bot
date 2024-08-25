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

use crate::graphql_queries;

#[derive(Deserialize, Debug)]
struct Response {
    data: Data,
}

#[derive(Deserialize, Debug)]
struct Data {
    Media: Media,
}

#[derive(Deserialize, Debug)]
struct Media {
    title: Title,
    season: String,
    seasonYear: u32,
    format: String,
    genres: Vec<String>,
    tags: Vec<Tag>,
    averageScore: u32,
    source: String,
}

#[derive(Deserialize, Debug)]
struct Title {
    romaji: String,
    english: String,
    native: String,
    userPreferred: String,
}

#[derive(Deserialize, Debug)]
struct Tag {
    name: String,
    rank: u32,
}


async fn anime_guessing_setup() {
    
}