use rusqlite::{Connection, Error, NO_PARAMS};

pub fn find_or_create_author(connection: &Connection, author: &str) -> Result<i64, Error> {
    let mut stmt = connection.prepare_cached("SELECT id FROM authors WHERE author=?1")?;

    stmt.query_row(&[author], |row| row.get(0))
        .or_else(|_| {
            connection
                .prepare_cached("INSERT INTO authors (author) VALUES (?)")
                .map(|mut stmt| stmt.execute(&[author]))
                .map(|_| connection.last_insert_rowid())
        })
        .map_err(|err| err.into())
}

pub fn create_authors_tables(connection: &mut Connection) -> Result<usize, Error> {
    trace!("Creating messages tables");
    connection.execute(
        "CREATE TABLE IF NOT EXISTS authors (
          id INTEGER PRIMARY KEY,
          author TEXT UNIQUE
        )",
        NO_PARAMS,
    )
}

pub fn create_authors_indices(_connection: &Connection) -> Result<usize, Error> {
    Ok(0)
}
