use rusqlite::{Connection, Error, NO_PARAMS};

const MIGRATION_VERSION_NUMBER: u32 = 1;

pub fn create_migrations_tables(connection: &Connection) -> Result<usize, Error> {
    trace!("Creating migrations tables");

    connection.execute(
        "CREATE TABLE IF NOT EXISTS migrations (
          id INTEGER PRIMARY KEY,
          version INTEGER
        )",
        NO_PARAMS,
    )
}

pub fn is_db_up_to_date(connection: &Connection) -> Result<bool, Error> {
    connection
        .query_row_and_then("SELECT version FROM migrations LIMIT 1", NO_PARAMS, |row| {
            row.get_checked(0)
        })
        .map(|version: u32| version == MIGRATION_VERSION_NUMBER)
        .or(Ok(false))
}

pub fn set_db_version(connection: &Connection) -> Result<usize, Error> {
    connection.execute(
        "INSERT INTO migrations (id, version) VALUES(0, ?)",
        &[&MIGRATION_VERSION_NUMBER],
    )
}
