
use diesel::prelude::*;
use rustc_serialize::json;
use rustful::{Context, Response, header, StatusCode};

use ::models::{GameServer};

pub fn get_all_servers(_ctx: Context, mut response: Response) {
    use ::schema::game_servers::dsl::*;

    let conn = match ::establish_connection() {
        Ok(c) => c,
        Err(e) => {
            error!("Connecting to the DB in get_all failed! {:?}", e);
            response.set_status(StatusCode::InternalServerError);
            return;
        }
    };
    response.headers_mut().set(header::ContentType::json());

    let all: Vec<GameServer> = match game_servers.load(&conn) {
        Ok(servers) => servers,
        Err(e) => {
            error!("Could not execute query query in get_all_regions: {:?}", e);
            response.set_status(StatusCode::InternalServerError);
            return;
        }
    };

    let encoded = match json::encode(&all) {
        Ok(v) => v,
        Err(e) => {
            error!("Could not encode results of get_all_servers query as json: {:?}", e);
            response.set_status(StatusCode::InternalServerError);
            return;
        }
    };

    response.send(format!("{{\"results\": {}, \"size\": {}}}", encoded, all.len()));
}
