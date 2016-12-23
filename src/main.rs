extern crate iron;
extern crate persistent;
extern crate router;
extern crate r2d2;
extern crate r2d2_sqlite;
extern crate rusqlite;
extern crate uuid;

use iron::prelude::*;
use router::Router;
use std::io::Read;
//use rusqlite::Connection;
use r2d2_sqlite::SqliteConnectionManager;
use uuid::Uuid;

type SqlitePool = r2d2::Pool<SqliteConnectionManager>;
type SqlitePooledConnection = r2d2::PooledConnection<SqliteConnectionManager>;

struct AppDb;
impl iron::typemap::Key for AppDb { type Value = SqlitePool; }

fn setup_connection_pool() -> SqlitePool {
    let config = r2d2::Config::default();
    let manager = r2d2_sqlite::SqliteConnectionManager::new("./db.sqlite3");
    r2d2::Pool::new(config, manager).unwrap()
}

fn post_pastebin(req: &mut Request) -> IronResult<Response> {
    let pool = req.get::<persistent::Read<AppDb>>().unwrap();
    let conn = pool.get().unwrap();

    let mut req_body = String::new();
    req.body.read_to_string(&mut req_body).unwrap();
    let id: String = format!("{}", Uuid::new_v4());
    conn.execute("INSERT INTO pastes VALUES ($1, $2)", &[&id, &req_body]).unwrap();
    let url = format!("{}{}", req.url, id);
    Ok(Response::with((iron::status::Ok, url)))
}

fn get_pastebin(req: &mut Request) -> IronResult<Response> {
    let pool = req.get::<persistent::Read<AppDb>>().unwrap();
    let conn = pool.get().unwrap();

    let id = req.extensions.get::<Router>().unwrap().find("id").unwrap();
    let body: String = conn.query_row("SELECT body FROM pastes WHERE name = $1", &[&id], |row| {
        row.get(0)
    }).unwrap();

    Ok(Response::with((iron::status::Ok, body)))
}

fn main() {
    let mut router = Router::new();
    router.post("/", post_pastebin, "post_pastebin");
    router.get("/:id", get_pastebin, "get_pastebin");

    let pool = setup_connection_pool();

    let conn = pool.get().unwrap();
    conn.execute("DROP TABLE IF EXISTS pastes", &[]).unwrap();
    conn.execute("CREATE TABLE pastes (name TEXT, body BLOB)", &[]).unwrap();

    let mut middleware = Chain::new(router);
    middleware.link(persistent::Read::<AppDb>::both(pool));

    Iron::new(middleware).http("localhost:3000").unwrap();
}
