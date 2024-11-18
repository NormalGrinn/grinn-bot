use serde::Serialize;
use warp::Filter;
use serde_json::json;

use crate::{database, types};


#[derive(Debug, Serialize, Clone)]
struct Data {
    teams: Vec<types::TeamMembers>,
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
                                let resp = Data {
                                    teams: teams,
                                    teamless_users: users,
                                };
                                let json_reply = json!(resp);
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