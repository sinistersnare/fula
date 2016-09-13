
use diesel;
use diesel::prelude::*;
use rustful::{Context, Response, header, StatusCode};

use ::models::NewGameServer;
use ::routes::{regions_allowed, AllowedRegion};

pub fn add_server(mut context: Context, mut response: Response) {
    use schema::game_servers;

    let conn = match ::establish_connection() {
        Ok(c) => c,
        Err(e) => {
            error!("Could connect to the DB in add_server: {:?}", e);
            response.set_status(StatusCode::InternalServerError);
            return;
        }
    };
    response.headers_mut().set(header::ContentType::json());
    let parsed_server: NewGameServer = match context.body.decode_json_body() {
        Ok(s) => s,
        Err(e) => {
            error!("Could not decode request JSON into a GameServer object: {:?}", e);
            response.set_status(StatusCode::BadRequest);
            return;
        }
    };
    match regions_allowed(&conn, Some(parsed_server.region.as_str()).into_iter()) {
        AllowedRegion::Success => {},
        AllowedRegion::Failure(failures) => {
            response.set_status(StatusCode::BadRequest);
            response.send(format!("\"Regions `{:?}` do not exist in the Database!\"", failures));
            return;
        },
        AllowedRegion::Panic => {
            error!("Failed in add_server/regions_allowed, can not complete request.");
            response.set_status(StatusCode::InternalServerError);
            return;
        }
    };

    match diesel::insert(&parsed_server).into(game_servers::table).execute(&conn) {
        Ok(_) => {},
        Err(e) => {
            error!("Failed to insert server into game_servers: {:?}", e);
            response.set_status(StatusCode::InternalServerError);
            return;
        }
    };
    response.send(format!("\"server `{}` added!\"", &parsed_server.name));
}
