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

pub type SqlitePool = r2d2::Pool<SqliteConnectionManager>;
pub type SqlitePooledConnection = r2d2::PooledConnection<SqliteConnectionManager>;

pub struct ConnectionPool;
impl iron::typemap::Key for ConnectionPool {
    type Value = SqlitePool;
}

struct DatabaseConnection {
    conn: SqlitePooledConnection,
}

impl DatabaseConnection {
    fn new(pool: &SqlitePool) -> DatabaseConnection {
        DatabaseConnection { conn: pool.get().unwrap() }
    }

    fn setup_database(&self) -> Result<i32, rusqlite::Error> {
        let query = "CREATE TABLE IF NOT EXISTS pastes (id TEXT, body BLOB)";
        self.conn.execute(query, &[])
    }

    fn insert_paste(&self, body: &str) -> Result<String, rusqlite::Error> {
        let id = uuid::Uuid::new_v4().to_string();
        let query = "INSERT INTO pastes VALUES ($1, $2)";
        self.conn.execute(query, &[&id, &body]).and(Ok(id))
    }

    fn get_paste_body_by_id(&self, id: &str) -> Option<String> {
        let query = "SELECT body FROM pastes WHERE id = $1";
        self.conn.query_row(query, &[&id], |row| row.get(0)).ok()
    }
}

fn post_pastebin(req: &mut Request) -> IronResult<Response> {
    let pool = req.get::<persistent::Read<ConnectionPool>>().unwrap();
    let conn = DatabaseConnection::new(&pool);

    let mut req_body = String::new();
    req.body.read_to_string(&mut req_body).unwrap();

    match conn.insert_paste(&req_body) {
        Ok(id) => Ok(Response::with((status::Ok, format!("{}{}\n", req.url, id)))),
        Err(_) => Ok(Response::with((status::ServiceUnavailable, ""))),
    }
}

fn get_pastebin(req: &mut Request) -> IronResult<Response> {
    let pool = req.get::<persistent::Read<ConnectionPool>>().unwrap();
    let conn = DatabaseConnection::new(&pool);

    let id = req.extensions.get::<router::Router>().unwrap().find("id").unwrap();

    match conn.get_paste_body_by_id(&id) {
        Some(body) => Ok(Response::with((status::Ok, body))),
        None => Ok(Response::with((status::NotFound, ""))),
    }
}

fn main() {
    let mut router = router::Router::new();
    router.post("/", post_pastebin, "post_pastebin");
    router.get("/:id", get_pastebin, "get_pastebin");

    let config = r2d2::Config::default();
    let manager = SqliteConnectionManager::new("./db.sqlite3");
    let pool = r2d2::Pool::new(config, manager).unwrap();
    DatabaseConnection::new(&pool).setup_database().unwrap();

    let mut middleware = Chain::new(router);
    middleware.link_before(persistent::Read::<ConnectionPool>::one(pool));

    Iron::new(middleware).http("localhost:3000").unwrap();
}
