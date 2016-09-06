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
use std::io::prelude::*;
use std::fs::File;

use postgres::{Connection, SslMode};

use rustful::{Server, Context, Response, TreeRouter};

use routes::{get_all, search_server, add_server};
mod game_server;
mod routes;


// TODO: Move all these TODOs to an issue tracker.
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
// TODO: game_server: make fields of GameServer private when switched to Diesel.


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
