extern crate iron;
extern crate persistent;
extern crate router;
extern crate r2d2;
extern crate r2d2_sqlite;
extern crate rusqlite;

use iron::prelude::*;
use iron::status;
use std::io::Read;
use r2d2_sqlite::SqliteConnectionManager;

mod database_connection;
use database_connection::DatabaseConnection;

pub type SqlitePool = r2d2::Pool<SqliteConnectionManager>;
pub type SqlitePooledConnection = r2d2::PooledConnection<SqliteConnectionManager>;

pub struct ConnectionPool;
impl iron::typemap::Key for ConnectionPool {
    type Value = SqlitePool;
}

fn setup_connection_pool() -> SqlitePool {
    let config = r2d2::Config::default();
    let manager = SqliteConnectionManager::new("./db.sqlite3");
    r2d2::Pool::new(config, manager).unwrap()
}

fn post_pastebin(req: &mut Request) -> IronResult<Response> {
    let conn = DatabaseConnection::new(req);

    let mut req_body = String::new();
    req.body.read_to_string(&mut req_body).unwrap();

    match conn.insert_paste(&req_body) {
        Ok(id) => Ok(Response::with((status::Ok, format!("{}{}\n", req.url, id)))),
        Err(_) => Ok(Response::with((status::ServiceUnavailable, ""))),
    }
}

fn get_pastebin(req: &mut Request) -> IronResult<Response> {
    let conn = DatabaseConnection::new(req);

    let id = req.extensions.get::<router::Router>().unwrap().find("id").unwrap();

    match conn.get_paste_body_by_id(&id) {
        Some(body) => {
            Ok(Response::with((status::Ok, body)))
        }
        None => Ok(Response::with((status::NotFound, ""))),
    }
}

fn setup_database(conn: &SqlitePooledConnection) {
    conn.execute("CREATE TABLE IF NOT EXISTS pastes (id TEXT, body BLOB)", &[]).unwrap();
}

fn main() {
    let mut router = router::Router::new();
    router.post("/", post_pastebin, "post_pastebin");
    router.get("/:id", get_pastebin, "get_pastebin");

    let pool = setup_connection_pool();
    let conn = pool.get().unwrap();
    setup_database(&conn);

    let mut middleware = Chain::new(router);
    middleware.link_before(persistent::Read::<ConnectionPool>::one(pool));

    Iron::new(middleware).http("localhost:3000").unwrap();
}
