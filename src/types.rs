use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Tag {
    pub(crate) name: String,
    pub(crate) rank: u64,
}

#[derive(Debug)]
pub enum Hint {
    Season(String),
    SeasonYear(u64),
    Format(String),
    Genres(Vec<String>),
    Tag(Vec<Tag>),
    AverageScore(u64),
    Source(String),
    UserScore(u64),
}

#[derive(Debug)]
pub struct AnimeGuess {
    pub(crate) id: u64,
    pub(crate) hints: Vec<Hint>, 
}