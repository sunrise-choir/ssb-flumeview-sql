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
extern crate serde_json;
extern crate base64;


mod errors;
mod napi;
mod napi_sys;
mod value;

use napi::*;
use napi_sys::*;
use serde::de::DeserializeSeed; 
use ssb_legacy_msg_data::json::JsonDeserializer;

#[no_mangle]
pub extern "C" fn parse_legacy(env: napi_env, info: napi_callback_info) -> napi_value {
    let arg = get_arg(env, info, 0);

    println!("about to do the thing");
    get_string(env, arg)
        .and_then(|string|{
            println!("got the string");
            let mut deserializer = JsonDeserializer::from_slice(&string.as_bytes());
            NapiEnv{env}.deserialize(&mut deserializer)
                .or(Err(errors::ErrorKind::ArgumentTypeError.into()))
        })
        .map(|result|result.value)
        .unwrap_or_else(|_| get_undefined_value(env))
}
