use flume_view_sql::*;
use rusqlite::{Connection, Error, NO_PARAMS};
use serde_json::Value;

pub fn insert_branches(connection: &Connection, message: &SsbMessage, message_key_id: i64) {
    if let Some(branches_value) = message.value.content.get("branch") {
        let mut insert_branch_stmt = connection
            .prepare_cached(
                "INSERT INTO branches_raw (link_from_key_id, link_to_key_id) VALUES (?, ?)",
            )
            .unwrap();

        let branches = match branches_value {
            Value::Array(arr) => arr
                .iter()
                .map(|value| value.as_str().unwrap().to_string())
                .collect(),
            Value::String(branch) => vec![branch.as_str().to_string()],
            _ => Vec::new(),
        };

        branches
            .iter()
            .map(|branch| find_or_create_key(connection, branch).unwrap())
            .for_each(|link_to_key_id| {
                insert_branch_stmt
                    .execute(&[&message_key_id, &link_to_key_id])
                    .unwrap();
            })
    }
}

pub fn create_branches_tables(connection: &Connection) -> Result<usize, Error> {
    trace!("Creating branches tables");

    connection.execute(
        "CREATE TABLE IF NOT EXISTS branches_raw (
          id INTEGER PRIMARY KEY,
          link_from_key_id INTEGER,
          link_to_key_id INTEGER
        )",
        NO_PARAMS,
    )
}

pub fn create_branches_indices(_connection: &Connection) -> Result<usize, Error> {
    trace!("Creating branches tables");
    Ok(0)
}
