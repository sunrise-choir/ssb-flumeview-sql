extern crate failure_derive;
#[macro_use]
extern crate failure;

#[macro_use]
extern crate log;

extern crate itertools;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate base64;
extern crate flumedb;
extern crate node_napi;
extern crate private_box;
extern crate rusqlite;

use failure::Error;

use itertools::Itertools;
use node_napi::napi::*;
use node_napi::napi_sys::*;
use std::debug_assert;
use std::os::raw::c_void;
use std::ptr::{null, null_mut};
use std::slice;

use flumedb::OffsetLogIter;
use flumedb::Sequence;

use private_box::SecretKey;

pub mod flume_view_sql;
pub use flume_view_sql::FlumeViewSql;

struct SsbQuery {
    view: FlumeViewSql,
    log_path: String,
}

impl SsbQuery {
    fn new(
        log_path: String,
        view_path: String,
        keys: Vec<SecretKey>,
        pub_key: &str,
    ) -> Result<SsbQuery, Error> {
        let view = FlumeViewSql::new(&view_path, keys, pub_key)?;

        Ok(SsbQuery { view, log_path })
    }

    fn get_latest(&self) -> Sequence {
        self.view.get_latest().unwrap()
    }

    fn process(&mut self, num_items: i64) {
        let latest = self.get_latest();

        //If the latest is 0, we haven't got anything in the db. Don't skip the very first
        //element in the offset log. I know this isn't super nice. It could be refactored later.
        let num_to_skip = match latest {
            0 => 0,
            _ => 1,
        };
        let log_path = self.log_path.clone();
        let file = std::fs::File::open(log_path.clone()).unwrap();

        let items_to_take = match num_items {
            -1 => std::usize::MAX,
            n => n as usize,
        };

        OffsetLogIter::<u32, std::fs::File>::with_starting_offset(file, latest)
            .skip(num_to_skip)
            .take(items_to_take)
            .map(|data| (data.id + latest, data.data_buffer)) //TODO log_latest might not be the right thing
            .chunks(1000)
            .into_iter()
            .for_each(|chunk| {
                self.view.append_batch(&chunk.collect_vec());
            })
    }
}

#[no_mangle]
extern "C" fn get_latest(env: napi_env, info: napi_callback_info) -> napi_value {
    let this = get_this(env, info);
    let mut ptr_ssb_query = null_mut();

    unsafe { napi_unwrap(env, this, &mut ptr_ssb_query) };

    let ssb_query = ptr_ssb_query as *mut SsbQuery;
    let latest = unsafe { (*ssb_query).get_latest() };

    wrap_unsafe_create::<i64>(env, latest as i64, napi_create_int64)
}

#[no_mangle]
extern "C" fn process(env: napi_env, info: napi_callback_info) -> napi_value {
    let this = get_this(env, info);

    let num_value = get_arg(env, info, 0);
    let num = wrap_unsafe_get(env, num_value, napi_get_value_int64);

    let mut ptr_ssb_query = null_mut();

    unsafe { napi_unwrap(env, this, &mut ptr_ssb_query) };

    let ssb_query = ptr_ssb_query as *mut SsbQuery;
    unsafe { (*ssb_query).process(num) };

    get_undefined_value(env)
}
#[no_mangle]
pub extern "C" fn define_view_class(env: napi_env) -> napi_value {
    let latest_property: napi_property_descriptor = napi_property_descriptor {
        utf8name: null(),
        name: create_string_utf8(env, "getLatest"),
        method: Some(get_latest),
        getter: None,
        setter: None,
        value: null_mut(),
        attributes: napi_property_attributes_napi_default,
        data: null_mut(),
    };
    let process_property: napi_property_descriptor = napi_property_descriptor {
        utf8name: null(),
        name: create_string_utf8(env, "process"),
        method: Some(process),
        getter: None,
        setter: None,
        value: null_mut(),
        attributes: napi_property_attributes_napi_default,
        data: null_mut(),
    };
    let properties = vec![latest_property, process_property];
    let data = null_mut();

    define_class(
        env,
        "SqlView",
        Some(construct_view_class),
        data,
        &properties,
    )
}

#[no_mangle]
pub extern "C" fn finalize_view(_: napi_env, data: *mut c_void, _: *mut c_void) {
    let ssb_query = data as *mut SsbQuery;
    unsafe { Box::from_raw(ssb_query) };
}

#[no_mangle]
pub extern "C" fn construct_view_class(env: napi_env, info: napi_callback_info) -> napi_value {
    let this = get_this(env, info);

    let path_to_offset_value = get_arg(env, info, 0);
    let path_to_db_value = get_arg(env, info, 1);
    let secret_key_value = get_arg(env, info, 2);
    let pub_key_value = get_arg(env, info, 3);

    let raw_parts = get_buffer_info(env, secret_key_value);

    let secret_key_bytes = unsafe { slice::from_raw_parts(raw_parts.0, raw_parts.1) };
    let secret_key = SecretKey::from_slice(secret_key_bytes).unwrap_or_else(|| {
        let empty_slice = vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ];
        SecretKey::from_slice(&empty_slice[0..32]).unwrap()
    });
    let keys = vec![secret_key];

    let path_to_offset = get_string(env, path_to_offset_value).unwrap();
    let path_to_db = get_string(env, path_to_db_value).unwrap();
    let pub_key = get_string(env, pub_key_value).unwrap();

    let mut wrapped_ref: napi_ref = null_mut();
    let finalize_hint: *mut c_void = null_mut();

    match SsbQuery::new(path_to_offset, path_to_db, keys, &pub_key) {
        Ok(query) => {
            let ssb_query = Box::new(query);

            let status = unsafe {
                napi_wrap(
                    env,
                    this,
                    Box::into_raw(ssb_query) as *mut c_void,
                    Some(finalize_view),
                    finalize_hint,
                    &mut wrapped_ref,
                )
            };

            debug_assert!(status == napi_status_napi_ok);
        }
        Err(err) => {
            throw_error(env, err);
        }
    }

    this
}
