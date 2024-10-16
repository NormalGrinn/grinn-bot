use rusqlite::{Connection, Result};
use crate::types;
use strsim::jaro_winkler;
use std::hash::{DefaultHasher, Hash, Hasher};
use poise::serenity_prelude as serenity;

const ANIME_GUESSING_PATH: &str = "databases/animeGuessing.db";
const TEAM_SWAPPING_PATH: &str = "databases/teamSwaps.db";

const GET_ANIME_GUESSING_ID: &str = "
SELECT anime_guessing.anime_id FROM anime_guessing WHERE channel_id = ?1;
";

const GET_HINTS: &str = "
SELECT hints FROM anime_guessing WHERE channel_id = ?1";

const GET_POTENIAL_HINTS: &str = "
SELECT potential_hints FROM anime_guessing WHERE channel_id = ?1;";

const GET_SYNONYMS: &str = "
SELECT anime_synonyms FROM anime_guessing WHERE channel_id = ?1;";

const GET_NAMES: &str = "
SELECT all_names FROM anime_guessing WHERE channel_id = ?1;";

const INSERT_ANIME_GUESSING: &str = "
INSERT INTO anime_guessing VALUES (?1, ?2, ?3, ?4, ?5, ?6);
";

const DELETE_ANIME_GUESSING: &str = "
    DELETE FROM anime_guessing WHERE channel_id = ?1;
";

const UPDATE_HINTS: &str = "
    UPDATE anime_guessing
    SET potential_hints = ?1, hints = ?2
    WHERE channel_id = ?3;
";

pub fn start_db() -> bool {
    let create_table = "CREATE TABLE anime_guessing(
    channel_id INTEGER PRIMARY KEY,
    anime_id INTEGER,
	potential_hints TEXT,
    hints TEXT,
	anime_synonyms TEXT,
    all_names TEXT
    );";
    let potential_conn = Connection::open(ANIME_GUESSING_PATH);
    let _ = match potential_conn {
        Ok(conn) => {
            conn.execute(&create_table,());
        },
        Err(_) => return false,
    };
    true
}

// Returns the id of the anime currently being guessed in the channel
// If there is no anime that is being guessed in the current channel return 0
pub async fn get_anime_id_by_channel_id(channel_id: u64) -> Result<u64> {
    let conn = Connection::open(ANIME_GUESSING_PATH)?;
    let res: u64 = conn.query_row(GET_ANIME_GUESSING_ID, rusqlite::params![channel_id], |row| row.get(0))?;
    Ok(res)
}

pub async fn give_up(channel_id: u64) -> Result<usize> {
    let conn = Connection::open(ANIME_GUESSING_PATH)?;
    let res = conn.execute(DELETE_ANIME_GUESSING, rusqlite::params![channel_id]);
    return res;
}

pub async fn get_anime_synonyms(channel_id: u64) -> Result<types::StringVecWrapper> {
    let conn = Connection::open(ANIME_GUESSING_PATH)?;
    let res: types::StringVecWrapper = conn.query_row(GET_SYNONYMS, rusqlite::params![channel_id], |row| row.get(0))?;
    Ok(res)
}

//Get the remaining and current hints
pub async fn get_hints(channel_id: u64) -> Result<(Vec<types::Hint>, Vec<String>)> {
    let conn = Connection::open(ANIME_GUESSING_PATH)?;
    let rem_hints: types::HintWrapper = conn.query_row(GET_POTENIAL_HINTS, rusqlite::params![channel_id], |row| row.get(0))?;
    let cur_hints: types::StringVecWrapper = conn.query_row(GET_HINTS, rusqlite::params![channel_id], |row| row.get(0))?;
    Ok((rem_hints.hints, cur_hints.stringvec))
}

//Filters all the names and returns the ones closest to the guess
pub async fn get_filtered_names(partial: &str, channel_id: u64) -> Vec<String> {
    let potentail_conn = Connection::open(ANIME_GUESSING_PATH);
    match potentail_conn {
        Ok(conn) => {
            let potentail_names: Result<types::StringVecWrapper> = conn.query_row(GET_NAMES, rusqlite::params![channel_id], |row| row.get(0));
            match potentail_names {
                Ok(names) => {
                    let mut similarity_tuples: Vec<(String, f64)> = names.stringvec
                    .iter()
                    .map(|s| (s.clone(), jaro_winkler(partial, s)))
                    .collect();
                    similarity_tuples.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                    let filtered_names: Vec<String> = similarity_tuples.into_iter().map(|(s, _)| s).collect();
                    return filtered_names
                    },
                Err(_) => {
                    let nothing: Vec<String> = Vec::new();
                    return nothing
                },
            }
        },
        Err(_) => {
            let nothing: Vec<String> = Vec::new();
            return nothing
        },
    }
}

