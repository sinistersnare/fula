//! Code revolving around the GameServer struct and its use throughout the codebase.
//! The GameServer struct should directly match up to the GameServer table on the Database.
//! Which is why we should be using Diesel.

use postgres::rows::Row;
use rustc_serialize::{Decoder, Decodable, DecoderHelpers};

#[derive(Debug, Clone, RustcEncodable)]
pub struct GameServer {
    pub id: i32,
    pub name: String,
    pub region: String,
    pub game_type: String,
    pub ip: String,
    pub max_users: i32,
    pub current_users: i32,
    pub current_premium_users: Option<i32>,
    pub max_premium_users: Option<i32>,
    pub tags: Vec<String>,
}

impl GameServer {
	pub fn from_row(row: &Row) -> GameServer {
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
