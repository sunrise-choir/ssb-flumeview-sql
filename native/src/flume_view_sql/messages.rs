use rusqlite::types::ToSql;
use rusqlite::{Connection, Error, NO_PARAMS};
use serde_json::Value;

use flume_view_sql::*;

pub fn insert_message(
    connection: &Connection,
    message: &SsbMessage,
    seq: i64,
    message_key_id: i64,
    is_decrypted: bool,
) -> Result<usize, Error> {
    trace!("prepare stmt");
    let mut insert_msg_stmt = connection.prepare_cached("INSERT INTO messages_raw (flume_seq, key_id, seq, received_time, asserted_time, root_id, fork_id, author_id, content_type, content, is_decrypted) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")?;


    trace!("get root key id");
    let root_key_id = match message.value.content["root"] {
        Value::String(ref key) => {
            let id = find_or_create_key(&connection, &key).unwrap();
            Some(id)
        }
        _ => None,
    };

    trace!("get fork key id");
    let fork_key_id = match message.value.content["fork"] {
        Value::String(ref key) => {
            let id = find_or_create_key(&connection, &key).unwrap();
            Some(id)
        }
        _ => None,
    };

    trace!("find or create author");
    let author_id = find_or_create_author(&connection, &message.value.author)?;

    trace!("insert message");
    insert_msg_stmt.execute(&[
        &seq as &ToSql,
        &message_key_id,
        &message.value.sequence,
        &message.timestamp,
        &message.value.timestamp,
        &root_key_id as &ToSql,
        &fork_key_id as &ToSql,
        &author_id,
        &message.value.content["type"].as_str() as &ToSql,
        &message.value.content as &ToSql,
        &is_decrypted as &ToSql,
    ])
}

pub fn create_messages_tables(connection: &mut Connection) -> Result<usize, Error> {
    trace!("Creating messages tables");
    connection.execute(
        "CREATE TABLE IF NOT EXISTS messages_raw (
          flume_seq INTEGER PRIMARY KEY,
          key_id INTEGER UNIQUE, 
          seq INTEGER,
          received_time REAL,
          asserted_time REAL,
          root_id INTEGER,
          fork_id INTEGER,
          author_id INTEGER,
          content_type TEXT,
          content JSON,
          is_decrypted BOOLEAN
        )",
        NO_PARAMS,
    )
}

pub fn create_messages_views(connection: &Connection) -> Result<usize, Error> {
    trace!("Creating messages views");
    connection.execute(
        "
        CREATE VIEW IF NOT EXISTS messages AS
        SELECT 
        flume_seq,
        key_id,
        seq,
        received_time,
        asserted_time,
        root_id,
        fork_id,
        author_id,
        content,
        content_type,
        is_decrypted,
        keys.key as key,
        root_keys.key as root,
        fork_keys.key as fork,
        authors.author as author
        FROM messages_raw 
        JOIN keys ON keys.id=messages_raw.key_id
        LEFT JOIN keys AS root_keys ON root_keys.id=messages_raw.root_id
        LEFT JOIN keys AS fork_keys ON fork_keys.id=messages_raw.fork_id
        JOIN authors ON authors.id=messages_raw.author_id
        ",
        NO_PARAMS,
    )
}

pub fn create_messages_indices(connection: &Connection) -> Result<usize, Error> {
    trace!("Creating messages indices");
    create_content_type_index(&connection)?;
    create_root_index(&connection)?;
    create_author_index(connection)
}

fn create_author_index(connection: &Connection) -> Result<usize, Error> {
    trace!("Creating author index");
    connection.execute(
        "CREATE INDEX IF NOT EXISTS author_id_index on messages_raw (author_id)",
        NO_PARAMS,
    )
}

fn create_root_index(connection: &Connection) -> Result<usize, Error> {
    trace!("Creating root index");
    connection.execute(
        "CREATE INDEX IF NOT EXISTS root_id_index on messages_raw (root_id)",
        NO_PARAMS,
    )
}

fn create_content_type_index(connection: &Connection) -> Result<usize, Error> {
    trace!("Creating content type index");
    connection.execute(
        "CREATE INDEX IF NOT EXISTS content_type_index on messages_raw (content_type)",
        NO_PARAMS,
    )
}
