use flume_view_sql::*;
use rusqlite::{Connection, Error, NO_PARAMS};

pub fn create_links_tables(connection: &Connection) -> Result<usize, Error> {
    trace!("Creating links tables");

    connection.execute(
        "CREATE TABLE IF NOT EXISTS links_raw (
          id INTEGER PRIMARY KEY,
          link_from_key_id INTEGER,
          link_to_key_id INTEGER
        )",
        NO_PARAMS,
    )
}

pub fn create_links_views(connection: &Connection) -> Result<usize, Error> {
    connection.execute(
        "
        CREATE VIEW IF NOT EXISTS links AS
        SELECT 
        links_raw.id as id, 
        links_raw.link_from_key_id as link_from_key_id, 
        links_raw.link_to_key_id as link_to_key_id, 
        keys.key as link_from_key, 
        keys2.key as link_to_key
        FROM links_raw 
        JOIN keys ON keys.id=links_raw.link_from_key_id
        JOIN keys AS keys2 ON keys2.id=links_raw.link_to_key_id
        ",
        NO_PARAMS,
    )
}

pub fn insert_links(connection: &Connection, links: &[&serde_json::Value], message_key_id: i64) {
    let mut insert_link_stmt = connection
        .prepare_cached("INSERT INTO links_raw (link_from_key_id, link_to_key_id) VALUES (?, ?)")
        .unwrap();

    links
        .iter()
        .filter(|link| link.is_string())
        .map(|link| link.as_str().unwrap())
        .filter(|link| link.starts_with('%'))
        .map(|link| find_or_create_key(&connection, link).unwrap())
        .for_each(|link_id| {
            insert_link_stmt
                .execute(&[&message_key_id, &link_id])
                .unwrap();
        });
}

pub fn create_links_indices(connection: &Connection) -> Result<usize, Error> {
    create_links_to_index(connection)
}

fn create_links_to_index(conn: &Connection) -> Result<usize, Error> {
    trace!("Creating links index");
    conn.execute(
        "CREATE INDEX IF NOT EXISTS links_to_id_index on links_raw (link_to_key_id, link_from_key_id)",
        NO_PARAMS,
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS links_from_id_index on links_raw (link_from_key_id, link_to_key_id)",
        NO_PARAMS,
    )
}
