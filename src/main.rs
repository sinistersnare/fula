#![feature(plugin, custom_derive)]

#[macro_use]
extern crate postgres;

#[macro_use]
extern crate rustful;

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate hyper;
extern crate rustc_serialize;

use std::iter::Iterator;
use std::error::Error;
use std::collections::HashSet;
use std::io::prelude::*;
use std::fs::File;

use postgres::{Connection, SslMode};

use rustful::{Server, Context, Response, TreeRouter, header};

use rustc_serialize::json;

// TODO: Connection pooling?
// TODO: Do not allow duplicate IP addresses -- make an UPDATE call instead
// TODO: Have HashSets of possible GameTypes and regions and check against them.
// TODO: Descriptive error messages on not providing fields for /add
// TODO: Do not panic! return appropriate HTTP codes.
// TODO: Flesh out search functionality
// TODO: Thorough logging
// TODO: Have a table of acceptable regions and gametypes
// TODO: Link the DB tables, make GameServer use Region and GameType
// TODO: Execute all sql files in bin/ at start of program.
// TODO: MethodNotAllowed (405) errors for all common verbs.
// TODO: regions_allowed: Is it possible to not make a HashSet on no failures?.
// TODO: Ability to search for multiple regions, gameTypes, and tags.
// TODO: Refactor regions_allowed to account for gameTypes too...?
// TODO: Use diesel instead of rust-postgres directly.

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
struct GameServer {
    id: i32,
    name: String,
    region: String,
    game_type: String,
    ip: String,
}

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

fn get_all(_ctx: Context, mut response: Response) {
    let conn = Connection::connect("postgres://fula@localhost", SslMode::None).expect("connect in say_hi");
    response.headers_mut().set(header::ContentType::json());

    let mut all = vec![];
    for row in &conn.query("SELECT id, name, region, gametype, ip FROM GameServer", &[]).expect("query in say_hi") {
        let g = GameServer {
            id: row.get(0),
            name: row.get(1),
            region: row.get(2),
            game_type: row.get(3),
            ip: row.get(4),
        };
        all.push(g);
    }
    response.send(format!("{{\"results\": {}}}", json::encode(&all).expect("couldnt encode")));
}

fn search_server(mut context: Context, mut response: Response) {
    let conn = Connection::connect("postgres://fula@localhost", SslMode::None).expect("connect in add_server");
    response.headers_mut().set(header::ContentType::json());

    let body = context.body.read_json_body().expect("Could not read json body");

    let search_region: Option<&str> = body.find("region")
                                             .and_then(|r| r.as_string())
                                             .and_then(|s| Some(s));

    let game_type: Option<String> = body.find("gameType")
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
            conn.query("SELECT id, name, region, gametype, ip FROM gameserver
                            WHERE region = $1 AND gametype = $2", &[&r, &g])
                .expect("Could not execute query on region and gametype")
        },
        (Some(r), None) => {
            conn.query("SELECT id, name, region, gametype, ip FROM gameserver WHERE region = $1", &[&r])
                .expect("Could not execute query on region only")
        },
        (None, Some(g)) => {
            conn.query("SELECT id, name, region, gametype, ip FROM gameserver WHERE gametype = $1", &[&g])
                .expect("could not execute query on gametype only")
        },
        (None, None) => {
            conn.query("SELECT id, name, region, gametype, ip FROM gameserver", &[]).expect("Could not execute * query")
        }
    };

    let mut results = vec![];
    for row in &selection {
        results.push(GameServer {
            id: row.get(0),
            name: row.get(1),
            region: row.get(2),
            game_type: row.get(3),
            ip: row.get(4)
        });
    }
    let json_response = json::encode(&results).expect("Could not encode search response as JSON");
    response.send(format!("{{\"results\": {}}}", json_response))
}

fn add_server(mut context: Context, mut response: Response) {
    let conn = Connection::connect("postgres://fula@localhost", SslMode::None).expect("connect in add_server");
    response.headers_mut().set(header::ContentType::json());

    let body = context.body.read_json_body().expect("Could not read json body");

    let region: String = body.find("region").expect("Key 'region' not found in json.")
                             .as_string().expect("Region could not be converted to string.").into();
    let game_type: String = body.find("gameType").expect("Key 'gameType' not found in json.")
                             .as_string().expect("the game type could not be converted to string.").into();
    let name: String = body.find("name").expect("Key 'name' not found in json.")
                             .as_string().expect("Name could not be converted to string.").into();
    let ip: String = body.find("ip").expect("Key 'ip' not found in json.")
                             .as_string().expect("IP Address could not be converted to string.").into();

    match regions_allowed(&conn, Some(region.as_str()).into_iter()) {
        None => {},
        Some(failures) => {
            response.send(format!("\" regions `{:?}` do not exist in the Database!\"", failures));
            return;
        }
    }

    conn.execute("INSERT INTO GameServer (name, region, gametype, ip) VALUES ($1, $2, $3, $4)",
                 &[&name, &region, &game_type, &ip]).expect("Could not add server to table");

    response.send(format!("\"server `{}` added!\"", name));
}

fn main() {
    env_logger::init().expect("env_logger init");

    let conn = Connection::connect("postgres://fula@localhost", SslMode::None).expect("connect in main");

    let mut f = File::open("bin/create_tables.sql").expect("Could not open create_tables.sql");
    let mut contents = String::new();
    f.read_to_string(&mut contents).expect("Could not read to string.");
    for cmd in contents.split(";") {
        conn.execute(cmd, &[]).expect("Could not execute create_tables.sql");
    }

    let server = Server {
        host: 8080.into(),
        handlers: insert_routes!{
            TreeRouter::new() => {
                // root
                Get: get_all as fn(Context, Response),

                // next level down
                "all" => {
                    Get: get_all as fn(Context, Response),
                },
                "search" => {
                    Post: search_server as fn(Context, Response),
                },
                "add" => {
                    Post: add_server as fn(Context, Response),
                },
            }
        },
        ..Server::default()
    }.run();

    match server {
        Ok(_) => {},
        Err(e) => error!("could not start server: {}", e.description())
    }
    println!("Ready to go! Send requests now!")
}
