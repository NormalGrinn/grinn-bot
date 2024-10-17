use warp::Filter;
use serde_json::json;

use crate::database;

pub fn get_teams() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path("get_teams")
        .and_then(move || {
            async {
                match database::get_teams() {
                    Ok(teams) => {
                        let json_reply = json!(teams);
                        Ok(warp::reply::json(&json_reply))
                    },
                    Err(_) => {
                        Err(warp::reject())
                    }
                }
            }
        })
}