use diesel;
use diesel::prelude::*;

use rustful::{Context, Response, StatusCode};

pub fn delete_server(context: Context, mut response: Response) {
	use ::schema::game_servers::dsl::*;

    let conn = match ::establish_connection() {
        Ok(c) => c,
        Err(e) => {
            error!("Could connect to the DB in add_server: {:?}", e);
            response.set_status(StatusCode::InternalServerError);
            return;
        }
    };
	let server_id: i32 = match context.variables.parse("id") {
		Ok(i) => i,
		Err(err_type) => match err_type {
			Some(a) => {
				error!("The id must be an integer: {:?}", a);
				response.set_status(StatusCode::BadRequest);
				return;
			},
			None => {
				error!("No id provided!");
				response.set_status(StatusCode::BadRequest);
				return;
			}
		}
	};
	match diesel::delete(game_servers.filter(id.eq(server_id))).execute(&conn) {
		Ok(v) => {
			if v != 1 {
				error!("Server does not exist, nothing deleted.");
				response.set_status(StatusCode::BadRequest);
			}
		},
		Err(e) => {
			error!("Encountered an error deleting server id {}: {:?}", server_id, e);
			response.set_status(StatusCode::InternalServerError);
		}
	}
}
