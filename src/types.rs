use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Tag {
    pub(crate) name: String,
    pub(crate) rank: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Staff {
    pub(crate) name: String,
    pub(crate) role: String,
}

#[derive(Debug)]
pub enum Hint {
    Season(String),
    SeasonYear(u64),
    Format(String),
    Genres(Vec<String>),
    Studios(Vec<String>),
    VoiceActors(Vec<String>),
    Tag(Vec<Tag>),
    Staff(Vec<Staff>),
    AverageScore(u64),
    Source(String),
    UserScore(u64),
}

#[derive(Debug)]
pub struct AnimeGuess {
    pub(crate) id: u64,
    pub(crate) synonyms: Vec<String>,
    pub(crate) hints: Vec<Hint>, 
}