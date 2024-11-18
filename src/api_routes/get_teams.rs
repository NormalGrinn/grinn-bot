use serde::Serialize;
use warp::Filter;
use serde_json::json;

use crate::database;

#[derive(Debug, Serialize, Clone)]
struct Users {
    teamless_users: Vec<String>,
}

pub fn get_teams() -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path("get_teams")
        .and_then(move || {
            async {
                match database::get_teams() {
                    Ok(teams) => {
                        match database::get_lonely_eligible_users() {
                            Ok(users) => {
                                let wrapped_users = Users {
                                    teamless_users: users,
                                };
                                let response = (teams, wrapped_users);
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