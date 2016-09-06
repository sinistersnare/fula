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

// TODO: Documentation? Doc comments would be nice.

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
