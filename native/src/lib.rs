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
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde_cbor;

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

#[no_mangle]
pub extern "C" fn to_cbor(env: napi_env, info: napi_callback_info) -> napi_value {
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

#[derive(Serialize, Deserialize, Debug)]
struct Value {
    previous: serde_json::Value,
    author: String,
    sequence: f64,
    timestamp: f64,
    hash: String,
    content: serde_json::Value,
    signature: String
}

impl Value {
    fn default()->Value{
        Value{
            previous: serde_json::Value::Null,
            author: String::default(),
            sequence: 0.0,
            timestamp: 0.0,
            hash: String::default(),
            content: serde_json::Value::Null,
            signature: String::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    key: String,
    value: Value,
    timestamp: f64,
}

impl Message {
    fn default()->Message{
        Message{
            key: String::default(),
            value: Value::default(),
            timestamp: 0.0
        }
    }
}

#[no_mangle]
pub extern "C" fn parse_json(env: napi_env, info: napi_callback_info) -> napi_value {
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

fn value_to_napi_value(env: napi_env, val: serde_json::Value)->napi_value{

    match val {
        serde_json::Value::Null => {
            get_null_value(env)
        },
        serde_json::Value::Bool(b) => {
            wrap_unsafe_create(env, b, napi_get_boolean)
        },
        serde_json::Value::Number(n) => {
            wrap_unsafe_create(env, n.as_f64().unwrap(), napi_create_double)
        },
        serde_json::Value::String(s) => {
            create_string_utf8(env, &s)
        },
        serde_json::Value::Array(a) => {
            let mut napi_array = NapiArray::with_capacity(env, a.len());

            for elem in a {
                napi_array.push(value_to_napi_value(env, elem))
            }
            napi_array.array
        },
        serde_json::Value::Object(o) => {
            let object = create_object(env);

            for (key, val) in o
            {
                //TODO: could use property descriptors here.
                unsafe { napi_set_property(env, object, create_string_utf8(env, &key), value_to_napi_value(env, val) ) };
            }

            object
        },
    }
}
#[no_mangle]
pub extern "C" fn parse_cbor_with_constructor(env: napi_env, info: napi_callback_info) -> napi_value {
    let arg = get_arg(env, info, 0);
    let constructor = get_arg(env, info, 1);

    let (p_buff, buff_size) = get_buffer_info(env, arg);

    let slice = unsafe { std::slice::from_raw_parts(p_buff, buff_size)};
    serde_cbor::from_slice::<Message>(slice)
        .map(|message|{
            //function Message (key, timestamp, previous, author, sequence, timestamp, hash, content, signature) {
            let args = [
                create_string_utf8(env, &message.key),
                wrap_unsafe_create(env, message.timestamp, napi_create_double),
                value_to_napi_value(env, message.value.previous),
                create_string_utf8(env, &message.value.author),
                wrap_unsafe_create(env, message.value.sequence, napi_create_double),
                wrap_unsafe_create(env, message.value.timestamp, napi_create_double),
                create_string_utf8(env, &message.value.hash),
                value_to_napi_value(env, message.value.content),
                create_string_utf8(env, &message.value.signature)
            ];

            let mut object = std::ptr::null_mut();
            let status = unsafe {napi_new_instance(env, constructor, args.len(), &args as *const napi_value, &mut object)};
            
            object
        })
        .unwrap_or_else(|_| get_undefined_value(env))
}
#[no_mangle]
pub extern "C" fn parse_json_with_constructor(env: napi_env, info: napi_callback_info) -> napi_value {
    let arg = get_arg(env, info, 0);
    let constructor = get_arg(env, info, 1);

    get_string(env, arg)
        .and_then(|string| {
            serde_json::from_str::<Message>(&string)
                .map_err(|_|{errors::ErrorKind::ParseError.into()})
        })
        .map(|message|{
            //function Message (key, timestamp, previous, author, sequence, timestamp, hash, content, signature) {
            let args = [
                create_string_utf8(env, &message.key),
                wrap_unsafe_create(env, message.timestamp, napi_create_double),
                value_to_napi_value(env, message.value.previous),
                create_string_utf8(env, &message.value.author),
                wrap_unsafe_create(env, message.value.sequence, napi_create_double),
                wrap_unsafe_create(env, message.value.timestamp, napi_create_double),
                create_string_utf8(env, &message.value.hash),
                value_to_napi_value(env, message.value.content),
                create_string_utf8(env, &message.value.signature)
            ];

            let mut object = std::ptr::null_mut();
            let status = unsafe {napi_new_instance(env, constructor, args.len(), &args as *const napi_value, &mut object)};
            
            object
        })
        .unwrap_or_else(|_| get_undefined_value(env))
}
