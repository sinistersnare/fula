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

use std::error::Error;
use std::collections::HashSet;
use std::io::prelude::*;
use std::fs::File;

use postgres::{Connection, SslMode};

use rustful::{Server, Context, Response, TreeRouter, header};

use rustc_serialize::json;

// TODO: Connection pooling?
// TODO: Do not allow duplicate IP addresses -- make an UPDATE call instead
// TODO: Have HashSets of possible GameTypes and Locations and check against them.
// TODO: Descriptive error messages on not providing fields for /add
// TODO: Do not panic! return appropriate HTTP codes.
// TODO: Flesh out search functionality
// TODO: Thorough logging
// TODO: Have a table of acceptable regions and gametypes
// TODO: Link the DB tables, make GameServer use Region and GameType
// TODO: Execute all sql files in bin/ at start of program.

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
struct GameServer {
    id: i32,
    name: String,
    location: String,
    gametype: String,
    ip: String,
}

fn is_region_allowed(region: &str) -> bool {

    let conn = Connection::connect("postgres://fula@localhost", SslMode::None).expect("connect in say_hi");
    let all_regions: HashSet<String> = conn.query("SELECT name FROM region", &[]).expect("could not select all regions.")
                                       .iter().map(|v| v.get::<usize, String>(0))
                                       .collect();

    // TODO: take an iterator of Strings and check them all.
    // TODO: list all regions that dont exist, not just the first one found.

    if !all_regions.contains(region) {
        return false;
    }
    return true;
}

fn get_all(_ctx: Context, mut response: Response) {
    let conn = Connection::connect("postgres://fula@localhost", SslMode::None).expect("connect in say_hi");
    response.headers_mut().set(header::ContentType::json());

    let mut all = vec![];
    for row in &conn.query("SELECT id, name, location, gametype, ip FROM GameServer", &[]).expect("query in say_hi") {
        let g = GameServer {
            id: row.get(0),
            name: row.get(1),
            location: row.get(2),
            gametype: row.get(3),
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

    let search_regions: Vec<String> = body.find("regions")
                                    .and_then(|r| r.as_array()).expect("not an array.")
                                    .iter().map(|r| r.as_string().expect("not a string").into())
                                    .collect();

    for region in &search_regions {
        if !is_region_allowed(region) {
            response.send(format!("\"Region `{}` does not exist in the Database!\"", region));
            return;
        }
    }

    response.send(format!("Regions asked for: {:?}", search_regions));
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

    if !is_region_allowed(&region) {
        response.send(format!("\"Region `{}` does not exist in the Database!\"", region));
        return;
    }

    conn.execute("INSERT INTO GameServer (name, location, gametype, ip) VALUES ($1, $2, $3, $4)",
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
    // conn.execute(&contents, &[]).expect("Running create_tables.sql");
    // conn.execute("CREATE TABLE IF NOT EXISTS GameServer (
    //                 id          SERIAL PRIMARY KEY,
    //                 name        VARCHAR NOT NULL,
    //                 location    VARCHAR NOT NULL,
    //                 gametype    VARCHAR NOT NULL,
    //                 ip          VARCHAR NOT NULL
    //              )", &[]).expect("create table");

    let server = Server {
        host: 8080.into(),
        handlers: insert_routes!{
            TreeRouter::new() => {
                // root
                Get: get_all as fn(Context, Response),

                // next level down
                "all" => Get: get_all as fn(Context, Response),
                "search" => Post: search_server as fn(Context, Response),
                "add" => Post: add_server as fn(Context, Response),
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
