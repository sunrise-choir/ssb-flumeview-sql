extern crate flumedb;
extern crate jsonrpc_tcp_server;
extern crate r2d2;
extern crate r2d2_sqlite;
extern crate rusqlite;

use jsonrpc_tcp_server::jsonrpc_core::*;
use r2d2_sqlite::SqliteConnectionManager;
use std::time::{Duration, Instant};

use flumedb::Sequence;
use rusqlite::NO_PARAMS;

fn main() {
    let manager = SqliteConnectionManager::file("/tmp/patchwork.sqlite3");
    let pool = r2d2::Pool::new(manager).unwrap();

    let mut io = MetaIoHandler::<()>::default();

    io.add_method("get_latest", move |params| {
        let now = Instant::now();
        println!("handle get_latest, params: {:?}", params);
        let connection = pool.clone().get().unwrap();
        let mut stmt = connection
            .prepare_cached("SELECT MAX(flume_seq) FROM messages_raw")
            .unwrap();

        let seq: i64 = stmt
            .query_row(NO_PARAMS, |row| {
                let res: i64 = row.get_checked(0).unwrap_or(0);
                res as Sequence
            })
            .unwrap() as i64;

        println!("{}", now.elapsed().as_micros());
        Ok(Value::Number(seq.into()))
    });

    let handler = ::std::thread::spawn(move || {
        let _server = jsonrpc_tcp_server::ServerBuilder::new(io)
            .start(&"0.0.0.0:9876".parse().unwrap())
            .expect("Server should start ok")
            .wait();
    });

    handler.join().unwrap();
}
