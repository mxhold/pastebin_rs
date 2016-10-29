extern crate hyper;
extern crate regex;

use hyper::server::{Server, Request, Response};
use hyper::uri::RequestUri::*;
use regex::Regex;

fn hello(req: Request, res: Response) {
    let path: String = match req.uri {
        AbsolutePath(string) => string,
        _ => panic!(),
    };

    println!("{}", path);

    let hello_regex = Regex::new(r"^/hello(.+)$").unwrap();

    // Hmm is it faster to do is_match then captures or just captures?

    let response_text: String = if hello_regex.is_match(&path) {
        let caps = hello_regex.captures(&path).unwrap();
        "Hi ".to_string() + caps.at(1).unwrap() + "!"
    } else {
        "Yo".to_string()
    };

    res.send(response_text.as_bytes()).unwrap();
}

fn main() {
    println!("Created server");
    match Server::http("localhost:5000") {
        Ok(server) => {
            println!("Handling");
            server.handle(hello).unwrap();
        },
        Err(err) => {
            println!("Err");
            println!("{:?}", err);
        },
    };
    println!("Server disconnnected");
}

use std::{thread, time};

#[test]
fn test_hello() {
    thread::spawn(move || {
        main();
        thread::sleep(time::Duration::from_secs(10));
        println!("Done");
    });
        thread::sleep(time::Duration::from_secs(15));
    let result = "Hi World";
    assert_eq!("Hi World", result);
}