//Sets the remaining hints and current hints
pub async fn set_hints(channel_id: u64, rem_hints: Vec<types::Hint>, cur_hints: &Vec<String>) -> Result<usize> {
    let conn = Connection::open(ANIME_GUESSING_PATH)?;
    let wrapped_rem_hints = types::HintWrapper { hints: rem_hints };
    let wrapped_cur_hints = types::StringVecWrapper { stringvec: cur_hints.to_vec() };
    let res = conn.execute(UPDATE_HINTS, (wrapped_rem_hints, wrapped_cur_hints, channel_id))?;
    Ok(res)
}

pub async fn set_anime_info(channel_id: u64, entry_info: types::AnimeGuess, gotten_hints:Vec<String>, names: Vec<String>) -> Result<usize> {
    let conn = Connection::open(ANIME_GUESSING_PATH)?;
    let wrapped_hints = types::HintWrapper { hints: entry_info.hints };
    let wrapped_names = types::StringVecWrapper { stringvec: names};
    let wrapped_gotten_hints = types::StringVecWrapper { stringvec: gotten_hints };
    let wrapped_synonyms = types::StringVecWrapper { stringvec: entry_info.synonyms };
    let res = conn.execute(INSERT_ANIME_GUESSING, ( channel_id, &entry_info.id, &wrapped_hints, &wrapped_gotten_hints, 
                                                                        &wrapped_synonyms, &wrapped_names))?;
    Ok(res)
}

pub async fn get_teams() -> Result<Vec<(String, String)>> {
    const GET_TEAMS: &str = "
    SELECT members.name, teams.team_name
    FROM members
    INNER JOIN teams ON members.team = teams.team_id; 
    ";
    let mut members: Vec<(String, String)> = Vec::new();
    let conn = Connection::open(TEAM_SWAPPING_PATH)?;
    let mut team_query = conn.prepare(GET_TEAMS)?;
    let team_iter = team_query.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
        ))
    })?;
    for member in team_iter {
        let (name, team_name) = member?;
        println!("Member: {}, Team: {}", name, team_name);
        members.push((name, team_name))
    }
    Ok(members)
}


/// Creates a team with 1-3 members and a name in the database
/// Creates a member in the member table for each user
pub async fn create_team(members: Vec<serenity::User>, team_name: String) -> Result<usize> {
    const CREATE_TEAM: &str = "
    INSERT INTO teams (team_id, team_name)
    VALUES(?1, ?2);
    ";
    const CREATE_MEMBER: &str = "
    INSERT INTO members VALUES (?1, ?2, ?3);
    ";
    let mut hasher = DefaultHasher::new();
    team_name.hash(&mut hasher);
    let team_id: i64 = hasher.finish() as i64;
    let conn = Connection::open(TEAM_SWAPPING_PATH)?;
    let res = conn.execute(CREATE_TEAM, rusqlite::params![team_id, team_name])?;
    println!("{:?}", res);
    for member in members {
        let id = member.id.get();
        let name = member.name;
        conn.execute(CREATE_MEMBER, rusqlite::params![id, name, team_id])?;
    };
    Ok(res)
}

pub async fn delete_teams() -> Result<usize> {
    const DELETE_MEMBERS: &str = "
    DELETE FROM members
    ";
    const DELETE_TEAMS: &str = "
    DELETE FROM teams;
    ";
    let conn = Connection::open(TEAM_SWAPPING_PATH)?;
    let mut res = conn.execute(DELETE_MEMBERS, ())?;
    res += conn.execute(DELETE_TEAMS, ())?;
    Ok(res)
}

pub async fn check_if_user_in_db(user_id :u64) -> Result<bool> {
    const CHECK_QUERY: &str = "
    SELECT COUNT(*) FROM members WHERE member_id = (?1);
    ";
    let conn = Connection::open(TEAM_SWAPPING_PATH)?;
    let mut query = conn.prepare(CHECK_QUERY)?;
    let count: u64 = query.query_row(rusqlite::params![user_id], |row| row.get(0))?;
    if count == 0 {
        return Ok(false)
    } else {
        return Ok(true);
    }
}

pub async fn check_if_team_exists(team_name: &String) -> Result<bool> {
    const CHECK_QUERY: &str = "
    SELECT COUNT(*) FROM teams WHERE team_name = (?1);
    "; 
    let conn = Connection::open(TEAM_SWAPPING_PATH)?;
    let mut query = conn.prepare(CHECK_QUERY)?;
    let count: u64 = query.query_row(rusqlite::params![team_name], |row| row.get(0))?;
    if count == 0 {
        return Ok(false)
    } else {
        return Ok(true);
    }
}