extern crate jsonrpc_ipc_server;
extern crate r2d2;
extern crate r2d2_sqlite;
extern crate rusqlite;
extern crate flumedb;

use jsonrpc_ipc_server::jsonrpc_core::*;
use r2d2_sqlite::SqliteConnectionManager;

use rusqlite::{NO_PARAMS};
use flumedb::Sequence;

fn main() {

    let manager = SqliteConnectionManager::file("/tmp/patchwork.sqlite3");
    let pool = r2d2::Pool::new(manager).unwrap();

	let mut io = MetaIoHandler::<()>::default();

	io.add_method("get_latest", move |_params| {
        println!("handle get_latest");
        let connection = pool.clone().get().unwrap();
        let mut stmt = connection
            .prepare_cached("SELECT MAX(flume_seq) FROM messages_raw").unwrap();

        let seq: i64 = stmt
            .query_row(NO_PARAMS, |row| {
                let res: i64 = row.get_checked(0).unwrap_or(0);
                res as Sequence
            })
            .unwrap() as i64;

		Ok(Value::Number(seq.into()))
	});

    let handler = ::std::thread::spawn(move || {
        let _server = jsonrpc_ipc_server::ServerBuilder::new(io)
            .start("/tmp/parity-example.ipc")
            .expect("Server should start ok")
            .wait();
    });

    handler.join().unwrap();
}
