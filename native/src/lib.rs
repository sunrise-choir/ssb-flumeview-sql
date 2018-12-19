#![recursion_limit = "1024"]
#[macro_use]
extern crate error_chain;

extern crate serde;

extern crate serde_derive;
extern crate serde_json;

mod napi;
mod napi_sys;
mod errors;

use napi::*;
use napi_sys::*;
use std::ptr::{null, null_mut};
use std::os::raw::{c_char, c_void};
use std::{debug_assert};
use std::slice;

#[no_mangle]
pub extern "C" fn define_view_class(env: napi_env) -> napi_value {
    null_mut()
}

