#![recursion_limit = "1024"]
#[macro_use]
extern crate error_chain;

extern crate ssb_legacy_msg;

mod errors;
mod napi;
mod napi_sys;

use napi::*;
use napi_sys::*;

#[no_mangle]
pub extern "C" fn validate(env: napi_env, info: napi_callback_info) -> napi_value {
    get_undefined_value(env)
}
