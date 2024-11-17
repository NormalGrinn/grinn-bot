use warp::Filter;
use serde_json::json;

use crate::database;

pub fn get_teams() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path("get_teams")
        .and_then(move || {
            async {
                match database::get_teams() {
                    Ok(teams) => {
                        match database::get_lonely_eligible_users() {
                            Ok(users) => {
                                let response = (teams, users);
                                let json_reply = json!(response);
                                Ok(warp::reply::json(&json_reply))
                            },
                            Err(_) => Err(warp::reject()),
                        }
                    },
                    Err(_) => Err(warp::reject())
                }
            }
        })
}