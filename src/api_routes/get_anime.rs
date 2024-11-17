use warp::Filter;
use serde_json::json;

use crate::database;

pub fn get_anime() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path("get_anime")
        .and_then(move || {
            async {
                match database::get_all_anime() {
                    Ok(mut anime) => {
                        anime.sort_by(|a1, a2| a1.anime_name.cmp(&a2.anime_name));
                        let json_reply = json!(anime);
                        Ok(warp::reply::json(&json_reply))
                    },
                    Err(_) => {
                        Err(warp::reject())
                    }
                }
            }
        })
}