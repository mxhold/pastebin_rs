extern crate iron;
extern crate persistent;
extern crate router;
extern crate r2d2;
extern crate r2d2_sqlite;
extern crate rusqlite;
extern crate uuid;

use iron::prelude::*;
use iron::status;
use std::io::Read;
use r2d2_sqlite::SqliteConnectionManager;

type SqlitePool = r2d2::Pool<SqliteConnectionManager>;
type SqlitePooledConnection = r2d2::PooledConnection<SqliteConnectionManager>;

struct ConnectionPool;
impl iron::typemap::Key for ConnectionPool {
    type Value = SqlitePool;
}

fn setup_connection_pool() -> SqlitePool {
    let config = r2d2::Config::default();
    let manager = SqliteConnectionManager::new("./db.sqlite3");
    r2d2::Pool::new(config, manager).unwrap()
}

fn get_connection(req: &mut Request) -> SqlitePooledConnection {
    let pool = req.get::<persistent::Read<ConnectionPool>>().unwrap();
    pool.get().unwrap()
}

fn post_pastebin(req: &mut Request) -> IronResult<Response> {
    let conn = get_connection(req);

    let mut req_body = String::new();
    req.body.read_to_string(&mut req_body).unwrap();

    match insert_paste(&conn, &req_body) {
        Ok(id) => Ok(Response::with((status::Ok, format!("{}{}\n", req.url, id)))),
        Err(_) => Ok(Response::with((status::ServiceUnavailable, ""))),
    }
}

fn get_pastebin(req: &mut Request) -> IronResult<Response> {
    let conn = get_connection(req);

    let id = req.extensions.get::<router::Router>().unwrap().find("id").unwrap();

    match get_paste_body_by_id(&conn, &id) {
        Some(body) => {
            Ok(Response::with((status::Ok, body)))
        }
        None => Ok(Response::with((status::NotFound, ""))),
    }
}

fn get_paste_body_by_id(conn: &SqlitePooledConnection, id: &str) -> Option<String> {
    let query = "SELECT body FROM pastes WHERE id = $1";
    conn.query_row(query, &[&id], |row| row.get(0)).ok()
}

fn insert_paste(conn: &SqlitePooledConnection, body: &str) -> Result<String, rusqlite::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    conn.execute("INSERT INTO pastes VALUES ($1, $2)", &[&id, &body]).and(Ok(id))
}

fn setup_database(conn: &SqlitePooledConnection) {
    conn.execute("DROP TABLE IF EXISTS pastes", &[]).unwrap();
    conn.execute("CREATE TABLE pastes (id TEXT, body BLOB)", &[]).unwrap();
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
