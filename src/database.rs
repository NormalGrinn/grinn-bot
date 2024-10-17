use rusqlite::{Connection, OptionalExtension, Result};
use crate::types;
use strsim::jaro_winkler;
use std::{collections::HashMap, hash::{DefaultHasher, Hash, Hasher}};
use poise::serenity_prelude as serenity;
use std::time::SystemTime;
use chrono::{Date, DateTime, Utc};

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

pub fn get_teams() -> Result<Vec<types::TeamMembers>> {
    const GET_TEAMS: &str = "
    SELECT members.member_id, members.name, teams.team_id, teams.team_name, teams.team_image_url
    FROM members
    INNER JOIN teams ON members.team = teams.team_id;
    ";
    let conn = Connection::open(TEAM_SWAPPING_PATH)?;
    let mut team_query = conn.prepare(GET_TEAMS)?;
    let team_iter = team_query.query_map([], |row| {
        Ok((
            types::Team {
                team_id: row.get(2)?,
                team_image_url: row.get(4)?,
                team_name: row.get(3)?,
            },
            types::Member {
                member_id: row.get(0)?,
                member_name: row.get(1)?,
            },
        ))
    })?;
    let mut teams: HashMap<i64, (types::Team, Vec<types::Member>)> = HashMap::new();
    for m in team_iter {
        match m {
            Ok((team, member)) => {
                teams.entry(team.team_id).or_insert_with(|| {
                    (team.clone(), Vec::new()) // Clone the team to insert into the HashMap
                }).1.push(member);
            },
            Err(_) => (),
        }
    }
    let team_members_list: Vec<types::TeamMembers> = teams
    .into_iter()
    .map(|(_, (team, members))| types::TeamMembers { team, members })
    .collect();
    Ok(team_members_list)
}


/// Creates a team with 1-3 members and a name in the database
pub fn create_team(members: Vec<serenity::User>, team_name: String) -> Result<usize> {
    const CREATE_TEAM: &str = "
    INSERT INTO teams (team_id, team_name)
    VALUES(?1, ?2);
    ";
    const UPDATE_MEMBERS: &str = "
    UPDATE members
    SET team = ?1
    WHERE member_id = ?2;
    ";
    let mut hasher = DefaultHasher::new();
    team_name.hash(&mut hasher);
    let team_id: i64 = hasher.finish() as i64;
    let conn = Connection::open(TEAM_SWAPPING_PATH)?;
    let res = conn.execute(CREATE_TEAM, rusqlite::params![team_id, team_name])?;
    for member in members {
        let id = member.id.get();
        conn.execute(UPDATE_MEMBERS, rusqlite::params![team_id, id])?;
    };
    Ok(res)
}

pub fn create_member(member: serenity::User) -> Result<usize> {
    const CREATE_MEMBER: &str = "
    INSERT INTO members VALUES (?1, ?2, NULL);
    ";
    let id = member.id.get();
    let name = member.name;
    let conn = Connection::open(TEAM_SWAPPING_PATH)?;
    let res = conn.execute(CREATE_MEMBER, rusqlite::params![id, name])?;
    Ok(res)
}

pub fn create_anime(anime_id: &u64, anime_name: &String, submitter_id: u64) -> Result<usize> {
    const CREATE_ANIME: &str = "
    INSERT INTO anime VALUES (?1, ?2, ?3);
    ";
    let conn = Connection::open(TEAM_SWAPPING_PATH)?;
    let res = conn.execute(CREATE_ANIME, rusqlite::params![anime_id, anime_name, submitter_id])?;
    Ok(res)
}

pub fn create_claimed_anime(anime_id: u64, team_id: i64, user_id :u64) -> Result<usize> {
    const CREATE_CLAIMED_ANIME: &str = "
    INSERT INTO claimed_anime (anime_id, team_id, claimed_by, claimed_on)
    VALUES (?1, ?2, ?3, ?4);
    ";
    let present_time: SystemTime = SystemTime::now();
    let present_time: DateTime<Utc> = present_time.into();
    let present_time: String = present_time.to_rfc3339();
    let conn: Connection = Connection::open(TEAM_SWAPPING_PATH)?;
    println!("{} {} {} {}", anime_id, team_id, user_id, present_time);
    let res = conn.execute(CREATE_CLAIMED_ANIME, rusqlite::params![anime_id, team_id, user_id, present_time])?;
    Ok(res)
}

