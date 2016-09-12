
use rustful::{Context, Response};

pub fn search_servers(_ctx: Context, response: Response) {
    response.send("Doing exactly 0 searching with your request!");
}
