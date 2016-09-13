
use diesel::prelude::*;
use rustful::{Context, Response, header, StatusCode};

use ::models::{UpdatedGameServer, GameServer};

pub fn update_server(mut context: Context, mut response: Response) {
    use ::schema::game_servers::dsl::*;

    let conn = match ::establish_connection() {
        Ok(c) => c,
        Err(e) => {
            error!("Connecting to the DB in update_server failed: {:?}", e);
            response.set_status(StatusCode::InternalServerError);
            return;
        }
    };
    response.headers_mut().set(header::ContentType::json());

    let server_id: i32 = match context.variables.parse::<_, i32>("id") {
        Ok(v) => v,
        Err(Some(e)) => {
            error!("Could not parse the ID as an i32: {:?}", e);
            response.set_status(StatusCode::BadRequest);
            return;
        },
        Err(None) => {
            error!("Another Error occured during context variable parsing.");
            response.set_status(StatusCode::InternalServerError);
            return;
        }
    };
    let mut server: GameServer = match game_servers.filter(id.eq(server_id)).first(&conn) {
        Ok(s) => s,
        Err(e) => {
            error!("Server ID does not exist: {:?}", e);
            response.set_status(StatusCode::BadRequest);
            return;
        }
    };
    let body = match context.body.read_json_body() {
        Ok(b) => b,
        Err(e) => {
            error!("Could not read update_server json body {:?}", e);
            response.set_status(StatusCode::InternalServerError);
            return;
        }
    };

    // FIXME: Need to verify that the id is valid.
    let mut updated_server = UpdatedGameServer::default();
    // FIXME: None of this accounts for parsing errors. DEAL WITH THEM
    updated_server.name =  body.find("name")
                               .and_then(|s| s.as_string())
                               .and_then(|s| Some(s.into()));
    // FIXME: check this against regions_allowed
    updated_server.region =  body.find("region")
                                 .and_then(|s| s.as_string())
                                 .and_then(|s| Some(s.into()));
    updated_server.game_type = body.find("game_type")
                                   .and_then(|s| s.as_string())
                                   .and_then(|s| Some(s.into()));
    updated_server.ip = body.find("ip")
                            .and_then(|s| s.as_string())
                            .and_then(|s| Some(s.into()));
    // FIXME: Casts == bad.
    updated_server.max_users = body.find("max_users")
                                    .and_then(|s| s.as_i64()
                                    .and_then(|v| Some(v as i32)));
    updated_server.max_premium_users = body.find("max_premium_users")
                                           .and_then(|s| s.as_i64()
                                           .and_then(|v| Some(v as i32)));
    match body.find("tags") {
        Some(t) => {
            match t.as_array() {
                Some(vals) => {
                    let mut tgs = vec![];
                    for val in vals {
                        match val.as_string() {
                            Some(s) => {
                                tgs.push(s.into());
                            },
                            None => {
                                error!("found a non-string tag!");
                                response.set_status(StatusCode::BadRequest);
                                return;
                            }
                        }
                    }
                    updated_server.tags = Some(tgs)
                },
                None => {
                    error!("tags is not an array!");
                    response.set_status(StatusCode::BadRequest);
                    return;
                }
            }
        },
        None => {}
    }

    server.update(updated_server);

    // FIXME: cant update tags because thats a lot of work.
    match server.save_changes::<GameServer>(&conn) {
        Ok(_n) => {},
        Err(e) => {
            error!("Unable to update game_servers in update_server: {:?}", e);
            response.set_status(StatusCode::InternalServerError);
            return;
        }
    }

    response.send("\"Update of server was successful\"");
}
