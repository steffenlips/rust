use rusqlite::{Connection, Result};

pub fn create_connection_and_table() -> Result<Connection> {
    let conn = Connection::open_in_memory()?;

    conn.execute(
        "CREATE TABLE person (
                id    INTEGER PRIMARY KEY,
                name  TEXT NOT NULL,
                password  TEXT
            )",
        [],
    )?;

    Ok(conn)
}
