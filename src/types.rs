use std::cmp::Ordering;

use serde::{Serialize, Deserialize};
use rusqlite::types::{FromSql, ToSql, ToSqlOutput};
use strum_macros::{EnumString, Display};
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

#[derive(Debug, Serialize, Clone)]
pub struct SubmittedAnime {
    pub(crate) anime_id: u64,
    pub(crate) anime_name: String,
    pub(crate) submitter_name: String,
    pub(crate) claimed_by_team: Option<String>,
    pub(crate) claimed_on: Option<String>,
}

#[derive(Debug)]
pub struct UserList {
    pub(crate) user_id: u64,
    pub(crate) user_name: String,
    pub(crate) user_score_type: ScoreType,
    pub(crate) anime: Vec<UserAnimeInfo>
}

#[derive(Debug)]
pub enum ScoreType {
    POINT_100,
    POINT_10_DECIMAL,
    POINT_10,
    POINT_5,
    POINT_3,
}

#[derive(Debug)]
pub enum AnimeStatus {
    CURRENT,
    PLANNING,
    COMPLETED,
    DROPPED,
    PAUSED,
    REPEATING,
}

#[derive(Debug)]
pub struct ListEntry {
    pub user_id: u64,
    pub anime_id: u64,
    pub anime_score: f64,
    pub is_favourite: bool,
    pub notes: Option<String>,
    pub rewatches: i64,
    pub completion_status: String,
    pub anime_names: Title,
    pub user_name: String,
    pub user_score_type: ScoreType,
}

impl ListEntry {
    fn normalize_score(&self) -> f64 {
        match self.user_score_type {
            ScoreType::POINT_100 => self.anime_score,
            ScoreType::POINT_10_DECIMAL => self.anime_score * 10.0,
            ScoreType::POINT_10 => self.anime_score * 10.0,
            ScoreType::POINT_5 => self.anime_score * 20.0,
            ScoreType::POINT_3 => self.anime_score * 33.33,
        }
    }
}

impl PartialOrd for ListEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.normalize_score().partial_cmp(&other.normalize_score())
    }
}

impl Ord for ListEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

impl PartialEq for ListEntry {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for ListEntry {}

#[derive(Debug)]
pub struct UserAnimeInfo {
    pub(crate) anime_id :u64,
    pub(crate) titles: Title,
    pub(crate) score: f64,
    pub(crate) favourite: bool,
    pub(crate) notes: Option<String>,
    pub(crate) status: AnimeStatus,
    pub(crate) repeats: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Title {
    pub(crate) romaji: Option<String>,
    pub(crate) native: Option<String>,
    pub(crate) english: Option<String>,
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

#[derive(EnumString, Display, Debug)]
pub enum SimilarityMeasure {
    CosineSim,
    MeanAbsoluteDifferenceNorm,
    MeanAbsoluteDifference,
}

#[derive(Debug)]
pub struct AnimeScored {
    pub(crate) id: u64,
    pub(crate) score: u64,
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

impl ToSql for ScoreType {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let score_type_str = match self {
            ScoreType::POINT_100 => "POINT_100",
            ScoreType::POINT_10_DECIMAL => "POINT_10_DECIMAL",
            ScoreType::POINT_10 => "POINT_10",
            ScoreType::POINT_5 => "POINT_5",
            ScoreType::POINT_3 => "POINT_3",
        };
        Ok(ToSqlOutput::from(score_type_str))
    }
}

impl ToSql for AnimeStatus {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let status_str = match self {
            AnimeStatus::CURRENT => "CURRENT",
            AnimeStatus::PLANNING => "PLANNING",
            AnimeStatus::COMPLETED => "COMPLETED",
            AnimeStatus::DROPPED => "DROPPED",
            AnimeStatus::PAUSED => "PAUSED",
            AnimeStatus::REPEATING => "REPEATING",
        };
        Ok(ToSqlOutput::from(status_str))
    }
}

impl ToSql for Title {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let json_string = serde_json::to_string(self)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
        Ok(ToSqlOutput::from(json_string))
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

impl FromSql for ScoreType {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let score_type_str = value.as_str()?;
        match score_type_str {
            "POINT_100" => Ok(ScoreType::POINT_100),
            "POINT_10_DECIMAL" => Ok(ScoreType::POINT_10_DECIMAL),
            "POINT_10" => Ok(ScoreType::POINT_10),
            "POINT_5" => Ok(ScoreType::POINT_5),
            "POINT_3" => Ok(ScoreType::POINT_3),
            _ => Err(rusqlite::types::FromSqlError::InvalidType)
        }
    }
}