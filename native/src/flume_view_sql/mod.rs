use failure::Error;
use flumedb::flume_view::*;

use base64::decode;
use rusqlite::types::ToSql;
use rusqlite::OpenFlags;
use rusqlite::{Connection, NO_PARAMS};
use serde_json::Value;

use private_box::SecretKey;

mod abouts;
mod authors;
mod blob_links;
mod blobs;
mod branches;
mod contacts;
mod keys;
mod links;
mod mentions;
mod messages;
mod migrations;
use self::abouts::*;
use self::authors::*;
use self::blob_links::*;
use self::blobs::*;
use self::branches::*;
use self::contacts::*;
use self::keys::*;
use self::links::*;
use self::mentions::*;
use self::messages::*;
use self::migrations::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct SsbValue {
    author: String,
    sequence: u32,
    timestamp: f64,
    content: Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SsbMessage {
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

fn create_connection(path: &str) -> Result<Connection, Error> {
    let flags: OpenFlags = OpenFlags::SQLITE_OPEN_READ_WRITE
        | OpenFlags::SQLITE_OPEN_CREATE
        | OpenFlags::SQLITE_OPEN_NO_MUTEX;

    Connection::open_with_flags(path, flags).map_err(|err| err.into())
}

impl FlumeViewSql {
    pub fn new(
        path: &str,
        secret_keys: Vec<SecretKey>,
        pub_key: &str,
    ) -> Result<FlumeViewSql, Error> {
        let mut connection = create_connection(path)?;

        if let Ok(false) = is_db_up_to_date(&connection) {
            info!("sqlite db is out of date. Deleting db and it will be rebuilt.");
            std::fs::remove_file(path).unwrap();

            connection = create_connection(path)?;

            create_tables(&connection)?;
            create_indices(&connection)?;
            create_views(&connection)?;

            set_db_version(&connection)?;
            set_author_that_is_me(&connection, pub_key)?;
        }

        set_pragmas(&connection);

        Ok(FlumeViewSql {
            connection,
            secret_keys,
        })
    }

    pub fn get_seq_by_key(&mut self, key: &str) -> Result<i64, Error> {
        let mut stmt = self
            .connection
            .prepare("SELECT flume_seq FROM messages_raw JOIN keys ON messages_raw.key_id=keys.id WHERE keys.key=?1")?;

        stmt.query_row(&[key], |row| row.get(0))
            .map_err(|err| err.into())
    }

    pub fn get_seqs_by_type(&mut self, content_type: &str) -> Result<Vec<i64>, Error> {
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

    pub fn get_seqs_by_author(&mut self, author: &str) -> Result<Vec<i64>, Error> {
        let mut stmt = self
            .connection
            .prepare("SELECT flume_seq FROM messages_raw JOIN authors ON messages_raw.author_id=authors.id WHERE author=?1")?;

        let rows = stmt.query_map(&[author], |row| row.get(0))?;

        let seqs = rows.fold(Vec::<i64>::new(), |mut vec, row| {
            vec.push(row.unwrap());
            vec
        });

        Ok(seqs)
    }

    pub fn append_batch(&mut self, items: &[(Sequence, Vec<u8>)]) {
        trace!("Start batch append");
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
                        Err(FlumeViewSqlError::DbFailedIntegrityCheck {}.into())
                    })
            })
    }

    pub fn get_latest(&self) -> Result<Sequence, Error> {
        let mut stmt = self
            .connection
            .prepare_cached("SELECT MAX(flume_seq) FROM messages_raw")?;

        stmt.query_row(NO_PARAMS, |row| {
            let res: i64 = row.get_checked(0).unwrap_or(0);
            trace!("got latest seq from db: {}", res);
            res as Sequence
        })
        .map_err(|err| err.into())
    }
}

