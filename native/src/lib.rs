extern crate serde;

extern crate serde_derive;
extern crate serde_json;

mod napi;
mod napi_sys;

use napi::*;
use napi_sys::*;
use std::ptr::{null, null_mut};
use std::os::raw::{c_char, c_void};
use std::{debug_assert};
use std::slice;

#[no_mangle]
pub extern "C" fn to_json(env: napi_env, info: napi_callback_info) -> napi_value {
    let arg = get_arg(env, info, 0);
    let message = NapiValue{env, value: arg};
    //serde_json::to_string(&message)
    ssb_legacy_msg_data::json::to_string(&message, true)
        .map(|string|{
            create_string_utf8(env, &string)
        })
        .unwrap_or_else(|err| {
            println!("Error: {:?}", err);
            get_undefined_value(env)
        })
}

