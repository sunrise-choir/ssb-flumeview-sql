use rusqlite::{Connection, Error, NO_PARAMS};

pub fn find_or_create_key(connection: &Connection, key: &str) -> Result<i64, Error> {
    let mut stmt = connection.prepare_cached("SELECT id FROM keys WHERE key=?1")?;

    stmt.query_row(&[key], |row| row.get(0))
        .or_else(|_| {
            connection
                .prepare_cached("INSERT INTO keys (key) VALUES (?)")
                .map(|mut stmt| stmt.execute(&[key]))
                .map(|_| connection.last_insert_rowid())
        })
        .map_err(|err| err.into())
}

pub fn create_keys_tables(connection: &Connection) -> Result<usize, Error> {
    trace!("Creating messages tables");
    connection.execute(
        "CREATE TABLE IF NOT EXISTS keys (
          id INTEGER PRIMARY KEY,
          key TEXT UNIQUE
        )",
        NO_PARAMS,
    )
}

pub fn create_keys_indices(_connection: &Connection) -> Result<usize, Error> {
    Ok(0)
}
