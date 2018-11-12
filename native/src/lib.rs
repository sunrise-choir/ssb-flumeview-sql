#![recursion_limit = "1024"]
#[macro_use]
extern crate error_chain;

extern crate encode_unicode;
extern crate indexmap;
extern crate ryu_ecmascript;
extern crate serde;
extern crate ssb_legacy_msg_data;
extern crate ssb_legacy_msg;
extern crate ssb_multiformats;
extern crate strtod;

extern crate base64;
#[macro_use]
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
use ssb_legacy_msg_data::value::ContentValue;
use ssb_legacy_msg::{Message, json};
use ssb_multiformats::multikey::Multikey;

#[no_mangle]
pub extern "C" fn parse_legacy(env: napi_env, info: napi_callback_info) -> napi_value {
    let arg = get_arg(env, info, 0);
    get_string(env, arg)
        .and_then(|string| {
            //let mut deserializer = JsonDeserializer::from_slice(&string.as_bytes());
            //let mut deserializer = serde_json::Deserializer::from_str(&string);
            let s: serde_json::Value = serde_json::from_str(&string).unwrap();
            match s {
                serde_json::Value::Object(_) => (), 
                _ => ()
            }
            Ok(get_undefined_value(env))

                //NapiEnv { env }
                //.deserialize(&mut deserializer)
                //.or(Err(errors::ErrorKind::ArgumentTypeError.into()))
        })
    //.map(|result| result.value)
    .unwrap_or_else(|_| get_undefined_value(env))
}
#[no_mangle]
pub extern "C" fn parse_legacy_buffer(env: napi_env, info: napi_callback_info) -> napi_value {

    let arg = get_arg(env, info, 0);
    let construct = get_arg(env, info, 1);
    let (ptr, size) = get_buffer_info(env, arg);

    let slice = unsafe { std::slice::from_raw_parts(ptr, size) };

    //let mut deserializer = JsonDeserializer::from_slice(slice);
    let mut deserializer = serde_json::Deserializer::from_slice(slice);
    NapiEnv { env }
    .deserialize(&mut deserializer)
        .map(|result| result.value)
        .or(Err(errors::ErrorKind::ArgumentTypeError.into()))
        .unwrap_or_else(|_: errors::ErrorKind| get_undefined_value(env))
}

#[derive(Serialize, Deserialize)]
struct TestThing {
    val: String
}

#[no_mangle]
pub extern "C" fn parse_legacy_with_constructor(env: napi_env, info: napi_callback_info) -> napi_value {
    let arg = get_arg(env, info, 0);
    let construct = get_arg(env, info, 1);
    get_string(env, arg)
        .and_then(|string| {
            //let mut deserializer = JsonDeserializer::from_slice(&string.as_bytes());
            //let mut deserializer = serde_json::Deserializer::from_str(&string);
            match json::from_legacy::<ContentValue>(string.as_bytes()) {
                Ok((msg, _)) => {
                    let val_string = create_string_utf8(env, &msg.author.to_legacy_string());
                    let mut result: napi_value = std::ptr::null_mut();
                    let args = [val_string];

                    let status = unsafe {napi_new_instance(env, construct, args.len(), &args[0] as *const napi_value, &mut result)};
                    debug_assert!(status == napi_status_napi_ok);

                    Ok(result)

                },
                Err(_) => bail!(errors::ErrorKind::ParseError)
            }
        })
    .unwrap_or_else(|_| get_undefined_value(env))
}
