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

use std::collections::HashMap;
use napi::*;
use napi_sys::*;
use serde::de::DeserializeSeed;
use ssb_legacy_msg_data::json::JsonDeserializer;
use ssb_legacy_msg_data::value::ContentValue;
use ssb_legacy_msg::{json};
use ssb_multiformats::multikey::Multikey;

use serde::{Deserialize};



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

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    key: String,
    value: Value
}

#[derive(Serialize, Deserialize, Debug)]
struct Value {
    previous: String,
    author: String,
    sequence: u32,
    timestamp: i64,
    hash: String,
    content: serde_json::Value,
    signature: String
}


fn value_to_napi_value(env: napi_env, json_value: &serde_json::Value ) -> napi_value {

    match *json_value {
        serde_json::Value::Null => {get_null_value(env)},
        serde_json::Value::Bool(b) => {wrap_unsafe_create(env, b, napi_get_boolean)},
        serde_json::Value::Number(ref n) => {wrap_unsafe_create(env, n.as_f64().unwrap(), napi_create_double)},
        serde_json::Value::String(ref s) => {create_string_utf8(env, &s)},
        serde_json::Value::Array(ref v) => {
            let mut array = NapiArray::with_capacity(env, v.len());

            for elem in v.iter() {
                array.push(value_to_napi_value(env, elem));
            }

            array.array
        
        },
        serde_json::Value::Object(ref o) => {
            let object = create_object(env);

            for (key, val) in o.iter(){
                let napi_key = create_string_utf8(env, key);
                let napi_value = value_to_napi_value(env, val);
                unsafe { napi_set_property(env, object, napi_key, napi_value) };
            }
        
            object
        }
    }
} 


#[no_mangle]
pub extern "C" fn parse_legacy_with_constructor(env: napi_env, info: napi_callback_info) -> napi_value {
    let arg = get_arg(env, info, 0);
    let construct = get_arg(env, info, 1);
    get_string(env, arg)
        .and_then(|string| {
            //let mut deserializer = JsonDeserializer::from_slice(&string.as_bytes());
            //let mut deserializer = serde_json::Deserializer::from_str(&string);
            match serde_json::from_str::<Value>(&string) {
                Ok(msg) => {
                    let mut content: napi_value = value_to_napi_value(env, &msg.content);

                    //function sig in js: Message (key, previous, author, sequence, timestamp, hash, content, signature) {
                    let args = [
                        get_undefined_value(env), //TODO:: do the key
                        create_string_utf8(env, &msg.previous),
                        create_string_utf8(env, &msg.author),
                        wrap_unsafe_create(env, msg.sequence, napi_create_uint32),
                        wrap_unsafe_create(env, msg.timestamp, napi_create_int64),
                        create_string_utf8(env, &msg.hash),
                        content,
                        create_string_utf8(env, &msg.signature)
                    ];

                    let mut result: napi_value = std::ptr::null_mut();
                    let status = unsafe {napi_new_instance(env, construct, args.len(), &args[0] as *const napi_value, &mut result)};
                    debug_assert!(status == napi_status_napi_ok);
                    
                    Ok(result)


                },
                Err(_) => {
                    println!("nope, couldn't parse the msg");
                    bail!(errors::ErrorKind::ParseError)
                }
            }
        })
    .unwrap_or_else(|_| get_undefined_value(env))
}
