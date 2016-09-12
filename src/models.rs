//! Models of the various data structure used in the codebase.

use std::default::Default;

use rustc_serialize::{Decoder, Decodable, DecoderHelpers};
use diesel::ExpressionMethods;

use ::schema::game_servers;
use ::then_impl::Then;

#[derive(Debug, Clone, RustcEncodable, Queryable)]
pub struct Region {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, RustcEncodable, Queryable)]
#[changeset_for(game_servers)]
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

#[derive(RustcEncodable)]
#[insertable_into(game_servers)]
pub struct NewGameServer {
    pub name: String,
    pub region: String,
    pub game_type: String,
    pub ip: String,
    pub max_users: i32,
    pub max_premium_users: Option<i32>,
    pub tags: Vec<String>,
}

#[changeset_for(game_servers)]
pub struct UpdatedGameServer {
    pub name: Option<String>,
    pub region: Option<String>,
    pub game_type: Option<String>,
    pub ip: Option<String>,
    pub max_users: Option<i32>,
    pub max_premium_users: Option<i32>,
    pub tags: Option<Vec<String>>,
}

impl Default for UpdatedGameServer {
    fn default() -> Self {
        UpdatedGameServer {
            name: None,
            region: None,
            game_type: None,
            ip: None,
            max_users: None,
            max_premium_users: None,
            tags: None,
        }
    }
}

impl GameServer {
    pub fn update(&mut self, updated: UpdatedGameServer) {
        updated.name.then(|v| self.name = v);
        updated.region.then(|v| self.region = v);
        updated.game_type.then(|v| self.game_type = v);
        updated.ip.then(|v| self.ip = v);
        updated.max_users.then(|v| self.max_users = v);
        updated.max_premium_users.then(|v| self.max_premium_users = Some(v));
        updated.tags.then(|v| self.tags = v);
    }
}

impl Decodable for NewGameServer {
    fn decode<D: Decoder>(d: &mut D) -> Result<NewGameServer, D::Error> {
        d.read_struct("GameServer", 7, |d| {
            let name = match d.read_struct_field("name", 0, |d| { d.read_str()}) {
                Ok(v) => v,
                Err(_e) => { return Err(d.error("Couldnt Decode a name from GameServer JSON")); },
            };
            let region = match d.read_struct_field("region", 1, |d| { d.read_str()}) {
                Ok(v) => v,
                Err(_e) => { return Err(d.error("Couldnt Decode a region from GameServer JSON")); },
            };
            let game_type = match d.read_struct_field("game_type", 2, |d| { d.read_str()}) {
                Ok(v) => v,
                Err(_e) => { return Err(d.error("Couldnt Decode a game_type from GameServer JSON")); },
            };
            let ip = match d.read_struct_field("ip", 3, |d| { d.read_str()}) {
                Ok(v) => v,
                Err(_e) => { return Err(d.error("Couldnt Decode an IP from GameServer JSON")); },
            };
            let max_users = match d.read_struct_field("max_users", 4, |d| { d.read_i32()}) {
                Ok(v) => v,
                Err(_e) => { return Err(d.error("Couldnt Decode max_users from GameServer JSON")); },
            };
            let max_premium_users: Option<i32> = match d.read_struct_field("max_premium_users",
                                                        5, |d| { d.read_i32()}) {
                Ok(v) => Some(v),
                Err(_e) => None,
            };
            let tags: Vec<String> = match d.read_struct_field("tags", 6, |d| {
                d.read_to_vec(|d| d.read_str())
            }) {
                Ok(v) => v,
                Err(_e) => { return Err(d.error("Could not decode tag as str."))}
            };

            Ok(NewGameServer {
                name: name,
                region: region,
                game_type: game_type,
                ip: ip,
                max_users: max_users,
                max_premium_users: max_premium_users,
                tags: tags
            })
        })
    }
}

Identifiable! {
    #[table_name(game_servers)]
    struct GameServer {
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
}
