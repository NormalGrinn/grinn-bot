use rusqlite::{params, Connection, Result};
use crate::types;
use strsim::jaro_winkler;
const ANIME_GUESSING_PATH: &str = "databases/animeGuessing.db";
const SERVER_LIST_PATH: &str = "databases/serverList.db";

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

pub async fn upsert_user(user_list: types::UserList) -> Result<()> {
    const UPSERT_USER: &str = "
    INSERT OR REPLACE INTO users (user_id, user_name, user_score_type)
    VALUES (?1, ?2, ?3);
    ";
    const UPSERT_ANIME: &str = "
    INSERT OR REPLACE INTO anime (anime_id, anime_names)
    VALUES (?1, ?2);
    ";
    const UPSERT_ENTRY: &str = "
    INSERT INTO list_entry_table (
        user_id, anime_id, anime_score, is_favourite, notes, rewatches, completion_status
    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
    ON CONFLICT (user_id, anime_id) DO UPDATE SET
        anime_score = excluded.anime_score,
        is_favourite = excluded.is_favourite,
        notes = excluded.notes,
        rewatches = excluded.rewatches,
        completion_status = excluded.completion_status;
    ";
    let mut conn = Connection::open(SERVER_LIST_PATH).map_err(|e| {
        eprintln!("Failed to open database: {}", e);
        e
    })?;

    let tx = conn.transaction().map_err(|e| {
        eprintln!("Failed to start transaction: {}", e);
        e
    })?;

    tx.execute(UPSERT_USER, rusqlite::params![user_list.user_id, user_list.user_name, user_list.user_score_type]).map_err(|e| {
        eprintln!("Failed to upsert user: {}", e);
        e
    })?;
    for anime in user_list.anime {
        tx.execute(UPSERT_ANIME, rusqlite::params![anime.anime_id, anime.titles]).map_err(|e| {
            eprintln!("Failed to upsert anime: {}", e);
            e
        })?;
        tx.execute(UPSERT_ENTRY, rusqlite::params![user_list.user_id, anime.anime_id, anime.score, anime.favourite, anime.notes, anime.repeats, anime.status]).map_err(|e| {
            eprintln!("Failed to upsert entry: {}", e);
            e
        })?;
    }

    tx.commit().map_err(|e| {
        eprintln!("Failed to commit transaction: {}", e);
        e
    })?;

    Ok(())
}

pub async fn get_server_anime_titles() -> Vec<String> {
    const GET_TITLES: &str = "
    SELECT anime_names FROM anime;
    ";
    let conn = Connection::open(SERVER_LIST_PATH).expect("Error making connection");
    let mut result_titles: Vec<String> = Vec::new();
    let mut titles_query = conn.prepare(GET_TITLES).expect("Error making query");
    let titles_iter = titles_query.query_map((), 
    |row| {
        let titles: String = row.get(0).expect("Error mapping SQL to String");
        Ok(titles)
    }).expect("Error with the query iter");
    for titles in titles_iter {
        match titles {
            Ok(t) => {
                let anime_titles: types::Title = serde_json::from_str(&t).expect("Error parsing titles JSON");
                anime_titles.romaji.map(|s| result_titles.push(s));
                anime_titles.native.map(|s| result_titles.push(s));
                anime_titles.english.map(|s| result_titles.push(s));
            },
            Err(_) => {
                eprintln!("Error with titles");
                continue;   
            },
        }
    }
    result_titles
}

pub async fn get_anime_info(anime_name: &str) -> Result<Vec<types::ListEntry>> {
    let GET_ANIME_INFO_QUERY: &str = "
    SELECT list_entry_table.*, anime.anime_names, users.user_name, users.user_score_type
    FROM list_entry_table
    JOIN anime ON list_entry_table.anime_id = anime.anime_id
    JOIN users ON list_entry_table.user_id = users.user_id
    WHERE LOWER(json_extract(anime.anime_names, '$.romaji')) LIKE LOWER(?1)
        OR LOWER(json_extract(anime.anime_names, '$.native')) LIKE LOWER(?1)
        OR LOWER(json_extract(anime.anime_names, '$.english')) LIKE LOWER(?1);
    ";
    let conn = Connection::open(SERVER_LIST_PATH)?;
    let mut result_info: Vec<types::UserAnimeInfo> = Vec::new();
    let mut info_query = conn.prepare(GET_ANIME_INFO_QUERY)?;
    let mut info_iter = info_query.query_map(rusqlite::params![anime_name],
    |row| {
        let anime_names_json: String = row.get(7)?;
        let anime_names: types::Title = serde_json::from_str(&anime_names_json)
            .map_err(|e| rusqlite::Error::ExecuteReturnedResults)?;
        Ok(types::ListEntry {
            user_id: row.get(0)?,
            anime_id: row.get(1)?,
            anime_score: row.get(2)?,
            is_favourite: row.get(3)?,
            notes: row.get(4)?,
            rewatches: row.get(5)?,
            completion_status: row.get(6)?,
            anime_names,
            user_name: row.get(8)?, // user_name is the 9th column (index 8)
            user_score_type: row.get(9)?, // user_score_type is the 10th column (index 9)
        })
    })?
    .collect::<Result<Vec<types::ListEntry>>>();
    info_iter
}