pub fn delete_teams() -> Result<usize> {
    const DELETE_MEMBERS: &str = "
    DELETE FROM members;
    ";
    const DELETE_TEAMS: &str = "
    DELETE FROM teams;
    ";
    let conn = Connection::open(TEAM_SWAPPING_PATH)?;
    let mut res = conn.execute(DELETE_MEMBERS, ())?;
    res += conn.execute(DELETE_TEAMS, ())?;
    Ok(res)
}

pub fn check_if_user_in_team(user_id :u64) -> Result<bool, rusqlite::Error> {
    const CHECK_QUERY: &str = "
    SELECT team IS NULL FROM members WHERE member_id = ?1;
    ";
    let conn = Connection::open(TEAM_SWAPPING_PATH)?;
    let in_team: Option<bool> = conn.query_row(CHECK_QUERY,
        rusqlite::params![user_id],
         |row| row.get(0)).optional()?;
    Ok(!in_team.unwrap_or(false))
}

pub fn check_if_team_exists(team_name: &String) -> Result<bool> {
    const CHECK_QUERY: &str = "
    SELECT COUNT(*) FROM teams WHERE team_name = ?1;
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

pub fn check_if_user_exists(user_id: u64) -> Result<bool> {
    const CHECK_QUERY: &str = "
    SELECT member_id FROM members WHERE member_id = ?1;
    ";
    let conn = Connection::open(TEAM_SWAPPING_PATH)?;
    let user_exists: Option<i64> = conn.query_row(
        CHECK_QUERY,
        rusqlite::params![user_id],
        |row| row.get(0),
    ).optional()?;
    Ok(user_exists.is_some())
}

pub fn check_if_anime_exists(anime_id: u64) -> Result<bool> {
    const CHECK_QUERY: &str = "
    SELECT anime_id FROM anime WHERE anime_id = ?1;
    ";
    let conn = Connection::open(TEAM_SWAPPING_PATH)?;
    let anime_exists: Option<i64> = conn.query_row(
        CHECK_QUERY,
        rusqlite::params![anime_id],
        |row| row.get(0),
    ).optional()?;
    Ok(anime_exists.is_some())
}

pub fn check_if_anime_is_claimed(anime_name: &String) -> Result<bool> {
    const CHECK_QUERY: &str = "
        SELECT EXISTS(
        SELECT 1 
        FROM anime a
        JOIN claimed_anime ca ON a.anime_id = ca.anime_id
        WHERE a.name = ?1
    )";
    let conn = Connection::open(TEAM_SWAPPING_PATH)?;
    let mut stmt = conn.prepare(CHECK_QUERY)?;
    let is_claimed: bool = stmt.query_row([anime_name], |row| row.get(0))?;
    Ok(is_claimed)
}

pub fn count_submitted_anime(user_id: u64) -> Result<u64> {
    const COUNT_QUERY: &str = "
    SELECT COUNT(*) AS anime_count FROM anime WHERE submitter = ?1; 
    ";
    let conn: Connection = Connection::open(TEAM_SWAPPING_PATH)?;
    let anime_count: u64 = conn.query_row(COUNT_QUERY, rusqlite::params![user_id], |row| row.get(0))?;
    Ok(anime_count)
}

pub fn get_submitted_anime(user_id: u64) -> Result<Vec<String>> {
    let ANIME_NAME_QUERY: &str = "
    SELECT name FROM anime WHERE submitter = ?1;
    ";
    let mut names: Vec<String> = Vec::new();
    let conn: Connection = Connection::open(TEAM_SWAPPING_PATH)?;
    let mut names_query = conn.prepare(ANIME_NAME_QUERY)?;
    let names_iter = names_query.query_map(rusqlite::params![user_id],
    |row| {
        Ok((
            row.get::<_, String>(0)?
        ))
    })?;
    for name in names_iter {
        match name {
            Ok(n,) => names.push(n),
            Err(_) => (),
        }
    }
    Ok(names)
}

pub fn get_all_anime() -> Result<Vec<types::SubmittedAnime>> {
    let GET_ANIME_QUERY: &str = "
    SELECT 
        a.anime_id,
        a.name AS anime_name,
        m.name AS submitter_name,
        t.team_name AS claimed_team_name,
        ca.claimed_on
    FROM 
        anime a
    JOIN 
        members m ON a.submitter = m.member_id
    LEFT JOIN 
        claimed_anime ca ON a.anime_id = ca.anime_id
    LEFT JOIN 
        teams t ON ca.team_id = t.team_id;
    ";
    let mut anime: Vec<types::SubmittedAnime> = Vec::new();
    let conn: Connection = Connection::open(TEAM_SWAPPING_PATH)?;
    let mut anime_query = conn.prepare(GET_ANIME_QUERY)?;
    let anime_iter = anime_query.query_map((),
    |row| {
        Ok(types::SubmittedAnime {
            anime_id: row.get(0)?,
            anime_name: row.get(1)?,
            submitter_name: row.get(2)?,
            claimed_by_team: row.get::<_, Option<String>>(3)?,
            claimed_on: row.get::<_, Option<String>>(4)?,
        })
    })?;
    for a in anime_iter {
        match a {
            Ok(valid_anime) => anime.push(valid_anime),
            Err(_) => (),
        }
    }
    Ok(anime)
}

pub fn get_anime_submitter(anime_name: &String) -> Result<u64> {
    let SUBMITTER_QUERY: &str = "
    SELECT submitter FROM anime WHERE name = ?1;
    ";
    let conn: Connection = Connection::open(TEAM_SWAPPING_PATH)?;
    let res: u64 = conn.query_row(SUBMITTER_QUERY, rusqlite::params![anime_name], |row| row.get(0))?;
    Ok(res)
}

pub fn get_member_with_team(user_id :u64) -> Result<(types::Member, i64)> {
    let MEMBER_QUERY: &str = "
    SELECT * FROM members WHERE member_id = ?1;
    ";
    let conn: Connection = Connection::open(TEAM_SWAPPING_PATH)?;
    let res: (types::Member, i64) = conn.query_row(MEMBER_QUERY, rusqlite::params![user_id], 
    |row| {
        Ok((types::Member {
            member_id: row.get(0)?,
            member_name: row.get(1)?,
        },
            row.get(2)?,
        ))
    })?;
    Ok(res)
}

pub fn get_anime_id_by_name(anime_name: &String) -> Result<Option<u64>> {
    let ID_QUERY: &str = "
    SELECT anime_id FROM anime WHERE name = ?1;
    ";
    let conn: Connection = Connection::open(TEAM_SWAPPING_PATH)?;
    let res = conn.query_row(ID_QUERY, rusqlite::params![anime_name], 
    |row| row.get(0),
    );
    match res {
        Ok(anime_id) => Ok(Some(anime_id)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn get_unclaimed_anime_names() -> Result<Vec<String>> {
    let UNCLAIMED_QUERY: &str = "
    SELECT a.name
    FROM anime a
    LEFT JOIN claimed_anime ca ON a.anime_id = ca.anime_id
    WHERE ca.anime_id IS NULL;
    ";
    let conn = Connection::open(TEAM_SWAPPING_PATH)?;
    let mut stmt = conn.prepare(UNCLAIMED_QUERY)?;
    let anime_names = stmt.query_map([], |row| row.get(0))?
        .filter_map(Result::ok)
        .collect::<Vec<String>>();
    Ok (anime_names)
}

pub fn delete_anime(anime_name: &String) -> Result<usize> {
    let DELETE_QUERY: &str = "
    DELETE FROM anime WHERE name = ?1;
    ";
    let conn: Connection = Connection::open(TEAM_SWAPPING_PATH)?;
    let res = conn.execute(DELETE_QUERY, rusqlite::params![anime_name])?;
    Ok(res)
}