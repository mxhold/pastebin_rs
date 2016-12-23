extern crate iron;
extern crate router;

use iron::prelude::*;
use router::Router;
use std::io::Read;

fn post_pastebin(req: &mut Request) -> IronResult<Response> {
    let mut req_body = String::new();
    req.body.read_to_string(&mut req_body).unwrap();
    let url = format!("{}123", req.url);
    Ok(Response::with((iron::status::Ok, url)))
}

fn get_pastebin(req: &mut Request) -> IronResult<Response> {
    let id = req.extensions.get::<Router>().unwrap().find("id").unwrap();
    if id == "123" {
        Ok(Response::with((iron::status::Ok, "hi")))
    } else {
        Ok(Response::with((iron::status::NotFound, "Not Found")))
    }
}

fn main() {
    let mut router = Router::new();
    router.post("/", post_pastebin, "post_pastebin");
    router.get("/:id", get_pastebin, "get_pastebin");
    Iron::new(router).http("localhost:3000").unwrap();
}
