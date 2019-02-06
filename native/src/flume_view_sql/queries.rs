use flume_view_sql::*;
use rusqlite::{Connection, Error, NO_PARAMS};

#[derive(Serialize, Deserialize, Debug)]
pub struct BackLink {
    id: String,
    author: String,
    timestamp: f64,
}

pub fn back_link_references(
    connection: &Connection,
    id: &str,
    timestamp: f64,
) -> Result<Vec<BackLink>, Error> {
    let mut stmt = connection.prepare_cached(
        "
        SELECT  
            links.link_from_key as id,
            messages.author as author,
            messages.received_time as timestamp
        FROM links 
        JOIN messages ON messages.key = links.link_from_key 
        WHERE link_to_key = ?
        AND NOT root = ? 
        AND NOT content_type = 'about'
        AND NOT content_type = 'vote'
        AND NOT content_type = 'tag'
",
    )?;

    let rows = stmt.query_map(&[&id, &id], |row| BackLink {
        id: row.get::<usize, String>(0),
        author: row.get::<usize, String>(1),
        timestamp: row.get::<usize, f64>(2),
    })?;

    rows.collect()
}

pub fn how_many_friends_follow_id() {}
pub fn who_is_friends_with_id() {}
pub fn who_does_id_follow_one_way() {}
pub fn who_does_follows_id_one_way() {}

pub fn friends_two_hops(connection: Connection) {
    //"
    //SELECT
    //author as id
    //FROM
    //authors
    //WHERE authors.id IN (
    //SELECT
    //contact_author_id
    //FROM contacts_raw
    //WHERE author_id == 1 AND state == 1
    //UNION
    //SELECT
    //friend_contacts_raw.contact_author_id
    //FROM contacts_raw
    //join contacts_raw AS friend_contacts_raw ON friend_contacts_raw.author_id == contacts_raw.contact_author_id
    //WHERE contacts_raw.author_id == 1
    //AND contacts_raw.state == 1
    //AND friend_contacts_raw.state == 1
    //EXCEPT
    //SELECT
    //contact_author_id
    //FROM contacts_raw
    //WHERE author_id == 1
    //AND state == -1)"
}
#[cfg(test)]
mod test {
    use flume_view_sql::queries::back_link_references;
    use flume_view_sql::*;
    use flumedb::offset_log::OffsetLogIter;
    use itertools::Itertools;
    use serde_json::*;

    #[test]
    fn find_backlinks_refs() {
        let view = create_test_db(
            5000,
            "/home/piet/.ssb/flume/log.offset",
            "/tmp/backlinks.sqlite3",
        );
        let connection = &view.connection;
        let links = back_link_references(
            connection,
            "%ZEuQdC7OBxDgRg2Vv/VgjArRIpE5YwIMo6ufXqaWaGg=.sha256",
            0.0,
        );
        assert_eq!(links.unwrap().len(), 1);
    }
    fn create_test_db(
        num_entries: usize,
        offset_filename: &str,
        db_filename: &str,
    ) -> FlumeViewSql {
        let keys = Vec::new();
        std::fs::remove_file(db_filename).unwrap_or(());
        let mut view = FlumeViewSql::new(db_filename, keys, "").unwrap();

        let file = std::fs::File::open(offset_filename.to_string()).unwrap();

        OffsetLogIter::<u32, std::fs::File>::new(file)
            .take(num_entries)
            .map(|data| (data.id, data.data_buffer))
            .chunks(1000 as usize)
            .into_iter()
            .for_each(|chunk| {
                view.append_batch(&chunk.collect_vec());
            });

        view
    }
}
