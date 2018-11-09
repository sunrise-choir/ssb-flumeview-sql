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

#[no_mangle]
pub extern "C" fn parse_legacy(env: napi_env, info: napi_callback_info) -> napi_value {

    let input = r#"
      {
        "a_boolean": true,
        "an_array": [3, 2, 1]
      }
    "#;


    // A JSON deserializer. You can use any Serde Deserializer here.
    let deserializer = serde_json::Deserializer::from_str(input);

    NapiEnv{env}.deserialize(deserializer).unwrap().value

}
