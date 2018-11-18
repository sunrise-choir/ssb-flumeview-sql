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
use std::ptr::{null, null_mut};
use std::os::raw::{c_char, c_void};
use std::{debug_assert};
use std::alloc::{alloc, dealloc, Layout};
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
    let mut deserializer = CborDeserializer::from_slice(slice); //this one is slightly faster
    //let mut deserializer = ssb_legacy_msg_data::cbor::from_slice::<Message>(slice)
    NapiEnv { env }
        .deserialize(&mut deserializer)
        .map(|result| result.value)
        .unwrap_or_else(|_| get_undefined_value(env))
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
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

fn value_to_napi_value(env: napi_env, val: &serde_json::Value)->napi_value{

    match *val {
        serde_json::Value::Null => {
            get_null_value(env)
        },
        serde_json::Value::Bool(b) => {
            wrap_unsafe_create(env, b, napi_get_boolean)
        },
        serde_json::Value::Number(ref n) => {
            wrap_unsafe_create(env, n.as_f64().unwrap(), napi_create_double)
        },
        serde_json::Value::String(ref s) => {
            create_string_utf8(env, &s)
        },
        serde_json::Value::Array(ref a) => {
            let mut napi_array = NapiArray::with_capacity(env, a.len());

            for elem in a {
                napi_array.push(value_to_napi_value(env, &elem))
            }
            napi_array.array
        },
        serde_json::Value::Object(ref o) => {
            let object = create_object(env);
            let descriptors: Vec<napi_property_descriptor> = o
                .iter()
                .map(|(key, val)|{
                    napi_property_descriptor{
                        utf8name: null(), // key.as_ptr() as *const c_char,
                        name: create_string_utf8(env, key),
                        method: None,
                        getter: None,
                        setter: None,
                        value: value_to_napi_value(env, val),
                        attributes: napi_property_attributes_napi_enumerable, 
                        data: null_mut() 
                    }
                })
                .collect();

            let status = unsafe { napi_define_properties(env, object, descriptors.len(), descriptors.as_ptr() as * const napi_property_descriptor) };
            debug_assert!(status == napi_status_napi_ok);

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
    ssb_legacy_msg_data::cbor::from_slice::<Message>(slice)
    //serde_cbor::from_slice::<Message>(slice)
        .map(|message|{
            //function Message (key, timestamp, previous, author, sequence, timestamp, hash, content, signature) {
            let args = [
                create_string_utf8(env, &message.key),
                wrap_unsafe_create(env, message.timestamp, napi_create_double),
                value_to_napi_value(env, &message.value.previous),
                create_string_utf8(env, &message.value.author),
                wrap_unsafe_create(env, message.value.sequence, napi_create_double),
                wrap_unsafe_create(env, message.value.timestamp, napi_create_double),
                create_string_utf8(env, &message.value.hash),
                value_to_napi_value(env, &message.value.content),
                create_string_utf8(env, &message.value.signature)
            ];

            let mut object = std::ptr::null_mut();
            let status = unsafe {napi_new_instance(env, constructor, args.len(), &args as *const napi_value, &mut object)};
            debug_assert!(status == napi_status_napi_ok);
            
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
                value_to_napi_value(env, &message.value.previous),
                create_string_utf8(env, &message.value.author),
                wrap_unsafe_create(env, message.value.sequence, napi_create_double),
                wrap_unsafe_create(env, message.value.timestamp, napi_create_double),
                create_string_utf8(env, &message.value.hash),
                value_to_napi_value(env, &message.value.content),
                create_string_utf8(env, &message.value.signature)
            ];

            let mut object = std::ptr::null_mut();
            let status = unsafe {napi_new_instance(env, constructor, args.len(), &args as *const napi_value, &mut object)};
            debug_assert!(status == napi_status_napi_ok);
            
            object
        })
        .unwrap_or_else(|_| get_undefined_value(env))
}


struct RawSlice {
    inner: (*const u8, usize)
}

impl Default for RawSlice {
    fn default()->RawSlice {
        RawSlice{inner: (null(), 0)}
    }
}

struct ParseContext<T: Default> {
    input_ref: napi_ref,
    cb_ref: napi_ref,
    constructor_ref: napi_ref,

    work: napi_async_work,
    message: Option<Result<Message, errors::ErrorKind>>,
    thing_to_parse: T
}

impl<T: Default> ParseContext<T> {
    /// Allocate. Will not be automatically dropped.
    fn alloc()-> *mut ParseContext<T>{
        let layout = Layout::new::<ParseContext<T>>();
        unsafe {alloc(layout) as *mut ParseContext<T>}
    }
}

extern "C" fn delete_context<T: Default>(arg: *mut c_void) {
    let context = unsafe { &mut *(arg as *mut ParseContext<T>) };
    let layout = Layout::for_value(&context);

    unsafe {dealloc(arg as *mut u8, layout)}
}

impl<T: Default> Default for ParseContext<T> {
    fn default()-> ParseContext<T> {
        ParseContext {
            input_ref: null_mut(),
            cb_ref: null_mut(),
            constructor_ref: null_mut(),

            work: null_mut(),
            message: None,
            thing_to_parse: T::default()
        }
    }
}

impl <T: Default> Drop for ParseContext<T> {
    fn drop(&mut self){
        println!("dropping");
    }
}

extern "C" fn parse_json_async_execute(_env: napi_env, data: *mut c_void) {
    println!("execute");
    let context = unsafe { data as *mut ParseContext<&[u8]> };

    let slice = unsafe { (*context).thing_to_parse.clone() };
    std::mem::forget(slice);

    println!("about to derserialize");
    unsafe {

        println!("message is {:?}",(*context).message );
        (*context).message = Some(serde_json::from_slice::<Message>(slice)
                                  .map_err(|_|{errors::ErrorKind::ParseError.into()}));

        println!("message is {:?}",(*context).message );
    };
    
}

extern "C" fn parse_json_async_complete(env: napi_env, _status: napi_status, data: *mut c_void) {
    println!("complete");
    let context = unsafe { &mut *(data as *mut ParseContext<&[u8]>) };
    println!("message is {:?}",(*context).message );

    let cb = get_reference_value(env, context.cb_ref);
    let constructor = get_reference_value(env, context.constructor_ref);

    let result = match context.message {
        Some(Ok(ref message)) => {
            println!("before we mess with the stuff in message");
            let args = [
                create_string_utf8(env, &message.key),
                wrap_unsafe_create(env, message.timestamp, napi_create_double),
                value_to_napi_value(env, &message.value.previous),
                create_string_utf8(env, &message.value.author),
                wrap_unsafe_create(env, message.value.sequence, napi_create_double),
                wrap_unsafe_create(env, message.value.timestamp, napi_create_double),
                create_string_utf8(env, &message.value.hash),
                value_to_napi_value(env, &message.value.content),
                create_string_utf8(env, &message.value.signature)
            ];

            println!("after we mess with the stuff in message");
            let mut object = std::ptr::null_mut();
            let status = unsafe {napi_new_instance(env, constructor, args.len(), &args as *const napi_value, &mut object)};
            debug_assert!(status == napi_status_napi_ok);

            object
        },
        _ => get_undefined_value(env)
    };

    let args = [get_undefined_value(env), result];
    let mut global: napi_value = null_mut();
    let mut return_value: napi_value = null_mut();

    unsafe {
        napi_get_global(env, &mut global);
        napi_call_function(
            env,
            global,
            cb,
            2,
            &args[0] as *const napi_value,
            &mut return_value,
        );
    };

    delete_reference(env, context.input_ref);
    delete_reference(env, context.cb_ref);
    delete_reference(env, context.constructor_ref);

    //let status = unsafe {napi_add_env_cleanup_hook(env, Some(delete_context::<RawSlice>), context as *mut ParseContext<&[u8]> as *mut c_void)};
    unsafe {
        napi_delete_async_work(env, context.work);
        delete_context::<&[u8]>(context as *mut ParseContext<&[u8]> as *mut c_void);
    }
}

#[no_mangle]
pub extern "C" fn parse_json_async(env: napi_env, info: napi_callback_info) -> napi_value { 
    
    let context = ParseContext::<&[u8]>::alloc();

    println!("added cleanup hook");
    
    let message = Some(Ok(Message::default()));
    
    println!("created default message");

    unsafe {
        (*context).message = message;
        println!("context message {:?}", (*context).message) 
    };

    let input = get_arg(env, info, 0);
    let constructor = get_arg(env, info, 1);
    let cb = get_arg(env, info, 2);

    unsafe {
        (*context).input_ref = create_reference(env, input);
        (*context).cb_ref = create_reference(env, cb);
        (*context).constructor_ref = create_reference(env, constructor);
    }

    println!("after creating refs");
    
    let (p_buff, buff_size) = get_buffer_info(env, input);
    let slice = unsafe { std::slice::from_raw_parts(p_buff, buff_size)};

    unsafe {
        (*context).thing_to_parse = slice; 
    }

    std::mem::forget(slice);

    let work_name = create_string_utf8(env, "parse_json_async");
    let status = unsafe {napi_create_async_work(env, null_mut(), work_name, Some(parse_json_async_execute), Some(parse_json_async_complete), context as *mut c_void, &mut (*context).work )};
    debug_assert!(status == napi_status_napi_ok);


    println!("about to queue work");
    let status = unsafe {napi_queue_async_work(env, (*context).work)};
    debug_assert!(status == napi_status_napi_ok);

    get_undefined_value(env)
}
