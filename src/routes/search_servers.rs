
use diesel::prelude::*;
use rustc_serialize::json;
use rustful::{Context, Response, header, StatusCode};

use ::routes::{AllowedRegion, regions_allowed};
use ::models::{GameServer};

pub fn search_servers(mut context: Context, mut response: Response) {
    use ::schema::game_servers::dsl::*;
    let conn = match ::establish_connection() {
        Ok(c) => c,
        Err(e) => {
            error!("Connecting to the DB in search_server failed! {:?}", e);
            response.set_status(StatusCode::InternalServerError);
            return;
        }
    };
    response.headers_mut().set(header::ContentType::json());

    let body = match context.body.read_json_body() {
        Ok(b) => b,
        Err(e) => {
            error!("Could not read search_server json body: {:?}", e);
            response.set_status(StatusCode::InternalServerError);
            return;
        }
    };

    let search_region: Option<&str> = body.find("region")
                                             .and_then(|r| r.as_string())
                                             .and_then(|s| Some(s));

    let search_game_type: Option<String> = body.find("game_type")
                                         .and_then(|r| r.as_string())
                                         .and_then(|s| Some(s.into()));

    match regions_allowed(&conn, search_region.into_iter()) {
        AllowedRegion::Success => {},
        AllowedRegion::Failure(failures) => {
            response.set_status(StatusCode::BadRequest);
            response.send(format!("\"Regions `{:?}` do not exist in the Database!\"", failures));
            return;
        },
        AllowedRegion::Panic => {
            error!("Failed in search_server/regions_allowed, can not complete request.");
            response.set_status(StatusCode::InternalServerError);
            return;
        }
    };

    let mut query = game_servers.into_boxed();
    if let Some(r) = search_region {
        query = query.filter(region.eq(r));
    }
    if let Some(g) = search_game_type {
        query = query.filter(game_type.eq(g));
    }

    let results = match query.load::<GameServer>(&conn) {
        Ok(v) => v,
        Err(e) => {
            error!("Server search filter failed: {:?}", e);
            response.set_status(StatusCode::InternalServerError);
            return;
        }
    };

    let json_response = match json::encode(&results) {
        Ok(r) => r,
        Err(e) => {
            error!("Could not encode search response as JSON: {:?}", e);
            response.set_status(StatusCode::InternalServerError);
            return;
        }
    };
    response.send(format!("{{\"results\": {}, \"size\": {}}}", json_response, results.len()))
}
