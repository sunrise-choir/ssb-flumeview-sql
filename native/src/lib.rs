#![recursion_limit = "1024"]
#[macro_use]
extern crate error_chain;

extern crate encode_unicode;
extern crate indexmap;
extern crate ryu_ecmascript;
extern crate serde;
extern crate ssb_legacy_msg_data;
extern crate strtod;

extern crate base64;
extern crate serde_derive;
extern crate serde_json;

mod errors;
mod napi;
mod napi_sys;
mod value;

use napi::*;
use napi_sys::*;
use serde::de::DeserializeSeed;
use ssb_legacy_msg_data::json::JsonDeserializer;
use ssb_legacy_msg_data::cbor::CborDeserializer;
use value::NapiValue;

#[no_mangle]
pub extern "C" fn stringify_legacy(env: napi_env, info: napi_callback_info) -> napi_value {
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

#[no_mangle]
pub extern "C" fn encode_cbor(env: napi_env, info: napi_callback_info) -> napi_value {
    let arg = get_arg(env, info, 0);
    let message = NapiValue{env, value: arg};
    ssb_legacy_msg_data::cbor::to_vec(&message)
        .map(|vec|{
            create_buffer_copy(env, &vec)
        })
        .unwrap_or_else(|_| get_undefined_value(env))
}

#[no_mangle]
pub extern "C" fn parse_cbor(env: napi_env, info: napi_callback_info) -> napi_value {
    let arg = get_arg(env, info, 0);
    let (p_buff, buff_size) = get_buffer_info(env, arg);

    let slice = unsafe { std::slice::from_raw_parts(p_buff, buff_size)};
    let mut deserializer = CborDeserializer::from_slice(slice);
    NapiEnv { env }
        .deserialize(&mut deserializer)
        .map(|result| result.value)
        .unwrap_or_else(|_| get_undefined_value(env))
}

#[no_mangle]
pub extern "C" fn parse_legacy(env: napi_env, info: napi_callback_info) -> napi_value {
    let arg = get_arg(env, info, 0);
    get_string(env, arg)
        .and_then(|string| {
            let mut deserializer = JsonDeserializer::from_slice(&string.as_bytes());
            //let mut deserializer = serde_json::Deserializer::from_str(&string);
            NapiEnv { env }
                .deserialize(&mut deserializer)
                .or_else(|err| {
                    println!("Error: {:?}", err);
                    Err(errors::ErrorKind::ArgumentTypeError.into())
                })
                .map(|result| result.value)
        })
    .unwrap_or_else(|_| get_undefined_value(env))
}
