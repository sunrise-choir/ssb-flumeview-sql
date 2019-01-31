use flume_view_sql::*;
use rusqlite::{Connection, Error, NO_PARAMS};

pub fn create_blob_links_tables(connection: &Connection) -> Result<usize, Error> {
    trace!("Creating blob_links tables");

    connection.execute(
        "CREATE TABLE IF NOT EXISTS blob_links_raw (
          id INTEGER PRIMARY KEY,
          link_from_key_id INTEGER,
          link_to_blob_id INTEGER
        )",
        NO_PARAMS,
    )
}

pub fn create_blob_links_views(connection: &Connection) -> Result<usize, Error> {
    connection.execute(
        "
        CREATE VIEW IF NOT EXISTS blob_links AS
        SELECT 
        blob_links_raw.id as id, 
        blob_links_raw.link_from_key_id as link_from_key_id, 
        blob_links_raw.link_to_blob_id as link_to_blob_id, 
        keys.key as link_from_key, 
        blobs.blob as link_to_blob
        FROM blob_links_raw 
        JOIN keys ON keys.id=blob_links_raw.link_from_key_id
        JOIN blobs ON blobs.id=blob_links_raw.link_to_blob_id
        ",
        NO_PARAMS,
    )
}

pub fn insert_blob_links(
    connection: &Connection,
    links: &[&serde_json::Value],
    message_key_id: i64,
) {
    let mut insert_link_stmt = connection
        .prepare_cached(
            "INSERT INTO blob_links_raw (link_from_key_id, link_to_blob_id) VALUES (?, ?)",
        )
        .unwrap();

    links
        .iter()
        .filter(|link| link.is_string())
        .map(|link| link.as_str().unwrap())
        .filter(|link| link.starts_with("&"))
        .map(|link| find_or_create_blob(&connection, link).unwrap())
        .for_each(|link_id| {
            insert_link_stmt
                .execute(&[&message_key_id, &link_id])
                .unwrap();
        });
}

pub fn create_blob_links_indices(connection: &Connection) -> Result<usize, Error> {
    create_blob_links_index(connection)
}

fn create_blob_links_index(conn: &Connection) -> Result<usize, Error> {
    trace!("Creating blob links index");
    conn.execute(
        "CREATE INDEX IF NOT EXISTS blob_links_index on blob_links_raw (link_to_blob_id, link_from_key_id)",
        NO_PARAMS,
    )
    .map_err(|err| err.into())
}
