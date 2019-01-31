use flume_view_sql::*;
use rusqlite::{Connection, Error, NO_PARAMS};

pub fn create_mentions_tables(connection: &Connection) -> Result<usize, Error> {
    trace!("Creating mentions tables");

    connection.execute(
        "CREATE TABLE IF NOT EXISTS mentions_raw (
          id INTEGER PRIMARY KEY,
          link_from_key_id INTEGER,
          link_to_author_id INTEGER
        )",
        NO_PARAMS,
    )
}

pub fn insert_mentions(connection: &Connection, links: &[&serde_json::Value], message_key_id: i64) {
    let mut insert_link_stmt = connection
        .prepare_cached(
            "INSERT INTO mentions_raw (link_from_key_id, link_to_author_id) VALUES (?, ?)",
        )
        .unwrap();

    links
        .iter()
        .filter(|link| link.is_string())
        .map(|link| link.as_str().unwrap())
        .filter(|link| link.starts_with("@"))
        .map(|link| find_or_create_key(&connection, link).unwrap())
        .for_each(|link_id| {
            insert_link_stmt
                .execute(&[&message_key_id, &link_id])
                .unwrap();
        });
}

pub fn create_mentions_views(connection: &Connection) -> Result<usize, Error> {
    connection.execute(
        "
        CREATE VIEW IF NOT EXISTS mentions AS
        SELECT 
        mentions_raw.id as id, 
        mentions_raw.link_from_key_id as link_from_key_id, 
        mentions_raw.link_to_author_id as link_to_author_id, 
        keys.key as link_from, 
        authors.author as link_to,
        messages_raw.flume_seq as flume_seq
        FROM mentions_raw 
        JOIN keys ON keys.id = mentions_raw.link_from_key_id
        JOIN authors ON authors.id = mentions_raw.link_to_author_id
        JOIN messages_raw ON messages_raw.key_id = mentions_raw.link_from_key_id
        ",
        NO_PARAMS,
    )
}
pub fn create_mentions_indices(connection: &Connection) -> Result<usize, Error> {
    create_mentions_to_index(connection)
}

fn create_mentions_to_index(conn: &Connection) -> Result<usize, Error> {
    trace!("Creating mentions index");
    conn.execute(
        "CREATE INDEX IF NOT EXISTS mentions_id_index on mentions_raw (link_to_author_id, link_from_key_id)",
        NO_PARAMS,
    )
    .map_err(|err| err.into())
}
