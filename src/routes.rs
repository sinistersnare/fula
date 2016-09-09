//! Routes used by the REST API

use std::collections::HashSet;

use postgres::{Connection, SslMode};
use rustful::{Context, Response, header, StatusCode};
use rustc_serialize::json;

use game_server::{GameServer};

enum AllowedRegion<T> {
    Success,
    Failure(T),
    Panic,
}

fn regions_allowed<'a, 'b, I>(conn: &'a Connection, regions: I) -> AllowedRegion<HashSet<&'b str>>
                    where I: Iterator<Item=&'b str> {
    let all_regions: HashSet<String> = match conn.query("SELECT name FROM region", &[]) {
        Ok(c) => c.into_iter().map(|v| v.get::<usize, String>(0)).collect(),
        Err(e) => {
            error!("Could not execute query in regions_allowed: {:?}", e);
            return AllowedRegion::Panic;
        }
    };
    // FIXME: Is it possible to not make a HashSet on no failures?.
    let failed: HashSet<&'b str> = regions.filter(|r| !all_regions.contains(*r)).collect();
    if failed.len() == 0 {
        AllowedRegion::Success
    } else {
        AllowedRegion::Failure(failed)
    }
}

pub fn get_all(_ctx: Context, mut response: Response) {
    let conn = match Connection::connect("postgres://fula@localhost", SslMode::None) {
        Ok(c) => c,
        Err(e) => {
            error!("Connecting to the DB in get_all failed! {:?}", e);
            response.set_status(StatusCode::InternalServerError);
            return;
        }
    };
    response.headers_mut().set(header::ContentType::json());

    let all: Vec<GameServer> = match conn.query("SELECT * FROM GameServer", &[]) {
        Ok(r) => r.iter().map(|r| GameServer::from_row(&r)).collect(),
        Err(e) => {
            error!("Could not execute the query in get_all: {:?}", e);
            response.set_status(StatusCode::InternalServerError);
            return;
        }
    };

    let encoded = match json::encode(&all) {
        Ok(v) => v,
        Err(e) => {
            error!("Could not encode results of get_all query as json: {:?}", e);
            response.set_status(StatusCode::InternalServerError);
            return;
        }
    };

    response.send(format!("{{\"results\": {}, \"size\": {}}}", encoded, all.len()));
}

pub fn update_server(mut context: Context, mut response: Response) {
    let conn = match Connection::connect("postgres://fula@localhost", SslMode::None) {
        Ok(c) => c,
        Err(e) => {
            error!("Connecting to the DB in update_server failed: {:?}", e);
            response.set_status(StatusCode::InternalServerError);
            return;
        }
    };
    response.headers_mut().set(header::ContentType::json());

    let server_id: i32 = match context.variables.parse("id") {
        Ok(id) => id,
        Err(Some(e)) => {
            error!("Could not parse the ID as an i32: {:?}", e);
            response.set_status(StatusCode::BadRequest);
            return;
        },
        Err(None) => {
            error!("Another Error occured during context variable parsing.");
            response.set_status(StatusCode::InternalServerError);
            return;
        }
    };

    let body = match context.body.read_json_body() {
        Ok(b) => b,
        Err(e) => {
            error!("Could not read update_server json body {:?}", e);
            response.set_status(StatusCode::InternalServerError);
            return;
        }
    };

    response.send(format!("\"Update recieved I DID NOTHING! Gonna change {}\"", server_id));
}

pub fn search_server(mut context: Context, mut response: Response) {
    let conn = match Connection::connect("postgres://fula@localhost", SslMode::None) {
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

    let game_type: Option<String> = body.find("game_type")
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

    // FIXME: This should not be 4 different queries.
    // Perhaps Diesel will fix this.
    let selection = match (search_region, game_type) {
        (Some(r), Some(g)) => {
            match conn.query("SELECT * FROM gameserver WHERE region = $1 AND game_type = $2", &[&r, &g]) {
                Ok(s) => s,
                Err(e) => {
                    error!("Could not execute search query on region and game_type: {:?}", e);
                    response.set_status(StatusCode::InternalServerError);
                    return;
                }

            }
        },
        (Some(r), None) => {
            match conn.query("SELECT * FROM gameserver WHERE region = $1", &[&r]) {
                Ok(s) => s,
                Err(e) => {
                    error!("Could not execute query on region only: {:?}", e);
                    response.set_status(StatusCode::InternalServerError);
                    return;
                }
            }
        },
        (None, Some(g)) => {
            match conn.query("SELECT * FROM gameserver WHERE game_type = $1", &[&g]) {
                Ok(s) => s,
                Err(e) => {
                    error!("Could not execute query on game_type only: {:?}", e);
                    response.set_status(StatusCode::InternalServerError);
                    return;
                }
            }
        },
        (None, None) => {
            match conn.query("SELECT * FROM gameserver", &[]) {
                Ok(s) => s,
                Err(e) => {
                    error!("Could not execute a query on all servers: {:?}", e);
                    response.set_status(StatusCode::InternalServerError);
                    return;
                }
            }
        }
    };

    let results: Vec<_> = selection.iter().map(|r| GameServer::from_row(&r)).collect();
    let json_response = match json::encode(&results) {
        Ok(r) => r,
        Err(e) => {
            error!("Could not encode search response as JSON: {:?}", e);
            response.set_status(StatusCode::InternalServerError);
            return;
        }
    };
    response.send(format!("{{\"results\": {}}}", json_response))
}

pub fn add_server(mut context: Context, mut response: Response) {
    let conn = match Connection::connect("postgres://fula@localhost", SslMode::None) {
        Ok(c) => c,
        Err(e) => {
            error!("Could connect to the DB in add_server: {:?}", e);
            response.set_status(StatusCode::InternalServerError);
            return;
        }
    };
    response.headers_mut().set(header::ContentType::json());
    let parsed_server: GameServer = match context.body.decode_json_body() {
        Ok(s) => s,
        Err(e) => {
            error!("Could not decode request JSON into a GameServer object: {:?}", e);
            response.set_status(StatusCode::InternalServerError);
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
    match conn.execute("INSERT INTO GameServer (name, region, game_type, ip, max_users,
                                            current_users, current_premium_users,
                                            max_premium_users, tags)
                                            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
                 &[&parsed_server.name, &parsed_server.region,
                   &parsed_server.game_type, &parsed_server.ip,
                   &parsed_server.max_users, &parsed_server.current_users,
                   &parsed_server.current_premium_users, &parsed_server.max_premium_users,
                   &parsed_server.tags]) {
        Ok(_rows_affected) => {},
        Err(e) => {
            error!("Unable to add server to table: {:?}", e);
            response.set_status(StatusCode::InternalServerError);
            return;
        }
    };
    response.send(format!("\"server `{}` added!\"", &parsed_server.name));
}
