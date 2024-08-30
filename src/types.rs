use serde::{Serialize, Deserialize};
use rusqlite::types::{FromSql, ToSql, ToSqlOutput, Value};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tag {
    pub(crate) name: String,
    pub(crate) rank: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Staff {
    pub(crate) name: String,
    pub(crate) role: String,
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
pub struct HintWrapper {
    pub(crate) hints: Vec<Hint>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StringVecWrapper {
    pub(crate) stringvec: Vec<String>,
}

impl ToSql for HintWrapper {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        match  serde_json::to_string(self) {
            Ok(serialized) => Ok(ToSqlOutput::from(serialized)),
            Err(e) => Err(rusqlite::Error::ToSqlConversionFailure(Box::new(e))),
        }
    }
}

impl ToSql for StringVecWrapper {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        match  serde_json::to_string(self) {
            Ok(serialized) => Ok(ToSqlOutput::from(serialized)),
            Err(e) => Err(rusqlite::Error::ToSqlConversionFailure(Box::new(e))),
        }
    }
}

impl From<&str> for HintWrapper {
    fn from(s: &str) -> Self {
        // Deserialize the JSON string into HintWrapper
        match serde_json::from_str(s) {
            Ok(hint_wrapper) => hint_wrapper,
            Err(_) => HintWrapper { hints: Vec::new() },
        }
    }
}

impl From<&str> for StringVecWrapper {
    fn from(s: &str) -> Self {
        // Deserialize the JSON string into StringVecWrapper
        match serde_json::from_str(s) {
            Ok(stringvec_wrapper) => stringvec_wrapper,
            Err(_) => StringVecWrapper { stringvec: Vec::new() },
        }
    }
}

impl FromSql for HintWrapper {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        value.as_str().map(Into::into)
    }
}

impl FromSql for StringVecWrapper {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        value.as_str().map(Into::into)
    }
}