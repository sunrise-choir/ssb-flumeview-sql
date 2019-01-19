use failure::Error;
use flumedb::flume_view::*;

use base64::decode;
use rusqlite::types::ToSql;
use rusqlite::OpenFlags;
use rusqlite::{Connection, NO_PARAMS};
use serde_json::Value;

use private_box::SecretKey;

use log;

#[derive(Serialize, Deserialize, Debug)]
struct SsbValue {
    author: String,
    sequence: u32,
    timestamp: f64,
    content: Value,
}

#[derive(Serialize, Deserialize, Debug)]
struct SsbMessage {
    key: String,
    value: SsbValue,
    timestamp: f64,
}

#[derive(Debug, Fail)]
pub enum FlumeViewSqlError {
    #[fail(display = "Db failed integrity check")]
    DbFailedIntegrityCheck {},
}

pub struct FlumeViewSql {
    connection: Connection,
    secret_keys: Vec<SecretKey>,
}

impl FlumeView for FlumeViewSql {
    fn append(&mut self, seq: Sequence, item: &[u8]) {
        append_item(&self.connection, &self.secret_keys, seq, item).unwrap()
    }
    fn latest(&self) -> Sequence {
        self.get_latest().unwrap()
    }
}

impl FlumeViewSql {
    pub fn new(path: &str, secret_keys: Vec<SecretKey>) -> FlumeViewSql {
        //let mut connection = Connection::open(path).expect("unable to open sqlite connection");
        let flags: OpenFlags = OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX;
        let mut connection =
            Connection::open_with_flags(path, flags).expect("unable to open sqlite connection");

        set_pragmas(&mut connection);
        create_tables(&mut connection);
        create_indices(&connection);

        FlumeViewSql {
            connection,
            secret_keys,
        }
    }

    pub fn get_seq_by_key(&mut self, key: String) -> Result<i64, Error> {
        let mut stmt = self
            .connection
            .prepare("SELECT flume_seq FROM messages_raw JOIN keys ON messages_raw.key_id=keys.id WHERE keys.key=?1")?;

        stmt.query_row(&[key], |row| row.get(0))
            .map_err(|err| err.into())
    }

    pub fn get_seqs_by_type(&mut self, content_type: String) -> Result<Vec<i64>, Error> {
        let mut stmt = self
            .connection
            .prepare("SELECT flume_seq FROM messages_raw WHERE content_type=?1")?;

        let rows = stmt.query_map(&[content_type], |row| row.get(0))?;

        let seqs = rows.fold(Vec::<i64>::new(), |mut vec, row| {
            vec.push(row.unwrap());
            vec
        });

        Ok(seqs)
    }

    pub fn append_batch(&mut self, items: Vec<(Sequence, Vec<u8>)>) {
        info!("Start batch append");
        let tx = self.connection.transaction().unwrap();

        for item in items {
            append_item(&tx, &self.secret_keys, item.0, &item.1).unwrap();
        }

        tx.commit().unwrap();
    }

    pub fn check_db_integrity(&mut self) -> Result<(), Error> {
        self.connection
            .query_row_and_then("PRAGMA integrity_check", NO_PARAMS, |row| {
                row.get_checked(0)
                    .map_err(|err| err.into())
                    .and_then(|res: String| {
                        if res == "ok" {
                            return Ok(());
                        }
                        return Err(FlumeViewSqlError::DbFailedIntegrityCheck {}.into());
                    })
            })
    }

    pub fn get_latest(&self) -> Result<Sequence, Error> {
        info!("Getting latest seq from db");

        let mut stmt = self
            .connection
            .prepare_cached("SELECT MAX(flume_seq) FROM messages")?;

        stmt.query_row(NO_PARAMS, |row| {
            let res: i64 = row.get_checked(0).unwrap_or(0);
            res as Sequence
        })
        .map_err(|err| err.into())
    }
}

fn find_values_in_object_by_key(
    obj: &serde_json::Value,
    key: &str,
    values: &mut Vec<serde_json::Value>,
) {
    match obj.get(key) {
        Some(val) => values.push(val.clone()),
        _ => (),
    };

    match obj {
        Value::Array(arr) => {
            for val in arr {
                find_values_in_object_by_key(val, key, values);
            }
        }
        Value::Object(kv) => {
            for val in kv.values() {
                match val {
                    Value::Object(_) => find_values_in_object_by_key(val, key, values),
                    Value::Array(_) => find_values_in_object_by_key(val, key, values),
                    _ => (),
                }
            }
        }
        _ => (),
    }
}

