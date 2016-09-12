#![feature(custom_derive, custom_attribute, plugin)]
#![plugin(diesel_codegen, dotenv_macros)]

#[macro_use]
extern crate rustful;
#[macro_use]
extern crate log;
#[macro_use]
extern crate diesel;
extern crate dotenv;

extern crate env_logger;
extern crate hyper;
extern crate rustc_serialize;

use std::error::Error;
use std::env;

// use postgres::{Connection, SslMode};
use rustful::{Server, Context, Response, TreeRouter};

use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;

use routes::{get_all_servers, add_server, update_server, search_servers};

mod schema;
mod models;
mod routes;
mod then_impl;

// TODO: Documentation? Doc comments would be nice.

fn establish_connection() -> Result<PgConnection, &'static str> {
    dotenv().ok();

    let database_url = match env::var("DATABASE_URL") {
        Ok(d) => d,
        Err(_e) => {return Err("DATABASE_URL must be set")},
    };
    match PgConnection::establish(&database_url) {
        Ok(c) => Ok(c),
        Err(_e) => Err("Error connecting to DATABASE_URL"),
    }
}

fn main() {
    env_logger::init().expect("env_logger init");

    let server = Server {
        host: 8080.into(),
        handlers: insert_routes!{
            TreeRouter::new() => {
                // root
                Get: get_all_servers as fn(Context, Response),

                // next level down
                "all" => {
                    Get: get_all_servers as fn(Context, Response),
                },
                "search" => {
                    Post: search_servers as fn(Context, Response),
                },
                "add" => {
                    Post: add_server as fn(Context, Response),
                },
                "update/:id" => {
                    Post: update_server as fn(Context, Response),
                }
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
