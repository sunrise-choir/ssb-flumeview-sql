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

pub fn create_abouts_views(connection: &Connection) -> Result<usize, Error> {
    trace!("Creating abouts views");
    //resolve all the links, get the content of the message.
    connection.execute(
        "
        CREATE VIEW IF NOT EXISTS abouts AS
        SELECT 
        abouts_raw.id as id, 
        abouts_raw.link_from_key_id as link_from_key_id, 
        abouts_raw.link_to_key_id as link_to_key_id, 
        abouts_raw.link_to_author_id as link_to_author_id, 
        abouts_raw.link_to_blob_id as link_to_blob_id, 
        keys_from.key as link_from_key, 
        keys_to.key as link_to_key, 
        authors_to.author as link_to_author,
        messages.content as content
        FROM abouts_raw 
        JOIN keys AS keys_from ON keys_from.id=abouts_raw.link_from_key_id
        JOIN messages ON link_from_key_id=messages.key_id
        LEFT JOIN keys AS keys_to ON keys_to.id=abouts_raw.link_to_key_id
        LEFT JOIN authors AS authors_to ON authors_to.id=abouts_raw.link_to_author_id
        ",
        NO_PARAMS,
    )
}