fn attempt_decryption(mut message: SsbMessage, secret_keys: &[SecretKey]) -> (bool, SsbMessage) {
    let mut is_decrypted = false;

    message = match message.value.content["type"] {
        Value::Null => {
            let content = message.value.content.clone();
            let strrr = &content.as_str().unwrap().trim_end_matches(".box");

            let bytes = decode(strrr).unwrap();

            message.value.content = secret_keys
                .get(0)
                .ok_or(())
                .and_then(|key| private_box::decrypt(&bytes, key))
                .and_then(|data| {
                    is_decrypted = true;
                    serde_json::from_slice(&data).map_err(|_| ())
                })
                .unwrap_or(Value::Null); //If we can't decrypt it, throw it away.

            message
        }
        _ => message,
    };

    (is_decrypted, message)
}

fn insert_links(connection: &Connection, message: &SsbMessage, message_key_id: i64) {
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

fn insert_message(connection: &Connection, message: &SsbMessage, seq: i64, message_key_id: i64, is_decrypted: bool){

    let mut insert_msg_stmt = connection.prepare_cached("INSERT INTO messages_raw (flume_seq, key_id, seq, received_time, asserted_time, root_id, fork_id, author_id, content_type, content, is_decrypted) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)").unwrap();

    let root_key_id = match message.value.content["root"] {
        Value::String(ref key) => {
            let id = find_or_create_key(&connection, &key).unwrap();
            Some(id)
        }
        _ => None,
    };

    let fork_key_id = match message.value.content["fork"] {
        Value::String(ref key) => {
            let id = find_or_create_key(&connection, &key).unwrap();
            Some(id)
        }
        _ => None,
    };


    let author_id = find_or_create_author(&connection, &message.value.author).unwrap();
    insert_msg_stmt
        .execute(&[
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
        .unwrap();

}

fn append_item(
    connection: &Connection,
    secret_keys: &[SecretKey],
    seq: Sequence,
    item: &[u8],
) -> Result<(), Error> {

    let message: SsbMessage = serde_json::from_slice(item).unwrap();

    let (is_decrypted, message) = attempt_decryption(message, secret_keys);

    let message_key_id = find_or_create_key(&connection, &message.key).unwrap();

    insert_links(connection, &message, message_key_id);
    insert_message(connection, &message, seq as i64, message_key_id, is_decrypted);

    Ok(())
}

fn set_pragmas(conn: &mut Connection) {
    conn.execute("PRAGMA synchronous = OFF", NO_PARAMS).unwrap();
    conn.execute("PRAGMA page_size = 8192", NO_PARAMS).unwrap();
}

fn find_or_create_author(conn: &Connection, author: &str) -> Result<i64, Error> {
    let mut stmt = conn.prepare_cached("SELECT id FROM authors WHERE author=?1")?;

    stmt.query_row(&[author], |row| row.get(0))
        .or_else(|_| {
            conn.prepare_cached("INSERT INTO authors (author) VALUES (?)")
                .map(|mut stmt| stmt.execute(&[author]))
                .map(|_| conn.last_insert_rowid())
        })
        .map_err(|err| err.into())
}

fn find_or_create_key(conn: &Connection, key: &str) -> Result<i64, Error> {
    let mut stmt = conn.prepare_cached("SELECT id FROM keys WHERE key=?1")?;

    stmt.query_row(&[key], |row| row.get(0))
        .or_else(|_| {
            conn.prepare_cached("INSERT INTO keys (key) VALUES (?)")
                .map(|mut stmt| stmt.execute(&[key]))
                .map(|_| conn.last_insert_rowid())
        })
        .map_err(|err| err.into())
}

fn create_author_index(conn: &Connection) -> Result<usize, Error> {
    info!("Creating author index");
    conn.execute(
        "CREATE INDEX author_id_index on messages_raw (author_id)",
        NO_PARAMS,
    )
    .map_err(|err| err.into())
}

fn create_root_index(conn: &Connection) -> Result<usize, Error> {
    info!("Creating root index");
    conn.execute(
        "CREATE INDEX root_id_index on messages_raw (root_id)",
        NO_PARAMS,
    )
    .map_err(|err| err.into())
}

fn create_links_to_index(conn: &Connection) -> Result<usize, Error> {
    info!("Creating links index");
    conn.execute(
        "CREATE INDEX links_to_id_index on links (link_to_id)",
        NO_PARAMS,
    )
    .map_err(|err| err.into())
}

fn create_content_type_index(conn: &Connection) -> Result<usize, Error> {
    info!("Creating content type index");
    conn.execute(
        "CREATE INDEX content_type_index on messages_raw (content_type)",
        NO_PARAMS,
    )
    .map_err(|err| err.into())
}

fn create_tables(conn: &mut Connection) {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS messages_raw (
          flume_seq INTEGER PRIMARY KEY,
          key_id INTEGER, 
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
    .unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS authors (
          id INTEGER PRIMARY KEY,
          author TEXT UNIQUE
        )",
        NO_PARAMS,
    )
    .unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS keys (
          id INTEGER PRIMARY KEY,
          key TEXT UNIQUE
        )",
        NO_PARAMS,
    )
    .unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS links_raw (
          id INTEGER PRIMARY KEY,
          link_from_id INTEGER,
          link_to_id INTEGER
        )",
        NO_PARAMS,
    )
    .unwrap();

    conn.execute(
        "
        CREATE VIEW links AS
        SELECT 
        links_raw.id as id, 
        links_raw.link_from_id as link_from_id, 
        links_raw.link_to_id as link_to_id, 
        keys.key as link_from, 
        keys2.key as link_to
        FROM links_raw 
        JOIN keys ON keys.id=links_raw.link_from_id
        JOIN keys AS keys2 ON keys2.id=links_raw.link_to_id
        ",
        NO_PARAMS,
    )
    .unwrap();

    conn.execute(
        "
        CREATE VIEW messages AS
        SELECT 
        messages_raw.flume_seq as flume_seq,
        messages_raw.key_id as key_id,
        messages_raw.seq as seq,
        messages_raw.received_time as received_time,
        messages_raw.asserted_time as asserted_time,
        messages_raw.root_id as root_id,
        messages_raw.fork_id as fork_id,
        messages_raw.author_id as author_id,
        messages_raw.content as content,
        messages_raw.content_type as content_type,
        messages_raw.is_decrypted as is_decrypted,
        keys.key as key,
        keys2.key as root,
        keys3.key as fork,
        authors.author as author
        FROM messages_raw 
        JOIN keys ON keys.id=messages_raw.key_id
        JOIN keys AS keys2 ON keys2.id=messages_raw.root_id
        JOIN keys AS keys3 ON keys3.id=messages_raw.fork_id
        JOIN authors ON authors.id=messages_raw.author_id
        ",
        NO_PARAMS,
    )
    .unwrap();

    conn.execute(
        "
    CREATE TABLE contacts(
        id INTEGER PRIMARY KEY,
        author_id INTEGER,
        contact_author_id INTEGER,
        state INTEGER
    ) 
    ",
        NO_PARAMS,
    )
    .unwrap();


    conn.execute(
        "
    CREATE TRIGGER contacts_block_trigger AFTER INSERT ON messages_raw
    WHEN NEW.content_type='contact'
    AND json_extract(NEW.content, '$.blocking')=true
    BEGIN
    REPLACE INTO contacts (author_id, contact_author_id, state)
    VALUES
    (NEW.author_id, json_extract(NEW.content, '$.contact'), -1);
    END
    ",
        NO_PARAMS,
    )
    .unwrap();

    conn.execute(
        "
    CREATE TRIGGER contacts_neutral_trigger AFTER INSERT ON messages_raw
    WHEN NEW.content_type='contact'
    AND json_extract(NEW.content, '$.blocking')=false
    AND json_extract(NEW.content, '$.following')=false
    BEGIN
    DELETE FROM contacts WHERE author_id=NEW.author_id AND contact_author_id=json_extract(NEW.content, '$.contact');
    END
    ",
        NO_PARAMS,
    )
    .unwrap();

    conn.execute(
        "
    CREATE TRIGGER contacts_follow_trigger AFTER INSERT ON messages_raw
    WHEN NEW.content_type='contact'
    AND json_extract(NEW.content, '$.blocking')=false
    AND json_extract(NEW.content, '$.following')=true
    BEGIN
    REPLACE INTO contacts (author_id, contact_author_id, state)
    VALUES
    (NEW.author_id, json_extract(NEW.content, '$.contact'), 1);
    END
    ",
    NO_PARAMS,
    )
    .unwrap();


}

fn create_indices(connection: &Connection) {
    create_author_index(&connection)
        .and_then(|_| create_links_to_index(&connection))
        .and_then(|_| create_content_type_index(&connection))
        .and_then(|_| create_root_index(&connection))
        .map(|_| ())
        .unwrap_or_else(|_| ());
}

#[cfg(test)]
mod test {
    use flume_view_sql::*;
    use flumedb::flume_view::*;
    use serde_json::*;

    #[test]
    fn find_values_in_object() {
        let obj = json!({ "key": 1, "value": {"link": "hello", "array": [{"link": "piet"}], "deeper": {"link": "world"}}});

        let mut vec = Vec::new();
        find_values_in_object_by_key(&obj, "link", &mut vec);

        assert_eq!(vec.len(), 3);
        assert_eq!(vec[0].as_str().unwrap(), "hello");
        assert_eq!(vec[1].as_str().unwrap(), "piet");
        assert_eq!(vec[2].as_str().unwrap(), "world");
    }

    #[test]
    fn open_connection() {
        let filename = "/tmp/test123456.sqlite3";
        let keys = Vec::new();
        std::fs::remove_file(filename.clone())
            .or::<Result<()>>(Ok(()))
            .unwrap();
        FlumeViewSql::new(filename, keys);
        assert!(true)
    }

    #[test]
    fn append() {
        let expected_seq = 1234;
        let filename = "/tmp/test12345.sqlite3";
        let keys = Vec::new();
        std::fs::remove_file(filename.clone())
            .or::<Result<()>>(Ok(()))
            .unwrap();

        let mut view = FlumeViewSql::new(filename, keys);
        let jsn = r#####"{
  "key": "%KKPLj1tWfuVhCvgJz2hG/nIsVzmBRzUJaqHv+sb+n1c=.sha256",
  "value": {
    "previous": "%xsMQA2GrsZew0GSxmDSBaoxDafVaUJ07YVaDGcp65a4=.sha256",
    "author": "@QlCTpvY7p9ty2yOFrv1WU1AE88aoQc4Y7wYal7PFc+w=.ed25519",
    "sequence": 4797,
    "timestamp": 1543958997985,
    "hash": "sha256",
    "content": {
      "type": "post",
      "root": "%9EdpeKC5CgzpQs/x99CcnbD3n6ugUlwm19F7ZTqMh5w=.sha256",
      "branch": "%sQV8QpyUNvh7fBAs2ts00Qo2gj44CQBmwonWJzm+AeM=.sha256",
      "reply": {
        "%9EdpeKC5CgzpQs/x99CcnbD3n6ugUlwm19F7ZTqMh5w=.sha256": "@+UMKhpbzXAII+2/7ZlsgkJwIsxdfeFi36Z5Rk1gCfY0=.ed25519",
        "%sQV8QpyUNvh7fBAs2ts00Qo2gj44CQBmwonWJzm+AeM=.sha256": "@vzoU7/XuBB5B0xueC9NHFr9Q76VvPktD9GUkYgN9lAc=.ed25519"
      },
      "channel": null,
      "recps": null,
      "text": "If I understand correctly, cjdns overlaying over old IP (which is basically all of the cjdns uses so far) still requires old IP addresses to introduce you to the cjdns network, so the chicken and egg problem is still there.",
      "mentions": []
    },
    "signature": "mi5j/buYZdsiH8l6CVWRqdBKe+0UG6tVTOoVVjMhYl38Nkmb8wiIEfe7zu0JWuiHkaAIq+0/ZqYr6aV14j4fAw==.sig.ed25519"
  },
  "timestamp": 1543959001933
}
"#####;
        view.append(expected_seq, jsn.as_bytes());
        let seq = view
            .get_seq_by_key("%KKPLj1tWfuVhCvgJz2hG/nIsVzmBRzUJaqHv+sb+n1c=.sha256".to_string())
            .unwrap();
        assert_eq!(seq, expected_seq as i64);

        let seqs = view.get_seqs_by_type("post".to_string()).unwrap();
        assert_eq!(seqs[0], expected_seq as i64);
    }

    #[test]
    fn test_db_integrity_ok() {
        let filename = "/tmp/test_integrity.sqlite3";
        let keys = Vec::new();
        std::fs::remove_file(filename.clone())
            .or::<Result<()>>(Ok(()))
            .unwrap();

        let mut view = FlumeViewSql::new(filename, keys);
        view.check_db_integrity().unwrap();
    }
    #[test]
    fn test_db_integrity_fails() {
        let filename = "/tmp/test_integrity_bad.sqlite3";
        let keys = Vec::new();
        std::fs::remove_file(filename.clone())
            .or::<Result<()>>(Ok(()))
            .unwrap();

        let mut view = FlumeViewSql::new(filename.clone(), keys);

        std::fs::write(filename, b"BANG").unwrap();

        match view.check_db_integrity() {
            Ok(_) => panic!(),
            Err(_) => assert!(true),
        }
    }
}
