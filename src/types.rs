use serde::{Serialize, Deserialize};
use rusqlite::types::{FromSql, ToSql, ToSqlOutput};
/**
 * Struct for describing an AL tag
 * name: the name of this tag
 * rank: the rating this score has on AL
 */
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tag {
    pub(crate) name: String,
    pub(crate) rank: u64,
}

/**
 * Struct for describing a staff member
 * name: the full romaji name of this staff member
 * role: the role they have on this show
 */
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Staff {
    pub(crate) name: String,
    pub(crate) role: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Team {
    pub(crate) team_id: u64,
    pub(crate) team_image_url: Option<String>,
    pub(crate) team_name: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct Member{
    pub(crate) member_id: u64,
    pub(crate) member_name: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct TeamMembers {
    pub(crate) team: Team,
    pub(crate) members: Vec<(Member)>,
}

#[derive(Debug, Serialize, Clone)]
pub struct SubmittedAnime {
    pub(crate) anime_id: u64,
    pub(crate) anime_name: String,
    pub(crate) submitter_name: String,
    pub(crate) claimed_by_team: Option<String>,
    pub(crate) claimed_on: Option<String>,
}

/**
 * Enum for describing the various types a hint can take on
 * Season: the airing season
 * SeasonYear: the year it started airing
 * Format: the format it was aired in (e.g. TV, OVA, ONA, etc)
 * Genres: a vector containing all the genres in string from
 * Studios: a vector of all the primary studios in string form
 * Voice Actors: a vector of all the VAs who voiced a main character
 * Tag: a vector of tags
 * Staff: a vectoring containing staff that worked on the show (who are not VAs)
 * AverageScore: the average score as listed on AL
 * Source: the source material of the show
 * UserScore: the score this user gave this show
 */
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

// Wrapper used for serialzing a Hint vector for SQL queries
#[derive(Serialize, Deserialize, Debug)]
pub struct HintWrapper {
    pub(crate) hints: Vec<Hint>,
}

// Wrapper used for serializng a String vector for SQL queries
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