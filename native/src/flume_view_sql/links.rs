use flume_view_sql::*;
use rusqlite::{Connection, Error, NO_PARAMS};

pub fn create_links_tables(connection: &mut Connection) -> Result<usize, Error> {
    trace!("Creating messages tables");

    connection.execute(
        "CREATE TABLE IF NOT EXISTS links_raw (
          id INTEGER PRIMARY KEY,
          link_from_id INTEGER,
          link_to_id INTEGER
        )",
        NO_PARAMS,
    )
}

pub fn insert_links(connection: &Connection, message: &SsbMessage, message_key_id: i64) {
    let mut insert_link_stmt = connection
        .prepare_cached("INSERT INTO links_raw (link_from_id, link_to_id) VALUES (?, ?)")
        .unwrap();

    let mut links = Vec::new();
    find_values_in_object_by_key(&message.value.content, "link", &mut links);

    links
        .iter()
        .filter(|link| link.is_string())
        .map(|link| link.as_str().unwrap())
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
        "CREATE INDEX IF NOT EXISTS links_id_index on links_raw (link_to_id, link_from_id)",
        NO_PARAMS,
    )
    .map_err(|err| err.into())
}
