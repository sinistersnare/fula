//! Routes used by the REST API

use std::collections::HashSet;

use postgres::{Connection, SslMode};
use rustful::{Context, Response, header};
use rustc_serialize::json;

use game_server::{GameServer, make_game_server_from_row};


fn regions_allowed<'a, 'b, I>(conn: &'a Connection, regions: I) -> Option<HashSet<&'b str>>
                    where I: Iterator<Item=&'b str> {
    let all_regions: HashSet<String> = conn.query("SELECT name FROM region", &[])
                                      .expect("could not select all regions.")
                                      .into_iter().map(|v| v.get::<usize, String>(0))
                                      .collect();
    let failed: HashSet<&'b str> = regions.filter(|r| !all_regions.contains(*r)).collect();
    if failed.len() == 0 {
        None
    } else {
        Some(failed)
    }
}

pub fn get_all(_ctx: Context, mut response: Response) {
    let conn = Connection::connect("postgres://fula@localhost", SslMode::None)
                    .expect("connect in get_all");
    response.headers_mut().set(header::ContentType::json());

    let all: Vec<GameServer> = conn.query("SELECT * FROM GameServer", &[]).expect("Query in get_all")
                                   .iter().map(make_game_server_from_row).collect();
    response.send(format!("{{\"results\": {}}}", json::encode(&all).expect("couldnt encode")));
}

pub fn search_server(mut context: Context, mut response: Response) {
    let conn = Connection::connect("postgres://fula@localhost", SslMode::None)
                    .expect("connect in add_server");
    response.headers_mut().set(header::ContentType::json());

    let body = context.body.read_json_body().expect("Could not read json body");

    let search_region: Option<&str> = body.find("region")
                                             .and_then(|r| r.as_string())
                                             .and_then(|s| Some(s));

    let game_type: Option<String> = body.find("game_type")
                                         .and_then(|r| r.as_string())
                                         .and_then(|s| Some(s.into()));

    match regions_allowed(&conn, search_region.into_iter()) {
        None => {},
        Some(failures) => {
            response.send(format!("\" regions `{:?}` do not exist in the Database!\"", failures));
            return;
        }
    }

    let selection = match (search_region, game_type) {
        (Some(r), Some(g)) => {
            conn.query("SELECT * FROM gameserver WHERE region = $1 AND game_type = $2", &[&r, &g])
                .expect("Could not execute query on region and game_type")
        },
        (Some(r), None) => {
            conn.query("SELECT * FROM gameserver WHERE region = $1", &[&r])
                .expect("Could not execute query on region only")
        },
        (None, Some(g)) => {
            conn.query("SELECT * FROM gameserver WHERE game_type = $1", &[&g])
                .expect("could not execute query on game_type only")
        },
        (None, None) => {
            conn.query("SELECT * FROM gameserver", &[]).expect("Could not execute * query")
        }
    };

    let mut results = vec![];
    for row in &selection {
        results.push(make_game_server_from_row(row));
    }

    let json_response = json::encode(&results).expect("Could not encode search response as JSON");
    response.send(format!("{{\"results\": {}}}", json_response))
}

pub fn add_server(mut context: Context, mut response: Response) {
    let conn = Connection::connect("postgres://fula@localhost", SslMode::None)
                    .expect("connect in add_server");
    response.headers_mut().set(header::ContentType::json());
    let parsed_server: GameServer = context.body.decode_json_body()
                                           .expect("Could not decode JSON into a GameServer object");

    match regions_allowed(&conn, Some(parsed_server.region.as_str()).into_iter()) {
        None => {},
        Some(failures) => {
            response.send(format!("\" regions `{:?}` do not exist in the Database!\"", failures));
            return;
        }
    }

    conn.execute("INSERT INTO GameServer (name, region, game_type, ip, max_users,
                                            current_users, current_premium_users,
                                            max_premium_users, tags)
                                            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
                 &[&parsed_server.name, &parsed_server.region,
                   &parsed_server.game_type, &parsed_server.ip,
                   &parsed_server.max_users, &parsed_server.current_users,
                   &parsed_server.current_premium_users, &parsed_server.max_premium_users,
                   &parsed_server.tags])
        .expect("Could not add server to table");

    response.send(format!("\"server `{}` added!\"", &parsed_server.name));
}
