use rusqlite::{Connection, Error, NO_PARAMS};

pub fn create_branches_tables(connection: &Connection) -> Result<usize, Error> {
    trace!("Creating branches tables");

    connection.execute(
        "CREATE TABLE IF NOT EXISTS branches_raw (
          id INTEGER PRIMARY KEY,
          link_from_key_id INTEGER,
          link_to_key_id INTEGER
        )",
        NO_PARAMS,
    )
}

pub fn create_branches_indices(_connection: &Connection) -> Result<usize, Error> {
    trace!("Creating branches tables");
    Ok(0)
}

