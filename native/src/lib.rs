#![recursion_limit = "1024"]
#[macro_use]
extern crate error_chain;

extern crate indexmap;
extern crate ssb_legacy_msg_data;
extern crate ryu_ecmascript;
extern crate strtod;
extern crate encode_unicode;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate base64;


mod errors;
mod napi;
mod napi_sys;
pub mod value;

use napi::*;
use napi_sys::*;

#[no_mangle]
pub extern "C" fn parse_legacy(env: napi_env, info: napi_callback_info) -> napi_value {
    // get the buffer contents as a slice.
    //
    // call from_slice
    //
    // return what we got
    get_undefined_value(env)
}