fn find_values_in_object_by_key<'a>(
    obj: &'a serde_json::Value,
    key: &str,
    values: &mut Vec<&'a serde_json::Value>,
) {
    if let Some(val) = obj.get(key) {
        values.push(val)
    }

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

            for secret_key in secret_keys {
                message.value.content = private_box::decrypt(&bytes, secret_key)
                    .and_then(|data| {
                        is_decrypted = true;
                        serde_json::from_slice(&data).map_err(|_| ())
                    })
                    .unwrap_or(Value::Null); //If we can't decrypt it, throw it away.

                if is_decrypted {
                    break;
                }
            }

            message
        }
        _ => message,
    };

    (is_decrypted, message)
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

    let mut links = Vec::new();
    find_values_in_object_by_key(&message.value.content, "link", &mut links);

    insert_links(connection, links.as_slice(), message_key_id);
    insert_mentions(connection, links.as_slice(), message_key_id);
    insert_blob_links(connection, links.as_slice(), message_key_id);

    insert_branches(connection, &message, message_key_id);
    insert_message(
        connection,
        &message,
        seq as i64,
        message_key_id,
        is_decrypted,
    )?;
    insert_or_update_contacts(connection, &message, message_key_id, is_decrypted);
    insert_abouts(connection, &message, message_key_id);

    Ok(())
}

fn set_pragmas(connection: &Connection) {
    connection
        .execute("PRAGMA synchronous = OFF", NO_PARAMS)
        .unwrap();
    connection
        .execute("PRAGMA page_size = 4096", NO_PARAMS)
        .unwrap();
}

fn create_tables(connection: &Connection) -> Result<(), Error> {
    create_migrations_tables(connection)?;
    create_messages_tables(connection)?;
    create_authors_tables(connection)?;
    create_keys_tables(connection)?;
    create_links_tables(connection)?;
    create_contacts_tables(connection)?;
    create_branches_tables(connection)?;
    create_mentions_tables(connection)?;
    create_abouts_tables(connection)?;
    create_blobs_tables(connection)?;
    create_blob_links_tables(connection)?;

    Ok(())
}

fn create_views(connection: &Connection) -> Result<(), Error> {
    create_messages_views(connection)?;
    create_links_views(connection)?;
    create_blob_links_views(connection)?;
    create_abouts_views(connection)?;
    create_mentions_views(connection)?;
    Ok(())
}

fn create_indices(connection: &Connection) -> Result<(), Error> {
    create_messages_indices(connection)?;
    create_links_indices(connection)?;
    create_blob_links_indices(connection)?;
    create_contacts_indices(connection)?;
    create_keys_indices(connection)?;
    create_branches_indices(connection)?;
    create_authors_indices(connection)?;
    create_abouts_indices(connection)?;
    create_mentions_indices(connection)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use flume_view_sql::*;
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
        FlumeViewSql::new(filename, keys, "").unwrap();
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

        let mut view = FlumeViewSql::new(filename, keys, "").unwrap();
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
            .get_seq_by_key("%KKPLj1tWfuVhCvgJz2hG/nIsVzmBRzUJaqHv+sb+n1c=.sha256")
            .unwrap();
        assert_eq!(seq, expected_seq as i64);

        let seqs = view.get_seqs_by_type("post").unwrap();
        assert_eq!(seqs[0], expected_seq as i64);
    }

    #[test]
    fn test_db_integrity_ok() {
        let filename = "/tmp/test_integrity.sqlite3";
        let keys = Vec::new();
        std::fs::remove_file(filename.clone())
            .or::<Result<()>>(Ok(()))
            .unwrap();

        let mut view = FlumeViewSql::new(filename, keys, "").unwrap();
        view.check_db_integrity().unwrap();
    }
    #[test]
    fn test_db_integrity_fails() {
        let filename = "/tmp/test_integrity_bad.sqlite3";
        let keys = Vec::new();
        std::fs::remove_file(filename.clone())
            .or::<Result<()>>(Ok(()))
            .unwrap();

        let mut view = FlumeViewSql::new(filename.clone(), keys, "").unwrap();

        std::fs::write(filename, b"BANG").unwrap();

        match view.check_db_integrity() {
            Ok(_) => panic!(),
            Err(_) => assert!(true),
        }
    }
}
