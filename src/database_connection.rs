extern crate iron;
extern crate uuid;
extern crate persistent;
extern crate rusqlite;
extern crate r2d2;
extern crate r2d2_sqlite;

use iron::prelude::*;
use ::SqlitePooledConnection;
use ::ConnectionPool;

pub struct DatabaseConnection {
    conn: SqlitePooledConnection,
}

impl DatabaseConnection {
    pub fn new(req: &mut Request) -> DatabaseConnection {
        let pool = req.get::<persistent::Read<ConnectionPool>>().unwrap();
        DatabaseConnection { conn: pool.get().unwrap() }
    }

    pub fn get_paste_body_by_id(&self, id: &str) -> Option<String> {
        let query = "SELECT body FROM pastes WHERE id = $1";
        self.conn.query_row(query, &[&id], |row| row.get(0)).ok()
    }

    pub fn insert_paste(&self, body: &str) -> Result<String, rusqlite::Error> {
        let id = uuid::Uuid::new_v4().to_string();
        self.conn.execute("INSERT INTO pastes VALUES ($1, $2)", &[&id, &body]).and(Ok(id))
    }
}
