#[macro_use]
extern crate rustful;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate hyper;

use std::error::Error;

use rustful::{Server, Context, Response, TreeRouter};
use hyper::header::ContentType;


fn say_hi(ctx: Context, mut response: Response) {
	response.headers_mut().set(ContentType::json());

	let person = match ctx.variables.get("name") {
		Some(name) => name,
		None => "World".into()
	};

	response.send(r#"{"test": "boop"}"#);
}


fn main() {
    env_logger::init().unwrap();

    let server = Server {
    	host: 8080.into(),
    	handlers: insert_routes!{
    		TreeRouter::new() => {
    			// root
    			Get: say_hi,

    			// next level down
    			":name" => Get: say_hi
    		}
    	},
    	..Server::default()
    }.run();

    match server {
    	Ok(_) => {},
    	Err(e) => error!("could not start server: {}", e.description())
    }
}
