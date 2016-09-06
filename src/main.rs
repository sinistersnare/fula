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

use std::default::Default;
use std::iter::Iterator;
use std::error::Error;
use std::collections::HashSet;
use std::io::prelude::*;
use std::fs::File;

use postgres::{Connection, SslMode, rows};

use rustful::{Server, Context, Response, TreeRouter, header};

use rustc_serialize::{json, Decodable, Decoder, DecoderHelpers};

// TODO: Move all these TODOs to an issue tracker.
// TODO: split out code to different files.
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
// TODO /add converts from i64 to i32. should probably harden the check.

#[derive(Debug, Clone, RustcEncodable)]
struct GameServer {
    id: i32,
    name: String,
    region: String,
    game_type: String,
    ip: String,
    max_users: i32,
    current_users: i32,
    current_premium_users: Option<i32>,
    max_premium_users: Option<i32>,
    tags: Vec<String>,
}

impl Decodable for GameServer {
    fn decode<D: Decoder>(d: &mut D) -> Result<GameServer, D::Error> {
        d.read_struct("GameServer", 10, |d| {
            let id: i32 = match d.read_struct_field("id", 0, |d| { d.read_i32()}) {
                Ok(v) => v,
                Err(_e) => -1,
            };
            let name = match d.read_struct_field("name", 1, |d| { d.read_str()}) {
                Ok(v) => v,
                Err(_e) => { return Err(d.error("Couldnt Decode a name from GameServer JSON")); },
            };
            let region = match d.read_struct_field("region", 2, |d| { d.read_str()}) {
                Ok(v) => v,
                Err(_e) => { return Err(d.error("Couldnt Decode a region from GameServer JSON")); },
            };
            let game_type = match d.read_struct_field("game_type", 3, |d| { d.read_str()}) {
                Ok(v) => v,
                Err(_e) => { return Err(d.error("Couldnt Decode a game_type from GameServer JSON")); },
            };
            let ip = match d.read_struct_field("ip", 4, |d| { d.read_str()}) {
                Ok(v) => v,
                Err(_e) => { return Err(d.error("Couldnt Decode an IP from GameServer JSON")); },
            };
            let max_users = match d.read_struct_field("max_users", 5, |d| { d.read_i32()}) {
                Ok(v) => v,
                Err(_e) => { return Err(d.error("Couldnt Decode max_users from GameServer JSON")); },
            };
            let current_users = match d.read_struct_field("current_users", 6, |d| { d.read_i32()}) {
                Ok(v) => v,
                Err(_e) => 0,
            };
            let current_premium_users = match d.read_struct_field("current_premium_users",
                                                        7, |d| { d.read_i32()}) {
                Ok(v) => Some(v),
                Err(_e) => None,
            };
            let max_premium_users: Option<i32> = match d.read_struct_field("max_premium_users",
                                                        8, |d| { d.read_i32()}) {
                Ok(v) => Some(v),
                Err(_e) => None,
            };
            let tags: Vec<String> = match d.read_struct_field("tags", 9, |d| {
                d.read_to_vec(|d| d.read_str())
            }) {
                Ok(v) => v,
                Err(_e) => { return Err(d.error("Could not decode tag as str."))}
            };

            Ok(GameServer {
                id: id,
                name: name,
                region: region,
                game_type: game_type,
                ip: ip,
                max_users: max_users,
                current_users: current_users,
                current_premium_users: current_premium_users,
                max_premium_users: max_premium_users,
                tags: tags
            })
        })
    }
}

impl Default for GameServer {
    fn default() -> Self {
        GameServer {
            id: 0,
            name: "".into(),
            region: "".into(),
            game_type: "".into(),
            ip: "".into(),
            max_users: 0,
            current_users: 0,
            current_premium_users: None,
            max_premium_users: None,
            tags: Vec::new(),
        }
    }
}

fn make_game_server_from_row(row: rows::Row) -> GameServer {
    GameServer {
        id: row.get(0),
        name: row.get(1),
        region: row.get(2),
        game_type: row.get(3),
        ip: row.get(4),
        max_users: row.get(5),
        current_users: row.get(6),
        current_premium_users: row.get(7),
        max_premium_users: row.get(8),
        tags: row.get(9)
    }
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
    let conn = Connection::connect("postgres://fula@localhost", SslMode::None)
                    .expect("connect in get_all");
    response.headers_mut().set(header::ContentType::json());

    let mut all = vec![];
    for row in &conn.query("SELECT * FROM GameServer", &[]).expect("query in get_all") {
        all.push(make_game_server_from_row(row))
    }
    response.send(format!("{{\"results\": {}}}", json::encode(&all).expect("couldnt encode")));
}

fn search_server(mut context: Context, mut response: Response) {
    let conn = Connection::connect("postgres://fula@localhost", SslMode::None)
                    .expect("connect in add_server");
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
            conn.query("SELECT * FROM gameserver WHERE region = $1 AND gametype = $2", &[&r, &g])
                .expect("Could not execute query on region and gametype")
        },
        (Some(r), None) => {
            conn.query("SELECT * FROM gameserver WHERE region = $1", &[&r])
                .expect("Could not execute query on region only")
        },
        (None, Some(g)) => {
            conn.query("SELECT * FROM gameserver WHERE gametype = $1", &[&g])
                .expect("could not execute query on gametype only")
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

fn add_server(mut context: Context, mut response: Response) {
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

fn main() {
    env_logger::init().expect("env_logger init");

    let conn = Connection::connect("postgres://fula@localhost", SslMode::None)
                    .expect("connect in main");

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
