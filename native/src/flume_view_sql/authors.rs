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
    trace!("Creating authors tables");
    connection.execute(
        "CREATE TABLE IF NOT EXISTS authors (
          id INTEGER PRIMARY KEY,
          author TEXT UNIQUE,
          is_me BOOLEAN 
        )",
        NO_PARAMS,
    )
}

pub fn set_author_that_is_me(connection: &Connection, my_key: &str) -> Result<usize, Error> {
    let my_key_id = find_or_create_author(connection, my_key)?;
    connection.execute("UPDATE authors SET is_me = 1 WHERE id = ?", &[&my_key_id])
}

pub fn create_authors_indices(connection: &Connection) -> Result<usize, Error> {
    create_is_me_index(connection)
}

fn create_is_me_index(connection: &Connection) -> Result<usize, Error>{
    trace!("Creating is_me index");
    connection.execute(
        "CREATE INDEX IF NOT EXISTS authors_is_me_index ON authors (is_me)",
        NO_PARAMS,
    )
}
