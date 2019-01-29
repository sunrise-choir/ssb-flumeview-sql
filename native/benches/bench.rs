#[macro_use]
extern crate criterion;
use criterion::Criterion;

extern crate base64;
extern crate flumedb;
extern crate private_box;
extern crate ssb_sql_napi;

use base64::{decode, encode};
use flumedb::flume_log::FlumeLog;
use flumedb::offset_log::OffsetCodec;
use flumedb::offset_log::OffsetLogIter;
use private_box::SecretKey;
use ssb_sql_napi::FlumeViewSql;

const NUM_ENTRIES: u32 = 100000;

fn create_test_db(num_entries: usize, offset_filename: &str, db_filename: &str) -> FlumeViewSql {
    let keys = Vec::new();
    std::fs::remove_file(db_filename).unwrap_or(());
    let mut view = FlumeViewSql::new(db_filename, keys);

    let file = std::fs::File::open(offset_filename.to_string()).unwrap();

    let buff: Vec<_> = OffsetLogIter::<u32, std::fs::File>::new(file)
        .take(num_entries)
        .map(|data| (data.id, data.data_buffer))
        .collect();

    view.append_batch(buff);
    view
}

fn flume_view_sql_insert_piets_entire_log(c: &mut Criterion) {
    let offset_filename = "/home/piet/.ssb/flume/log.offset";
    let db_filename = "/tmp/test.sqlite3";

    c.bench_function("flume view sql insert piets entire log", move |b| {
        b.iter(|| {
            let keys = Vec::new();
            std::fs::remove_file(db_filename.clone()).unwrap_or(());
            let mut view = FlumeViewSql::new(db_filename, keys);

            let file = std::fs::File::open(offset_filename.to_string()).unwrap();
            let buff: Vec<_> = OffsetLogIter::<u32, std::fs::File>::new(file)
                .map(|data| (data.id, data.data_buffer))
                .collect();

            view.append_batch(buff);
        })
    });
}

fn flume_view_sql_insert_piets_entire_log_with_decryption(c: &mut Criterion) {
    let offset_filename = "/home/piet/.ssb/flume/log.offset";
    let db_filename = "/tmp/test_private.sqlite3";
    let secret_str = std::env::vars()
        .find(|(key, _)| key == "SSB_SECRET")
        .map(|(_, val)| val)
        .unwrap();

    let secret_bytes = decode(&secret_str).unwrap();

    c.bench_function(
        "flume view sql insert piets entire log with decryptions",
        move |b| {
            b.iter(|| {
                let key = SecretKey::from_slice(&secret_bytes).unwrap();
                let keys = vec![key];
                std::fs::remove_file(db_filename.clone()).unwrap_or(());
                let mut view = FlumeViewSql::new(db_filename, keys);

                let file = std::fs::File::open(offset_filename.to_string()).unwrap();
                let buff: Vec<_> = OffsetLogIter::<u32, std::fs::File>::new(file)
                    .map(|data| (data.id, data.data_buffer))
                    .collect();

                view.append_batch(buff);
            })
        },
    );
}

fn flume_view_sql_insert(c: &mut Criterion) {
    let offset_filename = "/home/piet/.ssb/flume/log.offset";
    let db_filename = "/tmp/test.sqlite3";

    c.bench_function("flumeview sql insert", move |b| {
        b.iter(|| {
            let keys = Vec::new();
            std::fs::remove_file(db_filename.clone()).unwrap_or(());
            let mut view = FlumeViewSql::new(db_filename, keys);

            let file = std::fs::File::open(offset_filename.to_string()).unwrap();

            //TODO: this is ok for a benchmark but uses lots of memory.
            //Better would be to create a transaction and then append in a for_each loop.
            let buff: Vec<_> = OffsetLogIter::<u32, std::fs::File>::new(file)
                .take(NUM_ENTRIES as usize)
                .map(|data| (data.id, data.data_buffer))
                .collect();

            view.append_batch(buff);
        })
    });
}

fn all_messages_by_type(c: &mut Criterion) {
    let offset_filename = "/home/piet/.ssb/flume/log.offset";
    let db_filename = "/tmp/test.sqlite3";

    let mut view = create_test_db(NUM_ENTRIES as usize, offset_filename, db_filename);

    c.bench_function("flumeview all messages by type", move |b| {
        b.iter(|| {
            let seqs = view.get_seqs_by_type("post").unwrap();
        })
    });
}

fn all_messages_by_author(c: &mut Criterion) {
    let offset_filename = "/home/piet/.ssb/flume/log.offset";
    let db_filename = "/tmp/test.sqlite3";
    let author_key = "@U5GvOKP/YUza9k53DSXxT0mk3PIrnyAmessvNfZl5E0=.ed25519";

    let mut view = create_test_db(NUM_ENTRIES as usize, offset_filename, db_filename);

    c.bench_function("flumeview all messages by author", move |b| {
        b.iter(|| {
            let seqs = view.get_seqs_by_author(author_key).unwrap();
        })
    });
}
criterion_group! {
    name = sql_full_log;
    config = Criterion::default().sample_size(2);
    targets = flume_view_sql_insert_piets_entire_log_with_decryption, flume_view_sql_insert_piets_entire_log, flume_view_sql_insert
}

criterion_group!(sql, all_messages_by_type, all_messages_by_author);

criterion_main!(sql, sql_full_log);
