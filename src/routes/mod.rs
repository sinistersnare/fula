//! Routes used by the REST API

use std::collections::HashSet;

use diesel::prelude::*;
use diesel::pg::PgConnection;

use models::Region;

pub mod server;
pub mod region;

pub enum AllowedRegion<T> {
    Failure(T),
    Success,
    Panic,
}

pub fn regions_allowed<'a, 'b, I>(conn: &'a PgConnection, possible_regions: I)
                    -> AllowedRegion<HashSet<&'b str>> where I: Iterator<Item=&'b str> {
    use ::schema::regions::dsl::*;

    let all_regions: HashSet<String> = match regions.load::<Region>(conn) {
        Ok(c) => c.into_iter().map(|v| v.name).collect(),
        Err(e) => {
            error!("Could not execute query in regions_allowed: {:?}", e);
            return AllowedRegion::Panic;
        }
    };

    let failed: HashSet<&'b str> = possible_regions.filter(|r| !all_regions.contains(*r)).collect();
    if failed.len() == 0 {
        AllowedRegion::Success
    } else {
        AllowedRegion::Failure(failed)
    }
}

