
#[macro_use]
extern crate criterion;
use criterion::Criterion;

extern crate ssb_sql_napi;
extern crate flumedb;
extern crate private_box;
extern crate base64;

use flumedb::flume_log::FlumeLog;
use flumedb::offset_log::OffsetCodec;
use flumedb::offset_log::OffsetLogIter;
use ssb_sql_napi::FlumeViewSql;
use private_box::SecretKey;
use base64::{decode, encode};

const NUM_ENTRIES: u32 = 10000;

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
        .find(|(key, _)|{
            key == "SSB_SECRET"
        })
        .map(|(_, val)| val)
        .unwrap();

    let secret_bytes = decode(&secret_str).unwrap();

    c.bench_function("flume view sql insert piets entire log with decryptions", move |b| {
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
    });
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

criterion_group!{
    name = sql;
    config = Criterion::default().sample_size(2);
    targets = flume_view_sql_insert_piets_entire_log_with_decryption, flume_view_sql_insert, flume_view_sql_insert_piets_entire_log
}

criterion_main!(sql);
