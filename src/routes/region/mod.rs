
use diesel;
use diesel::prelude::*;
use rustful::{Context, Response, header, StatusCode};
use rustc_serialize::json;

use ::establish_connection;
use ::models::{NewRegion, Region};

pub fn get_all_regions(_context: Context, mut response: Response) {
	use ::schema::regions::dsl::*;

	let conn = match establish_connection() {
		Ok(c) => c,
		Err(e) => {
			error!("Could not establish connection to DB: {}", e);
			response.set_status(StatusCode::InternalServerError);
			return;
		}
	};

	response.headers_mut().set(header::ContentType::json());

	let all: Vec<Region> = match regions.load(&conn) {
		Ok(r) => r,
		Err(e) => {
            error!("Could not execute query query in get_all_regions: {:?}", e);
            response.set_status(StatusCode::InternalServerError);
            return;
		}
	};

	let encoded = match json::encode(&all) {
		Ok(v) => v,
		Err(e) => {
            error!("Could not encode results of get_all_regions query as json: {:?}", e);
            response.set_status(StatusCode::InternalServerError);
            return;
		}
	};

	response.send(format!("{{\"results\": {}, \"size\": {}}}", encoded, all.len()));
}

pub fn add_region(mut context: Context, mut response: Response) {
	use ::schema::regions;
	let conn = match establish_connection() {
		Ok(c) => c,
		Err(e) => {
			error!("Could not establish connection to DB: {}", e);
			response.set_status(StatusCode::InternalServerError);
			return;
		}
	};

	let body = match context.body.read_json_body() {
		Ok(b) => b,
		Err(e) => {
			error!("Could not decode JSON body in add_region: {}", e);
			response.set_status(StatusCode::BadRequest);
			return;
		}
	};
	let region_name: String = match body.find("name") {
		Some(v) => match v.as_string() {
			Some(s) => {
				s.into()
			},
			None => {
				error!("parameter `name` needs to be a JSON string.");
				response.set_status(StatusCode::BadRequest);
				return;
			}
		},
		None => {
			error!("add_region must have a name parameter.");
			response.set_status(StatusCode::BadRequest);
			return;
		}
	};
	let new_region = NewRegion {
		name: region_name,
	};
	match diesel::insert(&new_region).into(regions::table).execute(&conn) {
		Ok(_) => {},
		Err(e) => {
			error!("Failed to insert new region into regions: {:?}", e);
			response.set_status(StatusCode::InternalServerError);
			return;
		}
	}
	response.send(format!("\"Region `{}` added to DB!\"", &new_region.name));
}
