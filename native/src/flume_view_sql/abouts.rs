use flume_view_sql::*;
use rusqlite::{Connection, Error, NO_PARAMS};
use rusqlite::types::Null;

pub fn create_abouts_tables(connection: &Connection) -> Result<usize, Error> {
    trace!("Creating abouts tables");

    connection.execute(
        "CREATE TABLE IF NOT EXISTS abouts_raw (
          id INTEGER PRIMARY KEY,
          link_from_key_id INTEGER,
          link_to_author_id INTEGER,
          link_to_key_id INTEGER,
          link_to_blob_id INTEGER
        )",
        NO_PARAMS,
    )
}

pub fn insert_abouts(
    connection: &Connection,
    message: &SsbMessage,
    message_key_id: i64,
) {
    if let Value::String(about_key) = &message.value.content["about"] {
        let mut key;

        let (link_to_author_id, link_to_key_id, link_to_blob_id): (&ToSql,&ToSql,&ToSql ) = match about_key.get(0..1) {
            Some("@") => {
                key = find_or_create_author(connection, about_key).unwrap();
                (&key, &Null, &Null)
            },
            Some("%") => {
                key = find_or_create_key(connection, about_key).unwrap();
                (&Null, &key, &Null)
            },
            Some("&") => {
                //TODO;
                //let key = find_or_create_author(connection, about_key).unwrap();
                (&Null, &Null, &Null)
            },
            _ => (&Null, &Null, &Null) 
        };

        let mut insert_abouts_stmt = connection
            .prepare_cached("INSERT INTO abouts_raw (link_from_key_id, link_to_author_id, link_to_key_id, link_to_blob_id) VALUES (?, ?, ?, ?)")
            .unwrap();

        insert_abouts_stmt
            .execute(&[&message_key_id, link_to_author_id, link_to_key_id, link_to_blob_id])
            .unwrap();
    }
}

pub fn create_abouts_indices(connection: &Connection) -> Result<usize, Error> {
    trace!("Creating abouts index");
    connection.execute(
        "CREATE INDEX IF NOT EXISTS abouts_raw_index on abouts_raw (link_from_key_id, link_to_key_id, link_to_author_id, link_to_blob_id)",
        NO_PARAMS,
    )
}
