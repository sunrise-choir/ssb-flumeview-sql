use flume_view_sql::*;
use rusqlite::{Connection, Error, NO_PARAMS};
use serde_json::Value;

pub fn create_contacts_tables(connection: &mut Connection) -> Result<usize, Error> {
    trace!("Creating contacts tables");
    connection.execute(
        "
    CREATE TABLE IF NOT EXISTS contacts_raw(
        id INTEGER PRIMARY KEY,
        author_id INTEGER,
        contact_author_id INTEGER,
        is_decrypted BOOLEAN,
        state INTEGER
    ) 
    ",
        NO_PARAMS,
    )
}

pub fn insert_or_update_contacts(
    connection: &Connection,
    message: &SsbMessage,
    _message_key_id: i64,
    is_decrypted: bool,
) {
    if message.value.content["type"].as_str() == Some("contact") {
        let is_blocking = message.value.content["blocking"].as_bool().unwrap_or(false);
        let is_following = message.value.content["following"]
            .as_bool()
            .unwrap_or(false);
        let state = if is_blocking {
            -1
        } else if is_following {
            1
        } else {
            0
        };

        if let Value::String(contact) = &message.value.content["contact"] {
            let author_id = find_or_create_author(&connection, &message.value.author).unwrap();
            let mut insert_contacts_stmt = connection
               .prepare_cached("REPLACE INTO contacts_raw (author_id, contact_author_id, state, is_decrypted) VALUES (?, ?, ?, ?)")
               .unwrap();
            let contact_author_id = find_or_create_author(&connection, contact).unwrap();

            insert_contacts_stmt
                .execute(&[
                    &author_id,
                    &contact_author_id,
                    &state,
                    &is_decrypted as &ToSql,
                ])
                .unwrap();
        }
    }
}

pub fn create_contacts_indices(connection: &Connection) -> Result<usize, Error> {
    create_contacts_author_id_index(connection)?;
    create_contacts_state_index(connection)
}

fn create_contacts_state_index(conn: &Connection) -> Result<usize, Error> {
    trace!("Creating contacts state index");
    conn.execute(
        "CREATE INDEX IF NOT EXISTS contacts_raw_state_index on contacts_raw (state)",
        NO_PARAMS,
    )
}
fn create_contacts_author_id_index(conn: &Connection) -> Result<usize, Error> {
    trace!("Creating contacts author_id index");
    conn.execute(
        "CREATE INDEX IF NOT EXISTS contacts_raw_author_id_index on contacts_raw (author_id)",
        NO_PARAMS,
    )
}
